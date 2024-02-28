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

pub mod events;
pub mod game;
pub mod wchar;
pub mod widget_string;
pub mod widgets;

extern crate self as terminity;

use game::WidgetDisplayer;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::io::Write as _;
use std::iter::repeat;
use std::mem::size_of;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
use std::ptr::null;

// pub use terminity_proc::frame;
pub use terminity_proc::img;
pub use terminity_proc::wchar;
pub use terminity_proc::wline;
pub use terminity_proc::wstr;
// pub use terminity_proc::StructFrame;
pub use terminity_proc::WidgetDisplay;
use widgets::Widget;

/// Re-export for use through build_game macro
#[doc(hidden)]
pub use bincode as _bincode;
/// Re-export for use in proc macros
#[doc(hidden)]
pub mod _reexport {
	pub use crossterm::terminal::Clear;
	pub use crossterm::terminal::ClearType::UntilNewLine;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Size {
	pub width: u16,
	pub height: u16,
}

impl Add for Size {
	type Output = Self;
	fn add(self, rhs: Self) -> Self::Output {
		Self { width: self.width + rhs.width, height: self.height + rhs.height }
	}
}

impl Sub for Size {
	type Output = Self;
	fn sub(self, rhs: Self) -> Self::Output {
		Self { width: self.width - rhs.width, height: self.height - rhs.height }
	}
}

impl Mul<u16> for Size {
	type Output = Self;
	fn mul(self, rhs: u16) -> Self::Output {
		Self { width: self.width * rhs, height: self.height * rhs }
	}
}

impl Div<u16> for Size {
	type Output = Self;
	fn div(self, rhs: u16) -> Self::Output {
		Self { width: self.width / rhs, height: self.height / rhs }
	}
}

#[derive(Debug)]
#[repr(C)]
pub struct WidgetBuffer {
	pub width: u32,
	pub height: u32,
	pub content: *const u8,
}

pub struct DisplayToBuffer<'a> {
	pub buffer: &'a mut String,
	pub res_buffer: &'a mut WidgetBuffer,
}

impl WidgetDisplayer for DisplayToBuffer<'_> {
	fn run<W: Widget>(self, widget: &W) {
		*self.res_buffer = unsafe { WidgetBuffer::new(widget, self.buffer.as_mut_vec()) };
	}
}

impl WidgetBuffer {
	pub fn new_empty() -> Self {
		Self { width: 0, height: 0, content: null() }
	}

	pub unsafe fn new<W: Widget>(widget: &W, buffer: &mut Vec<u8>) -> Self {
		let Size { width, height } = widgets::Widget::size(widget);
		let (width, height) = (width as usize, height as usize);
		buffer.clear();

		// Reserve for the indexes of the lines
		buffer.extend(repeat(0).take((1 + height) * size_of::<u16>()));

		// TODO: better size heuristic + way to parameter
		buffer.reserve(width * height);

		for line in 0..height {
			let line_start = buffer.len();
			let bytes = line_start.to_le_bytes();

			buffer[line * size_of::<u16>()] = bytes[0];
			buffer[line * size_of::<u16>() + 1] = bytes[1];

			write!(buffer, "{}", LineDisp(line as u16, widget)).unwrap();
		}

		let line_end = buffer.len();
		let bytes = line_end.to_le_bytes();

		buffer[height * size_of::<u16>()] = bytes[0];
		buffer[height * size_of::<u16>() + 1] = bytes[1];

		Self { width: width as u32, height: height as u32, content: buffer.as_ptr() }
	}

	pub fn is_empty(&self) -> bool {
		self.content.is_null()
	}
}

pub struct LineDisp<'a, W: Widget + ?Sized>(pub u16, pub &'a W);
impl<W: Widget + ?Sized> Display for LineDisp<'_, W> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.1.display_line(f, self.0)
	}
}

#[macro_export]
macro_rules! build_game {
	($GAME: ident) => {
		mod __build_game {
			use super::$GAME;
			use std::fmt::Write as _;

			static mut GAME: Option<$GAME> = None;
			static mut DISP_BUFFER: Option<String> = None;
			static mut CMD_BUFFER: Option<Vec<u8>> = None;

			#[no_mangle]
			pub unsafe extern "C" fn start_game(
				data: $crate::game::GameData,
				width: u16,
				height: u16,
			) {
				let size = $crate::Size { width, height };
				let data = if data.content.is_null() {
					None
				} else {
					let data_vec = Vec::from_raw_parts(
						data.content,
						data.size as usize,
						data.capacity as usize,
					);
					$crate::_bincode::deserialize::<<$GAME as $crate::game::Game>::DataInput>(
						&data_vec,
					)
					.ok()
				};
				unsafe { DISP_BUFFER = Some(String::with_capacity(32)) }
				unsafe { CMD_BUFFER = Some(Vec::new()) }
				unsafe { GAME = Some($crate::game::Game::start(data, size)) }
			}

			#[no_mangle]
			pub extern "C" fn disp_game() -> $crate::WidgetBuffer {
				let mut buffer = unsafe { DISP_BUFFER.as_mut() }.unwrap();
				let game = unsafe { GAME.as_mut() }.unwrap();
				let mut res = $crate::WidgetBuffer::new_empty();
				$crate::game::Game::disp(
					game,
					$crate::DisplayToBuffer { buffer: &mut buffer, res_buffer: &mut res },
				);
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
				$crate::game::Game::update(game, &evt_reader);
				evt_reader.into_commands_data()
			}

			#[no_mangle]
			pub extern "C" fn close_game() -> $crate::game::GameData {
				let game = unsafe { GAME.take() }.unwrap();
				if let Some(game_state) = $crate::game::Game::finish(game) {
					let mut data = $crate::_bincode::serialize(&game_state).unwrap();
					let capacity = data.capacity();
					let size = data.len();
					let content = data.as_mut_ptr();
					$crate::game::GameData { content, size: size as u32, capacity: capacity as u32 }
				} else {
					$crate::game::GameData { content: std::ptr::null_mut(), size: 0, capacity: 0 }
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
			pub unsafe extern "C" fn free_game_data(data: $crate::game::GameData) {
				if !data.content.is_null() {
					// convert to de-allocate
					Vec::from_raw_parts(data.content, data.size as usize, data.capacity as usize);
				}
			}
		}
	};
}
