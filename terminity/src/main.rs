use crossterm::QueueableCommand;
use std::io::Write;
#[macro_use]
extern crate lazy_static;

use std::{io::stdout, panic::catch_unwind};

mod games;

fn main() -> std::io::Result<()> {
	games::get("SuperTicTacToe").unwrap().run()
}
