use std::{
	cell::RefCell,
	mem::{forget, size_of},
	ops::{Add, AddAssign, Sub, SubAssign},
};

pub use bincode as _bincode;
use serde::{Deserialize, Serialize};

use crate::Size;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
	pub line: u16,
	pub column: u16,
}

impl Add<Position> for Position {
	type Output = Self;
	fn add(self, rhs: Position) -> Self::Output {
		Self { line: self.line + rhs.line, column: self.column + rhs.column }
	}
}

impl AddAssign for Position {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs
	}
}

impl SubAssign for Position {
	fn sub_assign(&mut self, rhs: Self) {
		*self = *self - rhs
	}
}

impl Sub<Position> for Position {
	type Output = Self;
	fn sub(self, rhs: Position) -> Self::Output {
		Self {
			line: self.line.saturating_sub(rhs.line),
			column: self.column.saturating_sub(rhs.column),
		}
	}
}

pub trait PositionnalEvent {
	fn get_pos(&self) -> Position;
	fn set_pos(&mut self, pos: Position);
	fn with_pos(&self, pos: Position) -> Self;
}

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
	pub position: Position,
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

pub trait EventPoller {
	type Iter<'a>: Iterator<Item = Event> + 'a
	where
		Self: 'a;
	fn cmd(&self, command: CommandEvent);
	fn events(&self) -> Self::Iter<'_>;
}

pub struct EventReader<'a> {
	pub slice: &'a [u8],
	pub commands: RefCell<Vec<u8>>,
}

#[repr(C)]
pub struct TerminityCommandsData {
	pub content: *mut u8,
	pub len: u32,
	pub capacity: u32,
}

impl<'a> EventReader<'a> {
	pub fn new(events: &'a [u8], buffer: Vec<u8>) -> Self {
		Self { slice: events, commands: buffer.into() }
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
	pub evts: &'a EventReader<'evts>,
	pub pos: usize,
}

impl Iterator for EventReaderIter<'_, '_> {
	type Item = Event;

	fn next(&mut self) -> Option<Self::Item> {
		if self.pos == self.evts.slice.len() {
			return None;
		}
		let size = u16::from_le_bytes(
			self.evts.slice[self.pos..self.pos + size_of::<u16>()].try_into().unwrap(),
		) as usize;

		self.pos += size_of::<u16>() + size;
		let evt_slice = &self.evts.slice[self.pos - size..self.pos];

		Some(bincode::deserialize(evt_slice).unwrap())
	}
}

impl<'evts> EventPoller for &mut EventReader<'evts> {
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
}
