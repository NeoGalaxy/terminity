use std::fmt::Write;

use serde::{Deserialize, Serialize};

use crate::Size;

use super::{EventBubblingWidget, Widget};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Position {
	Start,
	Center,
	End,
}

macro_rules! div {
	($name:ident $(,($wfield:ident: $wty:ident))* $(,)?) => {
		#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
		pub struct $name<$($wty),*> {
			$(pub $wfield: $wty,)*
			pub horizontal: bool,
			pub content_alignment: Position,
			pub content_pos: Position,
			pub size: Size,
		}

		impl<$($wty: crate::widgets::Widget),*> crate::widgets::Widget for $name<$($wty),*> {
			fn display_line(&self, f: &mut std::fmt::Formatter<'_>, mut line: u16) -> std::fmt::Result {
				#[allow(unused_assignments)]
				if self.horizontal {
					todo!()
				} else {
					let content_height = 0 $(+ self.$wfield.size().height)*;
					let padding = self.size.height - content_height;
					let top_pad = match self.content_pos {
						Position::Start => 0,
						Position::Center => padding / 2,
						Position::End => padding,
					};
					if line < top_pad {
						return Spacing::line(self.size().width).display_line(f, 0);
					}
					line -= top_pad;

					$(
						if line < self.$wfield.size().height {
							let w_pad = self.size.width - self.$wfield.size().width;
							let (l_pad, r_pad) = match self.content_alignment {
								Position::Start => (0, w_pad),
								Position::Center => {
									let tmp = w_pad / 2;
									(tmp, w_pad - tmp)
								}
								Position::End => (w_pad, 0),
							};
							Spacing::line(l_pad).display_line(f, 0)?;
							self.$wfield.display_line(f, line)?;
							Spacing::line(r_pad).display_line(f, 0)?;
							return Ok(());
						}

						line -= self.$wfield.size().height;
					)*
					Spacing::line(self.size().width).display_line(f, 0)?;
				}
				Ok(())
			}

			fn size(&self) -> Size {
				self.size
			}
		}
	}
}

div!(Div1, (widget0: W0));
div!(Div2, (widget0: W0), (widget1: W1));
div!(Div3, (widget0: W0), (widget1: W1), (widget2: W2));

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Clip<W> {
	pub widget: W,
	pub size: Size,
	pub v_pos: Position,
	pub h_pos: Position,
}

impl<W: Widget> Widget for Clip<W> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, mut line: u16) -> std::fmt::Result {
		let content_height = self.widget.size().height;

		// Padding may be negative
		let padding = self.size.height as i16 - content_height as i16;

		let top_pad = match self.v_pos {
			Position::Start => 0,
			Position::Center => padding / 2,
			Position::End => padding,
		};
		if (line as i16) < top_pad {
			return Spacing::line(self.size().width).display_line(f, 0);
		}
		line = (line as i16 - top_pad).try_into().unwrap();

		if line < self.widget.size().height {
			let w_pad = self.size.width as i16 - self.widget.size().width as i16;
			let (l_pad, r_pad) = match self.h_pos {
				Position::Start => (0, w_pad),
				Position::Center => {
					let tmp = w_pad / 2;
					(tmp, w_pad - tmp)
				}
				Position::End => (w_pad, 0),
			};
			if l_pad < 0 {
				self.widget.display_line_in(f, line, (-l_pad as u16)..self.size.width)?;
			} else {
				Spacing::line(l_pad as u16).display_line(f, 0)?;
				self.widget.display_line(f, line)?;
			}
			if r_pad > 0 {
				Spacing::line(r_pad as u16).display_line(f, 0)?;
			}
			return Ok(());
		}

		Spacing::line(self.size().width).display_line(f, 0)?;
		Ok(())
	}

	fn size(&self) -> Size {
		self.size
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Spacing {
	pub size: Size,
}

impl Spacing {
	pub fn line(len: u16) -> Self {
		Self { size: Size { width: len, height: 1 } }
	}
}

impl Widget for Spacing {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, _: u16) -> std::fmt::Result {
		for _ in 0..self.size.width {
			f.write_char(' ')?;
		}
		Ok(())
	}

	fn size(&self) -> Size {
		self.size
	}
}

impl EventBubblingWidget for Spacing {
	type FinalWidgetData<'a> = ();

	fn bubble_event<'a, R, F: FnOnce(Self::FinalWidgetData<'a>, super::BubblingEvent) -> R>(
		&'a mut self,
		event: super::BubblingEvent,
		callback: F,
	) -> R {
		callback((), event)
	}
}
