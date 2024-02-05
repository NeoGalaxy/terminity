pub mod events;

use clap::Parser;
use crossterm::{
	event::{
		DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
		EnableFocusChange, EnableMouseCapture, KeyboardEnhancementFlags,
		PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
	},
	execute, QueueableCommand as _,
};
use libloading::{Library, Symbol};
use std::{
	io::{stdout, Write as _},
	mem::size_of,
	panic::{self, catch_unwind, resume_unwind},
	path::PathBuf,
	ptr::{null, null_mut},
	slice,
	sync::{Arc, Mutex},
	thread::sleep,
	time::Duration,
};
use terminity::{
	events::{CommandEvent, Event, KeyCode, KeyModifiers, KeyPress, TerminityCommandsData},
	GameData, Widget, WidgetBuffer,
};
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

struct CommandReader {
	close: bool,
}

impl CommandReader {
	fn read(data: &TerminityCommandsData) -> Self {
		let data = unsafe { std::slice::from_raw_parts(data.content, data.len as usize) };
		let mut pos = 0;
		let mut close = false;

		while pos != data.len() {
			let size =
				u16::from_le_bytes(data[pos..pos + size_of::<u16>()].try_into().unwrap()) as usize;

			pos += size_of::<u16>() + size;
			let cmd_slice = &data[pos - size..pos];
			let cmd: CommandEvent = bincode::deserialize(cmd_slice).unwrap();

			match cmd {
				CommandEvent::CloseApp => close = true,
			}
		}
		Self { close }
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
			free_command_buffer: self.game.get(b"free_command_buffer\0").unwrap(),
			free_game_data: self.game.get(b"free_game_data\0").unwrap(),
		}
	}
}

struct GameHandle<'a> {
	start_game: Symbol<'a, unsafe extern "C" fn(GameData)>,
	disp_game: Symbol<'a, unsafe extern "C" fn() -> WidgetBuffer>,
	update_game: Symbol<'a, unsafe extern "C" fn(*const u8, size: u32) -> TerminityCommandsData>,
	close_game: Symbol<'a, unsafe extern "C" fn() -> GameData>,
	free_command_buffer: Symbol<'a, unsafe extern "C" fn(TerminityCommandsData)>,
	free_game_data: Symbol<'a, unsafe extern "C" fn(data: GameData)>,
}

impl GameHandle<'_> {
	fn start_game(&self, data: GameData) {
		unsafe { (self.start_game)(data) }
	}
	fn disp_game(&self) -> WidgetBuffer {
		unsafe { (self.disp_game)() }
	}
	fn update_game(&self, events: *const u8, size: u32) -> TerminityCommandsData {
		unsafe { (self.update_game)(events, size) }
	}
	fn free_command_buffer(&self, data: TerminityCommandsData) {
		unsafe { (self.free_command_buffer)(data) }
	}
	fn close_game(&self) -> GameData {
		unsafe { (self.close_game)() }
	}
	fn free_game_data(&self, data: GameData) {
		unsafe { (self.free_game_data)(data) }
	}
}

fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	let game = unsafe { libloading::Library::new(args.game).unwrap() };
	let game = unsafe { GameLib::new(game) };
	let game = unsafe { game.handle() };

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

	let mut event_buffer = Vec::with_capacity(128);

	let res = catch_unwind(move || {
		game.start_game(GameData { content: null_mut(), size: 0, capacity: 0 });
		'mainloop: loop {
			let output = game.disp_game();
			if !output.content.is_null() {
				stdout()
					.queue(crossterm::cursor::MoveTo(0, 0))
					.unwrap()
					.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
					.unwrap()
					.flush()
					.unwrap();
				print!("{}", GameDisplay(output));
			}
			print!("\n\r");
			event_buffer.clear();
			while crossterm::event::poll(Duration::ZERO).unwrap() {
				let Some(event) = events::from_crossterm(crossterm::event::read().unwrap()) else {
					continue;
				};
				if matches!(
					event,
					Event::KeyPress(KeyPress {
						code: KeyCode::Char('c'),
						modifiers: KeyModifiers { shift: false, control: true, alt: false, .. },
						repeated: _
					})
				) {
					break 'mainloop;
				}
				let size_pos = event_buffer.len();
				event_buffer.extend_from_slice(&[0, 0]);
				bincode::serialize_into(&mut event_buffer, &event).unwrap();
				let size = event_buffer.len() - (size_pos + 2);
				let bytes = size.to_le_bytes();
				event_buffer[size_pos] = bytes[0];
				event_buffer[size_pos + 1] = bytes[1];
			}
			let cmds_data = game.update_game(event_buffer.as_ptr(), event_buffer.len() as u32);
			let cmds = CommandReader::read(&cmds_data);
			game.free_command_buffer(cmds_data);

			if cmds.close {
				break;
			}

			sleep(Duration::from_millis(20));
		}

		let output = game.disp_game();
		if !output.content.is_null() {
			stdout()
				.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::Purge))
				.unwrap()
				.queue(crossterm::cursor::MoveTo(0, 0))
				.unwrap()
				.flush()
				.unwrap();
			print!("{}", GameDisplay(output));
		}

		let data = game.close_game();
		game.free_game_data(data);
	});

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
	// Restore panic state and manage any error during game
	panic::set_hook(old_hook);
	match res {
		Ok(()) => (),
		Err(e) => {
			eprintln!("Thread panicked: {}", panic_buffer.lock().unwrap());
			resume_unwind(e)
		}
	}

	Ok(())
}
