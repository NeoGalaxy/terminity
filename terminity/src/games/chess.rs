use std::fmt::Write;
use std::io;
use std::ops::{Index, IndexMut};
use std::time::{Duration, Instant};

use crossterm::event::{
	KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
//use crossterm::{Style, Color as TermColor};
use crossterm::style::{Color as TermColor, ContentStyle};
use crossterm::{cursor, event, QueueableCommand};
use terminity_widgets::{Widget, WidgetDisplay};

use crate::games::Game;

type Pos = (usize, usize);

pub struct Chess();

impl Game for Chess {
	fn run(&self, out: &mut dyn io::Write) -> io::Result<()> {
		let mut board = Board::default();
		out.queue(cursor::Hide)?;
		let mut since_blink: Duration = Duration::new(0, 0);
		'mainloop: loop {
			loop {
				out.queue(crossterm::cursor::MoveTo(0, 0))?;
				write!(out, "{}", board)?;
				out.flush()?;
				let mut timeout: u64 = if board.selected == None { 400 } else { 100 };
				timeout = timeout.saturating_sub(since_blink.as_millis() as u64);
				let now = Instant::now();
				if event::poll(Duration::from_millis(timeout))? {
					since_blink += now.elapsed();
					break;
				}
				since_blink = Duration::new(0, 0);
				board.cursor_style_alt = !board.cursor_style_alt;
			}
			use event::Event::*;
			use KeyCode::*;
			use KeyEventKind::*;
			match event::read()? {
				Mouse(MouseEvent { kind, mut column, mut row, .. }) => {
					column = column / 2;
					if column < 1 || column > 8 || row >= 8 {
						continue;
					}
					if board.rotated {
						column = 8 - column;
					} else {
						column -= 1;
						row = 7 - row;
					}
					let new_pos = (column as usize, row as usize);
					match kind {
						MouseEventKind::Moved | MouseEventKind::Drag(_) => {
							if new_pos == board.cursor_pos {
								continue;
							} else {
								board.cursor_pos = new_pos;
							}
						}
						MouseEventKind::Down(MouseButton::Left) => {
							board.select();
						}
						MouseEventKind::Up(MouseButton::Left) => {
							board.play();
						}
						_ => (),
					}
				}
				Key(KeyEvent { code: Enter, kind: Press, .. }) => {
					if board.selected == None {
						board.select();
					} else {
						board.play();
					}
				}
				Key(KeyEvent { code: Left, kind: Press, .. }) => {
					if board.cursor_pos.0 > 0 {
						board.cursor_pos.0 -= 1;
					}
				}
				Key(KeyEvent { code: Right, kind: Press, .. }) => {
					if board.cursor_pos.0 < 7 {
						board.cursor_pos.0 += 1;
					}
				}
				Key(KeyEvent { code: Up, kind: Press, .. }) => {
					if board.cursor_pos.1 < 7 {
						board.cursor_pos.1 += 1;
					}
				}
				Key(KeyEvent { code: Down, kind: Press, .. }) => {
					if board.cursor_pos.1 > 0 {
						board.cursor_pos.1 -= 1;
					}
				}

				Key(KeyEvent {
					code: KeyCode::Char('c'),
					kind: KeyEventKind::Press,
					modifiers,
					..
				}) => {
					if modifiers.contains(KeyModifiers::CONTROL) {
						break 'mainloop;
					}
				}
				_ => continue, // Wait another event
			}
			since_blink = Duration::new(0, 0);
			board.cursor_style_alt = false;
		}
		Ok(())
	}
}

#[derive(WidgetDisplay)]
struct Board {
	tiles: [[Option<Tile>; 8]; 8],
	light_tile_style: ContentStyle,
	dark_tile_style: ContentStyle,
	checked_tile_style: ContentStyle,
	select_style: ContentStyle,
	invalid_style: ContentStyle,
	rotated: bool,
	cursor_pos: Pos,
	selected: Option<Pos>,
	cursor_style_alt: bool,
	player: Color,
	checked_by: Vec<Pos>,
	invalid: Option<(Pos, Pos)>,
}

impl Board {
	fn select(&mut self) {
		if Some(self.player) == self[self.cursor_pos].map(|t| t.1) {
			self.selected = Some(self.cursor_pos);
		}
	}
	fn play(&mut self) {
		self.checked_by = Vec::with_capacity(5);
		let cursor_pos = self.cursor_pos.clone();
		self.invalid = None;
		if let Some(selected) = self.selected {
			self.invalid = Some((selected, cursor_pos));
			if let Some(tile) = &self[selected] {
				let eaten = self[cursor_pos].clone();
				if eaten.map_or(true, |e| e.1 != tile.1)
					&& tile.move_valid(&selected, &cursor_pos, &self)
				{
					let mut tile = tile.clone();
					if tile.0 == Piece::Pawn && (cursor_pos.1 == 0 || cursor_pos.1 == 7) {
						tile.0 = Piece::Queen;
					}
					self[selected] = None;
					self[cursor_pos] = Some(tile.clone());

					let mut checkers = self.pieces_checking(self.player);
					if checkers.len() != 0 {
						// Revert
						self[selected] = Some(tile);
						self[cursor_pos] = eaten;
						self.checked_by.append(&mut checkers);
					} else {
						self.player.swap();
					}
					self.invalid = None;
				}
			}
		}
		self.checked_by.append(&mut self.pieces_checking(self.player));
		self.selected = None;
	}
	fn pieces_checking(&self, color: Color) -> Vec<Pos> {
		let (king_pos, _) = self
			.indexed_pieces()
			.find(|(_, p)| Some(Tile(Piece::King, color)) == **p)
			.expect("Error: no king on field");

		self.indexed_pieces()
			.filter(|(pos, piece)| match piece {
				None => false,
				Some(t) => t.1 != color && t.move_valid(&pos, &king_pos, self),
			})
			.map(|(pos, _)| pos)
			.collect()
	}
	fn indexed_pieces<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = (Pos, &Option<Tile>)>> {
		Box::new(
			self.tiles
				.iter()
				.enumerate()
				.flat_map(|(y, e)| e.iter().enumerate().map(move |(x, t)| ((x, y), t))),
		)
	}
}

impl Index<Pos> for Board {
	type Output = Option<Tile>;
	fn index(&self, (x, y): Pos) -> &Self::Output {
		&self.tiles[y][x]
	}
}
impl IndexMut<Pos> for Board {
	fn index_mut(&mut self, (x, y): Pos) -> &mut Self::Output {
		&mut self.tiles[y][x]
	}
}

impl Default for Board {
	fn default() -> Self {
		use Color::*;
		use Piece::*;
		Board {
			light_tile_style: ContentStyle {
				foreground_color: Some(TermColor::White),
				background_color: Some(TermColor::DarkGrey),
				underline_color: None,
				attributes: Default::default(),
			},
			dark_tile_style: ContentStyle {
				foreground_color: Some(TermColor::White),
				background_color: None,
				underline_color: None,
				attributes: Default::default(),
			},
			checked_tile_style: ContentStyle {
				foreground_color: Some(TermColor::White),
				background_color: Some(TermColor::DarkRed),
				underline_color: None,
				attributes: Default::default(),
			},
			select_style: ContentStyle {
				foreground_color: Some(TermColor::White),
				background_color: Some(TermColor::DarkBlue),
				underline_color: None,
				attributes: Default::default(),
			},
			invalid_style: ContentStyle {
				foreground_color: Some(TermColor::White),
				background_color: Some(TermColor::DarkYellow),
				underline_color: None,
				attributes: Default::default(),
			},
			tiles: [
				[
					None,
					//Some(Tile(Rook, White)),
					Some(Tile(Knight, White)),
					Some(Tile(Bishop, White)),
					Some(Tile(Queen, White)),
					Some(Tile(King, White)),
					Some(Tile(Bishop, White)),
					Some(Tile(Knight, White)),
					Some(Tile(Rook, White)),
				],
				[Some(Tile(Pawn, White)); 8],
				Default::default(),
				Default::default(),
				Default::default(),
				Default::default(),
				[Some(Tile(Pawn, Black)); 8],
				[
					Some(Tile(Rook, Black)),
					Some(Tile(Knight, Black)),
					Some(Tile(Bishop, Black)),
					Some(Tile(Queen, Black)),
					Some(Tile(King, Black)),
					Some(Tile(Bishop, Black)),
					Some(Tile(Knight, Black)),
					Some(Tile(Rook, Black)),
				],
			],
			rotated: false,
			cursor_pos: (4, 0),
			cursor_style_alt: false,
			selected: None,
			player: White,
			checked_by: vec![],
			invalid: None,
		}
	}
}

impl Widget for Board {
	fn size(&self) -> Pos {
		(18, 9)
	}
	fn displ_line(&self, f: &mut std::fmt::Formatter<'_>, mut line_nb: usize) -> std::fmt::Result {
		if line_nb == 8 {
			f.write_char(' ')?;
			f.write_char(' ')?;
			let write_column = |letter| {
				f.write_char(letter)?;
				f.write_char(' ')
			};
			let col_names = 'A'..='H';
			if self.rotated {
				col_names.rev().map(write_column).collect::<Result<_, _>>()?
			} else {
				col_names.map(write_column).collect::<Result<_, _>>()?
			};
		} else {
			if !self.rotated {
				// The bord begins at bottom left
				line_nb = 7 - line_nb;
			}
			let line = &self.tiles[line_nb];
			f.write_str(&(line_nb + 1).to_string())?;
			f.write_char(' ')?;

			let selected_style =
				if self.cursor_style_alt { None } else { Some(&self.select_style) };
			let write_tile = |(i, tile): (usize, &Option<Tile>)| {
				let pos = (i, line_nb);
				let style = if pos == self.cursor_pos && selected_style.is_some() {
					selected_style.unwrap()
				} else if self.checked_by.len() > 0 && *tile == Some(Tile(Piece::King, self.player))
				{
					&self.checked_tile_style
				} else if self.checked_by.contains(&pos) {
					&self.checked_tile_style
				} else if self.selected == Some(pos) {
					&self.select_style
				} else if self.invalid.map_or(false, |(p0, p1)| p0 == pos || p1 == pos) {
					&self.invalid_style
				} else if (line_nb + i) % 2 == 0 {
					&self.light_tile_style
				} else {
					&self.dark_tile_style
				};
				write!(
					f,
					"{}",
					style.clone().apply(match tile {
						None => "  ".to_owned(),
						Some(t) => t.0.to_char(t.1).to_string() + " ",
					})
				)
			};
			let line_iter = line.iter().enumerate();
			if self.rotated {
				line_iter.rev().map(write_tile).collect::<Result<_, _>>()?
			} else {
				line_iter.map(write_tile).collect::<Result<_, _>>()?
			};
		}
		Ok(())
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Tile(Piece, Color);

impl Tile {
	fn move_valid(&self, curr_pos: &Pos, new_pos: &Pos, board: &Board) -> bool {
		curr_pos != new_pos
			&& match self.0 {
				Piece::King => {
					(curr_pos.0).abs_diff(new_pos.0) <= 1 && (curr_pos.1).abs_diff(new_pos.1) <= 1
				}
				Piece::Rook => {
					if curr_pos.0 == new_pos.0 {
						let max = new_pos.1.max(curr_pos.1);
						let min = new_pos.1.min(curr_pos.1);
						((min + 1)..max).all(|y| board[(curr_pos.0, y)] == None)
					} else if curr_pos.1 == new_pos.1 {
						let max = new_pos.0.max(curr_pos.0);
						let min = new_pos.0.min(curr_pos.0);
						((min + 1)..max).all(|x| board[(x, curr_pos.1)] == None)
					} else {
						false
					}
				}
				Piece::Bishop => {
					let dx = (curr_pos.0).abs_diff(new_pos.0);
					let dy = (curr_pos.1).abs_diff(new_pos.1);
					dx == dy
						&& (1..dx)
							.map(|d| {
								(
									(curr_pos.0 as isize)
										+ d as isize * if curr_pos.0 < new_pos.0 { 1 } else { -1 },
									(curr_pos.1 as isize)
										+ d as isize * if curr_pos.1 < new_pos.1 { 1 } else { -1 },
								)
							})
							.all(|pos| board[(pos.0 as usize, pos.1 as usize)] == None)
				}
				Piece::Queen => {
					Tile(Piece::Rook, self.1).move_valid(curr_pos, new_pos, board)
						|| Tile(Piece::Bishop, self.1).move_valid(curr_pos, new_pos, board)
				}
				Piece::Knight => {
					let dx = (curr_pos.0).abs_diff(new_pos.0);
					let dy = (curr_pos.1).abs_diff(new_pos.1);
					dx == 1 && dy == 2 || dx == 2 && dy == 1
				}
				Piece::Pawn => {
					let going_formard = (self.1 == Color::White && curr_pos.1 + 1 == new_pos.1)
						|| (self.1 == Color::Black && curr_pos.1 - 1 == new_pos.1);
					// First move
					(
						(curr_pos.1 == 1 || curr_pos.1 == 6) // Didn't move (or 1 away from queen)
						&& curr_pos.0 == new_pos.0 // Move straight
						&& curr_pos.1.abs_diff(new_pos.1) == 2 // Moves by 2
						&& board[(new_pos.0, (curr_pos.1 + new_pos.1) / 2)] == None // No one in path
						&& board[*new_pos] == None // Not eating
					)
					// Other moves
					|| going_formard
						&& ((curr_pos.0).abs_diff(new_pos.0) == 1 && board[*new_pos] != None
							|| curr_pos.0 == new_pos.0 && board[*new_pos] == None)
				}
			}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Color {
	Black,
	White,
}

impl Color {
	fn swap(&mut self) {
		match self {
			Self::Black => *self = Self::White,
			Self::White => *self = Self::Black,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u16)]
#[allow(dead_code)]
enum Piece {
	King = '\u{2654}' as u16,
	Queen = '\u{2655}' as u16,
	Rook = '\u{2656}' as u16,
	Bishop = '\u{2657}' as u16,
	Knight = '\u{2658}' as u16,
	Pawn = '\u{2659}' as u16,
}

impl Piece {
	fn to_char(&self, color: Color) -> char {
		match color {
			Color::White => unsafe { std::char::from_u32_unchecked(*self as u32 + 6) },
			Color::Black => unsafe { std::char::from_u32_unchecked(*self as u32) },
		}
	}
}
