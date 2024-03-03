use std::collections::VecDeque;
use std::fmt::{Debug, Display};
use terminity::events::Event;
use terminity::widget_string::WidgetString;
use terminity::widgets::content::TextArea;

use terminity::game::{Game, GameContext};
use terminity::widgets::positionning::div::Div2;
use terminity::widgets::positionning::{Clip, Positionning};
use terminity::widgets::{AsWidget, Widget};
use terminity::{build_game, img, Size};

struct DebugAsDisplay<T: Debug>(T);

impl<T: Debug> Display for DebugAsDisplay<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl Game for HelloWorld {
	type DataInput = ();
	type DataOutput = ();

	fn start(_data: Option<Self::DataInput>, size: Size) -> Self {
		HelloWorld { events: VecDeque::with_capacity(40), frame: 0, size }
	}

	fn update<E: GameContext>(&mut self, events: E) {
		self.events.extend(events.events().map(|v| (v, 0)));

		self.frame += 1;
		let mut n = 0;
		while self.events.front().map_or(false, |v| v.1 > 50) {
			n += 1;
			self.events.pop_front().unwrap();
		}

		let top = img!(
			"#*#*------------------------",
			"                            ",
			"        Hello world!        ",
			"                            ",
			"#*#*------------------------",
		);
		let mut buffer = WidgetString::new();
		let text_size = self.size - Size { width: 0, height: top.size().height };

		buffer
			.push_in_line(
				format!(
					"   frame: {:?}, w_size: {:?}",
					self.frame,
					(text_size.width, text_size.height)
				)
				.as_str()
				.try_into()
				.unwrap(),
			)
			.newline();

		buffer.push_in_line(
			format!("   cleaned: {n}, buf_size: {}", self.events.len())
				.as_str()
				.try_into()
				.unwrap(),
		);

		for i in 0..(text_size.height - buffer.height()) {
			let Some((event, nb_iter)) = self.events.get_mut(i as usize) else {
				break;
			};
			*nb_iter += 1;
			buffer.newline().push_in_line(format!("{:?}", event).as_str().try_into().unwrap());
		}

		events.display(
			&Div2::new(top, TextArea::left(buffer).with_size(text_size))
				.with_exact_size(self.size)
				.as_widget(),
		);
	}

	fn finish(self) -> Option<Self::DataOutput> {
		None
	}
}

struct HelloWorld {
	frame: usize,
	events: VecDeque<(Event, usize)>,
	size: Size,
}

#[derive(terminity::WidgetDisplay)]
pub struct GameDisplay(pub build_game::WidgetBuffer);

impl Widget for GameDisplay {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		if self.0.is_empty() {
			return Ok(());
		}
		let bounds_index = line as usize * std::mem::size_of::<u16>();
		let bounds = unsafe {
			(
				u16::from_le_bytes([
					*self.0.content.add(bounds_index),
					*self.0.content.add(bounds_index + 1),
				]),
				u16::from_le_bytes([
					*self.0.content.add(bounds_index + 2),
					*self.0.content.add(bounds_index + 3),
				]),
			)
		};
		let content = unsafe {
			std::slice::from_raw_parts(
				self.0.content.add(bounds.0 as usize),
				(bounds.1 - bounds.0) as usize,
			)
		};
		let s = unsafe { std::str::from_utf8_unchecked(content) };
		write!(f, "{s}")
	}

	fn size(&self) -> terminity::Size {
		Size { width: self.0.width as u16, height: self.0.height as u16 }
	}
}

build_game!(HelloWorld);
