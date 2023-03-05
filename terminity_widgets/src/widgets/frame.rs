//! Defines the [Frame] widget.
use crate as terminity_widgets; // For the macros
use crate::Widget;
use crate::WidgetDisplay;

use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use unicode_segmentation::UnicodeSegmentation;

/// A Frame[^coll] is a widget containing a collection of widgets that it is able to display.
///
/// The contained collection of widgets can be of any type, as long as it is indexable and that the
/// indexes to access the widgets that the frame needs to be displayed are given. The contained
/// widgets' size are assumed to never change, and each line is assumed to be of the same length.
///
/// Once a collection is framed, it can actually still be used as the original collection since
/// frames implements [`Deref`] and [`DerefMut`].
///
/// The generics arguments of Frame are:
/// * `Idx`: the type of the indexes to access the collection's content
/// * `Item`: the type of the children widgets
/// * `Coll`: the type of the wrapped collection
///
/// Building a frame itself might not seem straightforward, so the [frame macro](crate::frame) is
/// given to help building it. Check it's documentation for more details.
///
/// ```
/// use terminity_widgets::frame;
/// use terminity_widgets::widgets::text::Text;
/// use terminity_widgets::widgets::frame::Frame;
/// let texts = vec![
/// 	Text::new(["Hello".into(), "-----".into()], 5),
/// 	Text::new(["World".into(), "".into()], 5),
/// ];
///
/// // Generics not needed here, but written for an example of how they work
/// let mut framed_texts: Frame<usize, Text<2>, Vec<_>> = frame!(
/// 	texts => { 'H': 0, 'W': 1 }
/// 	"*~~~~~~~~~~~~~~*"
/// 	"| HHHHH WWWWW! |"
/// 	"| HHHHH-WWWWW- |"
/// 	"*~~~~~~~~~~~~~~*"
/// );
/// framed_texts[1][1] = String::from("-----");
///
/// println!("{}", framed_texts);
/// ```
///
/// [^coll]: "Frame" may be referred as "Collection Frame" (but still named `Frame` in code) when
/// "Structure Frames" will be a thing. A structure frame will be implemented through a trait and a
/// macro, allowing more flexibility in the types of the frame's children.
#[derive(WidgetDisplay)]
pub struct Frame<Idx: ToOwned<Owned = Idx>, Item: Widget, Coll: Index<Idx, Output = Item>> {
	content: Vec<(String, Vec<((Idx, usize), String)>)>,
	widgets: Coll,
	size: (usize, usize),
	positions: HashMap<Idx, (usize, usize)>,
}

impl<
		// That's a lot of generics...
		Idx: ToOwned<Owned = Idx> + Eq + Hash + Clone,
		Item: Widget,
		Coll: Index<Idx, Output = Item>,
	> Frame<Idx, Item, Coll>
{
	/// Creates a frame out of the given widgets. Finds the frame's size using the first line.
	///
	/// The content of a line is described as a prefix followed by a (maybe empty) list of tuples
	/// containing data to display the appropriate widget's line and a suffix to this widget's line.
	/// The data data to display the appropriate widget's line is simply the widget's index and the
	/// index of the widget's line to display. For instance, a line of the form `"| aa | bb |"`
	/// where `aa` is the line n°0 of the widget of index `'a'` and `bb` is the line n°1 of the
	/// widget of index `'b'`, the line will be of the form
	/// `("| ", [(('a', 0), " | "), (('b', 1), " |")])`.
	///
	/// If this function seems too complicated to use, consider using the [`frame!`](crate::frame)
	/// macro, that actually just compiles to an assignation and a `Frame::new` invocation.
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
		// TODO: cleanup/adapt. This is code from when I tried to implement un-resizable widgets.
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
		Self { content, widgets, size, positions }
	}
}

/*impl<
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
		self.content[line].1.iter().enumerate().find(|(_, (el, _))| el.0 == element_index).map(
			|(i, _)| {
				self.content[line].1[0..i].iter().fold(
					str_len!(&self.content[line].0),
					|tot, ((widget_id, _), suffix)| {
						tot + self.widgets[(*widget_id).to_owned()].size().0 + str_len!(suffix)
					},
				)
			},
		)
	}
}*/

impl<Idx: ToOwned<Owned = Idx> + Eq + Hash, Item: Widget, Coll: Index<Idx, Output = Item>>
	Frame<Idx, Item, Coll>
{
	// TODO: example
	/// Gives the coordinates of the first occurrence of the element
	/// of index `element_index` in the collection.
	pub fn find_pos(&self, element_index: &Idx) -> Option<(usize, usize)> {
		self.positions.get(element_index).copied()
	}
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
		let img1 = Img { content: vec!["Hello".to_owned(), "~~~~~".to_owned()], size: (5, 2) };
		let img2 = Img { content: vec!["World!".to_owned(), "~~~~~~".to_owned()], size: (6, 2) };
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
			]
			.join(&format!("{}\n\r", crate::_reexport::Clear(crate::_reexport::UntilNewLine)))
		)
	}

	#[test]
	fn extern_collection() {
		let values = vec![
			Img { content: vec!["A".to_owned(), "1".to_owned(), "é".to_owned()], size: (1, 3) },
			Img { content: vec!["F".to_owned(), "2".to_owned(), "é".to_owned()], size: (1, 3) },
			Img { content: vec!["S".to_owned(), "3".to_owned(), "é".to_owned()], size: (1, 3) },
			Img { content: vec!["Q".to_owned(), "4".to_owned(), "é".to_owned()], size: (1, 3) },
			Img { content: vec!["E".to_owned(), "5".to_owned(), "é".to_owned()], size: (1, 3) },
			Img { content: vec!["Z".to_owned(), "6".to_owned(), "é".to_owned()], size: (1, 3) },
			Img { content: vec!["K".to_owned(), "7".to_owned(), "é".to_owned()], size: (1, 3) },
			Img { content: vec!["U".to_owned(), "8".to_owned(), "é".to_owned()], size: (1, 3) },
			Img { content: vec!["O".to_owned(), "9".to_owned(), "é".to_owned()], size: (1, 3) },
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
			]
			.join(&format!("{}\n\r", crate::_reexport::Clear(crate::_reexport::UntilNewLine)))
		)
	}
}
