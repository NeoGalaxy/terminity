use crate as terminity_widgets;
use crate::Widget;
use crate::WidgetDisplay;

use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use unicode_segmentation::UnicodeSegmentation;

#[derive(WidgetDisplay)]
pub struct Frame<Idx: ToOwned<Owned = Idx>, Item: Widget, Coll: Index<Idx, Output = Item>> {
	content: Vec<(String, Vec<((Idx, usize), String)>)>,
	widgets: Coll,
	size: (usize, usize),
	positions: HashMap<Idx, (usize, usize)>,
}

impl<Idx: ToOwned<Owned = Idx>, Item: Widget, Coll: Index<Idx, Output = Item>> Widget
	for Frame<Idx, Item, Coll>
{
	fn displ_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
		let (begin, widgets_line) = &self.content[line as usize];
		f.write_str(&begin)?;
		for ((widget_i, w_line), postfix) in widgets_line {
			self.widgets[widget_i.to_owned()].displ_line(f, *w_line)?;
			f.write_str(&postfix)?;
		}
		Ok(())
	}
	fn size(&self) -> (usize, usize) {
		self.size.clone()
	}
}

impl<
		Idx: ToOwned<Owned = Idx> + Eq + Hash + Clone,
		Item: Widget,
		Coll: Index<Idx, Output = Item>,
	> Frame<Idx, Item, Coll>
{
	pub fn new(content: Vec<(String, Vec<((Idx, usize), String)>)>, widgets: Coll) -> Self {
		macro_rules! str_len {
			($str:expr) => {
				String::from_utf8(strip_ansi_escapes::strip($str).unwrap())
					.unwrap()
					.graphemes(true)
					.count()
			};
		}

		let size = (content[0].0.len(), content.len());
		let mut positions = HashMap::new();
		for (y_pos, (prefix, line)) in content.iter().enumerate() {
			let mut x_pos = 0;
			let mut previous = prefix;
			for (item, suffix) in line {
				x_pos += str_len!(previous);
				if item.1 == 0 {
					positions.insert(item.0.clone(), (x_pos, y_pos));
				}
				x_pos += widgets[item.0.to_owned()].size().0;
				previous = suffix;
			}
		}
		Self {
			content,
			widgets,
			size,
			positions,
		}
	}
}

impl<
		Idx: ToOwned<Owned = Idx> + PartialEq + Clone,
		Item: Widget,
		Coll: Index<Idx, Output = Item>,
	> Frame<Idx, Item, Coll>
{
	/// Gives the x coordinate of the first occurence of the element
	/// of index `element_index` in the collection. Panics if the
	/// line is out of the frame. (As a frame has a fixed size, any
	/// access outside of it shouldn't occur)
	pub fn find_x(&self, line: usize, element_index: Idx) -> Option<usize> {
		macro_rules! str_len {
			($str:expr) => {
				String::from_utf8(strip_ansi_escapes::strip($str).unwrap())
					.unwrap()
					.graphemes(true)
					.count()
			};
		}
		self.content[line]
			.1
			.iter()
			.enumerate()
			.find(|(_, (el, _))| el.0 == element_index)
			.map(|(i, _)| {
				self.content[line].1[0..i].iter().fold(
					str_len!(&self.content[line].0),
					|tot, ((widget_id, _), suffix)| {
						tot + self.widgets[(*widget_id).to_owned()].size().0 + str_len!(suffix)
					},
				)
			})
	}
}

impl<Idx: ToOwned<Owned = Idx> + Eq + Hash, Item: Widget, Coll: Index<Idx, Output = Item>>
	Frame<Idx, Item, Coll>
{
	/// Gives the coordinates of the first occurence of the element
	/// of index `element_index` in the collection. Panics if the
	/// line is out of the frame. (As a frame has a fixed size, any
	/// access outside of it shouldn't occur)
	pub fn find_pos(&self, element_index: &Idx) -> Option<(usize, usize)> {
		self.positions.get(element_index).copied()
	}
}

impl<Idx: ToOwned<Owned = Idx>, Item: Widget, Coll: Index<Idx, Output = Item>> Deref
	for Frame<Idx, Item, Coll>
{
	type Target = Coll;
	fn deref(&self) -> &Self::Target {
		&self.widgets
	}
}

impl<Idx: ToOwned<Owned = Idx>, Item: Widget, Coll: Index<Idx, Output = Item>> DerefMut
	for Frame<Idx, Item, Coll>
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.widgets
	}
}

#[cfg(test)]
mod tests {
	use terminity_widgets_proc::frame;

	use super::*;
	struct Img {
		content: Vec<String>,
		size: (usize, usize),
	}

	impl Widget for Img {
		fn displ_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
			f.write_str(&self.content[line as usize])
		}
		fn size(&self) -> (usize, usize) {
			self.size.clone()
		}
	}

	#[test]
	fn new_array() {
		let img1 = Img {
			content: vec!["Hello".to_owned(), "~~~~~".to_owned()],
			size: (5, 2),
		};
		let img2 = Img {
			content: vec!["World!".to_owned(), "~~~~~~".to_owned()],
			size: (6, 2),
		};
		let frame0 = frame!(
			['H': img1, 'W': img2]
			r"/==================\"
			r"| * HHHHH WWWWWW * |"
			r"| * HHHHH WWWWWW * |"
			r"\==================/"
		);

		assert_eq!(
			frame0.to_string(),
			[
				r"/==================\",
				r"| * Hello World! * |",
				r"| * ~~~~~ ~~~~~~ * |",
				r"\==================/",
				""
			]
			.join(&format!(
				"{}\n\r",
				crate::_reexport::Clear(crate::_reexport::UntilNewLine)
			))
		)
	}

	#[test]
	fn extern_collection() {
		let values = vec![
			Img {
				content: vec!["A".to_owned(), "1".to_owned(), "é".to_owned()],
				size: (1, 3),
			},
			Img {
				content: vec!["F".to_owned(), "2".to_owned(), "é".to_owned()],
				size: (1, 3),
			},
			Img {
				content: vec!["S".to_owned(), "3".to_owned(), "é".to_owned()],
				size: (1, 3),
			},
			Img {
				content: vec!["Q".to_owned(), "4".to_owned(), "é".to_owned()],
				size: (1, 3),
			},
			Img {
				content: vec!["E".to_owned(), "5".to_owned(), "é".to_owned()],
				size: (1, 3),
			},
			Img {
				content: vec!["Z".to_owned(), "6".to_owned(), "é".to_owned()],
				size: (1, 3),
			},
			Img {
				content: vec!["K".to_owned(), "7".to_owned(), "é".to_owned()],
				size: (1, 3),
			},
			Img {
				content: vec!["U".to_owned(), "8".to_owned(), "é".to_owned()],
				size: (1, 3),
			},
			Img {
				content: vec!["O".to_owned(), "9".to_owned(), "é".to_owned()],
				size: (1, 3),
			},
		];
		let frame0 = frame!(
		values => {
			'0': 0, '1': 1, '2': 2,
			'3': 3, '4': 4, '5': 5,
			'6': 6, '7': 7, '8': 8
		}
		"#-#-#-#"
		"|0|1|2|"
		"|0|1|2|"
		"|0|1|2|"
		"#-#-#-#"
		"|3|4|5|"
		"|3|4|5|"
		"|3|4|5|"
		"#-#-#-#"
		"|6|7|8|"
		"|6|7|8|"
		"|6|7|8|"
		"#-#-#-#"
		);

		assert_eq!(
			frame0.to_string(),
			[
				"#-#-#-#",
				"|A|F|S|",
				"|1|2|3|",
				"|é|é|é|",
				"#-#-#-#",
				"|Q|E|Z|",
				"|4|5|6|",
				"|é|é|é|",
				"#-#-#-#",
				"|K|U|O|",
				"|7|8|9|",
				"|é|é|é|",
				"#-#-#-#",
				"",
			]
			.join(&format!(
				"{}\n\r",
				crate::_reexport::Clear(crate::_reexport::UntilNewLine)
			))
		)
	}
}
