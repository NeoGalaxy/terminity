use std::ops::{Index, IndexMut};
use std::fmt::Write as FmtWrite;
use std::{
	io,
	fmt::{self, Display, Formatter}
};

use super::Game;
use crossterm::terminal::Clear;
use crossterm::{QueueableCommand, cursor};
use crossterm::style::{Stylize, Color, ContentStyle, PrintStyledContent as PrintSt, StyledContent};
use crossterm::event::{self, KeyModifiers};
use Cell::*;

macro_rules! nl {
	() => {
		format!("{}\n\r", Clear(crossterm::terminal::ClearType::UntilNewLine))
	}
}

#[derive(Debug)]
pub struct SuperTTT ();

impl Game for SuperTTT {
	fn run(&self, out: &mut dyn io::Write) -> io::Result<()> {
		Table::new(out).run()
	}
}

type Player = u8;

struct Table<'a> {
	pub out: &'a mut dyn io::Write,
	pub values: [Zone; 9],
	pub selected: Selection,
	pub player: u8,
	pub text: String
}

#[derive(Debug, Copy, Clone)]
struct Selection {
	ty: SelectType,
	x: u8,
	y: u8,
}

#[derive(Debug, Copy, Clone)]
enum SelectType {
	SelCell(u8, u8),
	Zone,
}

#[derive(Debug)]
struct Zone {
	pub values: [Cell; 9],
	pub winner: Option<Cell>
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
enum Cell {
	X = 'x' as u8,
	O = 'o' as u8,
	Empty = ' ' as u8
}


impl Cell {
    fn from_player(player: Player) -> Self {
		if player == 0 { X }
		else if player == 1 { O }
		else { panic!("Whut?") }
    }
    fn get_color(&self) -> Color {
    	match self {
    	    X => {
    	    	Color::Red
    	    },
    	    O => {
    	    	Color::Blue
    	    },
    	    Empty => {
    	    	Color::White
    	    }
    	}
    }
}

impl Display for Cell {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
		fmt.write_char(*self as u8 as char)
	}
}

impl Default for Zone {
	fn default() -> Self {
		Zone {
			values: [Empty; 9],
			winner: None
		}
	}
}

impl Index<(u8, u8)> for Table<'_> {
	type Output = Zone;
	fn index(&self, (x, y): (u8, u8)) -> &Self::Output {
		&self.values[(x + 3*y) as usize]
	}
}
impl IndexMut<(u8, u8)> for Table<'_> {
	fn index_mut(&mut self, (x, y): (u8, u8)) -> &mut Self::Output {
		&mut self.values[(x + 3*y) as usize]
	}
}

impl Index<(u8, u8)> for Zone {
	type Output = Cell;
	fn index(&self, (x, y): (u8, u8)) -> &Self::Output {
		&self.values[(x + 3*y) as usize]
	}
}
impl IndexMut<(u8, u8)> for Zone {
	fn index_mut(&mut self, (x, y): (u8, u8)) -> &mut Self::Output {
		&mut self.values[(x + 3*y) as usize]
	}
}

impl<'a> Table<'a> {
	fn new(out: &'a mut dyn io::Write) -> Self {
		Self {
			out,
			values: Default::default(),
			selected: Selection { ty:SelectType::Zone, x:1, y:1 },
			player: 0,
			text: "Welcome to Super tic tac toe!".to_owned() + &nl!()
					+ "Choose in which zone you will play first. You won't be able to cancel!"
		}
	}

	fn run(&mut self) -> crossterm::Result<()> {
		use event::{Event::Key, KeyEvent, KeyCode::*, KeyEventKind::*};
		self.disp()?;
		let _winner = loop {
			match event::read()? {
				Key(KeyEvent { code: Left, kind: Press, .. }) =>
					if self.selected.x > 0 {self.selected.x -= 1},
				Key(KeyEvent { code: Right, kind: Press, .. }) =>
					if self.selected.x < 2 {self.selected.x += 1},
				Key(KeyEvent { code: Up, kind: Press, .. }) =>
					if self.selected.y > 0 {self.selected.y -= 1},
				Key(KeyEvent { code: Down, kind: Press, .. }) =>
					if self.selected.y < 2 {self.selected.y += 1},
				Key(KeyEvent { code: Enter, kind: Press, .. }) =>
					match self.selected.ty {
					SelectType::Zone => {
						if let Some(winner) = self[(self.selected.x, self.selected.y)].winner {
							self.text = if winner == Empty {
								format!("Nope, no more free tile over here.")
							} else {
								format!("Nope, you can't! The zone is already won by {}.", winner)
							} + &nl!() + "Choose in which zone you will play.";
						} else {
							self.selected.ty = SelectType::SelCell(self.selected.x, self.selected.y);
							self.selected.x = 1;
							self.selected.y = 1;
							self.text = "Right.".to_owned() + &nl!() + "Which tile?";
						}
					}
					SelectType::SelCell(zone_x, zone_y) => {
						match self.play(zone_x, zone_y, self.selected.x, self.selected.y) {
							Ok(None) => {
								self.text = "Really guys? Well, that's a draw.".to_owned() + &nl!()
									+ "Well played though! That was actually intense!";
								break Ok(None);
							}
							Ok(Some(winner)) => {
								self.text = "WOOOOOHOOOOO!!!! Seems like we have a winner!".to_owned() + &nl!()
								 + &format!("Well done player {}!", self.player + 1) + &nl!()
								 + &format!("Player {}, maybe you wanna ask a rematch?",
								 	(self.player + 1) % 2 + 1) + &nl!();
								break Ok(Some(winner));
							}
							Err(true) => {
								self.text = "Done.".to_owned() + &nl!()
								 + "Where to play now?";
								if self[(self.selected.x, self.selected.y)].winner == None {
									self.selected.ty = SelectType::SelCell(self.selected.x, self.selected.y);
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
								self.text = "Sneaky one, but you can't play where someone already played!".to_owned() + &nl!()
								 + "Choose on which tile you'll play.";
							}
						}
					},
				},
				Key(KeyEvent { code: Char('c'), kind: Press, modifiers, .. }) => {
					if modifiers.contains(KeyModifiers::CONTROL) {
						self.text = "Exiting the game....".to_owned() + &nl!();
						break Err(());
					}
				}
				_ => (),
			}
			self.disp()?;
		};
		self.disp()?;
        self.out.queue(crossterm::cursor::Show)?;
		Ok(())
	}

	fn play(&mut self, z_x: u8, z_y: u8, cx: u8, cy: u8) -> Result<Option<Player>, bool> {
		let cell_type = Cell::from_player(self.player);

		let cell = &mut self[(z_x, z_y)][(cx, cy)];
		if *cell != Empty {
			return Err(false);
		}
		*cell = cell_type;

		// Line is the same
		if     cell_type == self[(z_x, z_y)][((cx + 1) % 3, cy)]
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
			if     Some(cell_type) == self[((z_x + 1) % 3, z_y)].winner
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
				return Ok(Some(self.player))
			}
		} else if self[(z_x, z_y)].values.iter().all(|c| *c != Empty) {
			self[(z_x, z_y)].winner = Some(Empty);
		}

		if self[(z_x, z_y)].winner != None && self.values.iter().all(|z| z.winner != None) {
			Ok(None)
		} else {
			Err(true)
		}
	}

	fn disp(&mut self) -> io::Result<()> {
		for y in [0, 4, 8, 12] {
			self.out.queue(cursor::MoveTo(0, y))?
			.queue(PrintSt("#-------#-------#-------#".stylize()))?;
		}
		for zone_y in 0..3 {
			for cell_y in 0..3 {
				self.out.queue(cursor::MoveTo(0, 1 + (zone_y as u16 * 4) + cell_y as u16))?;
				for zone_x in 0..3 {
					let mut style = ContentStyle::new();
					//style.background_color = Some(Color::Black);
					if let Some(winner) = self[(zone_x, zone_y)].winner {
						style.background_color = Some(winner.get_color());
						style.foreground_color = Some(Color::Black);
					}
					if let Selection { ty: SelectType::Zone, x, y } = self.selected {
						if x == zone_x && y == zone_y {
							style.foreground_color = style.background_color;
							style.background_color = Some(Color::Grey);
						}
					}
					self.out.queue(PrintSt('|'.stylize()))?;
					for cell_x in 0..3 {
						self.out.queue(PrintSt(StyledContent::new(style.clone(), ' ')))?;
						let cell = self[(zone_x, zone_y)][(cell_x, cell_y)];
						let mut styled_cell = StyledContent::new(style.clone(), cell).bold();
						if style.foreground_color == None {
							styled_cell = styled_cell.with(cell.get_color()).bold();
						}
						self.out.queue(PrintSt(styled_cell))?;
					}
					self.out.queue(PrintSt(StyledContent::new(style, ' ')))?;
				}
				self.out.queue(PrintSt("|".stylize()))?;
			}
		}
		self.out
			.queue(cursor::MoveTo(0, 13))?
			.queue(PrintSt((format!(
				"Turn to player {} ({})",
				self.player + 1,
				Cell::from_player(self.player).to_string().with(Cell::from_player(self.player).get_color()).bold()
			) + &nl!()).stylize()))?
			.queue(cursor::MoveTo(0, 15))?
			.queue(PrintSt(self.text.clone().stylize()))?
			.queue(Clear(crossterm::terminal::ClearType::FromCursorDown))?;

		if let Selection { ty: SelectType::SelCell(zx, zy), x, y } = self.selected {
			self.out
				.queue(cursor::MoveTo((2 + 2*x + 8*zx) as u16, (1 + y + 4*zy) as u16))?
				.queue(cursor::Show)?;
		} else {
			self.out.queue(cursor::Hide)?;
		}
		self.out.flush()?;
		Ok(())
	}
}
