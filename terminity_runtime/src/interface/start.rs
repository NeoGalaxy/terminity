use terminity::{
	game::GameContext,
	img,
	widgets::{
		content::{Img, TextArea},
		positionning::{
			div::{Div1, Div3},
			Clip, Positionning, Spacing,
		},
		AsWidget, Widget,
	},
	wstr, Size,
};

#[derive(Debug)]
pub enum StartStage {
	Stage0,
	Stage1 { padding: i16 },
}

#[derive(Debug)]
pub struct StartScreen {
	tick: u64,
	pub size: Size,
	stage: StartStage,
	stage0: Div3<Spacing, Img<'static>, TextArea>,
	stage1: Div1<Clip<Img<'static>>>,
}

impl StartScreen {
	pub fn new(size: Size) -> Self {
		let padding = (size.height as i16 - TERMINITY_TEXT.size().height as i16 - 1) / 2;
		Self {
			tick: 0,
			size,
			stage: StartStage::Stage0,
			stage0: Div3::new(
				Spacing { size: Size { width: size.width, height: padding as u16 }, c: ' ' },
				TERMINITY_TEXT,
				TextArea::center(wstr!(r"Press any key to start.")),
			)
			.with_content_alignment(Positionning::Center)
			.with_content_pos(Positionning::Start)
			.with_exact_size(size),
			stage1: Div1::new(Clip {
				widget: TERMINITY_TEXT,
				size,
				v_pos: Positionning::End,
				h_pos: Positionning::Center,
			})
			.with_content_alignment(Positionning::Center)
			.with_content_pos(Positionning::Start)
			.with_exact_size(size),
		}
	}

	pub fn update<E: GameContext>(&mut self, poller: E) -> bool {
		for e in poller.events() {
			match e {
				terminity::events::Event::KeyPress(_) => {
					if let StartStage::Stage0 = self.stage {
						self.tick = 0;
						self.stage = StartStage::Stage1 {
							padding: self.stage0.widgets.0.size.height as i16,
						};
					}
				}

				terminity::events::Event::Resize(size) => {
					match &mut self.stage {
						StartStage::Stage0 => {
							let mut padding = self.stage0.widgets.0.size.height;
							// update by adding the difference
							padding += size.height / 2;
							padding = padding.saturating_sub(self.size.height / 2);

							self.stage0.widgets.0.size.height = padding;
						}
						StartStage::Stage1 { padding } => {
							*padding += size.height as i16 / 2 - self.size.height as i16 / 2;
						}
					};
					self.size = size;
				}
				_ => (),
			}
		}

		match &mut self.stage {
			StartStage::Stage0 => {
				if self.tick % 10 == 0 {
					match self.tick / 10 {
						1 => {
							*self.stage0.widgets.2 = wstr!("Press any key to start..").into();
						}
						2 => {
							*self.stage0.widgets.2 = wstr!("Press any key to start...").into();
						}
						3 => {
							*self.stage0.widgets.2 = wstr!("Press any key to start....").into();
						}
						4 => {
							*self.stage0.widgets.2 = wstr!("Press any key to start.....").into();
						}
						5 => {
							*self.stage0.widgets.2 = wstr!("Press any key to start").into();
						}
						_ => {
							self.tick = 0;
							*self.stage0.widgets.2 = wstr!("Press any key to start.").into();
						}
					}
				}
				self.tick += 1;
			}
			StartStage::Stage1 { padding } => {
				self.tick += 1;
				*padding -= 3;
				if *padding + TERMINITY_TEXT.size().height as i16 <= 0 {
					return true;
				}
			}
		}

		// Display
		match self.stage {
			StartStage::Stage0 => poller.display(&self.stage0.as_widget()),
			StartStage::Stage1 { padding } => {
				let mut size = TERMINITY_TEXT.size();
				let tmp = (size.height as i16) + padding;
				size.height = if tmp >= 0 { tmp as u16 } else { 0 };
				self.stage1.widgets.0.size = size;
				poller.display(&self.stage1.as_widget())
			}
		}

		false
	}
}

const TERMINITY_TEXT: Img = img!(
	r"███▀▀██▀▀███                                  ██              ██   ██             ",
	r"█▀   ██   ▀█                                                       ██             ",
	r"     ██      ▄▄█▀██▀███▄███▀████████▄█████▄ ▀███ ▀████████▄ ▀███ ██████▀██▀   ▀██▀",
	r"     ██     ▄█▀   ██ ██▀ ▀▀  ██    ██    ██   ██   ██    ██   ██   ██    ██   ▄█  ",
	r"     ██     ██▀▀▀▀▀▀ ██      ██    ██    ██   ██   ██    ██   ██   ██     ██ ▄█   ",
	r"     ██     ██▄    ▄ ██      ██    ██    ██   ██   ██    ██   ██   ██      ███    ",
	r"   ▄████▄    ▀█████▀████▄  ▄████  ████  ████▄████▄████  ████▄████▄ ▀████   ▄█     ",
	r"                                                                         ▄█       ",
	r"                                                                       ██▀        ",
);
