#[macro_use]
extern crate lazy_static;

use std::io::stdout;

mod games;

fn main() {
    games::get("SuperTicTacToe").unwrap().run(&mut stdout()).unwrap();
}
