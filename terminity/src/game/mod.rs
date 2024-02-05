use std::io;

use crate::Widget;
use serde::{Deserialize, Serialize};

use crate::events::EventPoller;

pub trait Game {
	type DataInput: for<'a> Deserialize<'a>;
	type DataOutput: Serialize;
	type WidgetKind: Widget;

	fn start<R: io::Read>(data: Option<Self::DataInput>) -> Self;

	fn disp<F: FnOnce(&Self::WidgetKind)>(&mut self, displayer: F);

	fn update<E: EventPoller>(&mut self, events: E);

	fn finish(self) -> Option<Self::DataOutput>;
}

#[repr(C)]
pub struct GameData {
	pub content: *mut u8,
	pub size: u32,
	pub capacity: u32,
}
