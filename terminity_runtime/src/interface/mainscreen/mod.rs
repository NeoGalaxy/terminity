pub mod install;
pub mod library;
pub mod options;

use std::fmt::Write;

use terminity::{
	events::Event,
	game::GameContext,
	img,
	widgets::{
		positionning::{
			div::{Div1, Div2, Div3},
			Positionning, Spacing,
		},
		AsWidget, Widget,
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

enum TabContentAsWidget<'a> {
	Library(library::DisplayWidget<'a>),
	Install(&'a mut InstallTab),
	Options(options::DisplayWidget<'a>),
	Emtpy(Spacing),
}

enum TabContentWidget<'a: 'b, 'b> {
	Library(<library::DisplayWidget<'a> as AsWidget>::WidgetType<'b>),
	Install(<InstallTab as AsWidget>::WidgetType<'b>),
	Options(<options::DisplayWidget<'a> as AsWidget>::WidgetType<'b>),
	Emtpy(Spacing),
}

impl<'w> AsWidget for TabContentAsWidget<'w> {
	type WidgetType<'a> = TabContentWidget<'w, 'a>
	where
		Self: 'a;

	fn as_widget(&mut self) -> Self::WidgetType<'_> {
		match self {
			TabContentAsWidget::Library(w) => TabContentWidget::Library(w.as_widget()),
			TabContentAsWidget::Install(w) => TabContentWidget::Install(w.as_widget()),
			TabContentAsWidget::Options(w) => TabContentWidget::Options(w.as_widget()),
			TabContentAsWidget::Emtpy(w) => TabContentWidget::Emtpy(*w.as_widget()),
		}
	}
}
impl Widget for TabContentWidget<'_, '_> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		match self {
			TabContentWidget::Library(w) => w.display_line(f, line),
			TabContentWidget::Install(w) => w.display_line(f, line),
			TabContentWidget::Options(w) => w.display_line(f, line),
			TabContentWidget::Emtpy(s) => s.display_line(f, line),
		}
	}

	fn size(&self) -> Size {
		match self {
			TabContentWidget::Library(w) => w.size(),
			TabContentWidget::Install(w) => w.size(),
			TabContentWidget::Options(w) => w.size(),
			TabContentWidget::Emtpy(s) => s.size(),
		}
	}
}

fn build_widget<'a>(
	tab_content: &'a mut TabContent,
	size: Size,
	games: &'a HubGames,
) -> impl AsWidget + 'a {
	let content = match tab_content.active {
		Some(ActiveTab::Library) => {
			TabContentAsWidget::Library(library::display(&mut tab_content.library, games))
		}
		Some(ActiveTab::Install) => TabContentAsWidget::Install(&mut tab_content.install),
		Some(ActiveTab::Options) => TabContentAsWidget::Options(tab_content.options.display()),
		None => TabContentAsWidget::Emtpy(Spacing::line(size.width)),
	};
	Div2::new(TabSelect::from_active(tab_content.active, games), content)
		.with_content_alignment(Positionning::Center)
		.with_content_pos(Positionning::Start)
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
		size.height -= 8; // Remove space taken by img, spacing and tab selection
		MainScreen {
			tick: 0,
			size,
			tabs: TabContent {
				library: LibraryTab::new(size),
				install: InstallTab::new(size),
				options: OptionsTab::new(size),
				active: None,
			},
		}
	}

	// pub fn disp<D: WidgetDisplayer>(&mut self, displayer: D, games: &HubGames) {
	// 	if self.tick < 2 {
	// 		game_ctx.display(
	// 			&Div1::new(img!("Terminity"))
	// 				.with_content_alignment(Positionning::Center)
	// 				.with_content_pos(Positionning::Start)
	// 				.with_exact_size(self.size),
	// 		)
	// 	} else if self.tick < 3 {
	// 		game_ctx.display(
	// 			&Div1::new(img!("         ", "Terminity"))
	// 				.with_content_alignment(Positionning::Center)
	// 				.with_content_pos(Positionning::Start)
	// 				.with_exact_size(self.size),
	// 		)
	// 	} else if self.tick < 4 {
	// 		game_ctx.display(
	// 			&Div2::new(
	// 				img!("         ", "Terminity"),
	// 				Spacing::line(self.size.width).with_char('─'),
	// 			)
	// 			.with_content_alignment(Positionning::Center)
	// 			.with_content_pos(Positionning::Start)
	// 			.with_exact_size(self.size),
	// 		)
	// 	} else if self.tick < 5 {
	// 		game_ctx.display(
	// 			&Div2::new(
	// 				img!("         ", "Terminity", "         "),
	// 				Spacing::line(self.size.width).with_char('─'),
	// 			)
	// 			.with_content_alignment(Positionning::Center)
	// 			.with_content_pos(Positionning::Start)
	// 			.with_exact_size(self.size),
	// 		)
	// 	} else if self.tick < 14 {
	// 		game_ctx.display(
	// 			&Div3::new(
	// 				img!("         ", "Terminity", "         "),
	// 				Spacing::line(self.size.width).with_char('─'),
	// 				TabSelect::from_active(self.tabs.active, games),
	// 			)
	// 			.with_content_alignment(Positionning::Center)
	// 			.with_content_pos(Positionning::Start),
	// 		)
	// 	} else {
	// 		game_ctx.display(
	// 			&Div3::new(
	// 				img!("         ", "Terminity", "         "),
	// 				Spacing::line(self.size.width).with_char('─'),
	// 				TabContentPreWidget(&mut self.tabs, &mut self.size, games).as_div(),
	// 			)
	// 			.with_content_alignment(Positionning::Center)
	// 			.with_content_pos(Positionning::Start),
	// 		)
	// 	}
	// }

	pub async fn update<Ctx: GameContext>(&mut self, game_ctx: Ctx, ctx: &mut Context) {
		self.tick += 1;
		if self.tick < 5 {
			for _ in game_ctx.events() {}
			if self.tick < 2 {
				game_ctx.display(
					&Div1::new(img!("Terminity"))
						.with_content_alignment(Positionning::Center)
						.with_content_pos(Positionning::Start)
						.with_exact_size(self.size)
						.as_widget(),
				)
			} else if self.tick < 3 {
				game_ctx.display(
					&Div1::new(img!("         ", "Terminity"))
						.with_content_alignment(Positionning::Center)
						.with_content_pos(Positionning::Start)
						.with_exact_size(self.size)
						.as_widget(),
				)
			} else if self.tick < 4 {
				game_ctx.display(
					&Div2::new(
						img!("         ", "Terminity"),
						Spacing::line(self.size.width).with_char('─'),
					)
					.with_content_alignment(Positionning::Center)
					.with_content_pos(Positionning::Start)
					.with_exact_size(self.size)
					.as_widget(),
				)
			} else if self.tick < 5 {
				game_ctx.display(
					&Div2::new(
						img!("         ", "Terminity", "         "),
						Spacing::line(self.size.width).with_char('─'),
					)
					.with_content_alignment(Positionning::Center)
					.with_content_pos(Positionning::Start)
					.with_exact_size(self.size)
					.as_widget(),
				)
			}
			return;
		}

		// let content_size = (&TabContentWidget(&mut self.tabs, &mut self.size)).size();

		let Some(active_tab) = &mut self.tabs.active else {
			if self.tick >= 11 {
				self.tabs.active = Some(ActiveTab::Library);
			}
			for _ in game_ctx.events() {}

			game_ctx.display(
				&Div3::new(
					img!("         ", "Terminity", "         "),
					Spacing::line(self.size.width).with_char('─'),
					TabSelect::from_active(self.tabs.active, &ctx.games),
				)
				.with_content_alignment(Positionning::Center)
				.with_content_pos(Positionning::Start)
				.as_widget(),
			);
			return;
		};
		let initial_active = *active_tab;

		let mapped_game_ctx = PollerMap::new(&game_ctx, |e| {
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
			ActiveTab::Library => self.tabs.library.update(mapped_game_ctx, ctx),
			ActiveTab::Install => self.tabs.install.update(mapped_game_ctx, ctx).await,
			ActiveTab::Options => self.tabs.options.update(mapped_game_ctx),
		}

		game_ctx.display(
			&Div3::new(
				img!("         ", "Terminity", "         "),
				Spacing::line(self.size.width).with_char('─'),
				build_widget(&mut self.tabs, self.size, &ctx.games),
			)
			.with_content_alignment(Positionning::Center)
			.with_content_pos(Positionning::Start)
			.as_widget(),
		);
	}
}
