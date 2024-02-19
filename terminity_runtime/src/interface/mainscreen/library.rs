use std::sync::Arc;

use terminity::{
	events::KeyCode,
	widgets::{
		positionning::{div::Div1, Position, Spacing},
		Widget,
	},
	Size,
};
use tokio::sync::mpsc::Sender;

use crate::{game_handling::GameLib, interface::GameData};

#[derive(Debug)]
pub struct LibraryTab {
	selected: usize,
	tick: u8,
	size: Size,
}

#[derive(Debug)]
pub struct GameEntry<'a>(&'a GameData, bool, u8);

impl Widget for GameEntry<'_> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, _: u16) -> std::fmt::Result {
		let GameData { uid, name, .. } = self.0;
		write!(
			f,
			" {} {name:.<100} (id: {uid:.>8})",
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
	}

	fn size(&self) -> Size {
		Size { width: (5 + self.0.name.len().max(100) + 6 + 8 + 1) as u16, height: 1 }
	}
}

impl LibraryTab {
	// add code here
	pub fn display_line(
		&self,
		f: &mut std::fmt::Formatter<'_>,
		line: u16,
		games: &[GameData],
	) -> std::result::Result<(), std::fmt::Error> {
		if let Some(game) = games.get(line as usize) {
			let selected = line as usize == self.selected;
			Div1::new(true, GameEntry(game, selected, self.tick))
				.with_forced_size(Size { width: self.size.width, height: 1 })
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
		games: &mut Vec<GameData>,
		run_game: &mut Sender<Arc<GameLib>>,
	) {
		self.tick = self.tick.wrapping_add(1);
		for e in poller.events() {
			if let terminity::events::Event::KeyPress(k) = e {
				match k.code {
					KeyCode::Up => self.selected = self.selected.saturating_sub(1),
					KeyCode::Down => self.selected = self.selected.saturating_add(1),
					KeyCode::PageUp => self.selected = self.selected.saturating_sub(30),
					KeyCode::PageDown => self.selected = self.selected.saturating_add(30),
					KeyCode::Delete => {
						games.remove(self.selected);
					}
					KeyCode::Enter => {
						run_game.try_send(games[self.selected].lib.clone());
					}
					_ => (),
				}
			}
		}

		if !games.is_empty() && self.selected >= games.len() {
			self.selected = games.len() - 1;
		}
	}
}
