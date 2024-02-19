use std::collections::VecDeque;
use std::fmt::{Debug, Display, Write as _};
use terminity::events::Event;
use terminity::game::WidgetDisplayer;
use terminity::{build_game, widgets, Size};
use terminity::{events::EventPoller, game::Game};

struct DebugAsDisplay<T: Debug>(T);

impl<T: Debug> Display for DebugAsDisplay<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0.fmt(f)
	}
}

impl Game for HelloWorld {
	type DataInput = ();
	type DataOutput = ();

	fn start(_data: Option<Self::DataInput>, _size: Size) -> Self {
		HelloWorld { events: VecDeque::with_capacity(40), frame: 0 }
	}

	fn disp<D: WidgetDisplayer>(&mut self, displayer: D) {
		self.frame += 1;
		let mut array: [String; 30] = Default::default();
		array[0] = "#*#*------------------------".into();
		array[1] = "".into();
		array[2] = "         Hello world!".into();
		array[3] = "".into();
		array[4] = "#*#*------------------------".into();
		array[5] = format!("                                 frame: {:?}", self.frame);
		let mut n = 0;
		while self.events.front().map_or(false, |v| v.1 > 50) {
			n += 1;
			self.events.pop_front().unwrap();
		}
		array[6] =
			format!("                              cleaned: {n}, size: {}", self.events.len());
		for (i, array_buf) in array.iter_mut().skip(7).enumerate() {
			let Some((event, nb_iter)) = self.events.get_mut(i) else {
				break;
			};
			*nb_iter += 1;
			write!(array_buf, "{:?}", event).unwrap();
		}
		displayer.run(&widgets::text::Text::new(array, 30));
	}

	fn update<E: EventPoller>(&mut self, events: E) {
		self.events.extend(events.events().map(|v| (v, 0)));
	}

	fn finish(self) -> Option<Self::DataOutput> {
		None
	}
}

struct HelloWorld {
	frame: usize,
	events: VecDeque<(Event, usize)>,
}

build_game!(HelloWorld);
