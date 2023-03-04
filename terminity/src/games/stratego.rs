///! Not yet implemented
use std::io::{self, Write};

use super::Game;

#[derive(Debug)]
pub struct Stratego();

impl Game for Stratego {
	// add code here
	fn run(&self, _out: &mut dyn Write) -> Result<(), io::Error> {
		unimplemented!()
	}
}
