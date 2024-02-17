//! Widgets for easier terminal UI building.
//!
//! This crate revolves around the trait [`Widget`], and defines various [widgets] to help building
//! your own. It also defines various other traits for more transparent usage of the widgets.
//!
//! This crate is currently at a very early development stage. The first changes it might have are
//! an api for un-resizeable widgets and more widgets.

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::mem::size_of;
use std::mem::transmute;
use std::ops::Deref;
use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::events;
use crate::events::Position;
use crate::Size;

pub mod auto_padder;
// pub mod canvas;
// pub mod frame;
pub mod content;
pub mod positionning;
pub mod text;

pub struct WidgetLineDisplay<'a, W: Widget + ?Sized> {
	pub widget: &'a W,
	pub line: u16,
}

impl<'a, W: Widget + ?Sized> Display for WidgetLineDisplay<'a, W> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.widget.display_line(f, self.line)
	}
}

/// An item displayable on multiple lines, assumed to be a rectangle (though not yet enforced).
///
/// This trait is mainly used for composition, like inside frame. The idea is that widgets can include
/// other widgets, and this trait allows to display only one line of the widget, allowing the parent
/// widget to display anything it wants on the left and on the right of any multi-line child widget.
/// For the parent to be able to prepare for the display of a child widget, it probably needs to know
/// the size of this one, explaining the existence of `size`.
///
/// For the widget to behave correctly, it has to output lines of exactly the expected length.
/// Multiple widgets here will break this rule if not used correctly (like by putting a Widget of
/// size different from the one expected into a Frame or putting text longer than anticipated into
/// a Text), `debug_assert`s might be added in the near future.
///
/// Be careful, to avoid breaking this rule all the time, if the content of your widget is not long
/// enough you should add padding.
///
/// In the current implementation, the support for ANSI sequences is assumed and can thus be used.
///
/// NB: A widget's size shouldn't change during its display, as this could have some unpredictable
/// behavior for the parent. This is usually ensured by the fact that the widget is immutably
/// borrowed during its display, and it needs to be mutable for its size to change.
pub trait Widget {
	/// Prints the given line of the widget.
	///
	/// ```
	/// use terminity_widgets::widgets::text::Text;
	/// use terminity_widgets::Widget;
	/// use format::lazy_format;
	///
	/// let text = Text::new(["Hello".into(), "World".into()], 5);
	/// let formatted = text.get_line_display(1).to_string();
	/// assert_eq!(formatted, "World");
	/// ```
	fn display_line(&self, f: &mut Formatter<'_>, line: u16) -> std::fmt::Result;
	/// The current size of the widget, composed of first the width, then the height.
	///
	/// ```
	/// use terminity_widgets::widgets::text::Text;
	/// use terminity_widgets::Widget;
	///
	/// let text = Text::new(["Hello".into(), "World".into()], 5);
	/// assert_eq!(text.size(), (5, 2));
	/// ```
	fn size(&self) -> Size;

	fn display_line_in(
		&self,
		f: &mut Formatter<'_>,
		line: u16,
		bounds: Range<u16>,
	) -> std::fmt::Result {
		let output = self.get_line_display(line).to_string();
		let res: std::fmt::Result =
			String::from_utf8(strip_ansi_escapes::strip(output).map_err(|_| fmt::Error)?)
				.unwrap()
				.graphemes(true)
				.enumerate()
				.skip_while(|(i, _)| *i < bounds.start as usize)
				.map_while(
					|(i, s)| if i < bounds.end as usize { Some(f.write_str(s)) } else { None },
				)
				.collect();
		res
	}

	fn get_line_display(&self, line: u16) -> WidgetLineDisplay<'_, Self> {
		WidgetLineDisplay { widget: self, line }
	}
}

/// A widget that supports mouse events handling.
///
/// When a widget implements this trait, it is assumed to be able to be able to handle mouse events.
/// The main purpose of this method is event bubbling: when a parent widget receives a mouse event,
/// it may manage it itself or pass it on to a child widget by adapting the coordinates of the mouse
/// event.
///
/// Be careful: when a parent widget implements Deref, it might still implement EventBubblingWidget,
/// meaning that calling `.bubble_event` will call the parent's implementation and not the child's
/// one. This is intended behaviour and designed to make code easier to write and read, assuming
/// both knows this behaviour.
///
/// The [AutoPadder](widgets::auto_padder::AutoPadder) widget is a great example:
///
/// ```
/// use std::fmt::Formatter;
/// use terminity_widgets::widgets::auto_padder::AutoPadder;
/// use crossterm::event::{MouseEvent, MouseEventKind, KeyModifiers};
/// use terminity_widgets::{Widget, EventBubblingWidget};
///
/// // Defining a custom widget of size `(3, 3)`
/// // that returns the obtained coordinates on a mouse event
/// struct MyWidget();
/// impl Widget for MyWidget {
/// 	fn size(&self) -> (usize, usize) { (3, 3) }
/// 	//...
/// 	# fn display_line(&self, f: &mut Formatter<'_>, line_nb: usize) -> std::fmt::Result {
/// 		# unimplemented!()
/// 	# }
/// }
/// impl EventBubblingWidget for MyWidget {
/// 	type HandledEvent = (usize, usize);
/// 	// Returns the obtained coordinates
/// 	fn bubble_event(&mut self, event: MouseEvent) -> Self::HandledEvent {
/// 		(event.row as usize, event.column as usize)
/// 	}
/// }
///
/// // Now that the custom widget is defined, let's use it
/// let mut my_widget = AutoPadder(MyWidget(), (5, 5)); // Has a padding of 1 all around
///
/// // Now that the custom widget is defined, let's define custom mouse events.
/// // One on (0, 0)...
/// let event0 = MouseEvent {
/// 	row: 0, column: 0,
/// 	# kind: MouseEventKind::Moved,
/// 	# modifiers: KeyModifiers::NONE,
/// 	// ...
/// };
/// // ... and one on (2, 2).
/// let event1 = MouseEvent {
/// 	row: 2, column: 2,
/// 	# kind: MouseEventKind::Moved,
/// 	# modifiers: KeyModifiers::NONE,
/// 	// ...
/// };
///
/// // (0, 0) is outside of the child, so AutoPadder returns None.
/// assert_eq!(my_widget.bubble_event(event0), None);
///
/// // (2, 2) is inside of the child, AutoPadder bubbles the event by adapting the coordinates.
/// assert_eq!(my_widget.bubble_event(event1), Some((1, 1)));
/// ```
pub trait EventBubblingWidget: Widget {
	type FinalWidgetData<'a>
	where
		Self: 'a;
	/// Handles a mouse event. see the [trait](Self)'s doc for more details.
	fn bubble_event<'a, R, F: FnOnce(Self::FinalWidgetData<'a>, BubblingEvent) -> R>(
		&'a mut self,
		event: BubblingEvent,
		callback: F,
	) -> R;
}
pub use terminity_proc::EventBubblingWidget;

pub struct BubblingEvent {
	pub event: events::Mouse,
	pub current_widget_pos: Position,
}

impl From<events::Mouse> for BubblingEvent {
	fn from(value: events::Mouse) -> Self {
		Self { event: value, current_widget_pos: Position { line: 0, column: 0 } }
	}
}

impl BubblingEvent {
	pub fn pos(&self) -> Position {
		self.event.position - self.current_widget_pos
	}
	pub fn absolute_pos(&self) -> Position {
		self.event.position
	}
	pub fn bubble_at(mut self, relative_pos: Position) -> Self {
		self.current_widget_pos += relative_pos;
		self
	}
}

/// A widget that supports resizing.
///
/// If the context needs the current widget to be resized, then it might need it to
/// implement this trait. Note that for the time being, a widget may be able to change size
/// independently from this trait. However, it is assumed to be the closest possible to any size
/// specified by `resize`
///
/// This is subject to upgrades.
pub trait ResizableWisget {
	/// Resizes the current widget to the given size.
	///
	/// If the widget can't take the given size, it will take the size the closest possible to the
	/// aimed size. This is subject to change.
	fn resize(&mut self, size: Size);
}

#[derive(Debug, Clone)]
pub struct WidgetString {
	content: Vec<u8>,
}

impl From<&WidgetStr> for WidgetString {
	fn from(value: &WidgetStr) -> Self {
		Self { content: value.content.to_vec() }
	}
}

impl Deref for WidgetString {
	type Target = WidgetStr;
	fn deref(&self) -> &Self::Target {
		unsafe { WidgetStr::from_raw(&self.content) }
	}
}

#[derive(Debug)]
pub struct WidgetStr {
	content: [u8],
}

struct LineInfo {
	pos: u16,
	width: u16,
}

impl LineInfo {
	fn from_bytes(bytes: &[u8]) -> Self {
		Self {
			pos: u16::from_le_bytes([bytes[0], bytes[1]]),
			width: u16::from_le_bytes([bytes[2], bytes[3]]),
		}
	}
}

impl WidgetStr {
	pub fn height(&self) -> u16 {
		if self.content.is_empty() {
			0
		} else {
			u16::from_le_bytes([self.content[0], self.content[1]])
		}
	}

	fn line_info(&self, line: u16) -> LineInfo {
		if self.content.is_empty() || line > self.height() {
			return LineInfo { pos: 0, width: 0 };
		}
		let len = self.content.len();
		let pos = len - line as usize * size_of::<LineInfo>();
		if line == self.height() {
			let line_pos = [self.content[pos - 2], self.content[pos - 1]];
			return LineInfo { pos: u16::from_le_bytes(line_pos), width: 0 };
		}
		LineInfo::from_bytes(&self.content[pos - (size_of::<LineInfo>())..pos])
	}

	pub fn line_width(&self, line: u16) -> Option<u16> {
		if self.content.is_empty() || line >= self.height() {
			return None;
		}
		let info = self.line_info(line);
		Some(info.width)
	}

	pub fn line_content(&self, line: u16) -> Option<&str> {
		if self.content.is_empty() || line >= self.height() {
			return None;
		}
		let info = self.line_info(line);
		let info_next = self.line_info(line + 1);
		Some(std::str::from_utf8(&self.content[info.pos as usize..info_next.pos as usize]).unwrap())
	}

	pub const unsafe fn from_raw(content: &[u8]) -> &Self {
		transmute(content)
	}

	pub fn max_width(&self) -> u16 {
		(0..self.height()).map(|l| self.line_width(l).unwrap()).max().unwrap_or(0)
	}
}
