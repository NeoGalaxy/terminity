#![allow(clippy::tabs_in_doc_comments)]

//! Crate to register multiple terminal games, allow to choose a game and set up an environment
//! to run them. This is at an extremely early development stage.
//!
//! To try it, clone the project and run `cargo run Chess` or `cargo run SuperTicTacToe`.
//!
//! The purposes and goals of this crate are to to:
//! 1. Make it easier to build good UI in terminal
//! 2. Make terminal games accessible to anyone and everyone. A windows `.exe` is to be expected,
//! and I'd love to make a smartphone app.
//!
//! I would also love to setup a P2P (peer to peer) system allowing to play the games online with
//! anyone, and giving the programmers an API to setup an online game without a mandatory need for a
//! server (although a way to create a server-client game is to be expected if the P2P system is
//! created). Making the P2P system seems feasible, but I don't know yet for the API.
//!
//! This project is of course not feasible alone. It needs at least a community to try it and give
//! feedback. If you want to support this project in any way, from being part of a
//! subreddit to be an active developer, please do!! You don't have to be a developer or have money
//! to spend to help, and any help is probably more helping than you expect.
//!
//! Currently, to ease the UI building, a bit of configuration is made to help making something of
//! quality. That is:
//!
//! * Enable paste, focus change, and mouse captures
//! * Put the terminal in "alternate screen"
//! * Enable raw mode
//! * Save cursor position and move it to 0,0
//! * When any unwind (and thus most panics) occurs, the terminal state is restored before the
//! unwinding data is displayed (the display may be improved though). Without that, the terminal
//! state keeps the configuration and the sh CLI becomes crappy.
//!
//! The result of doing all this can be seen for instance on the chess implementation, where
//! dragging with the mouse is supported, and any keyboard input is captured immediately.

// #![warn(missing_docs)]

use std::{fmt::Display, io};

pub use bincode as _bincode;
use events::EventPoller;
use serde::{Deserialize, Serialize};

pub use terminity_widgets::Widget;

pub mod events;
pub mod games;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
	width: u16,
	height: u16,
}

pub trait Game {
	type DataInput: for<'a> Deserialize<'a>;
	type DataOutput: Serialize;
	type WidgetKind: Widget;

	fn start<R: io::Read>(data: Option<Self::DataInput>) -> Self;

	fn disp<F: FnOnce(&Self::WidgetKind)>(&mut self, displayer: F);

	fn update<E: EventPoller>(&mut self, events: E);

	fn finish(self) -> Option<Self::DataOutput>;
}

#[repr(C)]
pub struct GameData {
	pub content: *mut u8,
	pub size: u32,
	pub capacity: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct WidgetBuffer {
	pub width: u32,
	pub height: u32,
	pub content: *const u8,
}

pub struct LineDisp<'a, W: Widget + ?Sized>(pub usize, pub &'a W);
impl<W: Widget + ?Sized> Display for LineDisp<'_, W> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.1.display_line(f, self.0)
	}
}

#[macro_export]
macro_rules! build_game {
	($GAME: ident) => {
		mod _game_definition {
			use super::$GAME;
			use std::fmt::Write as _;

			static mut GAME: Option<$GAME> = None;
			static mut DISP_BUFFER: Option<String> = None;
			static mut CMD_BUFFER: Option<Vec<u8>> = None;

			#[no_mangle]
			pub unsafe extern "C" fn start_game(data: $crate::GameData) {
				let data = if data.content.is_null() {
					None
				} else {
					let data_vec = Vec::from_raw_parts(
						data.content,
						data.size as usize,
						data.capacity as usize,
					);
					Some($crate::_bincode::deserialize::<<$GAME as $crate::Game>::DataInput>(
						&data_vec,
					))
				};
				unsafe { DISP_BUFFER = Some(String::with_capacity(32)) }
				unsafe { CMD_BUFFER = Some(Vec::new()) }
				unsafe { GAME = Some(<$GAME as $crate::Game>::start::<std::fs::File>(None)) }
			}

			#[no_mangle]
			pub extern "C" fn disp_game() -> $crate::WidgetBuffer {
				let buffer = unsafe { DISP_BUFFER.as_mut() }.unwrap();
				let game = unsafe { GAME.as_mut() }.unwrap();
				let mut res =
					$crate::WidgetBuffer { width: 0, height: 0, content: std::ptr::null() };
				$crate::Game::disp(game, |w| {
					let (width, height) = $crate::Widget::size(w);
					buffer.clear();
					buffer.reserve_exact((width + 1) * height);
					for line in 0..height {
						write!(buffer, "{}", $crate::LineDisp(line, w)).unwrap()
					}
					res = $crate::WidgetBuffer {
						width: width as u32,
						height: height as u32,
						content: buffer.as_ptr(),
					};
				});
				res
			}

			#[no_mangle]
			pub unsafe extern "C" fn update_game(
				events: *const u8,
				size: u32,
			) -> $crate::events::TerminityCommandsData {
				let events = unsafe { std::slice::from_raw_parts(events, size as usize) };
				let commands_buffer = unsafe { CMD_BUFFER.take().unwrap() };
				let mut evt_reader = $crate::events::EventReader::new(events, commands_buffer);
				let game = unsafe { GAME.as_mut() }.unwrap();
				$crate::Game::update(game, &mut evt_reader);
				evt_reader.into_commands_data()
			}

			#[no_mangle]
			pub extern "C" fn close_game() -> $crate::GameData {
				let game = unsafe { GAME.take() }.unwrap();
				if let Some(game_state) = $crate::Game::finish(game) {
					let mut data = $crate::_bincode::serialize(&game_state).unwrap();
					let capacity = data.capacity();
					let size = data.len();
					let content = data.as_mut_ptr();
					$crate::GameData { content, size: size as u32, capacity: capacity as u32 }
				} else {
					$crate::GameData { content: std::ptr::null_mut(), size: 0, capacity: 0 }
				}
			}

			#[no_mangle]
			pub unsafe extern "C" fn free_command_buffer(
				buff: $crate::events::TerminityCommandsData,
			) {
				unsafe {
					CMD_BUFFER = Some(Vec::from_raw_parts(
						buff.content,
						buff.len as usize,
						buff.capacity as usize,
					))
				}
			}

			#[no_mangle]
			pub unsafe extern "C" fn free_game_data(data: $crate::GameData) {
				if !data.content.is_null() {
					// convert to deallocate
					Vec::from_raw_parts(data.content, data.size as usize, data.capacity as usize);
				}
			}
		}
	};
}
