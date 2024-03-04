//! Widgets for easier terminal UI building.
//!
//! This crate revolves around the trait [`Widget`], and defines various [widgets] to help building
//! your own. It also defines various other traits for more transparent usage of the widgets.
//!
//! This crate is currently at a very early development stage. The first changes it might have are
//! an api for un-resizeable widgets and more widgets.

use std::collections::btree_map;
use std::collections::hash_map;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Write;
use std::iter::Enumerate;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::RangeBounds;
use std::slice;

// use unicode_segmentation::UnicodeSegmentation;

use crate::events;
use crate::events::Position;
use crate::Size;

pub use terminity_proc::Widget;
use unicode_width::UnicodeWidthChar;

pub mod content;
pub mod positionning;

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

	fn display_line_in<R: RangeBounds<u16>>(
		&self,
		f: &mut Formatter<'_>,
		line: u16,
		bounds: R,
	) -> std::fmt::Result {
		let output = self.get_line_display(line).to_string();
		let mut chars = output.chars();

		let min = match bounds.start_bound() {
			std::ops::Bound::Included(i) => *i,
			std::ops::Bound::Excluded(e) => e + 1,
			std::ops::Bound::Unbounded => 0,
		};
		let mut w = 0;
		while w < min {
			let Some(c) = chars.next() else {
				return Ok(());
			};

			w += c.width().unwrap_or(0) as u16;
		}
		while bounds.contains(&w) {
			let Some(c) = chars.next() else {
				return Ok(());
			};

			w += c.width().unwrap_or(0) as u16;
			f.write_char(c)?;
		}
		Ok(())
		// let res: std::fmt::Result =
		// 	String::from_utf8(strip_ansi_escapes::strip(output).map_err(|_| fmt::Error)?)
		// 		.unwrap()
		// 		.graphemes(true)
		// 		.enumerate()
		// 		.skip_while(|(i, _)| *i < bounds.start as usize)
		// 		.map_while(
		// 			|(i, s)| if i < bounds.end as usize { Some(f.write_str(s)) } else { None },
		// 		)
		// 		.collect();
		// res
	}

	fn get_line_display(&self, line: u16) -> WidgetLineDisplay<'_, Self> {
		WidgetLineDisplay { widget: self, line }
	}
}

pub trait AsWidget {
	type WidgetType<'a>: Widget
	where
		Self: 'a;

	fn as_widget(&mut self) -> Self::WidgetType<'_>;
}

impl<W: Widget> AsWidget for W {
	type WidgetType<'a> = &'a mut Self where Self: 'a;

	fn as_widget(&mut self) -> Self::WidgetType<'_> {
		self
	}
}

impl<W: Widget> Widget for &mut W {
	fn display_line(&self, f: &mut Formatter<'_>, line: u16) -> std::fmt::Result {
		self.deref().display_line(f, line)
	}

	fn size(&self) -> Size {
		self.deref().size()
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
///		let lines_start = self.lines.iter().position(|l| l.pos > start).unwrap_or(self.lines.len()) - 1;

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
pub trait EventBubbling {
	type FinalData<'a>
	where
		Self: 'a;
	/// Handles a mouse event. see the [trait](Self)'s doc for more details.
	fn bubble_event<'a, R, F: FnOnce(Self::FinalData<'a>, BubblingEvent) -> R>(
		&'a mut self,
		event: BubblingEvent,
		callback: F,
	) -> R;
}
pub use terminity_proc::EventBubbling;

impl<T: EventBubbling> EventBubbling for &mut T {
	type FinalData<'a> = T::FinalData<'a> where Self: 'a;

	fn bubble_event<'a, R, F: FnOnce(Self::FinalData<'a>, BubblingEvent) -> R>(
		&'a mut self,
		event: BubblingEvent,
		callback: F,
	) -> R {
		self.deref_mut().bubble_event(event, callback)
	}
}

pub struct BubblingEvent {
	pub event: events::Mouse,
	pub current_widget_pos: Position,
}

impl From<events::Mouse> for BubblingEvent {
	fn from(value: events::Mouse) -> Self {
		Self { event: value, current_widget_pos: Position { line: 0, column: 0 } }
	}
}

impl From<BubblingEvent> for events::Mouse {
	fn from(value: BubblingEvent) -> Self {
		let mut event = value.event;
		event.position -= value.current_widget_pos;
		event
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

pub trait AsIndexedIterator {
	type Index<'a>
	where
		Self: 'a;
	type Value;
	type Iter<'a>: Iterator<Item = (Self::Index<'a>, &'a mut Self::Value)>
	where
		Self: 'a;
	fn as_iterator(&mut self) -> Self::Iter<'_>;
}

impl<T> AsIndexedIterator for [T] {
	type Index<'a> = usize where T: 'a;
	type Value = T;

	type Iter<'a> = Enumerate<slice::IterMut<'a, T>> where T: 'a;

	fn as_iterator(&mut self) -> Self::Iter<'_> {
		self.iter_mut().enumerate()
	}
}

impl<T> AsIndexedIterator for Vec<T> {
	type Index<'a> = usize where T: 'a;
	type Value = T;

	type Iter<'a> = Enumerate<slice::IterMut<'a, T>> where T: 'a;

	fn as_iterator(&mut self) -> Self::Iter<'_> {
		self.iter_mut().enumerate()
	}
}

impl<K, V, S> AsIndexedIterator for HashMap<K, V, S> {
	type Index<'a> = &'a K where Self: 'a;
	type Value = V;

	type Iter<'a> = hash_map::IterMut<'a, K, V> where Self: 'a;

	fn as_iterator(&mut self) -> Self::Iter<'_> {
		self.iter_mut()
	}
}

impl<K, V> AsIndexedIterator for BTreeMap<K, V> {
	type Index<'a> = &'a K where Self: 'a;
	type Value = V;

	type Iter<'a> = btree_map::IterMut<'a, K, V> where Self: 'a;

	fn as_iterator(&mut self) -> Self::Iter<'_> {
		self.iter_mut()
	}
}

/*#[cfg(test)]
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
*/
