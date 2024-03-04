use std::ops::{Deref, DerefMut};

use unicode_width::UnicodeWidthChar;

use super::{
	positionning::{Positionning, Spacing},
	AsWidget, EventBubbling, Widget,
};
use crate::{
	widget_string::{LineInfo, WidgetStr, WidgetString},
	Size,
};

#[derive(Debug, Clone)]
pub struct TextArea {
	content: WidgetString,
	horizontal_alignment: Positionning,
	size: Option<Size>,
}

impl TextArea {
	pub fn center<S: Into<WidgetString>>(text: S) -> Self {
		let text = text.into();
		Self { content: text, horizontal_alignment: Positionning::Center, size: None }
	}

	pub fn left<S: Into<WidgetString>>(text: S) -> Self {
		let text = text.into();
		Self { content: text, horizontal_alignment: Positionning::Start, size: None }
	}

	pub fn right<S: Into<WidgetString>>(text: S) -> Self {
		let text = text.into();
		Self { content: text, horizontal_alignment: Positionning::End, size: None }
	}

	pub fn with_size(mut self, size: Size) -> Self {
		self.set_size(Some(size));
		self
	}

	pub fn content(&self) -> &WidgetString {
		&self.content
	}

	pub fn content_mut(&mut self) -> &mut WidgetString {
		&mut self.content
	}

	pub fn set_size(&mut self, size: Option<Size>) {
		self.size = size;
	}
}

pub struct TextAreaWidget<'a> {
	text: WidgetStr<'a>,
	lines: Vec<LineInfo>,
	size: Size,
	horizontal_alignment: Positionning,
}

impl Deref for TextArea {
	type Target = WidgetString;

	fn deref(&self) -> &Self::Target {
		&self.content
	}
}

impl DerefMut for TextArea {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.content
	}
}

impl AsWidget for TextArea {
	type WidgetType<'a> = TextAreaWidget<'a>;

	fn as_widget(&mut self) -> Self::WidgetType<'_> {
		if let Some(size) = self.size {
			let mut lines = Vec::with_capacity(self.content.height() as usize);
			for (line_content, line_info) in self
				.content
				.lines_infos()
				.iter()
				.enumerate()
				.map(|(i, line)| (self.content.line_details(i as u16).unwrap(), line))
			{
				let mut remaining_width = line_info.width;
				let mut next_pos = 0;
				let mut chars = line_content.char_indices();

				while remaining_width > size.width {
					lines.push(LineInfo { pos: line_info.pos + next_pos, width: size.width });
					let mut w = 0;
					for (pos, c) in chars.by_ref() {
						let char_width = c.width().unwrap() as u16;
						if w + char_width > size.width {
							next_pos = pos as u16;
							break;
						} else {
							w += char_width;
						}
					}
					remaining_width -= w;
				}
				lines.push(LineInfo { pos: line_info.pos + next_pos, width: remaining_width });
			}
			TextAreaWidget {
				text: self.content.as_wstr(),
				horizontal_alignment: self.horizontal_alignment,
				lines,
				size,
			}
		} else {
			TextAreaWidget {
				text: self.content.as_wstr(),
				horizontal_alignment: self.horizontal_alignment,
				lines: self.content.lines_infos().to_owned(),
				size: Size { width: self.content.max_width(), height: self.content.height() },
			}
		}
	}
}

impl Widget for TextAreaWidget<'_> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		let text =
			unsafe { WidgetStr::from_content_unchecked(self.text.content_raw(), &self.lines) };

		let Some(line_details) = text.line_details(line) else {
			return Spacing::line(self.size().width).display_line(f, line);
		};

		debug_assert!(self.size().width >= line_details.width());
		let padding = self.size().width - line_details.width();

		let (l_padding, r_padding) = match self.horizontal_alignment {
			Positionning::Start => (0, padding),
			Positionning::Center => (padding / 2, padding - padding / 2),
			Positionning::End => (padding, 0),
		};

		Spacing::line(l_padding).display_line(f, line)?;
		line_details.display_line(f, 0)?;
		Spacing::line(r_padding).display_line(f, line)?;
		Ok(())
	}

	fn size(&self) -> Size {
		self.size
	}
}

impl EventBubbling for TextArea {
	type FinalData<'a> = &'a Self;

	fn bubble_event<'a, R, F: FnOnce(Self::FinalData<'a>, super::BubblingEvent) -> R>(
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

	// fn resize(&mut self, _: Size) -> Size {
	// 	self.size
	// }
}
