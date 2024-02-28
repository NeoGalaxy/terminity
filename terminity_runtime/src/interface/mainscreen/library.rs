use terminity::{
	events::KeyCode,
	widgets::{
		positionning::{div::Div1, Position, Spacing},
		Widget,
	},
	Size,
};

use crate::interface::{
	del_game, game_repository::GameDataLatest, hub_games::HubGames, load_game, Context, GameStatus,
};

#[derive(Debug)]
pub struct LibraryTab {
	selected: usize,
	tick: u8,
	size: Size,
}

#[derive(Debug)]
pub struct GameEntry<'a>(Option<&'a (GameDataLatest, GameStatus)>, bool, u8);

impl Widget for GameEntry<'_> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, _: u16) -> std::fmt::Result {
		if let Some((GameDataLatest { subpath }, status)) = self.0 {
			let status = match status {
				GameStatus::Unloaded => "unloaded  ",
				GameStatus::Loading(_) => "loading...",
				GameStatus::Loaded(_) => "ready     ",
				GameStatus::Running(_) => "running...",
			};
			let subpath = subpath.display();
			write!(
				f,
				" {} {subpath:.<100} {status}",
				if self.1 {
					match (self.2 / 2) % 5 {
						0 => " *-",
						1 => "*- ",
						2 => ".--",
						3 => "-.-",
						4 => "  *",
						_ => unreachable!(),
					}
				} else {
					"   "
				}
			)
		} else {
			Spacing::line(self.size().width).with_char('-').display_line(f, 0)
		}
	}

	fn size(&self) -> Size {
		Size { width: 5 + 100 + 1 + 10, height: 1 }
	}
}

impl LibraryTab {
	// add code here
	pub fn display_line(
		&self,
		f: &mut std::fmt::Formatter<'_>,
		line: u16,
		games: &HubGames,
	) -> std::result::Result<(), std::fmt::Error> {
		let selected = line as usize == self.selected;
		if let Some(&game_id) = games.list.get(line as usize) {
			let game = games.get(game_id);
			Div1::new(true, GameEntry(game, selected, self.tick))
				.with_exact_size(Size { width: self.size.width, height: 1 })
				.with_content_pos(Position::Center)
				.display_line(f, 0)
		} else {
			Spacing::line(self.size.width).display_line(f, 0)
		}
	}

	pub(crate) fn new(size: Size) -> LibraryTab {
		LibraryTab { selected: 0, tick: 0, size }
	}

	pub(crate) fn update<P: terminity::events::EventPoller>(
		&mut self,
		poller: P,
		ctx: &mut Context,
	) {
		self.tick = self.tick.wrapping_add(1);
		for e in poller.events() {
			if let terminity::events::Event::KeyPress(k) = e {
				match k.code {
					KeyCode::Up => self.selected = self.selected.saturating_sub(1),
					KeyCode::Down => self.selected = self.selected.saturating_add(1),
					KeyCode::PageUp => self.selected = self.selected.saturating_sub(30),
					KeyCode::PageDown => self.selected = self.selected.saturating_add(30),
					KeyCode::Delete | KeyCode::Backspace => {
						if let Some(game) = ctx
							.games
							.list
							.get(self.selected)
							.copied()
							.and_then(|g| ctx.games.remove(g))
						{
							del_game(&ctx.root_path, &game.0.subpath);
						}
					}
					KeyCode::Enter => {
						if let Some(game) = ctx
							.games
							.list
							.get(self.selected)
							.copied()
							.and_then(|g| ctx.games.get_mut(g))
						{
							match &game.1 {
								crate::interface::GameStatus::Loaded(lib) => {
									let _ = ctx.run_game.0.try_send(lib.clone());
								}
								GameStatus::Unloaded => {
									game.1 = GameStatus::Loading(load_game(
										&ctx.root_path,
										&game.0.subpath,
									))
								}
								_ => (),
							}
						}
					}
					_ => (),
				}
			}
		}

		ctx.games.list.append(&mut ctx.games.unlisted);
		if !ctx.games.list.is_empty() && self.selected >= ctx.games.list.len() {
			self.selected = ctx.games.list.len() - 1;
		}
	}
}
