use std::fmt::Formatter;
use crate::Widget;
use crate::WidgetDisplay;

#[derive(WidgetDisplay)]
pub struct Frame<Item: Widget> {
	content: Vec<(String, Vec<((usize, u16), String)>)>,
	widgets: Vec<Item>,
	size: (u16, u16)
}

impl<Item: Widget> Widget for Frame<Item> {
	fn displ_line(&self, f: &mut Formatter<'_>, line: u16) -> std::fmt::Result {
		let (begin, widgets_line) = &self.content[line as usize];
		f.write_str(&begin)?;
		for ((widget_i, w_line), postfix) in widgets_line {
			self.widgets[*widget_i].displ_line(f, *w_line)?;
			f.write_str(&postfix)?;
		}
		Ok(())
	}
	fn size(&self) -> &(u16, u16) {
		&self.size
	}
}

impl<Item: Widget> Frame<Item> {
	pub fn new(content: Vec<(String, Vec<((usize, u16), String)>)>, widgets: Vec<Item>) -> Self {
		let size = (0, content.len() as u16);
		Self {
			content,
			widgets,
			size,
		}
	}

	pub fn widgets(&mut self) -> &mut[Item] {
		&mut self.widgets
	}
}
