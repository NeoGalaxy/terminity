pub mod div;

use std::fmt::Write;

use crate::{events::Position, Size};

use super::{AsWidget, EventBubbling, Widget};

#[derive(Debug, Clone, Copy)]
pub enum Positionning {
	Start,
	Center,
	End,
}

#[derive(Debug, Clone, Copy)]
pub struct Clip<W: AsWidget> {
	pub widget: W,
	pub size: Size,
	pub v_pos: Positionning,
	pub h_pos: Positionning,
}

#[derive(Debug, Clone, Copy)]
pub struct ClipWidget<W: Widget> {
	widget: W,
	size: Size,
	top_padding: i16,
	left_padding: i16,
	right_padding: i16,
}

impl<W: AsWidget> AsWidget for Clip<W> {
	type WidgetType<'a> = ClipWidget<W::WidgetType<'a>> where W: 'a;

	fn as_widget(&mut self) -> Self::WidgetType<'_> {
		let widget = self.widget.as_widget();
		let content_height = widget.size().height;

		// Padding may be negative
		let h_padding = self.size.height as i16 - content_height as i16;

		let top_padding = match self.v_pos {
			Positionning::Start => 0,
			Positionning::Center => h_padding / 2,
			Positionning::End => h_padding,
		};

		let w_paddig = self.size.width as i16 - widget.size().width as i16;
		let left_padding = match self.h_pos {
			Positionning::Start => 0,
			Positionning::Center => w_paddig / 2,
			Positionning::End => w_paddig,
		};

		ClipWidget {
			widget,
			size: self.size,
			top_padding,
			left_padding,
			right_padding: w_paddig - left_padding,
		}
	}
}

impl<W: Widget> Widget for ClipWidget<W> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, mut line: u16) -> std::fmt::Result {
		if (line as i16) < self.top_padding {
			return Spacing::line(self.size().width).display_line(f, 0);
		}
		line = (line as i16 - self.top_padding).try_into().unwrap();

		if line >= self.widget.size().height {
			return Spacing::line(self.size().width).display_line(f, 0);
		}

		if self.left_padding + self.right_padding < 0 {
			self.widget.display_line_in(f, line, (-self.left_padding as u16)..self.size.width)?;
		} else {
			Spacing::line(self.left_padding as u16).display_line(f, 0)?;
			self.widget.display_line(f, line)?;
		}
		if self.right_padding > 0 {
			Spacing::line(self.right_padding as u16).display_line(f, 0)?;
		}
		Ok(())
	}

	fn size(&self) -> Size {
		self.size
	}
}

impl<W: EventBubbling + Widget> EventBubbling for ClipWidget<W> {
	type FinalData<'a> = Result<W::FinalData<'a>, &'a mut Self> where W: 'a;

	fn bubble_event<'a, R, F: FnOnce(Self::FinalData<'a>, super::BubblingEvent) -> R>(
		&'a mut self,
		event: super::BubblingEvent,
		callback: F,
	) -> R {
		let w_size = self.widget.size();
		if (self.top_padding..self.top_padding + w_size.height as i16).contains(&event.pos().line)
			&& (self.left_padding..self.left_padding + w_size.width as i16)
				.contains(&event.pos().column)
		{
			self.widget.bubble_event(
				event.bubble_at(Position { line: self.top_padding, column: self.left_padding }),
				|data, evt| callback(Ok(data), evt),
			)
		} else {
			callback(Err(self), event)
		}
	}
}

#[derive(Debug, Clone, Copy, EventBubbling)]
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
