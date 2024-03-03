use std::sync::Arc;
use terminity::{game::GameContext, widgets::AsWidget, wstr, Size};

use crate::game_handling::{GameCommands, GameHandle, GameLib};
use ouroboros::self_referencing;
use terminity::{
	events::Event,
	img,
	widget_string::WidgetString,
	widgets::positionning::{div::Div3, Clip, Positionning, Spacing},
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
	pub fn open(lib: Arc<GameLib>, init_size: Size) -> Self {
		let (snd, rcv) = kanal::bounded(516);
		GameScreenBuilder {
			lib,
			game_builder: |lib: &Arc<GameLib>| unsafe { lib.start(rcv, init_size).unwrap() },
			display: WidgetString::from(wstr!("")),
			events: snd,
		}
		.build()
	}

	pub(crate) fn update<Ctx: GameContext>(&mut self, ctx: Ctx, size: Size) -> GameCommands {
		for e in ctx.events() {
			let _ = self.with_events_mut(|events| events.try_send(e));
		}
		let res = self.with_game_mut(|g| g.tick());
		if let Some(display) = res.1 {
			let mut clip_size = size;
			clip_size.height -= 2;
			ctx.display(
				&Div3::new(
					img!("Running Game"),
					Spacing::line(size.width).with_char('-'),
					Clip {
						widget: display,
						size: clip_size,
						v_pos: Positionning::Center,
						h_pos: Positionning::Center,
					},
				)
				.with_content_alignment(Positionning::Center)
				.with_content_pos(Positionning::Start)
				.with_exact_size(size)
				.as_widget(),
			)
		}
		res.0
	}

	pub(crate) fn finish(mut self) -> Arc<GameLib> {
		self.with_game_mut(|g| g.close_save());
		self.into_heads().lib
	}
}
