#![warn(missing_docs)]

/*! Widgets for easier terminal UI building.

	This crate revolves around the trait [`Widget`], and defines various [widgets] to help building
	your own. It also defines various other traits for more transparent usage of the widgets.

	This crate is currently at a very early development stage. The first changes it might have are
	an api for un-resizeable widgets and more widgets.

*/

use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Range;

pub use terminity_widgets_proc::frame;
pub use terminity_widgets_proc::StructFrame;
pub use terminity_widgets_proc::WidgetDisplay;
use unicode_segmentation::UnicodeSegmentation;

pub mod widgets;

// Re-export for internal use
#[doc(hidden)]
pub mod _reexport {
	pub use crossterm::terminal::Clear;
	pub use crossterm::terminal::ClearType::UntilNewLine;
}

pub struct WidgetLineDisplay<'a, W: Widget + ?Sized> {
	pub widget: &'a W,
	pub line: usize,
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
	fn display_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result;
	/// The current size of the widget, composed of first the width, then the height.
	///
	/// ```
	/// use terminity_widgets::widgets::text::Text;
	/// use terminity_widgets::Widget;
	///
	/// let text = Text::new(["Hello".into(), "World".into()], 5);
	/// assert_eq!(text.size(), (5, 2));
	/// ```
	fn size(&self) -> (usize, usize);

	fn display_line_in(
		&self,
		f: &mut Formatter<'_>,
		line: usize,
		bounds: Range<usize>,
	) -> std::fmt::Result {
		let output = self.get_line_display(line).to_string();
		let res: std::fmt::Result =
			String::from_utf8(strip_ansi_escapes::strip(output).map_err(|_| fmt::Error)?)
				.unwrap()
				.graphemes(true)
				.enumerate()
				.skip_while(|(i, _)| *i < bounds.start)
				.map_while(|(i, s)| if i < bounds.end { Some(f.write_str(s)) } else { None })
				.collect();
		res
	}

	fn get_line_display(&self, line: usize) -> WidgetLineDisplay<'_, Self> {
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
/// Be careful: when a parent widget implements Deref, it might still implement EventHandleingWidget,
/// meaning that calling `.handle_event` will call the parent's implementation and not the child's
/// one. This is intended behaviour and designed to make code easier to write and read, assuming
/// both knows this behaviour.
///
/// The [AutoPadder](widgets::auto_padder::AutoPadder) widget is a great example:
///
/// ```
/// use std::fmt::Formatter;
/// use terminity_widgets::widgets::auto_padder::AutoPadder;
/// use crossterm::event::{MouseEvent, MouseEventKind, KeyModifiers};
/// use terminity_widgets::{Widget, EventHandleingWidget};
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
/// impl EventHandleingWidget for MyWidget {
/// 	type HandledEvent = (usize, usize);
/// 	// Returns the obtained coordinates
/// 	fn handle_event(&mut self, event: MouseEvent) -> Self::HandledEvent {
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
/// assert_eq!(my_widget.handle_event(event0), None);
///
/// // (2, 2) is inside of the child, AutoPadder bubbles the event by adapting the coordinates.
/// assert_eq!(my_widget.handle_event(event1), Some((1, 1)));
/// ```
pub trait EventHandleingWidget: Widget {
	/// The type of the return value of the `handle_event` call.
	type HandledEvent;
	/// Handles a mouse event. see the [trait](Self)'s doc for more details.
	fn handle_event(&mut self, event: crossterm::event::MouseEvent) -> Self::HandledEvent;
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
	fn resize(&mut self, size: (usize, usize));
}
