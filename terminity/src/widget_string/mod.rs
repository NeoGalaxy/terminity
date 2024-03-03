use self::line::WidgetLine;
use crate::wchar::WChar;

pub mod line;

#[derive(Debug, Clone, Copy)]
pub struct WidgetStr<'a> {
	content: &'a str,
	lines: &'a [LineInfo],
}

impl<'w> WidgetStr<'w> {
	/// # Safety
	///
	/// The data of the position and width of each line shall be accurate.
	pub const unsafe fn from_content_unchecked(content: &'w str, lines: &'w [LineInfo]) -> Self {
		Self { content, lines }
	}

	pub(crate) fn lines(&self) -> impl Iterator<Item = WidgetLine<'_>> {
		(0..self.height()).map(|i| self.line_details(i).unwrap())
	}

	pub fn content_raw(&self) -> &str {
		self.content
	}
}

// #[derive(Debug, Clone, Copy)]
// pub struct WidgetStrSlice<'a> {
// 	content: &'a str,
// 	lines: &'a [LineInfo],
// 	pos_offset: usize,
// 	width_offset: u16,
// }

#[derive(Debug, Clone)]
pub struct WidgetString {
	content: String,
	lines: Vec<LineInfo>,
}

#[derive(Debug, Clone)]
pub struct LineInfo {
	pub pos: u16,
	pub width: u16,
}

impl From<WidgetStr<'_>> for WidgetString {
	fn from(value: WidgetStr) -> Self {
		Self { content: value.content.into(), lines: value.lines.into() }
	}
}

// impl From<WidgetStrSlice<'_>> for WidgetString {
// 	fn from(value: WidgetStr) -> Self {
// 		Self { content: value.content.into(), lines: value.lines.into() }
// 	}
// }

macro_rules! widget_str {
	($ty:ty) => {
		impl $ty {
			pub fn height(&self) -> u16 {
				self.lines.len() as u16
			}

			pub fn line_details(&self, line: u16) -> Option<WidgetLine<'_>> {
				self.lines.get(line as usize).map(|line_info| {
					if let Some(next) = self.lines.get(line as usize + 1) {
						let end = next.pos;
						WidgetLine {
							width: line_info.width,
							content: &self.content[line_info.pos as usize..end as usize],
						}
					} else {
						WidgetLine {
							width: line_info.width,
							content: self.content.get(line_info.pos as usize..).unwrap_or(""),
						}
					}
				})
			}

			pub fn max_width(&self) -> u16 {
				self.lines.iter().map(|l| l.width).max().unwrap_or(0)
			}

			pub fn lines_infos(&self) -> &[LineInfo] {
				&self.lines
			}
		}
	};
}

widget_str!(WidgetStr<'_>);
widget_str!(WidgetString);

// impl WidgetStrSlice<'_> {
// 	pub fn height(&self) -> u16 {
// 		self.lines.len() as u16
// 	}

// 	pub fn line_details(&self, line: u16) -> Option<WidgetLine> {
// 		self.lines.get(line as usize).map(|line_info| {
// 			let width = line_info.width - if line == 0 { self.width_offset } else { 0 };

// 			let pos = line_info.pos as usize - self.pos_offset;

// 			if let Some(next) = self.lines.get(line as usize + 1) {
// 				let end = next.pos as usize - self.pos_offset;
// 				WidgetLine { width, content: &self.content[pos..end] }
// 			} else {
// 				WidgetLine { width, content: self.content.get(pos..).unwrap_or("") }
// 			}
// 		})
// 	}

// 	pub fn max_width(&self) -> u16 {
// 		let first = self.lines.get(0).map(|l| l.width - self.width_offset).into_iter();
// 		self.lines.iter().skip(1).map(|l| l.width).chain(first).max().unwrap_or(0)
// 	}
// }

impl Default for WidgetString {
	fn default() -> Self {
		Self::new()
	}
}

impl WidgetString {
	pub fn new() -> Self {
		Self { content: "".into(), lines: vec![LineInfo { pos: 0, width: 0 }] }
	}

	pub fn push_char(&mut self, c: WChar) -> &mut Self {
		let line = self.lines.last_mut().unwrap();
		line.width += c.width();
		self.content.push(*c);
		self
	}

	pub fn push_in_line(&mut self, s: WidgetLine<'_>) -> &mut Self {
		let line = self.lines.last_mut().unwrap();
		line.width += s.width();
		self.content.push_str(s.content);
		self
	}

	pub fn push_str(&mut self, s: WidgetStr<'_>) -> &mut Self {
		let str_pos = self.content.len() as u16;
		let line = self.lines.last_mut().unwrap();
		let first = s.lines.first().unwrap();
		line.width += first.width;
		self.content.push_str(s.content);
		if let Some(remaining) = s.lines.get(1..) {
			self.lines.extend(
				remaining.iter().map(|l| LineInfo { pos: l.pos + str_pos, width: l.width }),
			);
		};
		self
	}

	pub fn newline(&mut self) -> &mut Self {
		self.lines.push(LineInfo { pos: self.content.len() as u16, width: 0 });
		self
	}

	pub fn as_wstr(&self) -> WidgetStr<'_> {
		WidgetStr { content: &self.content, lines: &self.lines }
	}
}
