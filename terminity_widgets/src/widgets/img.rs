use std::fmt::Formatter;
use crate::Widget;

pub struct Img {
	pub content: Vec<String>,
	pub size: (u16, u16)
}

impl Widget for Img {
	fn displ_line(&self, f: &mut Formatter<'_>, line: u16) -> std::fmt::Result {
		f.write_str(&self.content[line as usize])
	}
	fn size(&self) -> &(u16, u16) {
		&self.size
	}
}
