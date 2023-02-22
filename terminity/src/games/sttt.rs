use std::fmt::Write as FmtWrite;
use std::ops::{Index, IndexMut};
use std::time::Duration;
use std::{
	fmt::{self, Display, Formatter},
	io,
};

use super::Game;
use crossterm::event::{self, KeyModifiers};
use crossterm::style::{Color, ContentStyle, Stylize};
use crossterm::terminal::Clear;
use crossterm::{cursor, QueueableCommand};
use terminity_widgets::widgets::frame::Frame;
use terminity_widgets::widgets::text::{Align, Text};
use terminity_widgets::{frame, Widget};
use Tile::*;

#[derive(Debug)]
pub struct SuperTTT();

impl Game for SuperTTT {
	fn run(&self, out: &mut dyn io::Write) -> io::Result<()> {
		GameArea::new(out).run()
	}
}

type Player = u8;

struct GameArea<'a> {
	pub out: &'a mut dyn io::Write,
	pub frame: Frame<usize, Zone, [Zone; 9]>,
	pub selected: Selection,
	pub player: u8,
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

#[derive(Debug)]
struct Zone {
	pub values: [Tile; 9],
	pub winner: Option<Tile>,
	pub selected: bool,
}

impl Widget for Zone {
	fn displ_line(&self, f: &mut Formatter<'_>, line: usize) -> std::fmt::Result {
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

impl Default for Zone {
	fn default() -> Self {
		Self {
			values: [Empty; 9],
			winner: None,
			selected: false,
		}
	}
}

impl Index<(u8, u8)> for GameArea<'_> {
	type Output = Zone;
	fn index(&self, (x, y): (u8, u8)) -> &Self::Output {
		&self.frame[(x + 3 * y) as usize]
	}
}
impl IndexMut<(u8, u8)> for GameArea<'_> {
	fn index_mut(&mut self, (x, y): (u8, u8)) -> &mut Self::Output {
		&mut self.frame[(x + 3 * y) as usize]
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

impl<'a> GameArea<'a> {
	fn new(out: &'a mut dyn io::Write) -> Self {
		let mut values: [Zone; 9] = Default::default();
		values[4].selected = true;
		Self {
			out,
			selected: Selection {
				ty: SelectType::Zone,
				x: 1,
				y: 1,
			},
			player: 0,
			frame: frame!(
			values => {
				'0': 0, '1': 1, '2': 2,
				'3': 3, '4': 4, '5': 5,
				'6': 6, '7': 7, '8': 8
			}
			"                      #-------#-------#-------#                      "
			"                      |0000000|1111111|2222222|                      "
			"                      |0000000|1111111|2222222|                      "
			"                      |0000000|1111111|2222222|                      "
			"                      #-------#-------#-------#                      "
			"                      |3333333|4444444|5555555|                      "
			"                      |3333333|4444444|5555555|                      "
			"                      |3333333|4444444|5555555|                      "
			"                      #-------#-------#-------#                      "
			"                      |6666666|7777777|8888888|                      "
			"                      |6666666|7777777|8888888|                      "
			"                      |6666666|7777777|8888888|                      "
			"                      #-------#-------#-------#                      "
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
				width: 70,
			},
		}
	}

	fn run(&mut self) -> crossterm::Result<()> {
		use event::{Event::Key, KeyCode::*, KeyEvent, KeyEventKind::*};
		self.disp()?;
		let winner = loop {
			let coords = (self.selected.x, self.selected.y);
			self[coords].selected = false;
			match event::read()? {
				Key(KeyEvent {
					code: Left,
					kind: Press,
					..
				}) => {
					if self.selected.x > 0 {
						self.selected.x -= 1
					}
				}
				Key(KeyEvent {
					code: Right,
					kind: Press,
					..
				}) => {
					if self.selected.x < 2 {
						self.selected.x += 1
					}
				}
				Key(KeyEvent {
					code: Up,
					kind: Press,
					..
				}) => {
					if self.selected.y > 0 {
						self.selected.y -= 1
					}
				}
				Key(KeyEvent {
					code: Down,
					kind: Press,
					..
				}) => {
					if self.selected.y < 2 {
						self.selected.y += 1
					}
				}
				Key(KeyEvent {
					code: Enter,
					kind: Press,
					..
				}) => match self.selected.ty {
					SelectType::Zone => {
						self.text.clear();
						if let Some(winner) = self[(self.selected.x, self.selected.y)].winner {
							self.text[2] = if winner == Empty {
								format!("Nope, no more free tile over here.")
							} else {
								format!("Nope, you can't! The zone is already won by {}.", winner)
							};
							self.text[3] = "Choose in which zone you will play.".to_string();
						} else {
							self.selected.ty =
								SelectType::SelCell(self.selected.x, self.selected.y);
							self.selected.x = 1;
							self.selected.y = 1;
							self.text[2] = "Right.".to_owned();
							self.text[3] = "Which tile?".to_owned();
						}
					}
					SelectType::SelCell(zone_x, zone_y) => {
						match self.play(zone_x, zone_y, self.selected.x, self.selected.y) {
							Ok(None) => {
								self.text.clear();
								self.text[2] = "Really guys? That's a draw.".to_owned();
								self.text[3] = "Well played though, that was intense!".to_owned();
								break Ok(None);
							}
							Ok(Some(winner)) => {
								self.text.clear();
								self.text[2] =
									"WOOOOOHOOOOO!!!! Seems like we have a winner!".to_owned();
								self.text[3] = format!("Well done player {}!", self.player + 1);
								self.text[4] = format!(
									"Player {}, maybe you wanna ask a rematch?",
									(self.player + 1) % 2 + 1
								);
								break Ok(Some(winner));
							}
							Err(true) => {
								self.text.clear();
								self.text[2] = "Done.".to_owned();
								self.text[3] = "Where to play now?".to_owned();
								if self[(self.selected.x, self.selected.y)].winner == None {
									self.selected.ty =
										SelectType::SelCell(self.selected.x, self.selected.y);
									self.selected.x = 1;
									self.selected.y = 1;
								} else {
									self.selected.ty = SelectType::Zone;
									self.selected.x = 1;
									self.selected.y = 1;
								}
								self.player = (1 + self.player) % 2;
							}
							Err(false) => {
								self.text.clear();
								self.text[2] =
									"Sneaky one, but you can't play where someone already played!"
										.to_owned();
								self.text[3] = "Choose on which tile you'll play.".to_string();
							}
						}
					}
				},
				Key(KeyEvent {
					code: Char('c'),
					kind: Press,
					modifiers,
					..
				}) => {
					if modifiers.contains(KeyModifiers::CONTROL) {
						self.text.clear();
						self.text[2] = "Exiting the game....".to_owned();
						break Err(());
					}
				}
				_ => (),
			}
			if self.selected.ty == SelectType::Zone {
				let coords = (self.selected.x, self.selected.y);
				self[coords].selected = true;
			}
			self.disp()?;
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
			self.text[6] = texts[i].to_owned();
			i = (i + 1) % texts.len();
			self.disp()?;
			self.out.queue(crossterm::cursor::Hide)?;
			self.out.flush()?;
			if event::poll(Duration::from_millis(600))? {
				break;
			}
		}
		Ok(())
	}

	fn play(&mut self, z_x: u8, z_y: u8, cx: u8, cy: u8) -> Result<Option<Player>, bool> {
		let cell_type = Tile::from_player(self.player);

		let cell = &mut self[(z_x, z_y)][(cx, cy)];
		if *cell != Empty {
			return Err(false);
		}
		*cell = cell_type;

		// Line is the same
		if cell_type == self[(z_x, z_y)][((cx + 1) % 3, cy)]
			&& cell_type == self[(z_x, z_y)][((cx + 2) % 3, cy)]
		// Column is the same
		||     cell_type == self[(z_x, z_y)][(cx, (cy + 1) % 3)]
			&& cell_type == self[(z_x, z_y)][(cx, (cy + 2) % 3)]
		// On the first diagonal and same as all on the diagonal
		||     cx == cy
			&& cell_type == self[(z_x, z_y)][((cx + 1) % 3, (cy + 1) % 3)]
			&& cell_type == self[(z_x, z_y)][((cx + 2) % 3, (cy + 2) % 3)]
		// On the second diagonal and same as all on the diagonal
		||     cx + cy == 2
			&& cell_type == self[(z_x, z_y)][((cx + 1) % 3, (cy + 2) % 3)]
			&& cell_type == self[(z_x, z_y)][((cx + 2) % 3, (cy + 1) % 3)]
		{
			// Mark zone as winned
			self[(z_x, z_y)].winner = Some(cell_type);

			// If line is the same
			if Some(cell_type) == self[((z_x + 1) % 3, z_y)].winner
				&& Some(cell_type) == self[((z_x + 2) % 3, z_y)].winner
			// column is the same
			||     Some(cell_type) == self[(z_x, (z_y + 1) % 3)].winner
				&& Some(cell_type) == self[(z_x, (z_y + 2) % 3)].winner
			// on the first diagonal and same as all on the diagonal
			||     z_x == z_y
				&& Some(cell_type) == self[((z_x + 1) % 3, (z_y + 1) % 3)].winner
				&& Some(cell_type) == self[((z_x + 2) % 3, (z_y + 2) % 3)].winner
			// on the second diagonal and same as all on the diagonal
			||     z_x + z_y == 2
				&& Some(cell_type) == self[((z_x + 1) % 3, (z_y + 2) % 3)].winner
				&& Some(cell_type) == self[((z_x + 2) % 3, (z_y + 1) % 3)].winner
			{
				return Ok(Some(self.player));
			}
		} else if self[(z_x, z_y)].values.iter().all(|c| *c != Empty) {
			self[(z_x, z_y)].winner = Some(Empty);
		}

		if self[(z_x, z_y)].winner != None && self.frame.iter().all(|z| z.winner != None) {
			Ok(None)
		} else {
			Err(true)
		}
	}

	fn disp(&mut self) -> io::Result<()> {
		self.text[0] = format!(
			"Turn to player {} ({})",
			self.player + 1,
			Tile::from_player(self.player)
				.to_string()
				.with(Tile::from_player(self.player).get_color())
				.bold()
		);

		self.out.queue(cursor::MoveTo(0, 0))?;
		write!(self.out, "{}", self.frame)?;
		self.out.queue(cursor::MoveTo(0, 13))?;
		write!(self.out, "{}", self.text)?;
		//.queue(PrintSt(self.text.clone().stylize()))?
		self.out
			.queue(Clear(crossterm::terminal::ClearType::FromCursorDown))?;

		if let Selection {
			ty: SelectType::SelCell(zx, zy),
			x,
			y,
		} = self.selected
		{
			let y_index = (1 + y + 4 * zy) as usize;
			let x_index =
				1 + 2 * x as u16
					+ self.frame.find_x(y_index, (zx + zy * 3) as usize).unwrap() as u16;
			self.out
				.queue(cursor::MoveTo(x_index as u16, y_index as u16))?
				.queue(cursor::Show)?;
		} else {
			self.out.queue(cursor::Hide)?;
		}
		self.out.flush()?;
		Ok(())
	}
}
