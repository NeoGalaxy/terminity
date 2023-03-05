use crossterm::{
	event::{
		DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
		EnableFocusChange, EnableMouseCapture,
	},
	execute, QueueableCommand,
};
use lazy_static::lazy_static;
use std::{
	collections::HashMap,
	io::{stdout, Write},
	panic::{self, catch_unwind, resume_unwind, RefUnwindSafe},
	sync::{Arc, Mutex},
};

mod chess;
mod stratego;
mod sttt;

pub struct GameWrapper {
	game: Box<dyn Game + Send + Sync>,
	pub name: &'static str,
}

lazy_static! {
	/// This is an example for using doc comment attributes
	pub static ref REGISTERY: HashMap<&'static str, GameWrapper> = {
		let mut m = HashMap::new();
		m.insert("SuperTicTacToe", GameWrapper {
			game: Box::new(sttt::SuperTTT()),
			name: "SuperTicTacToe"
		}
		);
		/*m.insert("Stratego", GameWrapper {
			game: Box::new(stratego::Stratego()),
			name: "Stratego"
		}
		);*/
		m.insert("Chess", GameWrapper {
			game: Box::new(chess::Chess()),
			name: "Chess"
		}
		);
		m
	};
}

impl GameWrapper {
	pub fn run(&self) -> std::io::Result<()> {
		// Set up new hook
		let old_hook = panic::take_hook();
		let panic_buffer = Arc::new(Mutex::new(String::with_capacity(200)));
		panic::set_hook({
			let panic_buffer = panic_buffer.clone();
			Box::new(move |info| {
				panic_buffer.lock().unwrap().push_str(&format!(
					"payload: {:?}\nlocation: {:?}",
					info.payload().downcast_ref::<&str>(),
					info.location()
				));
			})
		});
		// Prepare game
		crossterm::terminal::enable_raw_mode()?;
		execute!(stdout(), EnableBracketedPaste, EnableFocusChange, EnableMouseCapture,)?;
		stdout()
			.queue(crossterm::cursor::SavePosition)?
			.queue(crossterm::terminal::EnterAlternateScreen)?
			.queue(crossterm::cursor::MoveTo(0, 0))?
			.flush()?;
		// Game!
		let res = catch_unwind(move || self.game.run(&mut stdout()));
		// Restore console state
		stdout()
			.queue(crossterm::terminal::LeaveAlternateScreen)?
			.queue(crossterm::cursor::RestorePosition)?
			.queue(crossterm::cursor::Show)?
			.flush()?;
		execute!(stdout(), DisableBracketedPaste, DisableFocusChange, DisableMouseCapture,)?;
		crossterm::terminal::disable_raw_mode()?;
		// Restore panic state and manage any error during game
		panic::set_hook(old_hook);
		match res {
			Ok(r) => r,
			Err(e) => {
				eprintln!("Thread panicked: {}", panic_buffer.lock().unwrap());
				resume_unwind(e)
			}
		}
	}
}

trait Game: RefUnwindSafe {
	fn run(&self, out: &mut dyn Write) -> std::io::Result<()>;
}

impl Game for () {
	fn run(&self, _: &mut dyn Write) -> std::io::Result<()> {
		panic!("Internal error: dummy game running")
	}
}

pub fn get(name: &str) -> Option<&'static GameWrapper> {
	REGISTERY.get(name)
}
