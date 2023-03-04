use std::fmt::Formatter;

pub use terminity_widgets_proc::frame;
pub use terminity_widgets_proc::WidgetDisplay;

pub mod widgets;

pub mod _reexport {
	pub use crossterm::terminal::Clear;
	pub use crossterm::terminal::ClearType::UntilNewLine;
}

/// A Widget is defined here as something able to be printed on a square area.
pub trait Widget {
	/// Prints the given line of the widget.
	fn displ_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result;
	/// The current size of the widget.
	fn size(&self) -> (usize, usize);
}

pub trait MouseEventWidget: Widget {
	type Res;
	#[must_use]
	fn mouse_event(&mut self, event: crossterm::event::MouseEvent) -> Self::Res;
}

pub trait ResizableWisget {
	/// Prints the given line of the widget.
	fn resize(&mut self, size: (usize, usize));
}
