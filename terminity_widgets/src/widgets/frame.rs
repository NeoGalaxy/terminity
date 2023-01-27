use std::fmt::Formatter;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ops::Index;
use crate::Widget;
use crate::WidgetDisplay;

#[derive(WidgetDisplay)]
pub struct Frame<Idx: ToOwned<Owned = Idx>, Item: Widget, Coll: Index<Idx, Output = Item>> {
	content: Vec<(String, Vec<((Idx, usize), String)>)>,
	widgets: Coll,
	size: (usize, usize)
}

impl<Idx: ToOwned<Owned = Idx>, Item: Widget, Coll: Index<Idx, Output = Item>> Widget for Frame<Idx, Item, Coll> {
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

impl<Idx: ToOwned<Owned = Idx>, Item: Widget, Coll: Index<Idx, Output = Item>> Frame<Idx, Item, Coll> {
	pub fn new(content: Vec<(String, Vec<((Idx, usize), String)>)>, widgets: Coll) -> Self {
		let size = (content[0].0.len(), content.len());
		Self {
			content,
			widgets,
			size,
		}
	}
}

impl<Idx: ToOwned<Owned = Idx>, Item: Widget, Coll: Index<Idx, Output = Item>> Deref for Frame<Idx, Item, Coll> {
	type Target = Coll;
	fn deref(&self) -> &Self::Target {
	    &self.widgets
	}
}

impl<Idx: ToOwned<Owned = Idx>, Item: Widget, Coll: Index<Idx, Output = Item>> DerefMut for Frame<Idx, Item, Coll> {
	fn deref_mut(&mut self) -> &mut Self::Target {
	    &mut self.widgets
	}
}
