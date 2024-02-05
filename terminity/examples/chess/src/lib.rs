mod board;

use std::io;
use std::time::{Duration, Instant};

use terminity::build_game;
use terminity::events::{Event, KeyCode, KeyPress};
use terminity::{events::EventPoller, game::Game};

use board::Board;

impl Game for Chess {
	type DataInput = ();
	type DataOutput = ();
	type WidgetKind = Board;

	fn start<R: io::Read>(_data: Option<Self::DataInput>) -> Self {
		Self { last_blink: Instant::now(), board: Board::default() }
	}

	fn disp<F: FnOnce(&Self::WidgetKind)>(&mut self, displayer: F) {
		displayer(&self.board)
	}

	fn update<E: EventPoller>(&mut self, events: E) {
		for event in events {
			match event {
				Event::Mouse(_e) => {
					// Using the terminity_widget mouse api.
					// The wrapping auto-padder filters out the events out of the board
					// and changes the column and line values to correspond to the position
					// on the board.
					todo!()
				}
				Event::KeyPress(KeyPress { code: KeyCode::Enter, .. }) => {
					if self.board.selected.is_none() {
						self.board.select();
					} else {
						self.board.play();
					}
				}
				Event::KeyPress(KeyPress { code: KeyCode::Left, .. }) => {
					if self.board.cursor_pos.0 > 0 {
						self.board.cursor_pos.0 -= 1;
					}
				}
				Event::KeyPress(KeyPress { code: KeyCode::Right, .. }) => {
					if self.board.cursor_pos.0 < 7 {
						self.board.cursor_pos.0 += 1;
					}
				}
				Event::KeyPress(KeyPress { code: KeyCode::Up, .. }) => {
					if self.board.cursor_pos.1 < 7 {
						self.board.cursor_pos.1 += 1;
					}
				}
				Event::KeyPress(KeyPress { code: KeyCode::Down, .. }) => {
					if self.board.cursor_pos.1 > 0 {
						self.board.cursor_pos.1 -= 1;
					}
				}
				// Use the auto-padder to handle resize
				// Event::Resize(Size { width, height }) => board.resize((w as usize, h as usize)),
				_ => continue, // Wait another event
			}
			// If no continue encountered, reset blinking
			self.last_blink = Instant::now();
			self.board.cursor_style_alt = false;
		}
		let timeout: u64 = if self.board.selected.is_none() { 400 } else { 100 };
		if self.last_blink.elapsed() > Duration::from_millis(timeout) {
			self.last_blink = Instant::now();
			self.board.cursor_style_alt = !self.board.cursor_style_alt;
		}
	}

	fn finish(self) -> Option<Self::DataOutput> {
		None
	}
}

struct Chess {
	last_blink: Instant,
	board: Board,
}

build_game!(Chess);
