//! Defines the [Text] widget.

use crate::Widget;
use crate::WidgetDisplay;
use std::fmt;
use std::fmt::Formatter;
use std::fmt::Write;
use std::ops::Index;
use std::ops::IndexMut;
use unicode_segmentation::UnicodeSegmentation;

/// Enum used in [`Text`]. Indicates where the text aligns
pub enum Align {
	/// Align text to the left
	Left,
	/// Align text to the right
	Right,
	/// Centers the text, with an extra space on the right if needed.
	Center,
}

/// A [`Widget`] describing a multi-line text. It has a constant number of lines, but that may be
/// subject to change.
///
/// For `Text` to be a well-defined Widget, it needs to know at all times its width and height.
/// Consequently, it has a `width` attribute, and any line will be padded accordingly to their
/// length.
#[derive(WidgetDisplay)]
pub struct Text<const H: usize> {
	/// The lines of text.
	pub content: [String; H],
	/// The width of the text.
	pub width: usize,
	/// How is the text aligned if not long enough.
	pub align: Align,
	/// Which character is used for padding. Defaults to `' '`
	pub padding: char,
}

/// Helper functions to build a [`Text`].
impl<const H: usize> Text<H> {
	/// A left-aligned text with `' '` as padding character
	pub fn new(text: [String; H], width: usize) -> Self {
		Self { content: text, align: Align::Left, padding: ' ', width }
	}
	/// A centered text with `' '` as padding character
	pub fn centered(text: [String; H], width: usize) -> Self {
		Self { content: text, align: Align::Center, padding: ' ', width }
	}
	/// A right-aligned text with `' '` as padding character
	pub fn right_aligned(text: [String; H], width: usize) -> Self {
		Self { content: text, align: Align::Right, padding: ' ', width }
	}
	/// Clears the Text's content.
	pub fn clear(&mut self) {
		for s in self.content.iter_mut() {
			s.clear();
		}
	}
}

impl<const H: usize> Widget for Text<H> {
	fn display_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
		let width = String::from_utf8(
			strip_ansi_escapes::strip(&self.content[line]).map_err(|_| fmt::Error)?,
		)
		.unwrap()
		.graphemes(true)
		.count();
		let diff = self.width.saturating_sub(width);
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
