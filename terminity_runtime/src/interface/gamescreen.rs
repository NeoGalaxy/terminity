use std::{sync::Arc, time::Duration};

use crate::game_handling::{GameCommands, GameHandle, GameLib};
use ouroboros::self_referencing;
use terminity::{
	events::Event,
	img,
	widgets::{
		positionning::{
			div::{Div1, Div3},
			Clip, Position, Spacing,
		},
		WidgetString,
	},
	wstr,
};

#[self_referencing]
#[derive(Debug)]
pub struct GameScreen {
	lib: Arc<GameLib>,
	#[covariant]
	#[borrows(lib)]
	game: GameHandle<'this>,
	display: WidgetString,
	events: kanal::Sender<Event>,
}

impl GameScreen {
	pub fn open(lib: Arc<GameLib>) -> Self {
		let (snd, rcv) = kanal::bounded(516);
		GameScreenBuilder {
			lib,
			game_builder: |lib: &Arc<GameLib>| unsafe { lib.start(rcv).unwrap() },
			display: WidgetString::from(wstr!("")),
			events: snd,
		}
		.build()
	}
	pub(crate) fn disp<D: terminity::game::WidgetDisplayer>(
		&self,
		displayer: D,
		size: terminity::Size,
	) {
		if let Some(display) = self.borrow_game().display() {
			let mut clip_size = size;
			clip_size.height -= 2;
			displayer.run(
				&Div3::new(
					false,
					img!("Running Game"),
					Spacing::line(size.width).with_char('-'),
					Clip {
						widget: display,
						size: clip_size,
						v_pos: Position::Center,
						h_pos: Position::Center,
					},
				)
				.with_content_alignment(Position::Center)
				.with_content_pos(Position::Start)
				.with_forced_size(size),
			)
		}
	}

	pub(crate) fn update<P: terminity::events::EventPoller>(&mut self, poller: P) -> GameCommands {
		for e in poller.events() {
			self.with_events_mut(|events| events.try_send(e));
		}
		self.with_game_mut(|g| g.tick())
	}

	pub(crate) fn finish(mut self) -> Arc<GameLib> {
		self.with_game_mut(|g| g.close_save());
		self.into_heads().lib
	}
}