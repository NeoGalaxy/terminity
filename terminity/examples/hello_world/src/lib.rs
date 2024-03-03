use std::collections::VecDeque;
use std::fmt::{Debug, Display};
use terminity::events::Event;
use terminity::game::WidgetDisplayer;
use terminity::widget_string::WidgetString;
use terminity::widgets::content::TextArea;

use terminity::widgets::positionning::div::Div2;
use terminity::widgets::positionning::{Clip, Positionning};
use terminity::widgets::Widget;
use terminity::{build_game, img, Size};
use terminity::{events::GameContext, game::Game};

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

	fn disp<D: WidgetDisplayer>(&mut self, displayer: D) {
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

		displayer.run(
			&Div2::new(false, top, TextArea::left(buffer).with_size(text_size))
				.with_exact_size(self.size),
		);
	}

	fn update<E: GameContext>(&mut self, events: E) {
		self.events.extend(events.events().map(|v| (v, 0)));
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

build_game!(HelloWorld);
