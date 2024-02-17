use super::{
	positionning::{Position, Spacing},
	Widget, WidgetStr, WidgetString,
};
use crate::Size;

#[derive(Debug, Clone)]
pub struct TextArea {
	content: WidgetString,
	horizontal_alignment: Position,
	size: Size,
}

impl TextArea {
	pub fn center(text: &WidgetStr) -> Self {
		Self {
			content: text.into(),
			horizontal_alignment: Position::Center,
			size: Size { width: text.max_width(), height: text.height() },
		}
	}
}

impl Widget for TextArea {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		let padding = self.size.width - self.content.line_width(line).unwrap();
		let (l_padding, r_padding) = match self.horizontal_alignment {
			Position::Start => (0, padding),
			Position::Center => (padding / 2, padding - padding / 2),
			Position::End => (padding, 0),
		};
		Spacing::line(l_padding).display_line(f, line)?;
		f.write_str(self.content.line_content(line).unwrap())?;
		Spacing::line(r_padding).display_line(f, line)?;
		Ok(())
	}

	fn size(&self) -> Size {
		self.size
	}
}

#[derive(Debug, Clone)]
pub struct Img {
	pub content: &'static WidgetStr,
	size: Size,
}

impl Img {
	pub const unsafe fn from_raw_parts(content: &'static WidgetStr, size: Size) -> Self {
		Img { content, size }
	}
}

impl Widget for Img {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		f.write_str(self.content.line_content(line).unwrap())
	}

	fn size(&self) -> Size {
		self.size
	}
}
