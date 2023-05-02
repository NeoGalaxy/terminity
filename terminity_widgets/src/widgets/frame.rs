//! Defines the [Frame] widget.
use crate as terminity_widgets;
use crate::EventHandleingWidget;
// For the macros
use crate::Widget;
use crossterm::event::MouseEvent;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use std::ops::IndexMut;
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
/// * `Coll`: the type of the collection, who's values when indexed by `Idx` should be Widgets
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
/// let mut framed_texts: Frame<usize, Vec<_>> = frame!(
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
pub struct Frame<Key, Coll> {
	content: Vec<(String, Vec<((Key, usize), String)>)>,
	widgets: Coll,
	size: (usize, usize),
	positions: HashMap<Key, (usize, usize)>,
	_phantom: PhantomData<Key>,
}

impl<Key, Coll> Frame<Key, Coll>
where
	Key: Eq + Hash + Clone,
	Coll: Index<Key>,
	Coll::Output: Widget,
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
	pub fn new(content: Vec<(String, Vec<((Key, usize), String)>)>, widgets: Coll) -> Self {
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
				let item = item.clone();
				x_pos += str_len!(previous);
				if item.1 == 0 {
					positions.insert(item.0.clone(), (x_pos, y_pos));
				}
				x_pos += widgets[item.0.clone()].size().0;
				previous = suffix;
			}
		}
		Self { content, widgets, size, positions, _phantom: PhantomData }
	}
}

impl<Key, Coll> Frame<Key, Coll>
where
	Key: Eq + Hash + Clone,
{
	// TODO: example
	/// Gives the coordinates of the first occurrence of the element
	/// of index `element_index` in the collection.
	pub fn find_pos(&self, element_index: &Key) -> Option<(usize, usize)> {
		self.positions.get(element_index).copied()
	}
}

impl<Key, Coll> Widget for Frame<Key, Coll>
where
	Key: Clone,
	Coll: Index<Key>,
	Coll::Output: Widget,
{
	fn display_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
		let (begin, widgets_line) = &self.content[line as usize];
		f.write_str(&begin)?;
		for ((widget_i, w_line), postfix) in widgets_line {
			self.widgets[widget_i.clone()].display_line(f, *w_line)?;
			f.write_str(&postfix)?;
		}
		Ok(())
	}
	fn size(&self) -> (usize, usize) {
		self.size.clone()
	}
}

impl<Key, Coll> Display for Frame<Key, Coll>
where
	Key: Clone,
	Coll: Index<Key>,
	Coll::Output: Widget,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for i in 0..self.size().1 {
			self.display_line(f, i)?;
			if i != self.size().1 - 1 {
				f.write_str(&format!(
					"{}\n\r",
					terminity_widgets::_reexport::Clear(terminity_widgets::_reexport::UntilNewLine)
				))?;
			}
		}
		Ok(())
	}
}

impl<Key, Coll> EventHandleingWidget for Frame<Key, Coll>
where
	Key: Clone,
	Coll: IndexMut<Key>,
	Coll::Output: EventHandleingWidget,
{
	type HandledEvent = Option<(Key, <Coll::Output as EventHandleingWidget>::HandledEvent)>;
	fn handle_event(&mut self, event: crossterm::event::MouseEvent) -> Self::HandledEvent {
		let MouseEvent { column: column_index, row: row_index, kind, modifiers } = event;
		// TODO: optimize
		let (prefix, row) = &self.content[row_index as usize];
		// TODO: find better way to get length without ansi
		let mut curr_col = String::from_utf8(strip_ansi_escapes::strip(&prefix).unwrap())
			.unwrap()
			.graphemes(true)
			.count();
		for (widget_data, suffix) in row {
			if curr_col > column_index as usize {
				break;
			}
			let widget = &mut self.widgets[widget_data.0.clone()];
			if curr_col + widget.size().0 > column_index as usize {
				return Some((
					widget_data.0.clone(),
					widget.handle_event(MouseEvent {
						column: column_index - curr_col as u16,
						row: widget_data.1 as u16,
						kind,
						modifiers,
					}),
				));
			}
			curr_col += widget.size().0
				+ String::from_utf8(strip_ansi_escapes::strip(&suffix).unwrap())
					.unwrap()
					.graphemes(true)
					.count();
		}
		None
	}
}

impl<Key, Coll> Deref for Frame<Key, Coll> {
	type Target = Coll;
	fn deref(&self) -> &Self::Target {
		&self.widgets
	}
}

impl<Key, Coll> DerefMut for Frame<Key, Coll> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.widgets
	}
}

#[cfg(test)]
mod tests {
	use crossterm::event::KeyModifiers;
	use terminity_widgets_proc::{frame, StructFrame};

	use super::*;
	struct Img {
		content: Vec<String>,
		size: (usize, usize),
		event_res: u64,
	}

	impl Widget for Img {
		fn display_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
			f.write_str(&self.content[line as usize])
		}
		fn size(&self) -> (usize, usize) {
			self.size.clone()
		}
	}

	impl EventHandleingWidget for Img {
		type HandledEvent = u64;
		fn handle_event(&mut self, _event: crossterm::event::MouseEvent) -> Self::HandledEvent {
			self.event_res
		}
	}

	/*impl Trait for Type {
		// add code here
	}*/

	#[test]
	fn new_array() {
		let img1 = Img {
			content: vec!["Hello ".to_owned(), "~~~~~ ".to_owned()],
			size: (6, 2),
			event_res: 0,
		};
		let img2 = Img {
			content: vec!["World!".to_owned(), "~~~~~~".to_owned()],
			size: (6, 2),
			event_res: 0,
		};
		let frame0 = frame!(
			['H': img1, 'W': img2]
			r"/==================\"
			r"| * HHHHHHWWWWWW * |"
			r"| * HHHHHHWWWWWW * |"
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
			Img {
				content: vec!["A".to_owned(), "1".to_owned(), "é".to_owned()],
				size: (1, 3),
				event_res: 0,
			},
			Img {
				content: vec!["F".to_owned(), "2".to_owned(), "é".to_owned()],
				size: (1, 3),
				event_res: 0,
			},
			Img {
				content: vec!["S".to_owned(), "3".to_owned(), "é".to_owned()],
				size: (1, 3),
				event_res: 0,
			},
			Img {
				content: vec!["Q".to_owned(), "4".to_owned(), "é".to_owned()],
				size: (1, 3),
				event_res: 0,
			},
			Img {
				content: vec!["E".to_owned(), "5".to_owned(), "é".to_owned()],
				size: (1, 3),
				event_res: 0,
			},
			Img {
				content: vec!["Z".to_owned(), "6".to_owned(), "é".to_owned()],
				size: (1, 3),
				event_res: 0,
			},
			Img {
				content: vec!["K".to_owned(), "7".to_owned(), "é".to_owned()],
				size: (1, 3),
				event_res: 0,
			},
			Img {
				content: vec!["U".to_owned(), "8".to_owned(), "é".to_owned()],
				size: (1, 3),
				event_res: 0,
			},
			Img {
				content: vec!["O".to_owned(), "9".to_owned(), "é".to_owned()],
				size: (1, 3),
				event_res: 0,
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
			]
			.join(&format!("{}\n\r", crate::_reexport::Clear(crate::_reexport::UntilNewLine)))
		)
	}

	#[test]
	fn ref_keys_hashmap_frame() {
		let values = HashMap::from([
			(
				"Foo".to_string(),
				Img {
					content: vec!["Foo".to_owned(), "Foo".to_owned()],
					size: (3, 2),
					event_res: 0,
				},
			),
			(
				"Bar".to_string(),
				Img {
					content: vec!["Bar".to_owned(), "Bar".to_owned()],
					size: (3, 2),
					event_res: 0,
				},
			),
			(
				"Foo2".to_string(),
				Img {
					content: vec!["Foo".to_owned(), "two".to_owned()],
					size: (3, 2),
					event_res: 0,
				},
			),
			(
				"Bar2".to_string(),
				Img {
					content: vec!["Bar".to_owned(), "two".to_owned()],
					size: (3, 2),
					event_res: 0,
				},
			),
		]);
		//let x = "aaa".to_string();
		//let y = x.into_maybe_ref();
		let names = ["Foo".to_string(), "Bar".to_string(), "Foo2".to_string(), "Bar2".to_string()];
		let frame0 = frame!(
		values => {
			'f': &names[0],
			'b': &names[1],
			'F': &names[2],
			'B': &names[3],
		}
		"#---#---#"
		"|fff|bbb|"
		"|fff|bbb|"
		"#---#---#"
		"|FFF|BBB|"
		"|FFF|BBB|"
		"#---#---#"
		);

		assert_eq!(
			frame0.to_string(),
			[
				"#---#---#",
				"|Foo|Bar|",
				"|Foo|Bar|",
				"#---#---#",
				"|Foo|Bar|",
				"|two|two|",
				"#---#---#",
			]
			.join(&format!("{}\n\r", crate::_reexport::Clear(crate::_reexport::UntilNewLine)))
		)
	}

	#[derive(StructFrame)]
	#[sf_impl(EventHandleingWidget)]
	#[sf_layout {
		"*-------------*",
		"| HHHHHHHHHHH |",
		"|   ccccccc   |",
		"| l ccccccc r |",
		"|   ccccccc   |",
		"| FFFFFFFFFFF |",
		"*-------------*",
	}]
	struct MyFrame {
		#[sf_layout(name = 'c')]
		content: Img,
		#[sf_layout(name = 'H')]
		header: Img,
		#[sf_layout(name = 'l')]
		left: Img,
		#[sf_layout(name = 'r')]
		right: Img,
		#[sf_layout(name = 'F')]
		footer: Img,
	}

	#[test]
	fn struct_frame() {
		let mut s_frame = MyFrame {
			content: Img { content: vec!["1234567".into(); 3], size: (7, 3), event_res: 1 },
			header: Img { content: vec!["abcdefghijk".into()], size: (11, 1), event_res: 2 },
			left: Img { content: vec!["A".into()], size: (1, 1), event_res: 3 },
			right: Img { content: vec!["B".into()], size: (1, 1), event_res: 4 },
			footer: Img { content: vec!["lmnopqrstuv".into()], size: (11, 1), event_res: 5 },
		};

		assert_eq!("*-------------*", &s_frame.get_line_display(0).to_string());
		assert_eq!("| abcdefghijk |", &s_frame.get_line_display(1).to_string());
		assert_eq!("|   1234567   |", &s_frame.get_line_display(2).to_string());
		assert_eq!("| A 1234567 B |", &s_frame.get_line_display(3).to_string());
		assert_eq!("|   1234567   |", &s_frame.get_line_display(4).to_string());
		assert_eq!("| lmnopqrstuv |", &s_frame.get_line_display(5).to_string());
		assert_eq!("*-------------*", &s_frame.get_line_display(6).to_string());

		assert_eq!(
			s_frame.handle_event(MouseEvent {
				kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
				column: 0,
				row: 0,
				modifiers: KeyModifiers::empty(),
			}),
			None
		);
		assert_eq!(
			s_frame.handle_event(MouseEvent {
				kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
				column: 2,
				row: 1,
				modifiers: KeyModifiers::empty(),
			}),
			Some(MyFrameMouseEvents::Header(2))
		);
		assert_eq!(
			s_frame.handle_event(MouseEvent {
				kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
				column: 4,
				row: 2,
				modifiers: KeyModifiers::empty(),
			}),
			Some(MyFrameMouseEvents::Content(1))
		);
	}

	#[derive(StructFrame)]
	#[sf_impl(EventHandleingWidget)]
	#[sf_layout {
		"*-------------*",
		"| HHHHHHHHHHH |",
		"|   ccccccc   |",
		"| l ccccccc r |",
		"|   ccccccc   |",
		"| FFFFFFFFFFF |",
		"*-------------*",
	}]
	struct MyTupleFrame(
		#[sf_layout(name = 'c')] Img,
		#[sf_layout(name = 'H')] Img,
		#[sf_layout(name = 'l')] Img,
		#[sf_layout(name = 'r')] Img,
		#[sf_layout(name = 'F')] Img,
	);

	#[test]
	fn tuple_frame() {
		let mut s_frame = MyTupleFrame(
			Img { content: vec!["1234567".into(); 3], size: (7, 3), event_res: 1 },
			Img { content: vec!["abcdefghijk".into()], size: (11, 1), event_res: 2 },
			Img { content: vec!["A".into()], size: (1, 1), event_res: 3 },
			Img { content: vec!["B".into()], size: (1, 1), event_res: 4 },
			Img { content: vec!["lmnopqrstuv".into()], size: (11, 1), event_res: 5 },
		);

		assert_eq!("*-------------*", &s_frame.get_line_display(0).to_string());
		assert_eq!("| abcdefghijk |", &s_frame.get_line_display(1).to_string());
		assert_eq!("|   1234567   |", &s_frame.get_line_display(2).to_string());
		assert_eq!("| A 1234567 B |", &s_frame.get_line_display(3).to_string());
		assert_eq!("|   1234567   |", &s_frame.get_line_display(4).to_string());
		assert_eq!("| lmnopqrstuv |", &s_frame.get_line_display(5).to_string());
		assert_eq!("*-------------*", &s_frame.get_line_display(6).to_string());

		assert_eq!(
			s_frame.handle_event(MouseEvent {
				kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
				column: 0,
				row: 0,
				modifiers: KeyModifiers::empty(),
			}),
			None
		);
		assert_eq!(
			s_frame.handle_event(MouseEvent {
				kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
				column: 2,
				row: 1,
				modifiers: KeyModifiers::empty(),
			}),
			Some(MyTupleFrameMouseEvents::_1(2))
		);
		assert_eq!(
			s_frame.handle_event(MouseEvent {
				kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
				column: 4,
				row: 2,
				modifiers: KeyModifiers::empty(),
			}),
			Some(MyTupleFrameMouseEvents::_0(1))
		);
	}
}
