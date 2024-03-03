mod board;

use std::time::{Duration, Instant};

use board::Board;

use terminity::events::{Event, KeyCode, KeyPress, MouseButton, MouseKind};
use terminity::game::{Game, GameContext};
use terminity::widgets::EventBubbling;
use terminity::{build_game, Size};

impl Game for Chess {
	type DataInput = ();
	type DataOutput = ();

	fn start(_data: Option<Self::DataInput>, _size: Size) -> Self {
		Self { last_blink: Instant::now(), board: Board::default() }
	}

	fn update<E: GameContext>(&mut self, poller: E) {
		for event in poller.events() {
			match event {
				Event::Mouse(e) => {
					let tile = self.board.bubble_event(e.clone().into(), |t, _| Some(t?.0));
					let Some(tile) = tile else { continue };

					if tile != self.board.cursor_pos {
						self.board.cursor_pos = tile
					}

					match &e.kind {
						MouseKind::Up(MouseButton::Left) => self.board.select(),
						MouseKind::Down(MouseButton::Left) => self.board.play(),
						_ => (),
					}
				}
				Event::KeyPress(KeyPress { code: KeyCode::Enter, .. }) => {
					if self.board.selected.is_none() {
						self.board.select();
					} else {
						self.board.play();
					}
				}
				Event::KeyPress(KeyPress { code: KeyCode::Left, .. }) => {
					if self.board.cursor_pos.x > 0 {
						self.board.cursor_pos.x -= 1;
					}
				}
				Event::KeyPress(KeyPress { code: KeyCode::Right, .. }) => {
					if self.board.cursor_pos.x < 7 {
						self.board.cursor_pos.x += 1;
					}
				}
				Event::KeyPress(KeyPress { code: KeyCode::Up, .. }) => {
					if self.board.cursor_pos.y < 7 {
						self.board.cursor_pos.y += 1;
					}
				}
				Event::KeyPress(KeyPress { code: KeyCode::Down, .. }) => {
					if self.board.cursor_pos.y > 0 {
						self.board.cursor_pos.y -= 1;
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

		poller.display(&self.board)
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
