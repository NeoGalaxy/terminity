use std::fmt::Formatter;
use terminity_widgets::Widget;
use terminity_widgets::frame;

struct Img {
	content: Vec<String>,
	size: (usize, usize)
}

impl Widget for Img {
	fn displ_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
		f.write_str(&self.content[line as usize])
	}
	fn size(&self) -> (usize, usize) {
		self.size.clone()
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
	let frame = frame!(
		'H': img1, 'W': img2,
		r"/==================\"
		r"| * HHHHH WWWWWW * |"
		r"| * HHHHH WWWWWW * |"
		r"\==================/"
	);
	let values = vec![
		Img {content: vec!["A".to_owned(), "1".to_owned(), "é".to_owned()], size: (1, 3)},
		Img {content: vec!["F".to_owned(), "2".to_owned(), "é".to_owned()], size: (1, 3)},
		Img {content: vec!["S".to_owned(), "3".to_owned(), "é".to_owned()], size: (1, 3)},
		Img {content: vec!["Q".to_owned(), "4".to_owned(), "é".to_owned()], size: (1, 3)},
		Img {content: vec!["E".to_owned(), "5".to_owned(), "é".to_owned()], size: (1, 3)},
		Img {content: vec!["Z".to_owned(), "6".to_owned(), "é".to_owned()], size: (1, 3)},
		Img {content: vec!["K".to_owned(), "7".to_owned(), "é".to_owned()], size: (1, 3)},
		Img {content: vec!["U".to_owned(), "8".to_owned(), "é".to_owned()], size: (1, 3)},
		Img {content: vec!["O".to_owned(), "9".to_owned(), "é".to_owned()], size: (1, 3)},
	];
	let frame2 = frame!(
		values => {
			'0': [0], '1': [1], '2': [2],
			'3': [3], '4': [4], '5': [5],
			'6': [6], '7': [7], '8': [8]
		},
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
	println!("{}", frame);
	println!("{}", frame2);
	//tokens!(=>)
}