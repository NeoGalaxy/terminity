use clap::Parser;
use libloading::{Library, Symbol};
use std::{
	path::PathBuf,
	ptr::{null, null_mut},
	slice,
};
use terminity::{GameData, Widget, WidgetBuffer};
use terminity_widgets::WidgetDisplay;

#[derive(Parser)]
struct Args {
	game: PathBuf,
}

#[derive(WidgetDisplay)]
struct GameDisplay(WidgetBuffer);

impl Widget for GameDisplay {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: usize) -> std::fmt::Result {
		let content = unsafe {
			slice::from_raw_parts(
				self.0.content.offset(line as isize * self.0.width as isize),
				self.0.width as usize,
			)
		};
		let s = unsafe { std::str::from_utf8_unchecked(content) };
		write!(f, "{s}")
	}

	fn size(&self) -> (usize, usize) {
		(self.0.width as usize, self.0.height as usize)
	}
}
struct GameLib {
	game: Library,
}

impl GameLib {
	unsafe fn new(game: Library) -> Self {
		Self { game }
	}

	unsafe fn handle(&self) -> GameHandle {
		GameHandle {
			start_game: self.game.get(b"start_game\0").unwrap(),
			disp_game: self.game.get(b"disp_game\0").unwrap(),
			update_game: self.game.get(b"update_game\0").unwrap(),
			close_game: self.game.get(b"close_game\0").unwrap(),
			deallocate_data: self.game.get(b"deallocate_data\0").unwrap(),
		}
	}
}

struct GameHandle<'a> {
	start_game: Symbol<'a, unsafe extern "C" fn(GameData)>,
	disp_game: Symbol<'a, unsafe extern "C" fn() -> WidgetBuffer>,
	update_game: Symbol<'a, unsafe extern "C" fn(events: *const u8, size: u32)>,
	close_game: Symbol<'a, unsafe extern "C" fn() -> GameData>,
	deallocate_data: Symbol<'a, unsafe extern "C" fn(data: GameData)>,
}

impl GameHandle<'_> {
	fn start_game(&self, data: GameData) {
		unsafe { (self.start_game)(data) }
	}
	fn disp_game(&self) -> WidgetBuffer {
		unsafe { (self.disp_game)() }
	}
	fn update_game(&self, events: *const u8, size: u32) {
		unsafe { (self.update_game)(events, size) }
	}
	fn close_game(&self) -> GameData {
		unsafe { (self.close_game)() }
	}
	fn deallocate_data(&self, data: GameData) {
		unsafe { (self.deallocate_data)(data) }
	}
}

fn main() {
	let args = Args::parse();
	let game = unsafe { libloading::Library::new(args.game).unwrap() };
	let game = unsafe { GameLib::new(game) };
	let game = unsafe { game.handle() };
	game.start_game(GameData { content: null_mut(), size: 0, capacity: 0 });
	let output = game.disp_game();
	println!("Game output:\n{}\n<end of output>", GameDisplay(output));
	game.update_game(null(), 0);
	let data = game.close_game();
	game.deallocate_data(data);
}
