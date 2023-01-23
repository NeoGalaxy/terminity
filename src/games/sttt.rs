use std::ops::{Index, IndexMut};
use std::io::Write as IOWrite;
//use std::fmt::Write as FmtWrite;
use std::{
	io,
	fmt::{self, Display, Formatter}
};

use super::CLGame;

#[derive(Debug)]
pub struct SuperTTT ();

impl CLGame for SuperTTT {
	fn run(term: console::Term) -> io::Result<()> {
		Table::new(term).run()
	}
}

type Player = u8;

#[derive(Debug)]
struct Table {
	pub term: console::Term,
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

use Cell::*;
use console::{Attribute, Key};

impl Cell {
    fn from_player(player: Player) -> Self {
		if player == 0 { X }
		else if player == 1 { O }
		else { panic!("Whut?") }
    }
    fn get_color(&self) -> console::Color {
    	match self {
    	    X => {
    	    	console::Color::Red
    	    },
    	    O => {
    	    	console::Color::Blue
    	    },
    	    Empty => {
    	    	console::Color::White
    	    }
    	}
    }
}

impl Display for Cell {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
		use console::*;
		write!(fmt, "{}", 
			Style::new().fg(self.get_color()).attr(Attribute::Bold)
			.apply_to(*self as u8 as char)
		)
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

impl Index<(u8, u8)> for Table {
	type Output = Zone;
	fn index(&self, (x, y): (u8, u8)) -> &Self::Output {
		&self.values[(x + 3*y) as usize]
	}
}
impl IndexMut<(u8, u8)> for Table {
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

impl Table {
	fn new(term: console::Term) -> Self {
		Self {
			term,
			values: Default::default(),
			selected: Selection { ty:SelectType::Zone, x:1, y:1 },
			player: 0,
			text: "Welcome to Super tic tac toe!\n".to_owned()
					+ "Choose in which zone you will play first. You won't be able to cancel!"
		}
	}

	fn run(&mut self) -> io::Result<()> {
		self.disp()?;
		let _winner = loop {
			match self.term.read_key()? {
				Key::ArrowLeft => if self.selected.x > 0 {self.selected.x -= 1},
				Key::ArrowRight => if self.selected.x < 2 {self.selected.x += 1},
				Key::ArrowUp => if self.selected.y > 0 {self.selected.y -= 1},
				Key::ArrowDown => if self.selected.y < 2 {self.selected.y += 1},
				Key::Enter => match self.selected.ty {
					SelectType::Zone => {
						if let Some(winner) = self[(self.selected.x, self.selected.y)].winner {
							self.text = if winner == Empty {
								format!("Nope, no more free tile over here.\n")
							} else {
								format!("Nope, you can't! The zone is already won by {}.\n", winner)
							} + "Choose in which zone you will play.";
						} else {
							self.selected.ty = SelectType::SelCell(self.selected.x, self.selected.y);
							self.selected.x = 1;
							self.selected.y = 1;
							self.text = "Right.\n".to_owned() + "Which tile?";
						}
					}
					SelectType::SelCell(zone_x, zone_y) => {
						match self.play(zone_x, zone_y, self.selected.x, self.selected.y) {
							Ok(None) => {
								self.text = "Really guys? Well, that's a draw.\n".to_owned()
									+ "Well played though! That was actually intense!";
								break None;
							}
							Ok(Some(winner)) => {
								self.text = "WOOOOOHOOOOO!!!! Seems like we have a winner!\n".to_owned()
								 + &format!("Well done player {}!\n", self.player + 1)
								 + &format!("Player {}, maybe you wanna ask a rematch?\n",
								 	(self.player + 1) % 2 + 1);
								break Some(winner);
							}
							Err(true) => {
								self.text = "Done.\n".to_owned()
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
								self.text = "Sneaky one, but you can't play where someone already played!\n".to_owned()
								 + "Choose on which tile you'll play.";
							}
						}
					},
				},
				_ => (),
			}
			self.disp()?;
		};
		self.disp()?;
		self.term.move_cursor_to(0, 19)?;
		Ok(())
	}

	fn play(&mut self, z_x: u8, z_y: u8, cx: u8, cy: u8) -> Result<Option<Player>, bool> {
		let cell_type = Cell::from_player(self.player);

		let cell = &mut self[(z_x, z_y)][(cx, cy)];
		if cell_type == *cell {
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
		self.term.clear_screen()?;
		for y in [0, 4, 8, 12] {
			self.term.move_cursor_to(0, y)?;
			write!(self.term, "#-------#-------#-------#")?;
		}
		for zone_y in 0..3 {
			for cell_y in 0..3 {
				self.term.move_cursor_to(0, 1 + (zone_y as usize * 4) + cell_y as usize)?;
				for zone_x in 0..3 {
					let mut style = console::Style::new();
					if let Some(winner) = self[(zone_x, zone_y)].winner {
						style = style.bg(winner.get_color()).on_bright();
					}
					if let Selection { ty: SelectType::Zone, x, y } = self.selected {
						if x == zone_x && y == zone_y {
							style = style.bg(console::Color::Black).on_bright();
						}
					}
					write!(self.term, "|")?;
					write!(self.term, "{}", style.apply_to(' '))?;
					for cell_x in 0..3 {
						write!(self.term, "{}{}",
							style.apply_to(self[(zone_x, zone_y)][(cell_x, cell_y)]),
							style.apply_to(' '))?;
					}
				}
				write!(self.term, "|\n")?;
			}
		}
		self.term.move_cursor_to(0, 13)?;
		write!(self.term, "Turn to player {} ({})\n", self.player + 1, Cell::from_player(self.player))?;
		self.term.move_cursor_to(0, 15)?;
		self.term.write_line(&self.text)?;

		if let Selection { ty: SelectType::SelCell(zx, zy), x, y } = self.selected {
			self.term.move_cursor_to((2 + 2*x + 8*zx) as usize, (1 + y + 4*zy) as usize)?;
			self.term.show_cursor()?;
		} else {
			self.term.hide_cursor()?;
		}
		Ok(())
	}
}
