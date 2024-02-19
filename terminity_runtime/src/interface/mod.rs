use std::{cell::RefCell, mem, sync::Arc};

use terminity::{
	events::{CommandEvent, Event, EventPoller, KeyCode, KeyModifiers, KeyPress},
	game::{Game, WidgetDisplayer},
	Size,
};
use tokio::sync::mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender};

use crate::game_handling::{GameCommands, GameLib};

use self::{
	game_repository::GameRepo, gamescreen::GameScreen, mainscreen::MainScreen, start::StartScreen,
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
pub struct Hub {
	screen: HubScreens,
	size: Size,
	games: Vec<GameData>,
	game_canal: (
		UnboundedSender<Result<(String, GameLib), libloading::Error>>,
		UnboundedReceiver<Result<(String, GameLib), libloading::Error>>,
	),
	run_game: (Sender<Arc<GameLib>>, Receiver<Arc<GameLib>>),
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

impl Game for Hub {
	type DataInput = GameRepo;
	type DataOutput = ();

	fn start(data: Option<Self::DataInput>, size: Size) -> Self {
		let game_canal = mpsc::unbounded_channel();
		let sender = game_canal.0.clone();
		if let Some(data) = data {
			tokio::spawn(async move {
				let data = data.open();
				let root = data.root_path;
				for game in data.games.into_iter() {
					if sender
						.send(
							unsafe { GameLib::new(root.join(&game.subpath)) }
								.map(|lib| (game.subpath.to_string_lossy().into(), lib)),
						)
						.is_err()
					{
						// Reciever closed, no need to continue
						break;
					}
				}
			});
		}
		Self {
			screen: HubScreens {
				start: StartScreen::new(size),
				main: MainScreen::new(size),
				current: HubScreen::Start,
			},
			size,
			games: vec![],
			game_canal,
			run_game: mpsc::channel(1),
		}
	}

	fn disp<D: WidgetDisplayer>(&mut self, displayer: D) {
		match &mut self.screen.current {
			HubScreen::Start => self.screen.start.disp(displayer),
			HubScreen::Main => self.screen.main.disp(displayer, &self.games),
			HubScreen::Game(game) => game.disp(displayer, self.size),
		}
	}

	fn update<E: terminity::events::EventPoller>(&mut self, poller: E) {
		if let Ok(game) = self.run_game.1.try_recv() {
			self.screen.current = HubScreen::Game(GameScreen::open(game))
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
				self.screen.main.update(
					poller,
					&mut self.games,
					&mut self.game_canal,
					&mut self.run_game.0,
				);
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

	fn finish(self) -> Option<Self::DataOutput> {
		None
	}
}
