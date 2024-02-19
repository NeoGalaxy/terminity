use std::{path::PathBuf, sync::Arc};

use terminity::{
	events::{Event, EventPoller, KeyCode, KeyPress},
	img,
	widgets::{
		content::{Img, TextArea},
		positionning::{
			div::{Div1, Div2},
			Position,
		},
		Widget,
	},
	wstr, Size,
};
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use crate::game_handling::GameLib;

#[derive(Debug)]
pub struct InstallTab {
	file_open: Arc<Mutex<()>>,
	last_dir: PathBuf,
	content: Div2<Img<'static>, TextArea>,
	tick: Option<u8>,
}

const DIALOG: Img = img![
	"This screen allows to choose a game to add to Terminity",
	"         Press Enter to open the file selector         ",
	"                                                       ",
];

impl InstallTab {
	pub fn display_line(
		&self,
		f: &mut std::fmt::Formatter<'_>,
		line: u16,
	) -> std::result::Result<(), std::fmt::Error> {
		self.content.display_line(f, line)
	}

	pub(crate) fn new(size: Size) -> Self {
		let last_dir = if let Some(d) = directories::UserDirs::new() {
			d.home_dir().to_owned()
		} else {
			".".into()
		};
		Self {
			file_open: Mutex::new(()).into(),
			last_dir,
			content: Div2::new(false, DIALOG, TextArea::center(wstr!("")))
				.with_content_alignment(Position::Center)
				.with_content_pos(Position::Center)
				.with_forced_size(size),
			tick: Some(u8::MAX),
		}
	}

	pub(crate) fn update<P: terminity::events::EventPoller>(
		&mut self,
		poller: P,
		send_game: &UnboundedSender<Result<(String, GameLib), libloading::Error>>,
	) {
		if let Some(t) = &mut self.tick {
			if *t < 30 {
				*t += 1;
				if *t == 3 {
					self.content.widget1_mut().set_content(wstr!("Added selected games"));
				}
			} else {
				self.content.widget1_mut().set_content(wstr!(""));
			}
		} else if self.file_open.try_lock().is_ok() {
			self.tick = Some(0);
		}
		let mut open = false;
		for e in poller.events() {
			if matches!(e, Event::KeyPress(KeyPress { code: KeyCode::Enter, .. })) {
				open = true;
			}
		}

		if open {
			self.tick = None;
			let lock_cpy = self.file_open.clone();
			let extensions = if cfg!(target_family = "unix") {
				["so", "dylib"].as_slice()
			} else {
				["dll"].as_slice()
			};
			let dialog = rfd::AsyncFileDialog::new()
				.set_directory(&self.last_dir)
				.add_filter("terminity game", extensions)
				.set_title("Install Terminity game");
			let send_game = send_game.clone();

			tokio::spawn(async move {
				let lock = lock_cpy.try_lock();
				if lock.is_ok() {
					let files = dialog.pick_files().await.unwrap_or_default();

					for file in files {
						let _ = send_game.send(
							unsafe { GameLib::new(file.path()) }.map(|l| (file.file_name(), l)),
						);
					}
				}
			});
		}
	}
}
