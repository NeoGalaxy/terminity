pub mod events;
pub mod game_handling;
mod interface;

use anyhow::bail;
use clap::Parser;
use crossterm::{
	event::{
		DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
		EnableFocusChange, EnableMouseCapture, KeyboardEnhancementFlags,
		PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
	},
	execute, QueueableCommand as _,
};
use game_handling::GameCommands;
use terminity::{
	events::{Event, EventPoller},
	game::{Game, WidgetDisplayer},
	LineDisp, Size,
};
use tokio::time::sleep;

use std::{
	cell::RefCell,
	fs::{self, File},
	io::{stdout, Write as _},
	path::PathBuf,
	time::Duration,
};

use crate::interface::Hub;

#[derive(Parser)]
struct Args {
	game: PathBuf,
}

struct NativeDisplayer;

impl WidgetDisplayer for NativeDisplayer {
	fn run<W: terminity::widgets::Widget>(self, widget: &W) {
		std::io::stdout()
			.queue(crossterm::cursor::MoveTo(0, 0))
			.unwrap()
			.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
			.unwrap()
			.flush()
			.unwrap();
		print!("{}", LineDisp(0, widget));
		for l in 1..widget.size().height {
			print!("\n\r{}", LineDisp(l, widget));
		}
		std::io::stdout().flush().unwrap();
	}
}

struct NativePoller {
	cmds: RefCell<GameCommands>,
}

impl NativePoller {
	fn new() -> Self {
		Self { cmds: GameCommands::default().into() }
	}
}

impl EventPoller for &NativePoller {
	type Iter<'a> = NativePollerIter where Self: 'a;
	fn cmd(&self, command: terminity::events::CommandEvent) {
		match command {
			terminity::events::CommandEvent::CloseApp => self.cmds.borrow_mut().close = true,
		}
	}

	fn events(&self) -> Self::Iter<'_> {
		NativePollerIter
	}
}

struct NativePollerIter;

impl Iterator for NativePollerIter {
	type Item = Event;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			break if let Ok(true) = crossterm::event::poll(Duration::ZERO) {
				let Some(e) = events::from_crossterm(crossterm::event::read().ok()?) else {
					continue;
				};
				Some(e)
			} else {
				None
			};
		}
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// let args = Args::parse();

	let tty_config = if let Some(d) = directories::ProjectDirs::from("", "", "Terminity") {
		d.config_dir().to_owned().join("global_conf.json")
	} else {
		println!("----------------------------------------------");
		println!("                 FATAL ERROR");
		println!("----------------------------------------------");
		println!("Could not retrive the config directory");
		println!("----------------------------------------------");
		bail!("Couldn't start Terminity");
	};
	crossterm::terminal::enable_raw_mode()?;
	stdout()
		.queue(crossterm::cursor::SavePosition)?
		.queue(crossterm::terminal::EnterAlternateScreen)?
		.queue(crossterm::cursor::MoveTo(0, 0))?
		.queue(crossterm::cursor::Hide)?
		.flush()?;
	execute!(
		stdout(),
		EnableBracketedPaste,
		EnableFocusChange,
		EnableMouseCapture,
		// PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES),
		// PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS),
		// PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES),
		PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all()),
	)?;

	let size = {
		let tmp = crossterm::terminal::size().unwrap_or((100, 30));
		Size { width: tmp.0, height: tmp.1 }
	};

	let games = File::open(&tty_config).ok().and_then(|f| serde_json::from_reader(f).ok());

	let mut close = false;
	let mut start_screen = Hub::start(games, size);
	while !close {
		let poller = NativePoller::new();
		start_screen.update(&poller);
		close = poller.cmds.borrow().close;

		start_screen.disp(NativeDisplayer);

		sleep(Duration::from_millis(50)).await;
	}

	execute!(
		stdout(),
		DisableBracketedPaste,
		DisableFocusChange,
		DisableMouseCapture,
		PopKeyboardEnhancementFlags
	)?;
	stdout()
		.queue(crossterm::terminal::LeaveAlternateScreen)?
		.queue(crossterm::cursor::RestorePosition)?
		.queue(crossterm::cursor::Show)?
		.flush()?;

	crossterm::terminal::disable_raw_mode()?;
	println!("Terminal restored.");
	println!("Closing Terminity...");
	let data = start_screen.finish();
	println!("Saving remaining data...");
	fs::create_dir_all(tty_config.parent().unwrap()).unwrap();
	serde_json::to_writer(File::create(&tty_config).unwrap(), &data).unwrap();
	println!("Saved.");
	Ok(())
}
