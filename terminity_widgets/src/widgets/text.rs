use crate as terminity_widgets;
use crate::Widget;
use crate::WidgetDisplay;
use std::fmt;
use std::fmt::Formatter;
use std::fmt::Write;
use std::ops::Index;
use std::ops::IndexMut;
use unicode_segmentation::UnicodeSegmentation;

pub enum Align {
	Left,
	Right,
	Center,
}

#[derive(WidgetDisplay)]
pub struct Text<const H: usize> {
	pub content: [String; H],
	pub align: Align,
	pub padding: char,
	pub width: usize,
}

impl<const H: usize> Widget for Text<H> {
	fn displ_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
		let width = String::from_utf8(
			strip_ansi_escapes::strip(&self.content[line]).map_err(|_| fmt::Error)?,
		)
		.unwrap()
		.graphemes(true)
		.count();
		let diff = self.width - width;
		let (left, right) = match self.align {
			Align::Left => (0, diff),
			Align::Right => (diff, 0),
			Align::Center => (diff / 2, diff - (diff / 2)),
		};
		for _ in 0..left {
			f.write_char(self.padding)?;
		}
		f.write_str(&self.content[line])?;
		for _ in 0..right {
			f.write_char(self.padding)?;
		}
		Ok(())
	}
	fn size(&self) -> (usize, usize) {
		(self.width, H)
	}
}

impl<const H: usize> Text<H> {
	pub fn clear(&mut self) {
		for s in self.content.iter_mut() {
			s.clear();
		}
	}
}

impl<const H: usize> Index<usize> for Text<H> {
	type Output = String;
	fn index(&self, i: usize) -> &Self::Output {
		&self.content[i]
	}
}
impl<const H: usize> IndexMut<usize> for Text<H> {
	fn index_mut(&mut self, i: usize) -> &mut Self::Output {
		&mut self.content[i]
	}
}

#[cfg(test)]
mod tests {
	//use super::*;

	#[test]
	fn align() {
		unimplemented!();
	}
}
