use std::fmt::Formatter;

pub use terminity_widgets_proc::WidgetDisplay;
pub use terminity_widgets_proc::frame;

pub mod widgets;

pub trait Widget {
	fn displ_line(&self, f: &mut Formatter<'_>, line: u16) -> std::fmt::Result;
	fn size(&self) -> &(u16, u16);
}
