use std::mem::{forget, size_of};

pub use bincode as _bincode;
use serde::{Deserialize, Serialize};

pub use terminity_widgets::Widget;

use crate::Size;
/// Represents a key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyCode {
	Backspace,
	Enter,
	Left,
	Right,
	Up,
	Down,
	// Home,
	// End,
	PageUp,
	PageDown,
	Tab,
	BackTab,
	Delete,
	// Insert,
	F(u8),
	Char(char),
	// Null,
	Esc,
	// CapsLock,
	// ScrollLock,
	// NumLock,
	// PrintScreen,
	// Pause,
	// Menu,
	// KeypadBegin,
	// Media(MediaKeyCode),
	// Modifier(ModifierKeyCode),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyModifiers {
	pub shift: bool,
	pub control: bool,
	pub alt: bool,
	pub start: bool,

	pub hyper: bool,
	pub meta: bool,

	pub keypad: bool,
	pub caps_lock: bool,
	pub num_lock: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRelease {
	pub code: KeyCode,
	pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPress {
	pub code: KeyCode,
	pub modifiers: KeyModifiers,
	pub repeated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseKind {
	Down(MouseButton),
	Up(MouseButton),
	Drag(MouseButton),
	Moved,
	ScrollDown,
	ScrollUp,
	ScrollLeft,
	ScrollRight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mouse {
	pub kind: MouseKind,
	pub line: u16,
	pub column: u16,
	pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
	KeyPress(KeyPress),
	KeyRelease(KeyRelease),
	FocusChange { has_focus: bool },
	Mouse(Mouse),
	// Paste(),
	Resize(Size),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandEvent {
	CloseApp,
}

pub trait EventPoller: Iterator<Item = Event> {
	fn cmd(&mut self, command: CommandEvent);
}

pub struct EventReader<'a> {
	pub slice: &'a [u8],
	pub pos: usize,
	pub commands: Vec<u8>,
}

#[repr(C)]
pub struct TerminityCommandsData {
	pub content: *mut u8,
	pub len: u32,
	pub capacity: u32,
}

impl<'a> EventReader<'a> {
	pub fn new(events: &'a [u8], buffer: Vec<u8>) -> Self {
		Self { slice: events, pos: 0, commands: buffer }
	}

	pub fn into_commands_data(mut self) -> TerminityCommandsData {
		let res = TerminityCommandsData {
			content: self.commands.as_mut_ptr(),
			len: self.commands.len() as u32,
			capacity: self.commands.capacity() as u32,
		};
		forget(self.commands);
		res
	}
}

impl Iterator for &mut EventReader<'_> {
	type Item = Event;

	fn next(&mut self) -> Option<Self::Item> {
		if self.pos == self.slice.len() {
			return None;
		}
		let size = u16::from_le_bytes(
			self.slice[self.pos..self.pos + size_of::<u16>()].try_into().unwrap(),
		) as usize;

		self.pos += size_of::<u16>() + size;
		let evt_slice = &self.slice[self.pos - size..self.pos];

		Some(bincode::deserialize(evt_slice).unwrap())
	}
}

impl EventPoller for &mut EventReader<'_> {
	fn cmd(&mut self, evt: CommandEvent) {
		// Reserve space to write the size
		self.commands.extend_from_slice(&[0, 0]);
		let len = self.commands.len();
		bincode::serialize_into(&mut self.commands, &evt).unwrap();
		let size = self.commands.len() - len;
		let bytes = size.to_le_bytes();
		// write the size
		self.commands[len - 2] = bytes[0];
		self.commands[len - 1] = bytes[1];
	}
}
