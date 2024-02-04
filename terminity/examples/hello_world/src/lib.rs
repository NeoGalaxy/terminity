use std::io;
use terminity::build_game;
use terminity::{EventPoller, Game};
use terminity_widgets::widgets;

impl Game for HelloWorld {
	type DataInput = ();
	type DataOutput = ();
	type WidgetKind = widgets::text::Text<1>;

	fn start<R: io::Read>(_data: Option<Self::DataInput>) -> Self {
		HelloWorld()
	}

	fn disp<F: FnOnce(&Self::WidgetKind)>(&mut self, displayer: F) {
		displayer(&widgets::text::Text::centered(["Hello world!".into()], 14))
	}

	fn update<E: EventPoller>(&mut self, _: E) {}

	fn finish(self) -> Option<Self::DataOutput> {
		None
	}
}

struct HelloWorld();

build_game!(HelloWorld);
