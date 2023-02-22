use std::fmt::Formatter;

pub use terminity_widgets_proc::WidgetDisplay;
pub use terminity_widgets_proc::frame;

pub mod widgets;

pub mod _reexport {
	pub use crossterm::terminal::Clear;
	pub use crossterm::terminal::ClearType::UntilNewLine;
}

pub trait Widget {
	fn displ_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result;
	fn size(&self) -> (usize, usize);
}
