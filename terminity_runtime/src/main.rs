pub mod events;
pub mod game_handling;

use clap::Parser;
use crossterm::{
	event::{
		DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
		EnableFocusChange, EnableMouseCapture, KeyboardEnhancementFlags,
		PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
	},
	execute, QueueableCommand as _,
};
use game_handling::GameLib;
use std::{thread::sleep, time::Duration};
use terminity::widgets::Widget;

use std::{
	io::{stdout, Write as _},
	path::PathBuf,
	sync::mpsc,
};

#[derive(Parser)]
struct Args {
	game: PathBuf,
}

fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	crossterm::terminal::enable_raw_mode()?;
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
	stdout()
		.queue(crossterm::cursor::SavePosition)?
		.queue(crossterm::terminal::EnterAlternateScreen)?
		.queue(crossterm::cursor::MoveTo(0, 0))?
		.queue(crossterm::cursor::Hide)?
		.flush()?;

	let (send, rcv) = mpsc::channel();
	let mut game = unsafe { GameLib::new(args.game) }?;
	let mut game = unsafe { game.start(rcv) }?;

	loop {
		if let Some(display) = game.display() {
			stdout()
				.queue(crossterm::cursor::MoveTo(0, 0))
				.unwrap()
				.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
				.unwrap()
				.flush()
				.unwrap();
			print!("{}", display);
			print!("\n\r");
		}

		sleep(Duration::from_millis(20));
		while crossterm::event::poll(Duration::ZERO).unwrap() {
			let Some(event) = events::from_crossterm(crossterm::event::read().unwrap()) else {
				continue;
			};
			send.send(event)?;
		}
		let cmds = game.tick();
		if cmds.close {
			break;
		}
	}

	game.close_save();

	stdout()
		.queue(crossterm::terminal::LeaveAlternateScreen)?
		.queue(crossterm::cursor::RestorePosition)?
		.queue(crossterm::cursor::Show)?
		.flush()?;
	crossterm::terminal::disable_raw_mode()?;
	execute!(
		stdout(),
		DisableBracketedPaste,
		DisableFocusChange,
		DisableMouseCapture,
		PopKeyboardEnhancementFlags
	)?;

	Ok(())
}
