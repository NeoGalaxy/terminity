pub mod div;

use std::fmt::Write;

use crate::Size;

use super::{EventBubblingWidget, Widget};

#[derive(Debug, Clone, Copy)]
pub enum Position {
	Start,
	Center,
	End,
}

#[derive(Debug, Clone, Copy)]
pub struct Clip<W> {
	pub widget: W,
	pub size: Size,
	pub v_pos: Position,
	pub h_pos: Position,
}

impl<W: Widget> Widget for Clip<W> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, mut line: u16) -> std::fmt::Result {
		let content_height = self.widget.size().height;

		// Padding may be negative
		let padding = self.size.height as i16 - content_height as i16;

		let top_pad = match self.v_pos {
			Position::Start => 0,
			Position::Center => padding / 2,
			Position::End => padding,
		};
		if (line as i16) < top_pad {
			return Spacing::line(self.size().width).display_line(f, 0);
		}
		line = (line as i16 - top_pad).try_into().unwrap();

		if line < self.widget.size().height {
			let w_pad = self.size.width as i16 - self.widget.size().width as i16;
			let (l_pad, r_pad) = match self.h_pos {
				Position::Start => (0, w_pad),
				Position::Center => {
					let tmp = w_pad / 2;
					(tmp, w_pad - tmp)
				}
				Position::End => (w_pad, 0),
			};
			if l_pad < 0 {
				self.widget.display_line_in(f, line, (-l_pad as u16)..self.size.width)?;
			} else {
				Spacing::line(l_pad as u16).display_line(f, 0)?;
				self.widget.display_line(f, line)?;
			}
			if r_pad > 0 {
				Spacing::line(r_pad as u16).display_line(f, 0)?;
			}
			return Ok(());
		}

		Spacing::line(self.size().width).display_line(f, 0)?;
		Ok(())
	}

	fn size(&self) -> Size {
		self.size
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Spacing {
	pub size: Size,
	pub c: char,
}

impl Spacing {
	pub fn line(len: u16) -> Self {
		Self { size: Size { width: len, height: 1 }, c: ' ' }
	}

	pub fn with_char(mut self, c: char) -> Self {
		self.c = c;
		self
	}
}

impl Widget for Spacing {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, _: u16) -> std::fmt::Result {
		for _ in 0..self.size.width {
			f.write_char(self.c)?;
		}
		Ok(())
	}

	fn size(&self) -> Size {
		self.size
	}
}

impl EventBubblingWidget for Spacing {
	type FinalWidgetData<'a> = ();

	fn bubble_event<'a, R, F: FnOnce(Self::FinalWidgetData<'a>, super::BubblingEvent) -> R>(
		&'a mut self,
		event: super::BubblingEvent,
		callback: F,
	) -> R {
		callback((), event)
	}
}
