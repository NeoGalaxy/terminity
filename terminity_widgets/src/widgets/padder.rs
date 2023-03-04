use crossterm::event::MouseEvent;

use crate as terminity_widgets;
use crate::MouseEventWidget;
use crate::ResizableWisget;
use crate::Widget;
use crate::WidgetDisplay;

use std::fmt::Formatter;
use std::fmt::Write;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(WidgetDisplay)]
pub struct AutoPadder<W: Widget>(pub W, pub (usize, usize));

impl<W: Widget> Widget for AutoPadder<W> {
	fn displ_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
		let content_size = self.0.size();
		let total_size = self.1;
		let top_padding = (total_size.1.saturating_sub(content_size.1)) / 2;
		let left_padding = (total_size.0.saturating_sub(content_size.0)) / 2;
		if line < top_padding || line >= top_padding + content_size.1 {
			for _ in 0..total_size.0 {
				f.write_char(' ')?;
			}
		} else {
			for _ in 0..left_padding {
				f.write_char(' ')?;
			}
			self.0.displ_line(f, line - top_padding)?;
			for _ in (left_padding + content_size.0)..(total_size.0) {
				f.write_char(' ')?;
			}
		}
		Ok(())
	}
	fn size(&self) -> (usize, usize) {
		self.1
	}
}

impl<W: MouseEventWidget> MouseEventWidget for AutoPadder<W> {
	type Res = Option<W::Res>;
	fn mouse_event(&mut self, event: MouseEvent) -> Self::Res {
		let MouseEvent { column, row, kind, modifiers } = event;
		let mut column = column as i32;
		let mut row = row as i32;
		let content_size = self.0.size();
		let total_size = self.1;
		let top_padding = (total_size.1.saturating_sub(content_size.1)) / 2;
		let left_padding = (total_size.0.saturating_sub(content_size.0)) / 2;
		column -= left_padding as i32;
		row -= top_padding as i32;

		if column >= 0
			&& (column as usize) < content_size.0
			&& row >= 0 && (row as usize) < content_size.1
		{
			Some(self.0.mouse_event(MouseEvent {
				kind,
				column: column as u16,
				row: row as u16,
				modifiers,
			}))
		} else {
			None
		}
	}
}

impl<W: Widget> ResizableWisget for AutoPadder<W> {
	fn resize(&mut self, size: (usize, usize)) {
		self.1 = size;
	}
}

impl<W: Widget> Deref for AutoPadder<W> {
	type Target = W;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<W: Widget> DerefMut for AutoPadder<W> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
