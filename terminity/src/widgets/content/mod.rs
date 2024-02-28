use std::ops::{Deref, DerefMut};

use super::{
	positionning::{Position, Spacing},
	EventBubblingWidget, Widget,
};
use crate::{
	widget_string::{line::WidgetLine, WidgetStr, WidgetString},
	Size,
};

#[derive(Debug, Clone)]
pub struct TextArea {
	content: WidgetString,
	horizontal_alignment: Position,
	size: Option<Size>,
	content_size: Size,
}

pub struct TextAreaContentGuard<'a> {
	content: &'a mut WidgetString,
	size: &'a mut Size,
}

impl Drop for TextAreaContentGuard<'_> {
	fn drop(&mut self) {
		*self.size = Size { width: self.content.max_width(), height: self.content.height() }
	}
}

impl TextArea {
	pub fn center<S: Into<WidgetString>>(text: S) -> Self {
		let text = text.into();
		let content_size = Size { width: text.max_width(), height: text.height() };
		Self { content: text, horizontal_alignment: Position::Center, size: None, content_size }
	}

	pub fn left<S: Into<WidgetString>>(text: S) -> Self {
		let text = text.into();
		let content_size = Size { width: text.max_width(), height: text.height() };
		Self { content: text, horizontal_alignment: Position::Start, size: None, content_size }
	}

	pub fn right<S: Into<WidgetString>>(text: S) -> Self {
		let text = text.into();
		let content_size = Size { width: text.max_width(), height: text.height() };
		Self { content: text, horizontal_alignment: Position::End, size: None, content_size }
	}

	pub fn with_size(mut self, size: Size) -> Self {
		self.set_size(Some(size));
		self
	}

	pub fn content(&self) -> &WidgetString {
		&self.content
	}

	pub fn content_mut(&mut self) -> TextAreaContentGuard<'_> {
		TextAreaContentGuard { content: &mut self.content, size: &mut self.content_size }
	}

	pub fn set_size(&mut self, size: Option<Size>) {
		self.size = size;
	}
}

// impl Deref for TextArea {
// 	type Target = WidgetString;

// 	fn deref(&self) -> &Self::Target {
// 		&self.content
// 	}
// }

// impl DerefMut for TextArea {
// 	fn deref_mut(&mut self) -> &mut Self::Target {
// 		&mut self.content
// 	}
// }

impl Widget for TextArea {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		let Some(line_details) = self.content.line_details(line) else {
			return Spacing::line(self.size().width).display_line(f, line);
		};

		let padding = self.size().width as i32 - line_details.width() as i32;

		let (l_padding, r_padding) = match self.horizontal_alignment {
			Position::Start => (0, padding),
			Position::Center => (padding / 2, padding - padding / 2),
			Position::End => (padding, 0),
		};

		let (left, center, right) = match (l_padding, r_padding) {
			(..=-1, ..=-1) => (
				Spacing::line(0),
				Some((-l_padding) as u16..(line_details.width() as i32 + r_padding) as u16),
				Spacing::line(0),
			),
			(0.., ..=-1) => (
				Spacing::line(l_padding as u16),
				Some(0..(line_details.width() as i32 + r_padding) as u16),
				Spacing::line(0),
			),
			(..=-1, 0..) => (
				Spacing::line(0),
				Some((-l_padding) as u16..line_details.width()),
				Spacing::line(r_padding as u16),
			),
			(0.., 0..) => (Spacing::line(l_padding as u16), None, Spacing::line(r_padding as u16)),
		};

		// write!(
		// 	f,
		// 	"{l_padding}, {r_padding} => ({}, {center:?}, {})",
		// 	left.size.width, right.size.width
		// )?;

		left.display_line(f, line)?;
		if let Some(range) = center {
			line_details.display_line_in(f, 0, range)?;
		} else {
			line_details.display_line(f, 0)?;
		}
		right.display_line(f, line)?;
		Ok(())
	}

	fn size(&self) -> Size {
		self.size.unwrap_or(self.content_size)
	}

	fn resize(&mut self, size: Size) -> Size {
		self.size = Some(size);
		size
	}
}

impl EventBubblingWidget for TextArea {
	type FinalWidgetData<'a> = &'a Self;

	fn bubble_event<'a, R, F: FnOnce(Self::FinalWidgetData<'a>, super::BubblingEvent) -> R>(
		&'a mut self,
		event: super::BubblingEvent,
		callback: F,
	) -> R {
		callback(self, event)
	}
}

#[derive(Debug, Clone)]
pub struct Img<'a> {
	pub content: WidgetStr<'a>,
	size: Size,
}

impl<'a> Img<'a> {
	pub fn from_wstr_cheked(content: WidgetStr<'a>) -> Option<Self> {
		let width = content.line_details(0).map_or(0, |l| l.width());
		for line in content.lines().skip(1) {
			if line.width() != width {
				return None;
			}
		}
		Some(Img { content, size: Size { width, height: content.height() } })
	}

	pub const unsafe fn from_raw_parts(content: WidgetStr<'a>, size: Size) -> Self {
		Img { content, size }
	}
}

impl Widget for Img<'_> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		f.write_str(&self.content.line_details(line).unwrap())
	}

	fn size(&self) -> Size {
		self.size
	}

	fn resize(&mut self, _: Size) -> Size {
		self.size
	}
}
