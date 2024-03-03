use crate::game::GameContext;
use crate::widgets::Widget;
use crate::Size;
use core::iter::repeat;
use core::ptr::null;
use std::fmt::Display;
use std::io::Write;
use std::mem::forget;

use std::cell::RefCell;

use crate::events::Event;

use std::mem::size_of;

use crate::events::CommandEvent;

#[derive(Debug)]
#[repr(C)]
pub struct WidgetBuffer {
	pub width: u32,
	pub height: u32,
	pub content: *const u8,
}

pub struct DisplayToBuffer<'a> {
	pub buffer: &'a mut String,
	pub res_buffer: &'a mut WidgetBuffer,
}

pub struct Context<'a> {
	pub events: &'a [u8],
	pub commands: RefCell<Vec<u8>>,
	pub disp_buffer: RefCell<DisplayToBuffer<'a>>,
}

#[repr(C)]
pub struct UpdateResults {
	pub commands: TerminityCommandsData,
	pub display: WidgetBuffer,
}

#[repr(C)]
pub struct TerminityCommandsData {
	pub content: *mut u8,
	pub len: u32,
	pub capacity: u32,
}

impl<'a> Context<'a> {
	pub fn new(events: &'a [u8], cmd_buffer: Vec<u8>, disp_buffer: DisplayToBuffer<'a>) -> Self {
		Self { events, commands: cmd_buffer.into(), disp_buffer: disp_buffer.into() }
	}

	pub fn into_commands_data(self) -> TerminityCommandsData {
		let mut commands = self.commands.take();
		let res = TerminityCommandsData {
			content: commands.as_mut_ptr(),
			len: commands.len() as u32,
			capacity: commands.capacity() as u32,
		};
		forget(commands);
		res
	}
}

pub struct EventReaderIter<'a, 'evts> {
	pub evts: &'a Context<'evts>,
	pub pos: usize,
}

impl Iterator for EventReaderIter<'_, '_> {
	type Item = Event;

	fn next(&mut self) -> Option<Self::Item> {
		if self.pos == self.evts.events.len() {
			return None;
		}
		let size = u16::from_le_bytes(
			self.evts.events[self.pos..self.pos + size_of::<u16>()].try_into().unwrap(),
		) as usize;

		self.pos += size_of::<u16>() + size;
		let evt_slice = &self.evts.events[self.pos - size..self.pos];

		Some(bincode::deserialize(evt_slice).unwrap())
	}
}

impl<'evts> GameContext for &Context<'evts> {
	type Iter<'a> = EventReaderIter<'a, 'evts> where Self: 'a;

	fn cmd(&self, evt: CommandEvent) {
		let mut commands = self.commands.borrow_mut();
		// Reserve space to write the size
		commands.extend_from_slice(&[0, 0]);
		let len = commands.len();
		bincode::serialize_into(&mut *commands, &evt).unwrap();
		let size = commands.len() - len;
		let bytes = size.to_le_bytes();
		// write the size
		commands[len - 2] = bytes[0];
		commands[len - 1] = bytes[1];
	}

	fn events(&self) -> Self::Iter<'_> {
		EventReaderIter { evts: self, pos: 0 }
	}

	fn display<W: Widget>(&self, widget: &W) {
		let mut disp_buffer = self.disp_buffer.borrow_mut();
		*disp_buffer.res_buffer =
			unsafe { WidgetBuffer::new(widget, disp_buffer.buffer.as_mut_vec()) };
	}
}

impl WidgetBuffer {
	pub fn new_empty() -> Self {
		Self { width: 0, height: 0, content: null() }
	}

	pub unsafe fn new<W: Widget>(widget: &W, buffer: &mut Vec<u8>) -> Self {
		let Size { width, height } = Widget::size(widget);
		let (width, height) = (width as usize, height as usize);
		buffer.clear();

		// Reserve for the indexes of the lines
		buffer.extend(repeat(0).take((1 + height) * size_of::<u16>()));

		// TODO: better size heuristic + way to parameter
		buffer.reserve(width * height);

		for line in 0..height {
			let line_start = buffer.len();
			let bytes = line_start.to_le_bytes();

			buffer[line * size_of::<u16>()] = bytes[0];
			buffer[line * size_of::<u16>() + 1] = bytes[1];

			write!(buffer, "{}", LineDisp(line as u16, widget)).unwrap();
		}

		let line_end = buffer.len();
		let bytes = line_end.to_le_bytes();

		buffer[height * size_of::<u16>()] = bytes[0];
		buffer[height * size_of::<u16>() + 1] = bytes[1];

		Self { width: width as u32, height: height as u32, content: buffer.as_ptr() }
	}

	pub fn is_empty(&self) -> bool {
		self.content.is_null()
	}
}

pub struct LineDisp<'a, W: Widget + ?Sized>(pub u16, pub &'a W);
impl<W: Widget + ?Sized> Display for LineDisp<'_, W> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.1.display_line(f, self.0)
	}
}

#[macro_export]
macro_rules! build_game {
	($GAME: ident) => {
		mod __build_game {
			use super::$GAME;
			use std::fmt::Write as _;

			static mut GAME: Option<$GAME> = None;
			static mut DISP_BUFFER: Option<String> = None;
			static mut CMD_BUFFER: Option<Vec<u8>> = None;

			#[no_mangle]
			pub unsafe extern "C" fn start_game(
				data: $crate::game::GameData,
				width: u16,
				height: u16,
			) {
				let size = $crate::Size { width, height };
				let data = if data.content.is_null() {
					None
				} else {
					let data_vec = Vec::from_raw_parts(
						data.content,
						data.size as usize,
						data.capacity as usize,
					);
					$crate::_bincode::deserialize::<<$GAME as $crate::game::Game>::DataInput>(
						&data_vec,
					)
					.ok()
				};
				unsafe { DISP_BUFFER = Some(String::with_capacity(32)) }
				unsafe { CMD_BUFFER = Some(Vec::new()) }
				unsafe { GAME = Some($crate::game::Game::start(data, size)) }
			}

			#[no_mangle]
			pub unsafe extern "C" fn update_game(
				events: *const u8,
				size: u32,
			) -> $crate::build_game::UpdateResults {
				let mut buffer = unsafe { DISP_BUFFER.as_mut() }.unwrap();
				let events = unsafe { std::slice::from_raw_parts(events, size as usize) };
				let commands_buffer = unsafe { CMD_BUFFER.take().unwrap() };
				let mut disp_res = $crate::build_game::WidgetBuffer::new_empty();
				let mut evt_reader = $crate::build_game::Context::new(
					events,
					commands_buffer,
					$crate::build_game::DisplayToBuffer {
						buffer: &mut buffer,
						res_buffer: &mut disp_res,
					},
				);
				let game = unsafe { GAME.as_mut() }.unwrap();
				$crate::game::Game::update(game, &evt_reader);
				$crate::build_game::UpdateResults {
					commands: evt_reader.into_commands_data(),
					display: disp_res,
				}
			}

			#[no_mangle]
			pub extern "C" fn close_game() -> $crate::game::GameData {
				let game = unsafe { GAME.take() }.unwrap();
				if let Some(game_state) = $crate::game::Game::finish(game) {
					let mut data = $crate::_bincode::serialize(&game_state).unwrap();
					let capacity = data.capacity();
					let size = data.len();
					let content = data.as_mut_ptr();
					$crate::game::GameData { content, size: size as u32, capacity: capacity as u32 }
				} else {
					$crate::game::GameData { content: std::ptr::null_mut(), size: 0, capacity: 0 }
				}
			}

			#[no_mangle]
			pub unsafe extern "C" fn free_command_buffer(
				buff: $crate::build_game::TerminityCommandsData,
			) {
				unsafe {
					CMD_BUFFER = Some(Vec::from_raw_parts(
						buff.content,
						buff.len as usize,
						buff.capacity as usize,
					))
				}
			}

			#[no_mangle]
			pub unsafe extern "C" fn free_game_data(data: $crate::game::GameData) {
				if !data.content.is_null() {
					// convert to de-allocate
					Vec::from_raw_parts(data.content, data.size as usize, data.capacity as usize);
				}
			}
		}
	};
}
