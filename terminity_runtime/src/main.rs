pub mod events;
pub mod game_handling;

use clap::Parser;
use crossterm::{
	event::{
		DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
		EnableFocusChange, EnableMouseCapture, KeyboardEnhancementFlags,
		PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
	},
	execute, QueueableCommand as _,
};
use game_handling::GameCommands;
use terminity::{
	events::{Event, EventPoller},
	game::{Game, WidgetDisplayer},
	img,
	widgets::{
		auto_padder::AutoPadder,
		content::{Img, TextArea},
		positionning::{Clip, Div1, Div2, Div3, Position, Spacing},
		text::Text,
		ResizableWisget, Widget, WidgetStr,
	},
	wstr, LineDisp, Size,
};
use tokio::time::sleep;

use std::{
	cell::RefCell,
	fs::File,
	io::{stdout, Write as _},
	path::PathBuf,
	time::Duration,
};

#[derive(Parser)]
struct Args {
	game: PathBuf,
}

#[derive(Debug)]
enum StartStage {
	Stage0 { subtitle: &'static WidgetStr, padding: i16 },
	Stage1 { padding: i16 },
}

struct StartScreen {
	tick: u64,
	size: Size,
	stage: StartStage,
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

impl Game for StartScreen {
	type DataInput = ();
	type DataOutput = ();

	fn start<R: std::io::Read>(_data: Option<Self::DataInput>) -> Self {
		let default_size = crossterm::terminal::window_size().unwrap();
		let size = Size { width: 120, height: 30 };
		Self {
			tick: 0,
			size,
			stage: StartStage::Stage0 {
				subtitle: wstr!(r"Press any key to start."),
				padding: (size.height as i16 - TERMINITY_TEXT.size().height as i16 - 1) / 2,
			},
		}
	}

	fn disp<D: WidgetDisplayer>(&mut self, displayer: D) {
		match self.stage {
			StartStage::Stage0 { subtitle, padding } => displayer.run(&Div3 {
				widget0: Spacing { size: Size { width: self.size.width, height: padding as u16 } },
				widget1: TERMINITY_TEXT,
				widget2: TextArea::center(subtitle),
				horizontal: false,
				content_alignment: Position::Center,
				content_pos: Position::Start,
				size: self.size,
			}),
			StartStage::Stage1 { padding } => {
				let mut size = TERMINITY_TEXT.size();
				let tmp = (size.height as i16) + padding;
				size.height = if tmp >= 0 { tmp as u16 } else { 0 };

				displayer.run(&Div1 {
					widget0: Clip {
						widget: TERMINITY_TEXT,
						size,
						v_pos: Position::End,
						h_pos: Position::Center,
					},
					horizontal: false,
					content_alignment: Position::Center,
					content_pos: Position::Start,
					size: self.size,
				})
			}
		}
	}

	fn update<E: terminity::events::EventPoller>(&mut self, poller: E) {
		for e in poller.events() {
			match e {
				terminity::events::Event::KeyPress(_) => {
					if let StartStage::Stage0 { padding, .. } = self.stage {
						self.tick = 0;
						self.stage = StartStage::Stage1 { padding };
					} else {
						poller.cmd(terminity::events::CommandEvent::CloseApp)
					}
				}
				terminity::events::Event::Resize(size) => {
					if let StartStage::Stage0 { padding, .. } | StartStage::Stage1 { padding } =
						&mut self.stage
					{
						// update by adding the difference
						*padding += size.height as i16 / 2 - self.size.height as i16 / 2;
					}
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
					poller.cmd(terminity::events::CommandEvent::CloseApp)
				}
			}
		}
	}

	fn finish(self) -> Option<Self::DataOutput> {
		None
	}
}

struct NativeDisplayer;

impl WidgetDisplayer for NativeDisplayer {
	fn run<W: terminity::widgets::Widget>(self, widget: &W) {
		std::io::stdout()
			.queue(crossterm::cursor::MoveTo(0, 0))
			.unwrap()
			.queue(crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
			.unwrap()
			.flush()
			.unwrap();
		print!("/");
		for _ in 0..widget.size().width {
			print!("-");
		}
		print!("\\\n\r");
		for l in 0..widget.size().height {
			print!("|{}|\n\r", LineDisp(l, widget));
		}
		print!("\\");
		for _ in 0..widget.size().width {
			print!("-");
		}
		print!("/\n\r");
	}
}

struct NativePoller {
	cmds: RefCell<GameCommands>,
}

impl NativePoller {
	fn new() -> Self {
		Self { cmds: GameCommands::default().into() }
	}
}

impl EventPoller for &mut NativePoller {
	type Iter<'a> = NativePollerIter where Self: 'a;
	fn cmd(&self, command: terminity::events::CommandEvent) {
		match command {
			terminity::events::CommandEvent::CloseApp => self.cmds.borrow_mut().close = true,
		}
	}

	fn events(&self) -> Self::Iter<'_> {
		NativePollerIter
	}
}

struct NativePollerIter;

impl Iterator for NativePollerIter {
	type Item = Event;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			break if let Ok(true) = crossterm::event::poll(Duration::ZERO) {
				let Some(e) = events::from_crossterm(crossterm::event::read().ok()?) else {
					continue;
				};
				Some(e)
			} else {
				None
			};
		}
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	// let args = Args::parse();
	crossterm::terminal::enable_raw_mode()?;
	execute!(
		stdout(),
		EnableBracketedPaste,
		EnableFocusChange,
		EnableMouseCapture,
		// PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES),
		// PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS),
		// PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES),
		PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::all()),
	)?;
	stdout()
		.queue(crossterm::cursor::SavePosition)?
		.queue(crossterm::terminal::EnterAlternateScreen)?
		.queue(crossterm::cursor::MoveTo(0, 0))?
		.queue(crossterm::cursor::Hide)?
		.flush()?;

	let mut start = false;
	let mut start_screen = StartScreen::start::<File>(None);
	while !start {
		let mut poller = NativePoller::new();
		start_screen.update(&mut poller);
		start = poller.cmds.borrow().close;

		start_screen.disp(NativeDisplayer);

		sleep(Duration::from_millis(50)).await;
	}

	stdout()
		.queue(crossterm::terminal::LeaveAlternateScreen)?
		.queue(crossterm::cursor::RestorePosition)?
		.queue(crossterm::cursor::Show)?
		.flush()?;
	crossterm::terminal::disable_raw_mode()?;
	execute!(
		stdout(),
		DisableBracketedPaste,
		DisableFocusChange,
		DisableMouseCapture,
		PopKeyboardEnhancementFlags
	)?;
	print!("Finished.");

	Ok(())
}
