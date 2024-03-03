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

pub mod build_game;
pub mod events;
pub mod game;
pub mod wchar;
pub mod widget_string;
pub mod widgets;

extern crate self as terminity;
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;
// pub use terminity_proc::frame;
pub use terminity_proc::img;
pub use terminity_proc::wchar;
pub use terminity_proc::wline;
pub use terminity_proc::wstr;
// pub use terminity_proc::StructFrame;
pub use terminity_proc::WidgetDisplay;

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
