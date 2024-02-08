use libloading::{Library, Symbol};
use std::{
	mem::{forget, size_of},
	path::PathBuf,
	ptr::null_mut,
	slice,
	sync::mpsc::Receiver,
};
use terminity::{
	events::{CommandEvent, Event, KeyCode, KeyModifiers, KeyPress, TerminityCommandsData},
	game::GameData,
	widgets::Widget,
	WidgetBuffer, WidgetDisplay,
};

#[derive(WidgetDisplay)]
pub struct GameDisplay(pub WidgetBuffer);

impl Widget for GameDisplay {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: usize) -> std::fmt::Result {
		if self.0.is_empty() {
			return Ok(());
		}
		let bounds_index = line * size_of::<u16>();
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
		write!(f, "{s}")
	}

	fn size(&self) -> (usize, usize) {
		(self.0.width as usize, self.0.height as usize)
	}
}

impl GameDisplay {
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}
}

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

pub struct GameLib {
	pub game: Library,
}

impl GameLib {
	pub unsafe fn new(game_path: PathBuf) -> Result<Self, libloading::Error> {
		let game = unsafe { libloading::Library::new(game_path)? };
		Ok(Self { game })
	}

	pub unsafe fn start(
		&mut self,
		event_canal: Receiver<Event>,
	) -> Result<GameHandle, libloading::Error> {
		let handle = GameBinding {
			start_game: self.game.get(b"start_game\0")?,
			disp_game: self.game.get(b"disp_game\0")?,
			update_game: self.game.get(b"update_game\0")?,
			close_game: self.game.get(b"close_game\0")?,
			free_command_buffer: self.game.get(b"free_command_buffer\0")?,
			free_game_data: self.game.get(b"free_game_data\0")?,
		};
		Ok(GameHandle::start(handle, event_canal))
	}
}

#[derive(Debug)]
pub struct GameBinding<'a> {
	start_game: Symbol<'a, unsafe extern "C" fn(GameData)>,
	disp_game: Symbol<'a, unsafe extern "C" fn() -> WidgetBuffer>,
	update_game: Symbol<'a, unsafe extern "C" fn(*const u8, size: u32) -> TerminityCommandsData>,
	close_game: Symbol<'a, unsafe extern "C" fn() -> GameData>,
	free_command_buffer: Symbol<'a, unsafe extern "C" fn(TerminityCommandsData)>,
	free_game_data: Symbol<'a, unsafe extern "C" fn(data: GameData)>,
}

impl GameBinding<'_> {
	pub fn start_game(&self, data: GameData) {
		unsafe { (self.start_game)(data) }
	}
	pub fn disp_game(&self) -> WidgetBuffer {
		unsafe { (self.disp_game)() }
	}
	pub fn update_game(&self, events: *const u8, size: u32) -> TerminityCommandsData {
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
	event_canal: Receiver<Event>,
}

impl<'a> GameHandle<'a> {
	fn start(binding: GameBinding<'a>, event_canal: Receiver<Event>) -> Self {
		binding.start_game(GameData { content: null_mut(), size: 0, capacity: 0 });
		Self { binding, event_buffer: Vec::with_capacity(128), event_canal }
	}

	#[must_use]
	pub fn display(&self) -> Option<GameDisplay> {
		let buffer = self.binding.disp_game();
		if buffer.is_empty() {
			None
		} else {
			Some(GameDisplay(buffer))
		}
	}

	pub fn tick(&mut self) -> GameCommands {
		self.event_buffer.clear();
		while let Ok(evt) = self.event_canal.try_recv() {
			if matches!(
				evt,
				Event::KeyPress(KeyPress {
					code: KeyCode::Char('c'),
					modifiers: KeyModifiers { shift: false, control: true, alt: false, .. },
					repeated: _
				})
			) {
				return GameCommands { close: true };
			}

			let size_pos = self.event_buffer.len();
			self.event_buffer.extend_from_slice(&[0, 0]);
			bincode::serialize_into(&mut self.event_buffer, &evt).unwrap();
			let size = self.event_buffer.len() - (size_pos + 2);
			let bytes = size.to_le_bytes();
			self.event_buffer[size_pos] = bytes[0];
			self.event_buffer[size_pos + 1] = bytes[1];
		}
		let cmds_data =
			self.binding.update_game(self.event_buffer.as_ptr(), self.event_buffer.len() as u32);
		let cmds = GameCommands::read(&cmds_data);
		self.binding.free_command_buffer(cmds_data);
		cmds
	}

	pub fn close_save(self) {
		let data = self.binding.close_game();
		self.binding.free_game_data(data);
		forget(self)
	}
}

impl Drop for GameHandle<'_> {
	fn drop(&mut self) {
		self.binding.free_game_data(self.binding.close_game());
	}
}