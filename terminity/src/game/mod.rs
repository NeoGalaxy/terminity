use std::io;

use crate::Widget;
use serde::{Deserialize, Serialize};

use crate::events::EventPoller;

pub trait Game {
	type DataInput: for<'a> Deserialize<'a>;
	type DataOutput: Serialize;

	fn start<R: io::Read>(data: Option<Self::DataInput>) -> Self;

	fn disp<D: WidgetDisplayer>(&mut self, displayer: D);

	fn update<E: EventPoller>(&mut self, events: E);

	fn finish(self) -> Option<Self::DataOutput>;
}

pub trait WidgetDisplayer {
	fn run<W: Widget>(self, widget: &W);
}

#[repr(C)]
pub struct GameData {
	pub content: *mut u8,
	pub size: u32,
	pub capacity: u32,
}
