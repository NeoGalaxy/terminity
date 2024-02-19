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
use std::ops::DerefMut;
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

impl<'a, W: Widget> Widget for &'a W {
	fn display_line(&self, f: &mut Formatter<'_>, line: u16) -> std::fmt::Result {
		(*self).display_line(f, line)
	}

	fn size(&self) -> Size {
		(*self).size()
	}

	fn display_line_in(
		&self,
		f: &mut Formatter<'_>,
		line: u16,
		bounds: Range<u16>,
	) -> std::fmt::Result {
		(*self).display_line_in(f, line, bounds)
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

impl DerefMut for WidgetString {
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe { WidgetStr::from_raw_mut(&mut self.content) }
	}
}

impl WidgetString {
	pub fn push(&mut self, s: &WidgetStr) {
		if s.content.is_empty() {
			return;
		}
		if self.content.is_empty() {
			*self = s.into();
			return;
		}
		let additionnal_size =
			(s.height() - 1) as usize * size_of::<LineInfo>() + s.tot_len() as usize;
		let old_buffer_size = self.content.len();

		if additionnal_size + self.used_size() as usize > old_buffer_size {
			self.content.reserve(additionnal_size + self.used_size() as usize - old_buffer_size);
			for v in self.content.spare_capacity_mut() {
				v.write(0);
			}
			unsafe { self.content.set_len(self.content.capacity()) }

			if old_buffer_size == 0 {
				// The content is fully set to 0: we need to initialize it
				self.clear();
			} else {
				// From the end of the buffer to the beginning. Also copy the entry
				// giving the full size
				for i in 0..=self.height() as usize {
					// assume the old buffer as valid
					let info = unsafe {
						WidgetStr::from_raw(&self.content[0..old_buffer_size]).line_info(i as u16)
					};
					// update on the new buffer
					self.set_line_info(i as u16, &info);
				}
			}
		}

		let mut tot_len = self.tot_len() as usize;
		let mut curr_line = self.height() - 1;
		let mut curr_line_width = self.line_width(curr_line).unwrap();
		{
			let tmp = self.height() + s.height() - 1;
			self.set_height(tmp);
		}

		// do NOT use self.line_width in this loop, it is broken because there's data missing
		for line_i in 0..s.height() {
			let line = s.line_content(line_i).unwrap();
			self.content[tot_len..tot_len + line.len()].copy_from_slice(line.as_bytes());
			tot_len += line.len();
			let mut info = self.line_info(curr_line);
			info.width += s.line_width(line_i).unwrap();
			self.set_line_info(curr_line, &info);
			self.set_line_info(
				curr_line + 1,
				&LineInfo { pos: info.pos + curr_line_width + line.len() as u16, width: 0 },
			);
			curr_line += 1;
			curr_line_width = 0;
		}

		debug_assert_eq!(curr_line, self.height());
	}

	pub fn used_size(&self) -> u16 {
		self.height() * size_of::<LineInfo>() as u16 + self.tot_len() + 1
	}

	pub fn clear(&mut self) {
		self.set_height(1);
		self.set_line_info(0, &LineInfo { pos: 2, width: 0 });
		self.set_line_info(1, &LineInfo { pos: 2, width: 0 });
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

	fn to_bytes(&self) -> [u8; 4] {
		let pos = self.pos.to_le_bytes();
		let width = self.width.to_le_bytes();
		[pos[0], pos[1], width[0], width[1]]
	}
}

impl WidgetStr {
	pub fn height(&self) -> u16 {
		if self.content.is_empty() {
			1
		} else {
			u16::from_le_bytes([self.content[0], self.content[1]])
		}
	}

	fn set_height(&mut self, h: u16) {
		if !self.content.is_empty() {
			self.content[0..2].copy_from_slice(&h.to_le_bytes());
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

	fn set_line_info(&mut self, line: u16, info: &LineInfo) {
		if self.content.is_empty() || line > self.height() {
			return;
		}
		let len = self.content.len();
		let pos = len - line as usize * size_of::<LineInfo>();
		if line == self.height() {
			let [pos0, pos1, ..] = info.to_bytes();
			self.content[pos - 2] = pos0;
			self.content[pos - 1] = pos1;
		} else {
			self.content[pos - (size_of::<LineInfo>())..pos].copy_from_slice(&info.to_bytes())
		}
	}

	pub fn tot_len(&self) -> u16 {
		self.line_info(self.height()).pos
	}

	pub fn line_width(&self, line: u16) -> Option<u16> {
		if line >= self.height() {
			return None;
		}
		let info = self.line_info(line);
		Some(info.width)
	}

	pub fn line_content(&self, line: u16) -> Option<&str> {
		if line >= self.height() {
			return None;
		}
		if self.content.is_empty() {
			return Some("");
		}
		let info = self.line_info(line);
		let info_next = self.line_info(line + 1);
		Some(std::str::from_utf8(&self.content[info.pos as usize..info_next.pos as usize]).unwrap())
	}

	pub const unsafe fn from_raw(content: &[u8]) -> &Self {
		transmute(content)
	}

	pub unsafe fn from_raw_mut(content: &mut [u8]) -> &mut Self {
		transmute(content)
	}

	pub fn max_width(&self) -> u16 {
		(0..self.height()).map(|l| self.line_width(l).unwrap()).max().unwrap_or(0)
	}
}

#[cfg(test)]
mod tests {
	use terminity_proc::wstr;

	use super::*;

	#[test]
	fn wstr_push() {
		let mut s = WidgetString::from(wstr!("Hello I'm a\nwstr on multiple\nlines."));
		assert_eq!(s.height(), 3);
		assert_eq!(s.line_content(0), Some("Hello I'm a"));
		assert_eq!(s.line_content(1), Some("wstr on multiple"));
		assert_eq!(s.line_content(2), Some("lines."));

		s.push(wstr!(" And now we have\nmore lines.\n"));
		assert_eq!(s.height(), 5);
		assert_eq!(s.line_content(0), Some("Hello I'm a"));
		assert_eq!(s.line_content(1), Some("wstr on multiple"));
		assert_eq!(s.line_content(2), Some("lines. And now we have"));
		assert_eq!(s.line_content(3), Some("more lines."));
		assert_eq!(s.line_content(4), Some(""));

		s.push(wstr!("And even\nmore"));
		assert_eq!(s.height(), 6);
		assert_eq!(s.line_content(4), Some("And even"));
		assert_eq!(s.line_content(5), Some("more"));

		s.push(wstr!(""));
		assert_eq!(s.height(), 6);
		assert_eq!(s.line_content(5), Some("more"));
	}

	#[test]
	fn wstr_push_empty() {
		let mut s = WidgetString::from(wstr!(""));
		assert_eq!(s.content.len(), 0);
		assert_eq!(s.height(), 1);

		s.push(wstr!("Hello\nworld!"));
		assert_eq!(s.height(), 2);
		assert_eq!(s.line_content(0), Some("Hello"));
		assert_eq!(s.line_content(1), Some("world!"));
	}
}
