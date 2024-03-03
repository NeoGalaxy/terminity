use terminity::game::GameContext;
use terminity::img;
use terminity::widgets::content::Img;
use terminity::widgets::positionning::div::Div1;
use terminity::widgets::positionning::Positionning;
use terminity::Size;

#[derive(Debug)]
pub struct OptionsTab {
	size: Size,
}

pub type DisplayWidget<'a> = Div1<Img<'a>>;

impl OptionsTab {
	pub(crate) fn display(&mut self) -> DisplayWidget<'_> {
		Div1::new(img![
			"This view is still a work in progress",
			"                                     ",
			"              Coming soon!           ",
		])
		.with_content_alignment(Positionning::Center)
		.with_content_pos(Positionning::Center)
		.with_exact_size(self.size)
	}

	pub(crate) fn new(size: Size) -> Self {
		Self { size }
	}

	pub(crate) fn update<P: GameContext>(&self, poller: P) {
		for _ in poller.events() {}
	}
}
