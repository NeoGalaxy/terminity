use std::{
	path::{Path, PathBuf},
	sync::Arc,
};

use rfd::FileHandle;
use terminity::{
	events::{Event, KeyCode, KeyPress},
	img,
	widgets::{
		content::Img,
		positionning::{
			div::{CollDiv, Div4},
			Position, Spacing,
		},
		Widget,
	},
	Size,
};
use tokio::{
	fs::{self, File},
	io,
	sync::Mutex,
	task::JoinHandle,
};

use crate::interface::{game_repository::GameDataLatest, load_game, Context, GameStatus};

#[derive(Debug)]
enum Status {
	Running(JoinHandle<Result<GameDataLatest, String>>),
	Success(usize, usize, String),
	Fail(usize, String),
}

impl Widget for Status {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, _line: u16) -> std::fmt::Result {
		match self {
			Status::Running(_) => write!(f, "Importing game..."),
			Status::Success(_, id, n) => write!(f, "Import sucessful. Id: {id}, Name: {n}"),
			Status::Fail(_, s) => write!(f, "Import failed: {s}"),
		}
	}

	fn size(&self) -> Size {
		let width = match self {
			Status::Fail(_, s) => s.len() as u16 + 15,
			Status::Success(_, id, name) => {
				22 + id.to_string().len() as u16 + 8 + name.len() as u16
			}
			_ => 17,
		};
		Size { width, height: 1 }
	}
}

#[derive(Debug)]
pub struct InstallTab {
	file_open: Arc<Mutex<()>>,
	last_dir: PathBuf,
	// content: Div2<Img<'static>, TextArea>,
	tick: usize,
	popup: Option<JoinHandle<Vec<FileHandle>>>,
	copying: CollDiv<Vec<Status>, Status>,
}

const TITLE1: Img = img![
	"==============================================",
	">>   Press Enter to open the file picker.   <<",
	"==============================================",
];
const TITLE2: Img = img!["                ", ". Import list: .", r" -────────────- "];

impl InstallTab {
	pub fn display_line(
		&self,
		f: &mut std::fmt::Formatter<'_>,
		line: u16,
		size: Size,
	) -> std::result::Result<(), std::fmt::Error> {
		Div4::new(false, Spacing::line(size.width).with_char(' '), TITLE1, TITLE2, &self.copying)
			.with_content_alignment(Position::Start)
			.with_content_pos(Position::Start)
			.with_forced_size(size)
			.display_line(f, line)
	}

	pub(crate) fn new() -> Self {
		let last_dir = if let Some(d) = directories::UserDirs::new() {
			d.home_dir().to_owned()
		} else {
			".".into()
		};
		Self {
			file_open: Mutex::new(()).into(),
			last_dir,
			// content: Div2::new(false, DIALOG, TextArea::center(wstr!("")))
			// 	.with_content_alignment(Position::Center)
			// 	.with_content_pos(Position::Center)
			// 	.with_forced_size(size),
			tick: 0,
			popup: None,
			copying: CollDiv::new(false, vec![]),
		}
	}

	pub(crate) async fn update<P: terminity::events::EventPoller>(
		&mut self,
		poller: P,
		ctx: &mut Context,
	) {
		self.tick += 1;

		if let Some(p) = &mut self.popup {
			if p.is_finished() {
				if let Ok(files) = p.await {
					self.popup = None;
					for file in files {
						let root_dir = ctx.root_path.clone();

						let f = async move {
							let ext = file.path().extension().unwrap_or_default();
							let name = file
								.path()
								.file_stem()
								.ok_or_else(|| "path with no stem".to_string())?
								.to_string_lossy();
							let mut full_name = Path::new(&*name).with_extension(ext);
							let mut i = 0;
							let mut output = loop {
								match File::options()
									.write(true)
									.create_new(true)
									.open(&root_dir.join(&full_name))
									.await
								{
									Ok(o) => break o,
									Err(e) => match e.kind() {
										io::ErrorKind::AlreadyExists => (),

										e => {
											return Err(format!(
												"When opening {:?}: {e}",
												root_dir.join(full_name)
											))
										}
									},
								}
								full_name = Path::new(&format!("{name}_{i}")).with_extension(ext);
								i += 1;
							};

							io::copy(
								&mut File::open(file.path()).await.map_err(|e| e.to_string())?,
								&mut output,
							)
							.await
							.map_err(|e| e.to_string())?;
							Ok(GameDataLatest { subpath: full_name })
						};

						self.copying.collection_mut().push(Status::Running(tokio::spawn(f)));
					}
				}
			}
		}

		for i in (0..self.copying.len()).rev() {
			let mut copying = self.copying.collection_mut();
			let game = &mut copying[i];
			match game {
				Status::Running(h) => {
					if h.is_finished() {
						let res = h.await;
						match res.map_err(|e| e.to_string()).and_then(|v| v) {
							Ok(data) => {
								let name = data.subpath.display().to_string();
								// let handle = load_game(&ctx.root_path, &data.subpath);
								// let id = ctx.games.add((data, GameStatus::Loading(handle)));
								let id = ctx.games.add((data, GameStatus::Unloaded));
								*game = Status::Success(self.tick, id, name)
							}
							Err(e) => *game = Status::Fail(self.tick, e),
						}
					}
				}
				Status::Success(t, ..) | Status::Fail(t, _) if self.tick > *t + 100 => {
					copying.remove(i);
				}
				_ => (),
			}
		}

		let mut open = false;
		for e in poller.events() {
			if matches!(e, Event::KeyPress(KeyPress { code: KeyCode::Enter, .. })) {
				open = true;
			}
		}

		if open && self.popup.is_none() {
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

			let task = {
				async move {
					let lock = lock_cpy.try_lock();

					if lock.is_err() {
						return vec![];
					}

					dialog.pick_files().await.unwrap_or_default()
				}
			};
			self.popup = Some(tokio::spawn(task));
		}
	}
}