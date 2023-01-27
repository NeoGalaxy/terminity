use std::fmt::Formatter;
use terminity_widgets::Widget;
use terminity_widgets::frame;

struct Img {
	content: Vec<String>,
	size: (u16, u16)
}

impl Widget for Img {
	fn displ_line(&self, f: &mut Formatter<'_>, line: u16) -> std::fmt::Result {
		f.write_str(&self.content[line as usize])
	}
	fn size(&self) -> &(u16, u16) {
		&self.size
	}
}

fn main() {
	let img1 = Img {
		content: vec!["Hello".to_owned(), "~~~~~".to_owned()],
		size: (5, 2)
	};
	let img2 = Img {
		content: vec!["World!".to_owned(), "~~~~~~".to_owned()],
		size: (6, 2)
	};
	let frame = Box::new(
		frame!('H': img1, 'W': img2,
			r"/==================\"
			r"| * HHHHH WWWWWW * |"
			r"| * HHHHH WWWWWW * |"
			r"\==================/")
	);
	println!("{}", frame);
	//tokens!(r"=============" "=============")
}