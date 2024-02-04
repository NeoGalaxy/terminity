//! A "super tic tac toe". TODO: Explan rules

#![allow(missing_docs)]

use core::slice;
use std::fmt::Write as FmtWrite;
use std::ops::{Index, IndexMut};
use std::time::{Duration, Instant};
use std::{
	fmt::{self, Display, Formatter},
	io,
};

use super::Game;
use crossterm::cursor::MoveTo;
use crossterm::event::{self, KeyModifiers, MouseEvent};
use crossterm::style::{Color, ContentStyle, Stylize};
use crossterm::terminal::{self, Clear};
use crossterm::QueueableCommand;
use format::lazy_format;
use terminity_widgets::widgets::auto_padder::AutoPadder;
use terminity_widgets::widgets::frame::Frame;
use terminity_widgets::widgets::text::{Align, Text};
use terminity_widgets::{
	frame, EventHandleingWidget, ResizableWisget, StructFrame, Widget, WidgetDisplay,
};
use Tile::*;

#[derive(Debug)]
pub struct SuperTTT();

const FRAME_TIME: Duration = Duration::from_millis(100);

impl Game for SuperTTT {
	fn run(&self, out: &mut dyn io::Write) -> io::Result<()> {
		let mut game_state = AutoPadder(
			GameState::new(),
			terminal::size().map(|(a, b)| (a as usize, b as usize)).unwrap_or((0, 0)),
		);
		use event::{
			Event::{self, Key},
			KeyCode::*,
			KeyEvent,
			KeyEventKind::*,
		};
		out.queue(Clear(crossterm::terminal::ClearType::All))?.queue(MoveTo(0, 0))?;
		out.queue(crossterm::cursor::Hide)?;
		write!(out, "{game_state}")?;
		out.flush()?;
		let mut last_disp = Instant::now();
		let winner = loop {
			event::poll(FRAME_TIME.saturating_sub(last_disp.elapsed()))?;
			if last_disp.elapsed() >= FRAME_TIME {
				out.queue(Clear(crossterm::terminal::ClearType::All))?.queue(MoveTo(0, 0))?;
				out.queue(crossterm::cursor::Hide)?;
				write!(out, "{game_state}")?;
				out.flush()?;
				last_disp = Instant::now();
			}
			let coords = (game_state.selected.x, game_state.selected.y);
			game_state.area[coords].selected = false;
			match event::read()? {
				Event::Mouse(e) => {
					let _ = game_state.handle_event(e);
				}
				Key(KeyEvent { code: Left, kind: Press, .. }) => {
					if game_state.selected.x > 0 {
						game_state.selected.x -= 1
					}
				}
				Key(KeyEvent { code: Right, kind: Press, .. }) => {
					if game_state.selected.x < 2 {
						game_state.selected.x += 1
					}
				}
				Key(KeyEvent { code: Up, kind: Press, .. }) => {
					if game_state.selected.y > 0 {
						game_state.selected.y -= 1
					}
				}
				Key(KeyEvent { code: Down, kind: Press, .. }) => {
					if game_state.selected.y < 2 {
						game_state.selected.y += 1
					}
				}
				Key(KeyEvent { code: Enter, kind: Press, .. }) => match game_state.selected.ty {
					SelectType::Zone => {
						game_state.text.clear();
						if let Some(winner) =
							game_state.area[(game_state.selected.x, game_state.selected.y)].winner
						{
							game_state.text[2] = if winner == Empty {
								format!("Nope, no more free tile over here.")
							} else {
								format!("Nope, you can't! The zone is already won by {}.", winner)
							};
							game_state.text[3] = "Choose in which zone you will play.".to_string();
						} else {
							game_state.selected.ty =
								SelectType::SelCell(game_state.selected.x, game_state.selected.y);
							game_state.selected.x = 1;
							game_state.selected.y = 1;
							game_state.text[2] = "Right.".to_owned();
							game_state.text[3] = "Which tile?".to_owned();
						}
					}
					SelectType::SelCell(zone_x, zone_y) => {
						let game_state = &mut *game_state;
						match game_state.play(
							zone_x,
							zone_y,
							game_state.selected.x,
							game_state.selected.y,
						) {
							Ok(None) => {
								game_state.text.clear();
								game_state.text[2] = "Really guys? That's a draw.".to_owned();
								game_state.text[3] =
									"Well played though, that was intense!".to_owned();
								break Ok(None);
							}
							Ok(Some(winner)) => {
								game_state.text.clear();
								game_state.text[2] =
									"WOOOOOHOOOOO!!!! Seems like we have a winner!".to_owned();
								game_state.text[3] =
									format!("Well done player {}!", game_state.player + 1);
								game_state.text[4] = format!(
									"Player {}, maybe you wanna ask a rematch?",
									(game_state.player + 1) % 2 + 1
								);
								break Ok(Some(winner));
							}
							Err(true) => {
								game_state.text.clear();
								game_state.text[2] = "Done.".to_owned();
								game_state.text[3] = "Where to play now?".to_owned();
								if game_state.area[(game_state.selected.x, game_state.selected.y)]
									.winner == None
								{
									game_state.selected.ty = SelectType::SelCell(
										game_state.selected.x,
										game_state.selected.y,
									);
									game_state.selected.x = 1;
									game_state.selected.y = 1;
								} else {
									game_state.selected.ty = SelectType::Zone;
									game_state.selected.x = 1;
									game_state.selected.y = 1;
								}
								game_state.player = (1 + game_state.player) % 2;
							}
							Err(false) => {
								game_state.text.clear();
								game_state.text[2] =
									"Sneaky one, but you can't play where someone already played!"
										.to_owned();
								game_state.text[3] =
									"Choose on which tile you'll play.".to_string();
							}
						}
					}
				},
				Key(KeyEvent { code: Char('c'), kind: Press, modifiers, .. }) => {
					if modifiers.contains(KeyModifiers::CONTROL) {
						game_state.text.clear();
						game_state.text[2] = "Exiting the game....".to_owned();
						break Err(());
					}
				}
				Event::Resize(w, h) => game_state.resize((w as usize, h as usize)),
				_ => (),
			}
			if game_state.selected.ty == SelectType::Zone {
				let coords = (game_state.selected.x, game_state.selected.y);
				game_state.area[coords].selected = true;
			}
		};
		if winner == Err(()) {
			return Ok(());
		}
		let texts = [
			"Press any key to exit   ",
			"Press any key to exit.  ",
			"Press any key to exit.. ",
			"Press any key to exit...",
		];
		let mut i = 0;
		loop {
			game_state.text[6] = texts[i].to_owned();
			i = (i + 1) % texts.len();
			out.queue(Clear(crossterm::terminal::ClearType::All))?.queue(MoveTo(0, 0))?;
			write!(out, "{game_state}")?;
			out.queue(crossterm::cursor::Hide)?;
			out.flush()?;
			if event::poll(Duration::from_millis(600))? {
				break;
			}
		}
		Ok(())
	}
}

type Player = u8;

#[derive(StructFrame, WidgetDisplay)]
#[sf_impl(EventHandleingWidget)]
#[sf_layout {
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"                       #########################                       ",
	"TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT",
	"TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT",
	"TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT",
	"TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT",
	"TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT",
	"TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT",
	"TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT",
}]
struct GameState {
	#[sf_layout(name = '#')]
	pub area: Frame<(u8, u8), GameArea>,
	pub selected: Selection,
	pub player: u8,
	#[sf_layout(name = 'T', ignore_mouse_event)]
	pub text: Text<7>,
}

#[derive(Debug, Copy, Clone)]
struct Selection {
	ty: SelectType,
	x: u8,
	y: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SelectType {
	SelCell(u8, u8),
	Zone,
}

//#[derive(Debug)]
struct Zone {
	pub values: Frame<usize, [Tile; 9]>,
	pub winner: Option<Tile>,
	pub selected: bool,
}

impl Default for Zone {
	fn default() -> Self {
		let content = [Tile::Empty; 9];
		Self {
			values: frame! {
				content of size<1, 1> => {repeat 'X': 0..9}
				" X X X "
				" X X X "
				" X X X "
			},
			winner: None,
			selected: false,
		}
	}
}

/*impl Widget for Zone {
	fn display_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
		let mut style = ContentStyle::new();
		if let Some(winner) = self.winner {
			style.background_color = Some(winner.get_color());
			style.foreground_color = Some(Color::Black);
		}
		if self.selected {
			style.background_color = Some(Color::Grey);
			style.foreground_color = style.background_color;
		}
		for cell_x in 0..3 {
			f.write_fmt(format_args!("{}", &style.apply(' ').to_string()))?;
			let cell = self[(cell_x, line as u8)];
			let mut styled_cell = style.apply(cell).bold();
			if style.foreground_color == None {
				styled_cell = styled_cell.with(cell.get_color()).bold();
			}
			f.write_fmt(format_args!("{}", styled_cell))?;
		}
		f.write_fmt(format_args!("{}", &style.apply(' ').to_string()))?;
		Ok(())
	}
	fn size(&self) -> (usize, usize) {
		(7, 3)
	}
}*/

impl Widget for Zone {
	fn display_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
		let mut style = ContentStyle::new();
		if let Some(winner) = self.winner {
			style.background_color = Some(winner.get_color());
			style.foreground_color = Some(Color::Black);
		}
		if self.selected {
			style.background_color = Some(Color::Grey);
			style.foreground_color = style.background_color;
		}
		let to_disp = self.values.get_line_display(line).to_string();
		f.write_fmt(format_args!("{}", style.apply(to_disp)))
	}
	fn size(&self) -> (usize, usize) {
		self.values.size()
	}
}

impl EventHandleingWidget for Zone {
	type HandledEvent = Option<(usize, usize)>;
	fn handle_event(&mut self, event: MouseEvent) -> Self::HandledEvent {
		let (elem_index, ()) = self.values.handle_event(event)?;
		Some((elem_index % 3, elem_index / 3))
	}
}

#[derive(Default)]
struct GameArea([Zone; 9]);

impl Index<(u8, u8)> for GameArea {
	type Output = Zone;
	fn index(&self, (x, y): (u8, u8)) -> &Self::Output {
		&self.0[(x + 3 * y) as usize]
	}
}
impl IndexMut<(u8, u8)> for GameArea {
	fn index_mut(&mut self, (x, y): (u8, u8)) -> &mut Self::Output {
		&mut self.0[(x + 3 * y) as usize]
	}
}

impl GameArea {
	fn iter(&self) -> slice::Iter<Zone> {
		self.0.iter()
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum Tile {
	X = 'x' as u8,
	O = 'o' as u8,
	Empty = ' ' as u8,
}

impl Tile {
	fn from_player(player: Player) -> Self {
		if player == 0 {
			X
		} else if player == 1 {
			O
		} else {
			panic!("Whut?")
		}
	}
	fn get_color(&self) -> Color {
		match self {
			X => Color::Red,
			O => Color::Blue,
			Empty => Color::White,
		}
	}
}

impl Display for Tile {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
		fmt.write_char(*self as u8 as char)
	}
}

impl Widget for Tile {
	fn display_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
		self.fmt(f)
	}
	#[inline]
	fn size(&self) -> (usize, usize) {
		(1, 1)
	}
}

impl EventHandleingWidget for Tile {
	type HandledEvent = ();
	fn handle_event(&mut self, event: crossterm::event::MouseEvent) -> Self::HandledEvent {
		()
	}
}

impl Index<(u8, u8)> for Zone {
	type Output = Tile;
	fn index(&self, (x, y): (u8, u8)) -> &Self::Output {
		&self.values[(x + 3 * y) as usize]
	}
}
impl IndexMut<(u8, u8)> for Zone {
	fn index_mut(&mut self, (x, y): (u8, u8)) -> &mut Self::Output {
		&mut self.values[(x + 3 * y) as usize]
	}
}

impl GameState {
	fn new() -> Self {
		let mut area: GameArea = Default::default();
		area[(1, 1)].selected = true;
		Self {
			selected: Selection { ty: SelectType::Zone, x: 1, y: 1 },
			player: 0,
			area: frame!(
				area => {
					'0': (0, 0), '1': (1, 0), '2': (2, 0),
					'3': (0, 1), '4': (1, 1), '5': (2, 1),
					'6': (0, 2), '7': (1, 2), '8': (2, 2),
				}
				"#-------#-------#-------#"
				"|0000000|1111111|2222222|"
				"|0000000|1111111|2222222|"
				"|0000000|1111111|2222222|"
				"#-------#-------#-------#"
				"|3333333|4444444|5555555|"
				"|3333333|4444444|5555555|"
				"|3333333|4444444|5555555|"
				"#-------#-------#-------#"
				"|6666666|7777777|8888888|"
				"|6666666|7777777|8888888|"
				"|6666666|7777777|8888888|"
				"#-------#-------#-------#"
			),
			text: Text {
				content: [
					"".to_owned(),
					"".to_owned(),
					"Welcome to Super tic tac toe!".to_owned(),
					"Choose in which zone you will play first. You won't be able to cancel!"
						.to_owned(),
					"".to_owned(),
					"".to_owned(),
					"".to_owned(),
				],
				align: Align::Center,
				padding: ' ',
				width: 71,
			},
		}
	}

	fn play(&mut self, z_x: u8, z_y: u8, cx: u8, cy: u8) -> Result<Option<Player>, bool> {
		let cell_type = Tile::from_player(self.player);

		let cell = &mut self.area[(z_x, z_y)][(cx, cy)];
		if *cell != Empty {
			return Err(false);
		}
		*cell = cell_type;

		// Line is the same
		if cell_type == self.area[(z_x, z_y)][((cx + 1) % 3, cy)]
			&& cell_type == self.area[(z_x, z_y)][((cx + 2) % 3, cy)]
		// Column is the same
		||     cell_type == self.area[(z_x, z_y)][(cx, (cy + 1) % 3)]
			&& cell_type == self.area[(z_x, z_y)][(cx, (cy + 2) % 3)]
		// On the first diagonal and same as all on the diagonal
		||     cx == cy
			&& cell_type == self.area[(z_x, z_y)][((cx + 1) % 3, (cy + 1) % 3)]
			&& cell_type == self.area[(z_x, z_y)][((cx + 2) % 3, (cy + 2) % 3)]
		// On the second diagonal and same as all on the diagonal
		||     cx + cy == 2
			&& cell_type == self.area[(z_x, z_y)][((cx + 1) % 3, (cy + 2) % 3)]
			&& cell_type == self.area[(z_x, z_y)][((cx + 2) % 3, (cy + 1) % 3)]
		{
			// Mark zone as winned
			self.area[(z_x, z_y)].winner = Some(cell_type);

			// If line is the same
			if Some(cell_type) == self.area[((z_x + 1) % 3, z_y)].winner
				&& Some(cell_type) == self.area[((z_x + 2) % 3, z_y)].winner
			// column is the same
			||     Some(cell_type) == self.area[(z_x, (z_y + 1) % 3)].winner
				&& Some(cell_type) == self.area[(z_x, (z_y + 2) % 3)].winner
			// on the first diagonal and same as all on the diagonal
			||     z_x == z_y
				&& Some(cell_type) == self.area[((z_x + 1) % 3, (z_y + 1) % 3)].winner
				&& Some(cell_type) == self.area[((z_x + 2) % 3, (z_y + 2) % 3)].winner
			// on the second diagonal and same as all on the diagonal
			||     z_x + z_y == 2
				&& Some(cell_type) == self.area[((z_x + 1) % 3, (z_y + 2) % 3)].winner
				&& Some(cell_type) == self.area[((z_x + 2) % 3, (z_y + 1) % 3)].winner
			{
				return Ok(Some(self.player));
			}
		} else if self.area[(z_x, z_y)].values.iter().all(|c| *c != Empty) {
			self.area[(z_x, z_y)].winner = Some(Empty);
		}

		if self.area[(z_x, z_y)].winner != None && self.area.iter().all(|z| z.winner != None) {
			Ok(None)
		} else {
			Err(true)
		}
	}
}
