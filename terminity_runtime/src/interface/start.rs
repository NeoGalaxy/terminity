use terminity::{
	game::WidgetDisplayer,
	img,
	widget_string::WidgetStr,
	widgets::{
		content::{Img, TextArea},
		positionning::{
			div::{Div1, Div3},
			Clip, Position, Spacing,
		},
		Widget,
	},
	wstr, Size,
};

#[derive(Debug)]
pub enum StartStage {
	Stage0 { subtitle: WidgetStr<'static>, padding: i16 },
	Stage1 { padding: i16 },
}

#[derive(Debug)]
pub struct StartScreen {
	tick: u64,
	pub size: Size,
	stage: StartStage,
}

impl StartScreen {
	pub fn new(size: Size) -> Self {
		Self {
			tick: 0,
			size,
			stage: StartStage::Stage0 {
				subtitle: wstr!(r"Press any key to start."),
				padding: (size.height as i16 - TERMINITY_TEXT.size().height as i16 - 1) / 2,
			},
		}
	}

	pub fn disp<D: WidgetDisplayer>(&mut self, displayer: D) {
		match self.stage {
			StartStage::Stage0 { subtitle, padding } => {
				let widget = &Div3::new(
					false,
					Spacing {
						size: Size { width: self.size.width, height: padding as u16 },
						c: ' ',
					},
					TERMINITY_TEXT,
					TextArea::center(subtitle),
				)
				.with_content_alignment(Position::Center)
				.with_content_pos(Position::Start)
				.with_exact_size(self.size);

				displayer.run(widget);
			}
			StartStage::Stage1 { padding } => {
				let mut size = TERMINITY_TEXT.size();
				let tmp = (size.height as i16) + padding;
				size.height = if tmp >= 0 { tmp as u16 } else { 0 };

				displayer.run(
					&Div1::new(
						false,
						Clip {
							widget: TERMINITY_TEXT,
							size,
							v_pos: Position::End,
							h_pos: Position::Center,
						},
					)
					.with_content_alignment(Position::Center)
					.with_content_pos(Position::Start)
					.with_exact_size(self.size),
				)
			}
		}
	}

	pub fn update<E: terminity::events::EventPoller>(&mut self, poller: E) -> bool {
		for e in poller.events() {
			match e {
				terminity::events::Event::KeyPress(_) => {
					if let StartStage::Stage0 { padding, .. } = self.stage {
						self.tick = 0;
						self.stage = StartStage::Stage1 { padding };
					}
				}

				terminity::events::Event::Resize(size) => {
					let (StartStage::Stage0 { padding, .. } | StartStage::Stage1 { padding }) =
						&mut self.stage;
					// update by adding the difference
					*padding += size.height as i16 / 2 - self.size.height as i16 / 2;
					self.size = size;
				}
				_ => (),
			}
		}

		match &mut self.stage {
			StartStage::Stage0 { subtitle, .. } => {
				if self.tick % 10 == 0 {
					match self.tick / 10 {
						1 => {
							*subtitle = wstr!("Press any key to start..");
						}
						2 => {
							*subtitle = wstr!("Press any key to start...");
						}
						3 => {
							*subtitle = wstr!("Press any key to start....");
						}
						4 => {
							*subtitle = wstr!("Press any key to start.....");
						}
						5 => {
							*subtitle = wstr!("Press any key to start");
						}
						_ => {
							self.tick = 0;
							*subtitle = wstr!("Press any key to start.");
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
