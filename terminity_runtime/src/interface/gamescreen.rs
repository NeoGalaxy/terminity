use std::sync::Arc;
use terminity::{
	build_game::WidgetBuffer,
	game::GameContext,
	widgets::{content::Img, positionning::div::Div3WidgetElement, AsWidget, EventBubbling},
	Size,
};

use crate::game_handling::{GameCommands, GameDisplay, GameHandle, GameLib};
use ouroboros::self_referencing;
use terminity::{
	events::Event,
	img,
	widgets::positionning::{div::Div3, Clip, Positionning, Spacing},
};

#[self_referencing]
#[derive(Debug)]
pub struct GameScreen {
	lib: Arc<GameLib>,
	#[covariant]
	#[borrows(lib)]
	game: GameScreenInner<'this>,
}

#[derive(Debug)]
pub struct GameScreenInner<'g> {
	game: GameHandle<'g>,
	display: Div3<Img<'static>, Spacing, Clip<GameDisplay>>,
	events: kanal::Sender<Event>,
}

impl GameScreen {
	pub fn open(lib: Arc<GameLib>, size: Size) -> Self {
		let (snd, rcv) = kanal::bounded(516);
		GameScreenBuilder {
			lib,
			game_builder: |lib: &Arc<GameLib>| GameScreenInner {
				game: unsafe { lib.start(rcv, size - Size { width: 0, height: 2 }).unwrap() },
				display: Div3::new(
					img!("Running Game"),
					Spacing::line(size.width).with_char('-'),
					Clip {
						widget: GameDisplay(WidgetBuffer::new_empty()),
						size: size - Size { width: 0, height: 2 },
						v_pos: Positionning::Center,
						h_pos: Positionning::Center,
					},
				)
				.with_content_alignment(Positionning::Center)
				.with_content_pos(Positionning::Start)
				.with_exact_size(size),
				events: snd,
			},
		}
		.build()
	}

	pub(crate) fn update<Ctx: GameContext>(&mut self, ctx: Ctx) -> GameCommands {
		self.with_game_mut(|g| g.update(ctx))
	}

	pub(crate) fn finish(mut self) {
		self.with_game_mut(|g| g.finish())
	}
}

impl GameScreenInner<'_> {
	pub(crate) fn update<Ctx: GameContext>(&mut self, ctx: Ctx) -> GameCommands {
		let mut tmp_widget = self.display.as_widget();
		for e in ctx.events() {
			let e = match e {
				Event::Mouse(mouse_e) => {
					let e = tmp_widget.bubble_event(mouse_e.into(), |d, evt| match d {
						Ok(Div3WidgetElement::W2(Ok(_))) => Some(Event::Mouse(evt.into())),
						_ => None,
					});

					if let Some(e) = e {
						e
					} else {
						continue;
					}
				}
				_ => e,
			};
			let _ = self.events.try_send(e);
		}
		let res = self.game.tick();
		if let Some(display) = res.1 {
			self.display.widgets.2.widget = display;
			ctx.display(&self.display.as_widget());
		}
		res.0
	}

	pub(crate) fn finish(&mut self) {
		self.game.close_save();
	}
}
