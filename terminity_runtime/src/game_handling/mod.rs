use libloading::{Library, Symbol};
use std::{convert::AsRef, ffi::OsStr, fmt::Debug, mem::size_of, ptr::null_mut, slice};
use terminity::{
	build_game::{TerminityCommandsData, UpdateResults, WidgetBuffer},
	events::{CommandEvent, Event},
	game::GameData,
	widgets::{EventBubbling, Widget},
	Size, WidgetDisplay,
};

#[derive(WidgetDisplay, EventBubbling)]
pub struct GameDisplay(pub WidgetBuffer);

impl Debug for GameDisplay {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<GameDisplay>")
	}
}

impl Widget for GameDisplay {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		if line as u32 >= self.0.height {
			return Ok(());
		}
		let bounds_index = line as usize * size_of::<u16>();
		let bounds = unsafe {
			(
				u16::from_le_bytes([
					*self.0.content.add(bounds_index),
					*self.0.content.add(bounds_index + 1),
				]),
				u16::from_le_bytes([
					*self.0.content.add(bounds_index + 2),
					*self.0.content.add(bounds_index + 3),
				]),
			)
		};
		let content = unsafe {
			slice::from_raw_parts(
				self.0.content.add(bounds.0 as usize),
				(bounds.1 - bounds.0) as usize,
			)
		};
		let s = unsafe { std::str::from_utf8_unchecked(content) };
		// let s = unsafe { std::str::from_utf8_unchecked(content) };
		write!(f, "{s}")
	}

	fn size(&self) -> terminity::Size {
		Size { width: self.0.width as u16, height: self.0.height as u16 }
	}
}

impl GameDisplay {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

#[derive(Debug, Default)]
#[must_use]
pub struct GameCommands {
	pub close: bool,
}

impl GameCommands {
	pub fn read(data: &TerminityCommandsData) -> Self {
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

#[derive(Debug)]
pub struct GameLib {
	pub game: Library,
}

impl GameLib {
	pub unsafe fn new<P: AsRef<OsStr>>(game_path: P) -> Result<Self, libloading::Error> {
		let game = unsafe { libloading::Library::new(game_path)? };
		Ok(Self { game })
	}

	pub unsafe fn start(
		&self,
		event_canal: kanal::Receiver<Event>,
		init_size: Size,
	) -> Result<GameHandle, libloading::Error> {
		let handle = GameBinding {
			start_game: self.game.get(b"start_game\0")?,
			update_game: self.game.get(b"update_game\0")?,
			close_game: self.game.get(b"close_game\0")?,
			free_command_buffer: self.game.get(b"free_command_buffer\0")?,
			free_game_data: self.game.get(b"free_game_data\0")?,
		};
		Ok(GameHandle::start(handle, event_canal, init_size))
	}
}

#[derive(Debug)]
pub struct GameBinding<'a> {
	start_game: Symbol<'a, unsafe extern "C" fn(GameData, u16, u16)>,
	update_game: Symbol<'a, unsafe extern "C" fn(*const u8, size: u32) -> UpdateResults>,
	close_game: Symbol<'a, unsafe extern "C" fn() -> GameData>,
	free_command_buffer: Symbol<'a, unsafe extern "C" fn(TerminityCommandsData)>,
	free_game_data: Symbol<'a, unsafe extern "C" fn(data: GameData)>,
}

impl GameBinding<'_> {
	pub fn start_game(&self, data: GameData, init_size: Size) {
		unsafe { (self.start_game)(data, init_size.width, init_size.height) }
	}
	pub fn update_game(&self, events: *const u8, size: u32) -> UpdateResults {
		unsafe { (self.update_game)(events, size) }
	}
	pub fn free_command_buffer(&self, data: TerminityCommandsData) {
		unsafe { (self.free_command_buffer)(data) }
	}
	pub fn close_game(&self) -> GameData {
		unsafe { (self.close_game)() }
	}
	pub fn free_game_data(&self, data: GameData) {
		unsafe { (self.free_game_data)(data) }
	}
}

#[derive(Debug)]
pub struct GameHandle<'a> {
	binding: GameBinding<'a>,
	event_buffer: Vec<u8>,
	event_canal: kanal::Receiver<Event>,
}

impl<'a> GameHandle<'a> {
	fn start(
		binding: GameBinding<'a>,
		event_canal: kanal::Receiver<Event>,
		init_size: Size,
	) -> Self {
		binding.start_game(GameData { content: null_mut(), size: 0, capacity: 0 }, init_size);
		Self { binding, event_buffer: Vec::with_capacity(128), event_canal }
	}

	pub fn tick(&mut self) -> (GameCommands, Option<GameDisplay>) {
		self.event_buffer.clear();
		while let Ok(Some(evt)) = self.event_canal.try_recv() {
			// if matches!(
			// 	evt,
			// 	Event::KeyPress(KeyPress {
			// 		code: KeyCode::Char('c'),
			// 		modifiers: KeyModifiers { shift: false, control: true, alt: false, .. },
			// 		repeated: _
			// 	})
			// ) {
			// 	return (GameCommands { close: true }, None);
			// }

			let size_pos = self.event_buffer.len();
			self.event_buffer.extend_from_slice(&[0, 0]);
			bincode::serialize_into(&mut self.event_buffer, &evt).unwrap();
			let size = self.event_buffer.len() - (size_pos + 2);
			let bytes = size.to_le_bytes();
			self.event_buffer[size_pos] = bytes[0];
			self.event_buffer[size_pos + 1] = bytes[1];
		}
		let UpdateResults { commands, display } =
			self.binding.update_game(self.event_buffer.as_ptr(), self.event_buffer.len() as u32);
		let cmds = GameCommands::read(&commands);
		self.binding.free_command_buffer(commands);
		(cmds, if display.is_empty() { None } else { Some(GameDisplay(display)) })
	}

	pub fn close_save(&mut self) {
		let data = self.binding.close_game();
		self.binding.free_game_data(data);
	}
}

/*struct GameTask {
	handle: tokio::task::JoinHandle<()>,
}

fn run_game_task(game: GameLib, init_size: Size) -> GameTask {
	let handle = tokio::spawn(async move {
		let (send, rcv) = kanal::bounded(200);
		let mut game = unsafe { game.start(rcv, init_size) }.unwrap();

		loop {
			while crossterm::event::poll(Duration::ZERO).unwrap() {
				let Some(event) = events::from_crossterm(crossterm::event::read().unwrap()) else {
					continue;
				};
				send.as_async().send(event).await.unwrap();
			}
			let (cmds, display) = game.tick();
			if let Some(display) = display {
				std::io::stdout()
					.queue(crossterm::cursor::MoveTo(0, 0))
					.unwrap()
					.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
					.unwrap()
					.flush()
					.unwrap();
				print!("{}", display);
				print!("\n\r");
			}

			if cmds.close {
				break;
			}

			sleep(Duration::from_millis(20)).await;
		}

		game.close_save();
	});
	GameTask { handle }
}*/
