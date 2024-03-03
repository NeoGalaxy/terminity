use crate::{
	events::{CommandEvent, Event},
	widgets::Widget,
	Size,
};
use serde::{Deserialize, Serialize};

pub trait Game {
	type DataInput: for<'a> Deserialize<'a>;
	type DataOutput: Serialize;

	fn start(data: Option<Self::DataInput>, size: Size) -> Self;

	fn update<Ctx: GameContext>(&mut self, ctx: Ctx);

	fn finish(self) -> Option<Self::DataOutput>;
}

#[repr(C)]
pub struct GameData {
	pub content: *mut u8,
	pub size: u32,
	pub capacity: u32,
}

pub trait GameContext {
	type Iter<'a>: Iterator<Item = Event> + 'a
	where
		Self: 'a;
	fn cmd(&self, command: CommandEvent);
	fn events(&self) -> Self::Iter<'_>;
	fn display<W: Widget>(&self, widget: &W);
}
