pub mod install;
pub mod library;
pub mod options;

use std::fmt::Write;

use terminity::{
	events::{Event, EventPoller},
	game::WidgetDisplayer,
	img,
	widgets::{
		positionning::{
			div::{Div1, Div2, Div3},
			Position, Spacing,
		},
		Widget,
	},
	Size,
};

use self::{install::InstallTab, library::LibraryTab, options::OptionsTab};

use super::{Context, HubGames, PollerMap};

#[derive(Debug)]
pub enum Side {
	Left,
	Right,
}

#[derive(Debug, Default)]
pub struct Border {
	state: Option<Side>,
}

impl Widget for Border {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		let c = match line {
			0 => match self.state {
				Some(Side::Left) => '┓',
				Some(Side::Right) => '┏',
				None => '-',
			},
			1 => {
				if self.state.is_some() {
					'┃'
				} else {
					'│'
				}
			}
			2 => match self.state {
				Some(Side::Left) => '┛',
				Some(Side::Right) => '┗',
				None => '-',
			},
			_ => panic!(),
		};
		f.write_char(c)
	}

	fn size(&self) -> Size {
		Size { width: 1, height: 3 }
	}
}

struct Tab {
	select: TabSelect,
	content: TabContent,
}

#[derive(Debug, Clone, Copy)]
enum ActiveTab {
	Library,
	Install,
	Options,
}

impl ActiveTab {
	pub fn next(&self) -> Self {
		match self {
			Self::Library => Self::Install,
			Self::Install => Self::Options,
			Self::Options => Self::Library,
		}
	}
	pub fn prev(&self) -> Self {
		match self {
			Self::Library => Self::Options,
			Self::Install => Self::Library,
			Self::Options => Self::Install,
		}
	}
}

#[derive(Debug)]
struct TabContent {
	library: LibraryTab,
	install: InstallTab,
	options: OptionsTab,
	active: Option<ActiveTab>,
}

struct TabContentWidget<'a, 'b, 'c>(&'a mut TabContent, &'b mut Size, &'c HubGames);

impl TabContentWidget<'_, '_, '_> {
	fn as_div(&self) -> Div2<TabSelect, &Self> {
		let size = *self.1;
		Div2::new(false, TabSelect::from_active(self.0.active, self.2), self)
			.with_content_alignment(Position::Center)
			.with_content_pos(Position::Start)
		// .with_forced_size(size)
	}
}

impl Widget for &TabContentWidget<'_, '_, '_> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		let size = self.size();
		match &self.0.active {
			None => Spacing::line(size.width).display_line(f, 0),
			Some(ActiveTab::Library) => self.0.library.display_line(f, line, self.2),
			Some(ActiveTab::Install) => self.0.install.display_line(f, line),
			Some(ActiveTab::Options) => self.0.options.display_line(f, line, size),
		}
	}

	fn size(&self) -> Size {
		let mut res = *self.1;
		res.height -= TabSelect::from_active(self.0.active, self.2).size().height;
		res
	}
}

#[derive(Debug)]
struct Num(usize);

impl Widget for Num {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, _: u16) -> std::fmt::Result {
		write!(f, "{:_>4}", self.0)
	}

	fn size(&self) -> Size {
		Size { width: 4, height: 1 }
	}
}

#[derive(Debug, Widget)]
#[widget_layout(
	{
		'0' => .borders[0],
		'1' => .borders[1],
		'2' => .borders[2],
		'3' => .borders[3],
		'.' => .nb_games,
	},
	"0 ─────── 1 ──────────── 2 ─────── 3",
	"0 Library 1 Install Game 2 Options 3",
	"0 ─────── 1 ──────────── 2 ─────── 3",
	"          [nb games: ....]          "
)]
struct TabSelect {
	borders: [Border; 4],
	nb_games: Num,
}
impl TabSelect {
	fn from_active(active: Option<ActiveTab>, games: &HubGames) -> TabSelect {
		let mut res = Self { borders: Default::default(), nb_games: Num(games.list.len()) };
		let i = match active {
			None => return res,
			Some(ActiveTab::Library) => 0,
			Some(ActiveTab::Install) => 1,
			Some(ActiveTab::Options) => 2,
		};
		res.borders[i].state = Some(Side::Right);
		res.borders[i + 1].state = Some(Side::Left);
		res
	}
}

#[derive(Debug)]
pub struct MainScreen {
	tick: usize,
	size: Size,
	tabs: TabContent,
}

impl MainScreen {
	pub fn new(mut size: Size) -> Self {
		size.height -= 5; // Remove space taken by img and spacing
		MainScreen {
			tick: 0,
			size,
			tabs: TabContent {
				library: LibraryTab::new(size),
				install: InstallTab::new(size),
				options: OptionsTab::new(),
				active: None,
			},
		}
	}

	pub fn disp<D: WidgetDisplayer>(&mut self, displayer: D, games: &HubGames) {
		if self.tick < 2 {
			displayer.run(
				&Div1::new(false, img!("Terminity"))
					.with_content_alignment(Position::Center)
					.with_content_pos(Position::Start)
					.with_exact_size(self.size),
			)
		} else if self.tick < 3 {
			displayer.run(
				&Div1::new(false, img!("         ", "Terminity"))
					.with_content_alignment(Position::Center)
					.with_content_pos(Position::Start)
					.with_exact_size(self.size),
			)
		} else if self.tick < 4 {
			displayer.run(
				&Div2::new(
					false,
					img!("         ", "Terminity"),
					Spacing::line(self.size.width).with_char('─'),
				)
				.with_content_alignment(Position::Center)
				.with_content_pos(Position::Start)
				.with_exact_size(self.size),
			)
		} else if self.tick < 5 {
			displayer.run(
				&Div2::new(
					false,
					img!("         ", "Terminity", "         "),
					Spacing::line(self.size.width).with_char('─'),
				)
				.with_content_alignment(Position::Center)
				.with_content_pos(Position::Start)
				.with_exact_size(self.size),
			)
		} else if self.tick < 14 {
			displayer.run(
				&Div3::new(
					false,
					img!("         ", "Terminity", "         "),
					Spacing::line(self.size.width).with_char('─'),
					TabSelect::from_active(self.tabs.active, games),
				)
				.with_content_alignment(Position::Center)
				.with_content_pos(Position::Start),
			)
		} else {
			displayer.run(
				&Div3::new(
					false,
					img!("         ", "Terminity", "         "),
					Spacing::line(self.size.width).with_char('─'),
					TabContentWidget(&mut self.tabs, &mut self.size, games).as_div(),
				)
				.with_content_alignment(Position::Center)
				.with_content_pos(Position::Start),
			)
		}
	}

	pub async fn update<P: EventPoller>(&mut self, poller: P, ctx: &mut Context) {
		self.tick += 1;
		if self.tick < 5 {
			for _ in poller.events() {}
			return;
		}

		// let content_size = (&TabContentWidget(&mut self.tabs, &mut self.size)).size();

		let Some(active_tab) = &mut self.tabs.active else {
			if self.tick >= 11 {
				self.tabs.active = Some(ActiveTab::Library);
			}
			for _ in poller.events() {}
			return;
		};
		let initial_active = *active_tab;

		let poller = PollerMap::new(&poller, |e| {
			if let Event::KeyPress(kp) = &e {
				match kp.code {
					terminity::events::KeyCode::Tab => {
						*active_tab = active_tab.next();
						return None;
					}
					terminity::events::KeyCode::BackTab => {
						*active_tab = active_tab.prev();
						return None;
					}
					_ => (),
				}
			}
			Some(e)
		});
		match initial_active {
			ActiveTab::Library => self.tabs.library.update(poller, ctx),
			ActiveTab::Install => self.tabs.install.update(poller, ctx).await,
			ActiveTab::Options => self.tabs.options.update(poller),
		}
	}
}
