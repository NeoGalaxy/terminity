use std::{
	cell::RefCell,
	collections::HashMap,
	mem,
	path::{Path, PathBuf},
	sync::Arc,
};

use rand::distributions::{Distribution, Standard};
use terminity::{
	events::{CommandEvent, Event, EventPoller, KeyCode, KeyModifiers, KeyPress},
	game::WidgetDisplayer,
	Size,
};
use tokio::{
	fs, io,
	sync::mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender},
	task::JoinHandle,
};

use crate::game_handling::{GameCommands, GameLib};

use self::{
	game_repository::{GameDataLatest, GameRepo, GameRepoLatest},
	gamescreen::GameScreen,
	hub_games::HubGames,
	mainscreen::MainScreen,
	start::StartScreen,
};

mod game_repository;
mod gamescreen;
mod mainscreen;
mod start;

#[derive(Debug)]
pub struct GameData {
	uid: usize,
	name: String,
	lib: Arc<GameLib>,
}

#[derive(Debug)]
pub enum GameStatus {
	Unloaded,
	Loading(JoinHandle<Result<GameLib, libloading::Error>>),
	Loaded(Arc<GameLib>),
	Running(Arc<GameLib>),
}

mod hub_games {
	use super::*;

	#[derive(Debug, Default)]
	pub struct HubGames {
		content: HashMap<usize, Option<(GameDataLatest, GameStatus)>>,
		pub list: Vec<usize>,
		pub unlisted: Vec<usize>,
		deleted: usize,
	}

	fn new_id<R, T>(map: &HashMap<R, T>) -> R
	where
		Standard: Distribution<R>,
		R: std::cmp::Eq,
		R: std::hash::Hash,
	{
		for _ in 0..1000 {
			let res = rand::random();
			if !map.contains_key(&res) {
				return res;
			}
		}
		panic!("Couldn't build an ID after 1000 tries");
	}

	impl HubGames {
		pub fn new() -> Self {
			Default::default()
		}

		pub fn add(&mut self, game: (GameDataLatest, GameStatus)) -> usize {
			let id = new_id(&self.content);
			self.unlisted.push(id);
			self.content.insert(id, Some(game));
			id
		}

		pub fn remove(&mut self, id: usize) -> Option<(GameDataLatest, GameStatus)> {
			let res = self.content.get_mut(&id).and_then(|v| v.take());
			if res.is_some() {
				self.deleted += 1;
				if let Some(idx) = self.unlisted.iter().position(|v| *v == id) {
					self.unlisted.remove(idx);
				} else if let Some(idx) = self.list.iter().position(|v| *v == id) {
					self.list.remove(idx);
				}
			}
			res
		}

		pub fn get(&self, id: usize) -> Option<&(GameDataLatest, GameStatus)> {
			self.content.get(&id).and_then(|v| v.as_ref())
		}

		pub fn get_mut(&mut self, id: usize) -> Option<&mut (GameDataLatest, GameStatus)> {
			self.content.get_mut(&id).and_then(|v| v.as_mut())
		}

		pub fn into_values(self) -> impl Iterator<Item = (GameDataLatest, GameStatus)> {
			self.content.into_values().flatten()
		}

		pub fn contains_key(&self, id: usize) -> bool {
			self.get(id).is_some()
		}

		pub(crate) fn values(&self) -> impl Iterator<Item = &(GameDataLatest, GameStatus)> {
			self.content.values().flatten()
		}

		pub(crate) fn values_mut(
			&mut self,
		) -> impl Iterator<Item = &mut (GameDataLatest, GameStatus)> {
			self.content.values_mut().flatten()
		}
	}
}

#[derive(Debug)]
pub struct Context {
	root_path: PathBuf,
	games: hub_games::HubGames,
	add_game: (UnboundedSender<GameDataLatest>, UnboundedReceiver<GameDataLatest>),
	run_game: (Sender<Arc<GameLib>>, Receiver<Arc<GameLib>>),
}

#[derive(Debug)]
pub struct Hub {
	screen: HubScreens,
	size: Size,
	ctx: Context,
}

#[derive(Debug)]
struct HubScreens {
	start: StartScreen,
	main: MainScreen,
	current: HubScreen,
}

#[derive(Debug)]
enum HubScreen {
	Start,
	Main,
	Game(GameScreen),
}

pub(crate) struct PollerMap<'a, P: EventPoller, F: FnMut(Event) -> Option<Event>>(
	&'a P,
	RefCell<F>,
);
pub(crate) struct PollerMapIter<'a, P: EventPoller + 'a, F: FnMut(Event) -> Option<Event>>(
	P::Iter<'a>,
	&'a RefCell<F>,
);

impl<'a, P: EventPoller, F: FnMut(Event) -> Option<Event>> PollerMap<'a, P, F> {
	pub fn new(poller: &'a P, f: F) -> Self {
		Self(poller, f.into())
	}
}

impl<P: EventPoller, F: FnMut(Event) -> Option<Event>> EventPoller for PollerMap<'_, P, F> {
	type Iter<'a> = PollerMapIter<'a, P, F> where Self: 'a;

	fn cmd(&self, command: CommandEvent) {
		self.0.cmd(command)
	}

	fn events(&self) -> Self::Iter<'_> {
		PollerMapIter(self.0.events(), &self.1)
	}
}

impl<P: EventPoller, F: FnMut(Event) -> Option<Event>> Iterator for PollerMapIter<'_, P, F> {
	type Item = Event;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let Some(e) = (self.1.borrow_mut())(self.0.next()?) else {
				continue;
			};
			break Some(e);
		}
	}
}

pub fn load_game(
	parent_path: &Path,
	subpath: &Path,
) -> JoinHandle<Result<GameLib, libloading::Error>> {
	let path = parent_path.join(subpath);
	tokio::spawn(async move { unsafe { GameLib::new(path) } })
}

pub fn del_game(parent_path: &Path, subpath: &Path) -> JoinHandle<io::Result<()>> {
	let path = parent_path.join(subpath);
	tokio::spawn(fs::remove_file(path))
}

impl Hub {
	pub async fn start(data: Option<GameRepo>, size: Size) -> Self {
		let (root_path, games) = if let Some(data) = data {
			let GameRepoLatest { root_path, games: games_repo } = data.open();
			let mut games = HubGames::new();
			for g in games_repo {
				let path = g.subpath.clone();
				let handle = tokio::spawn(async move { unsafe { GameLib::new(path) } });
				games.add((g, GameStatus::Loading(handle)));
			}
			(root_path, games)
		} else {
			let root = if let Some(d) = directories::ProjectDirs::from("", "", "Terminity") {
				d.data_dir().join("games")
			} else if let Some(d) = directories::UserDirs::new() {
				d.home_dir().join(".terminty/games")
			} else {
				PathBuf::from(".terminty/games")
			};
			let _ = fs::create_dir_all(&root).await;
			let root = root.canonicalize().unwrap_or(root);
			(root, Default::default())
		};

		Self {
			screen: HubScreens {
				start: StartScreen::new(size),
				main: MainScreen::new(size),
				current: HubScreen::Start,
			},
			size,
			ctx: Context {
				root_path,
				games,
				run_game: mpsc::channel(1),
				add_game: mpsc::unbounded_channel(),
			},
		}
	}

	pub fn disp<D: WidgetDisplayer>(&mut self, displayer: D) {
		match &mut self.screen.current {
			HubScreen::Start => self.screen.start.disp(displayer),
			HubScreen::Main => self.screen.main.disp(displayer, &self.ctx.games),
			HubScreen::Game(game) => game.disp(displayer, self.size),
		}
	}

	pub async fn update<E: terminity::events::EventPoller>(&mut self, poller: E) {
		if let Ok(game) = self.ctx.run_game.1.try_recv() {
			self.screen.current =
				HubScreen::Game(GameScreen::open(game, self.size - Size { width: 2, height: 4 }))
		}

		while let Ok(game) = self.ctx.add_game.1.try_recv() {
			let game_path = game.subpath.clone();
			let handle = load_game(&self.ctx.root_path, &game_path);
			self.ctx.games.add((game, GameStatus::Loading(handle)));
		}

		for game in self.ctx.games.values_mut() {
			if let GameStatus::Loading(handle) = &mut game.1 {
				if handle.is_finished() {
					let res = handle.await;
					game.1 = if let Ok(Ok(lib)) = res {
						GameStatus::Loaded(lib.into())
					} else {
						GameStatus::Unloaded
					}
				}
			}
		}

		let mut exit_game = false;
		let running_game = matches!(self.screen.current, HubScreen::Game(_));
		let poller = PollerMap::new(&poller, |e| {
			if matches!(
				e,
				Event::KeyPress(KeyPress {
					code: KeyCode::Char('c'),
					modifiers: KeyModifiers { control: true, .. },
					..
				})
			) {
				poller.cmd(CommandEvent::CloseApp);
				None
			} else if running_game
				&& matches!(e, Event::KeyPress(KeyPress { code: KeyCode::Esc, .. }))
			{
				exit_game = true;
				None
			} else {
				Some(e)
			}
		});

		match &mut self.screen.current {
			HubScreen::Start => {
				let finished = self.screen.start.update(poller);
				if finished {
					self.screen.current = HubScreen::Main;
				}
			}
			HubScreen::Main => {
				self.screen.main.update(poller, &mut self.ctx).await;
			}
			HubScreen::Game(g) => {
				let GameCommands { close } = g.update(poller);
				exit_game = exit_game || close;
			}
		}

		if exit_game {
			let mut game = HubScreen::Main;
			mem::swap(&mut self.screen.current, &mut game);
			let HubScreen::Game(game) = game else { unreachable!() };
			game.finish();
		}
	}

	pub fn finish(self) -> Option<GameRepo> {
		let repo = GameRepoLatest {
			root_path: self.ctx.root_path,
			games: self.ctx.games.into_values().map(|g| g.0).collect(),
		};
		Some(repo.close())
	}
}
