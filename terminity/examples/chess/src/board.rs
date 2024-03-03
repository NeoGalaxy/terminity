use crossterm::style::{Color as TermColor, ContentStyle};
use std::{
	fmt::Write as _,
	ops::{Index, IndexMut},
};
use terminity::{
	events::Position,
	widgets::{EventBubbling, Widget},
	Size,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pos {
	pub x: u16,
	pub y: u16,
}

/// The style configuration for the board. Check out Board::default for default value
pub struct BoardStyle {
	pub light_tile_style: ContentStyle,
	pub dark_tile_style: ContentStyle,
	pub checked_tile_style: ContentStyle,
	pub select_style: ContentStyle,
	pub selected_style: ContentStyle,
	pub invalid_style: ContentStyle,
}

// Represents a piece on a board's tile.
// NB: it is not actually a tile since it can't be empty
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Tile(Piece, Color);

impl Tile {
	/// Checks if the current Piece can make said move.
	/// Implicitly checks if the aimed position can be eaten or not.
	/// Doesn't check the color of the aimed piece though, this can be made easily beforehand
	fn move_valid(&self, curr_pos: &Pos, new_pos: &Pos, board: &Board) -> bool {
		curr_pos != new_pos
			&& match self.0 {
				Piece::King => {
					(curr_pos.x).abs_diff(new_pos.x) <= 1 && (curr_pos.y).abs_diff(new_pos.y) <= 1
				}
				Piece::Rook => {
					if curr_pos.x == new_pos.x {
						let max = new_pos.y.max(curr_pos.y);
						let min = new_pos.y.min(curr_pos.y);
						// Check if there's no piece between curr_pos and new_pos, both excluded.
						((min + 1)..max).all(|y| board[Pos { x: curr_pos.x, y }].is_none())
					} else if curr_pos.y == new_pos.y {
						let max = new_pos.x.max(curr_pos.x);
						let min = new_pos.x.min(curr_pos.x);
						// Ditto
						((min + 1)..max).all(|x| board[Pos { x, y: curr_pos.y }].is_none())
					} else {
						false
					}
				}
				Piece::Bishop => {
					let dx = (curr_pos.x).abs_diff(new_pos.x);
					let dy = (curr_pos.y).abs_diff(new_pos.y);
					// Moves as much on x axis as on y axis
					dx == dy
					// No one in the way
						&& (1..dx)
							// Create positions
							.map(|d| {
								(
									(curr_pos.x as isize)
										+ d as isize * if curr_pos.x < new_pos.x { 1 } else { -1 },
									(curr_pos.y as isize)
										+ d as isize * if curr_pos.y < new_pos.y { 1 } else { -1 },
								)
							})
							.all(|pos| board[Pos{x: pos.0 as u16, y: pos.1 as u16}].is_none())
				}
				Piece::Queen => {
					Tile(Piece::Rook, self.1).move_valid(curr_pos, new_pos, board)
						|| Tile(Piece::Bishop, self.1).move_valid(curr_pos, new_pos, board)
				}
				Piece::Knight => {
					let dx = (curr_pos.x).abs_diff(new_pos.x);
					let dy = (curr_pos.y).abs_diff(new_pos.y);
					dx == 1 && dy == 2 || dx == 2 && dy == 1
				}
				Piece::Pawn => {
					let going_formard = (self.1 == Color::White && curr_pos.y + 1 == new_pos.y)
						|| (self.1 == Color::Black && curr_pos.y - 1 == new_pos.y);
					// First move
					(
						(curr_pos.y == 1 || curr_pos.y == 6) // Didn't move (or 1 away from queen)
						&& curr_pos.x == new_pos.x // Move straight
						&& curr_pos.y.abs_diff(new_pos.y) == 2 // Moves by 2
						&& board[Pos{x: new_pos.x, y: (curr_pos.y + new_pos.y) / 2}].is_none() // No one in path
						&& board[*new_pos].is_none() // Not eating
					)
					// Other moves
					|| going_formard
						&& ((curr_pos.x).abs_diff(new_pos.x) == 1 && board[*new_pos].is_some()
							|| curr_pos.x == new_pos.x && board[*new_pos].is_none())
				}
			}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
	Black,
	White,
}

impl Color {
	/// swaps the color of self
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
	fn to_char(self, color: Color) -> char {
		match color {
			Color::White => unsafe { std::char::from_u32_unchecked(self as u32 + 6) },
			Color::Black => unsafe { std::char::from_u32_unchecked(self as u32) },
		}
	}
}

impl Index<Pos> for Board {
	type Output = Option<Tile>;
	fn index(&self, Pos { x, y }: Pos) -> &Self::Output {
		&self.tiles[y as usize][x as usize]
	}
}

impl IndexMut<Pos> for Board {
	fn index_mut(&mut self, Pos { x, y }: Pos) -> &mut Self::Output {
		&mut self.tiles[y as usize][x as usize]
	}
}

/// The chess board and its metadata (mainly for display purpose)
pub struct Board {
	pub tiles: [[Option<Tile>; 8]; 8],
	pub style: BoardStyle,
	/// Whether the board is 90° rotated (for black-side view). Not recently tested.
	pub rotated: bool,
	/// Current cursor position
	pub cursor_pos: Pos,
	/// The position of the selected piece to move
	pub selected: Option<Pos>,
	/// Determinates if the cursor is in alternative style (i.e. blinking) or not
	pub cursor_style_alt: bool,
	/// The color of the next player to play
	pub player: Color,
	/// List of all pieces that are checking the king (or will check if requested move was made)
	pub checked_by: Vec<Pos>,
	/// Positions of the invalid move that was tried to be made
	pub invalid: Option<(Pos, Pos)>,
}

impl Board {
	/// Mark the cursor's position as selected (selects the piece to move)
	pub fn select(&mut self) {
		if Some(self.player) == self[self.cursor_pos].map(|t| t.1) {
			self.selected = Some(self.cursor_pos);
		}
	}
	/// Tries to move the piece at the position of self.selected to the current cursor's position
	pub fn play(&mut self) {
		// Reset the checking pieces list (will be populated later)
		self.checked_by = Vec::with_capacity(5);

		let cursor_pos = self.cursor_pos;
		self.invalid = None;

		if let Some(selected) = self.selected {
			// No move was actually asked, do as if nothing happened
			if selected != cursor_pos {
				// If current move is not invalid, it will be marked as valid later on
				self.invalid = Some((selected, cursor_pos));

				if let Some(tile) = &self[selected] {
					// The piece that's being attacked
					let eaten = self[cursor_pos];
					// if the attacked piece is not to the player and the move is valid
					if eaten.map_or(true, |e| e.1 != tile.1)
						&& tile.move_valid(&selected, &cursor_pos, self)
					{
						let mut tile = *tile;
						// Promotion (assumes that the pawn didn't find a way to move backwards)
						if tile.0 == Piece::Pawn && (cursor_pos.y == 0 || cursor_pos.y == 7) {
							tile.0 = Piece::Queen;
						}
						// Move piece
						self[selected] = None;
						self[cursor_pos] = Some(tile);

						// Checks out if any check occured
						let mut checkers = self.pieces_checking(self.player);
						if !checkers.is_empty() {
							// Revert and signal checking pieces that dissallowed move
							self[selected] = Some(tile);
							self[cursor_pos] = eaten;
							self.checked_by.append(&mut checkers);
						} else {
							// confirms by changing current player
							self.player.swap();
						}
						// Mark current move as valid
						self.invalid = None;
					}
				}
			}
		}
		// Signal any currently checking pieces
		self.checked_by.append(&mut self.pieces_checking(self.player));
		// Whether the move was sucessful or not, we want to stop trying to move.
		self.selected = None;
	}
	/// Lists the positions of the pieces that are checking the current color's king
	pub fn pieces_checking(&self, color: Color) -> Vec<Pos> {
		let (king_pos, _) = self
			.indexed_tiles()
			.find(|(_, p)| Some(Tile(Piece::King, color)) == **p)
			.expect("Error: no king on field"); // Yeah... Custom boards might panic

		self.indexed_tiles()
			.filter(|(pos, piece)| match piece {
				None => false,
				Some(t) => t.1 != color && t.move_valid(pos, &king_pos, self),
			})
			.map(|(pos, _)| pos)
			.collect()
	}
	/// Lists all the tiles with their coordinates
	pub fn indexed_tiles<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = (Pos, &Option<Tile>)>> {
		Box::new(self.tiles.iter().enumerate().flat_map(|(y, e)| {
			e.iter().enumerate().map(move |(x, t)| (Pos { x: x as u16, y: y as u16 }, t))
		}))
	}
}

impl Default for Board {
	fn default() -> Self {
		use Color::*;
		use Piece::*;
		Board {
			style: BoardStyle {
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
				selected_style: ContentStyle {
					foreground_color: Some(TermColor::White),
					background_color: Some(TermColor::DarkGreen),
					underline_color: None,
					attributes: Default::default(),
				},
				invalid_style: ContentStyle {
					foreground_color: Some(TermColor::White),
					background_color: Some(TermColor::DarkYellow),
					underline_color: None,
					attributes: Default::default(),
				},
			},
			tiles: [
				[
					Some(Tile(Rook, White)),
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
			cursor_pos: Pos { x: 4, y: 0 },
			cursor_style_alt: false,
			selected: None,
			player: White,
			checked_by: vec![],
			invalid: None,
		}
	}
}

impl Widget for Board {
	fn size(&self) -> Size {
		Size { width: 18, height: 9 }
	}
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, mut line_nb: u16) -> std::fmt::Result {
		if line_nb == 8 {
			f.write_char(' ')?;
			f.write_char(' ')?;
			let write_column = |letter| {
				f.write_char(letter)?;
				f.write_char(' ')
			};
			let mut col_names = 'A'..='H';
			if self.rotated {
				col_names.rev().try_for_each(write_column)?
			} else {
				col_names.try_for_each(write_column)?
			};
		} else {
			if !self.rotated {
				// The bord begins at bottom left
				line_nb = 7 - line_nb;
			}
			let line = &self.tiles[line_nb as usize];
			f.write_str(&(line_nb + 1).to_string())?;
			f.write_char(' ')?;

			let selected_style =
				if self.cursor_style_alt { None } else { Some(&self.style.select_style) };

			#[allow(clippy::if_same_then_else)]
			let write_tile = |(i, tile): (usize, &Option<Tile>)| {
				let pos = Pos { x: i as u16, y: line_nb };
				let style = if pos == self.cursor_pos && selected_style.is_some() {
					selected_style.unwrap()
				} else if !self.checked_by.is_empty()
					&& *tile == Some(Tile(Piece::King, self.player))
				{
					&self.style.checked_tile_style
				} else if self.checked_by.contains(&pos) {
					&self.style.checked_tile_style
				} else if self.selected == Some(pos) {
					&self.style.selected_style
				} else if self.invalid.map_or(false, |(p0, p1)| p0 == pos || p1 == pos) {
					&self.style.invalid_style
				} else if (line_nb as usize + i) % 2 == 0 {
					&self.style.light_tile_style
				} else {
					&self.style.dark_tile_style
				};
				write!(
					f,
					"{}",
					(*style).apply(match tile {
						None => "  ".to_owned(),
						Some(t) => t.0.to_char(t.1).to_string() + " ",
					})
				)
			};
			let mut line_iter = line.iter().enumerate();
			if self.rotated {
				line_iter.rev().try_for_each(write_tile)?
			} else {
				line_iter.try_for_each(write_tile)?
			};
		}
		Ok(())
	}
}

impl EventBubbling for Board {
	type FinalData<'a> = Option<(Pos, Option<Tile>)>;

	fn bubble_event<
		'a,
		R,
		F: FnOnce(Self::FinalData<'a>, terminity::widgets::BubblingEvent) -> R,
	>(
		&'a mut self,
		event: terminity::widgets::BubblingEvent,
		callback: F,
	) -> R {
		// NB: the event will be filtered and re-indexed by the wrapping Auto-Padder
		let pos = event.pos();
		let mut row = pos.line;
		let mut column = pos.column / 2;
		if !(1..=8).contains(&column) || row >= 8 {
			return callback(None, event.bubble_at(pos));
		}
		if self.rotated {
			column = 8 - column;
		} else {
			column -= 1;
			row = 7 - row;
		}
		let new_pos = Pos { x: column as u16, y: row as u16 };
		callback(Some((new_pos, self[new_pos])), event.bubble_at(Position { line: row, column }))
	}
}
