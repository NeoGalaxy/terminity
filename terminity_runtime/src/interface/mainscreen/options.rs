use terminity::img;
use terminity::widgets::content::Img;
use terminity::widgets::positionning::div::Div1;
use terminity::widgets::positionning::Position;

use terminity::widgets::Widget;
use terminity::Size;

#[derive(Debug)]
pub struct OptionsTab {}

const CONTENT: Img = img![
	"This view is still a work in progress",
	"                                     ",
	"              Coming soon!           ",
];

impl OptionsTab {
	pub fn display_line(
		&self,
		f: &mut std::fmt::Formatter<'_>,
		line: u16,
		size: Size,
	) -> std::result::Result<(), std::fmt::Error> {
		Div1::new(false, CONTENT)
			.with_content_alignment(Position::Center)
			.with_content_pos(Position::Center)
			.with_forced_size(size)
			.display_line(f, line)
	}

	pub(crate) fn new() -> Self {
		Self {}
	}

	pub(crate) fn update<P: terminity::events::EventPoller>(&self, poller: P) {
		for _ in poller.events() {}
	}
}
