use std::ops::{Add, AddAssign, Sub, SubAssign};

pub use bincode as _bincode;
use serde::{Deserialize, Serialize};

use crate::Size;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
	pub line: i16,
	pub column: i16,
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
