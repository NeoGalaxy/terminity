use crossterm::event::MouseEvent;

use crate::{
	events::Position,
	widgets::{EventBubblingWidget, ResizableWisget},
	Size, Widget,
};

#[derive(Debug)]
pub struct Canvas<E, BG> {
	pub background: BG,
	elements: Vec<CanvasEntity<E>>,
}

#[derive(Debug)]
pub struct CanvasPos {
	pub x: i32,
	pub y: i32,
}

impl From<(i32, i32)> for CanvasPos {
	fn from(value: (i32, i32)) -> Self {
		Self { x: value.0, y: value.1 }
	}
}
impl From<(u32, u32)> for CanvasPos {
	fn from(value: (u32, u32)) -> Self {
		Self { x: value.0 as i32, y: value.1 as i32 }
	}
}
impl From<(usize, usize)> for CanvasPos {
	fn from(value: (usize, usize)) -> Self {
		Self { x: value.0 as i32, y: value.1 as i32 }
	}
}

#[derive(Debug)]
struct CanvasEntity<W> {
	pos: CanvasPos,
	widget: W,
}

impl<E: Widget, BG: Widget> Canvas<E, BG> {
	pub fn new(bg: BG) -> Self {
		Self { background: bg, elements: vec![] }
	}
	pub fn fill<P, I>(mut self, elements: I) -> Self
	where
		P: Into<CanvasPos>,
		I: IntoIterator<Item = (P, E)>,
	{
		self.elements.extend(
			elements.into_iter().map(|(pos, elm)| CanvasEntity { pos: pos.into(), widget: elm }),
		);
		self
	}
	pub fn add_entity<P: Into<CanvasPos>>(&mut self, entity: E, pos: P) {
		self.elements.push(CanvasEntity { pos: pos.into(), widget: entity })
	}
}

impl<E: Widget, BG: Widget> Widget for Canvas<E, BG> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		self.background.display_line(f, line)?;
		for elm in &self.elements {
			let elm_size = elm.widget.size();
			let elm_line = line as i32 - elm.pos.y;
			if elm_line >= 0 || (elm_line) < elm_size.height as i32 {
				elm.widget.display_line(f, elm_line as u16)?;

				// If pos < 0, then clip
				let x_start = 0.max(-elm.pos.x);

				// If pos + size > self.size, then clip
				let x_end = self
					.size()
					.width
					.min(0.max(elm.pos.x + elm_size.width as i32).try_into().unwrap());

				elm.widget.display_line_in(
					f,
					elm_line as u16,
					x_start.try_into().unwrap()..x_end,
				)?;
			}
		}
		Ok(())
	}
	fn size(&self) -> Size {
		self.background.size()
	}
	// TODO: display_line_in
}

pub enum CanvasEvent<EEvt, BGEvt> {
	Entity(EEvt),
	Background(BGEvt),
}

// impl<E: EventBubblingWidget, BG: EventBubblingWidget> EventBubblingWidget for Canvas<E, BG> {
// 	type FinalWidgetData<'a> = ();
// 	/// Handles a mouse event. see the [trait](Self)'s doc for more details.
// 	fn bubble_event<'a, R, F: FnOnce(Self::FinalWidgetData<'a>) -> R>(
// 		&'a mut self,
// 		event: crossterm::event::MouseEvent,
// 		widget_pos: Position,
// 		callback: F,
// 	) -> R {
// 		todo!()
// 		// let MouseEvent { column, row, kind, modifiers } = event;
// 		// for elm in &mut self.elements {
// 		// 	let w_size = elm.widget.size();
// 		// 	let end = (elm.pos.x + w_size.0 as i32, elm.pos.y + w_size.1 as i32);
// 		// 	if (elm.pos.x..end.0).contains(&(row as i32))
// 		// 		&& (elm.pos.x..end.0).contains(&(column as i32))
// 		// 	{
// 		// 		return CanvasEvent::Entity(elm.widget.bubble_event(MouseEvent {
// 		// 			row: row - elm.pos.x as u16,
// 		// 			column: column - elm.pos.y as u16,
// 		// 			kind,
// 		// 			modifiers,
// 		// 		}));
// 		// 	}
// 		// }
// 		// CanvasEvent::Background(self.background.bubble_event(MouseEvent {
// 		// 	row,
// 		// 	column,
// 		// 	kind,
// 		// 	modifiers,
// 		// }))
// 	}
// }

impl<E, BG: ResizableWisget> ResizableWisget for Canvas<E, BG> {
	fn resize(&mut self, size: Size) {
		self.background.resize(size)
	}
}
