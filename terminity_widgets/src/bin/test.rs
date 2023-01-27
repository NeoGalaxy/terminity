use terminity_widgets::{widgets::img::Img, Widget};
use terminity_widgets_proc::{frame, tokens};

fn main() {
	let img1 = Img {
		content: vec!["Hello".to_owned(), "~~~~~".to_owned()],
		size: (5, 2)
	};
	let img2 = Img {
		content: vec!["World!".to_owned(), "~~~~~~".to_owned()],
		size: (6, 2)
	};
	let frame: Box<dyn Widget> = Box::new(frame!('H': img1, 'W': img2,
			r##"/==================\"##
			r##"| * HHHHH WWWWWW * |"##
			r##"\==================/"##));
	println!("{}", frame);
	//tokens!(r"=============" "=============")
}