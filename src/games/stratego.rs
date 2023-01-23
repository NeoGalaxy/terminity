use std::io;

use super::CLGame;

#[derive(Debug)]
struct Stratego ();

impl CLGame for Stratego {
	// add code here
	fn run(_term: console::Term) -> Result<(), io::Error> {
		todo!()
	}
}
