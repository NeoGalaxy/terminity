use crate::{
	events::Position,
	widgets::{
		self,
		positionning::{Positionning, Spacing},
		AsIndexedIterator, AsWidget, EventBubbling,
	},
};
use std::ops::{Deref, DerefMut};

use crate::Size;

use super::Widget;

macro_rules! setters_getters {
	($field_name:ident: $field_ty:ty $(, $($others:tt)*)?) => {
		setters_getters!{$($($others)*)?}

		concat_idents::concat_idents!(with_name = with_, $field_name {
			pub fn with_name(mut self, val: $field_ty) -> Self {
				self.$field_name = val;
				self
			}
		});
		concat_idents::concat_idents!(set_name = set_, $field_name {
			pub fn set_name(&mut self, val: $field_ty) {
				self.$field_name = val;
			}
		});

		pub fn $field_name(&self) -> $field_ty {
			self.$field_name
		}
	};
	() => {}
}

macro_rules! div {
	($name:ident $(,($windex:tt: $wty:ident))* $(,)?) => {
		#[derive(Debug, Clone, Copy)]
		pub struct $name<$($wty),*> {
			pub widgets: ($($wty,)*),
			pub horizontal: bool,
			pub content_alignment: Positionning,
			pub content_pos: Positionning,
			pub min_height: Option<u16>,
			pub min_width: Option<u16>,
			pub max_height: Option<u16>,
			pub max_width: Option<u16>,
		}
		impl<$($wty: crate::widgets::AsWidget),*> $name<$($wty),*> {

			#[allow(clippy::too_many_arguments)]
			pub fn new($(
				concat_idents::concat_idents!(field = widget, $windex {
					field
				}): $wty,
			)*) -> Self {
				Self {
					widgets: ($(concat_idents::concat_idents!(field = widget, $windex {
						field
					}),)*),
					horizontal: false,
					content_alignment: Positionning::Start,
					content_pos: Positionning::Start,
					min_height: None,
					min_width: None,
					max_height: None,
					max_width: None,
				}
			}

			setters_getters!{
				horizontal: bool,
				content_pos: Positionning,
				content_alignment: Positionning,
				min_height: Option<u16>,
				min_width: Option<u16>,
				max_height: Option<u16>,
				max_width: Option<u16>,
			}

			pub fn with_max_size(mut self, val: Size) -> Self {
				self.set_max_size(val);
				self
			}

			pub fn with_min_size(mut self, val: Size) -> Self {
				self.set_min_size(val);
				self
			}

			pub fn with_exact_size(mut self, val: Size) -> Self {
				self.set_exact_size(val);
				self
			}

			pub fn set_max_size(&mut self, val: Size) {
				self.max_width = Some(val.width);
				self.max_height = Some(val.height);
			}

			pub fn set_min_size(&mut self, val: Size) {
				self.min_width = Some(val.width);
				self.min_height = Some(val.height);
			}

			pub fn set_exact_size(&mut self, val: Size) {
				self.set_max_size(val);
				self.set_min_size(val);
			}
		}

		concat_idents::concat_idents!(D = $name, Widget {
			pub struct D<$($wty),*> {
				lines_data: Vec<(u8, (u16, u16), Option<u16>)>,
				widgets: ($($wty,)*),
				start_padding: u16,
				end_padding: u16,
				horizontal: bool,
				size: Size,
			}
			use D as DivWidget;
		});

		impl<$($wty: crate::widgets::AsWidget),*> crate::widgets::AsWidget for $name<$($wty),*> {
				type WidgetType<'a> = DivWidget<$($wty::WidgetType<'a>),*> where Self: 'a;

				fn as_widget(&mut self) -> <Self as widgets::AsWidget>::WidgetType<'_> {
					let (lines_data, widgets, start_padding, end_padding, size) = if self.horizontal {
						let mut tot_width = 0;
						let mut max_height = 0;
						let widgets = ($(
							{
								let w = self.widgets.$windex.as_widget();
								tot_width += w.size().width;
								max_height = max_height.max(w.size().height);
								w
							},
						)*);
						let size = Size {
							width: tot_width
								.min(self.max_width.unwrap_or(tot_width))
								.max(self.min_width.unwrap_or(tot_width)),
							height: max_height
								.min(self.max_height.unwrap_or(max_height))
								.max(self.min_height.unwrap_or(max_height)),
						};
						let w_padding = size.width - tot_width;
						let (left_pad, right_pad) = match self.content_pos {
							Positionning::Start => (w_padding, 0),
							Positionning::Center => (w_padding / 2, w_padding - w_padding / 2),
							Positionning::End => (0, w_padding),
						};

						let lines_data = vec![$(
							{
								let w = &widgets.$windex;
								let padding = size.height - w.size().height;
								let (top_pad, bot_pad) = match self.content_alignment {
									Positionning::Start => (padding, 0),
									Positionning::Center => (padding / 2, padding - padding / 2),
									Positionning::End => (0, padding),
								};
								($windex, (top_pad, bot_pad), None)
							},
						)*];
						(lines_data, widgets, left_pad, right_pad, size)
					} else {
						let mut max_width = 0;
						let mut tot_height = 0;
						let widgets = ($(
							{
								let w = self.widgets.$windex.as_widget();
								tot_height += w.size().height;
								max_width = max_width.max(w.size().width);
								w
							},
						)*);
						let size = Size {
							width: max_width
								.min(self.max_width.unwrap_or(max_width))
								.max(self.min_width.unwrap_or(max_width)),
							height: tot_height
								.min(self.max_height.unwrap_or(tot_height))
								.max(self.min_height.unwrap_or(tot_height)),
						};
						let h_padding = size.height - tot_height;
						let (top_pad, bot_pad) = match self.content_pos {
							Positionning::Start => (h_padding, 0),
							Positionning::Center => (h_padding / 2, h_padding - h_padding / 2),
							Positionning::End => (0, h_padding),
						};


						let lines_data = [$((
								$windex,
								size.width - widgets.$windex.size().width,
								widgets.$windex.size().height
						)),*].into_iter().flat_map(|(i, padding, widget_height)| {
							let (left_pad, right_pad) = match self.content_alignment {
								Positionning::Start => (padding, 0),
								Positionning::Center => (padding / 2, padding - padding / 2),
								Positionning::End => (0, padding),
							};
							(0..widget_height).map(move |l| (i, (left_pad, right_pad), Some(l)))
						}).collect();
						(lines_data, widgets, top_pad, bot_pad, size)
					};
					DivWidget {
						lines_data,
						widgets,
						start_padding,
						end_padding,
						horizontal: self.horizontal,
						size,
					}
				}
		}

		impl<$($wty: crate::widgets::Widget),*> DivWidget<$($wty),*> {
			fn widget_size(&self, i: u8) -> Size {
				match i {
					$($windex => self.widgets.$windex.size(),)*
					_ => panic!("There's no widget n°{i} on a {}", stringify!($name)),
				}
			}
			fn widget_display_line(&self, i: u8, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result  {
				match i {
					$($windex => self.widgets.$windex.display_line(f, line),)*
					_ => panic!("There's no widget n°{i} on a {}", stringify!($name)),
				}
			}
		}

		impl<$($wty: crate::widgets::Widget),*> crate::widgets::Widget for DivWidget<$($wty),*> {
			fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
				if self.horizontal {
					Spacing::line(self.start_padding).display_line(f, line)?;
					for (i, (top_pad, bot_pad), _) in &self.lines_data {
						if line < *top_pad || line > self.size.height - bot_pad {
							Spacing::line(self.widget_size(*i).width).display_line(f, line)?;
						} else {
							self.widget_display_line(*i, f, line - top_pad)?;
						}
					}
					Spacing::line(self.end_padding).display_line(f, line)?;
				} else if line < self.start_padding || line > self.size.height - self.end_padding {
					Spacing::line(self.size.width).display_line(f, line)?;
				} else {
					let (i, (left_pad, right_pad), Some(w_line)) =
						&self.lines_data[line as usize - self.start_padding as usize]
					else {
						panic!("Internal error: vertical Div has no index value");
					};
					Spacing::line(*left_pad).display_line(f, line)?;
					self.widget_display_line(*i, f, *w_line)?;
					Spacing::line(*right_pad).display_line(f, line)?;
				}
				Ok(())
			}
			fn size(&self) -> Size {
				self.size
			}
		}

		concat_idents::concat_idents!(E = $name, WidgetElement {
			pub enum E<$($wty),*> {
				$($wty($wty)),*
			}
			use E as DivWidgetElement;
		});

		impl<$($wty: Widget + EventBubbling),*> EventBubbling for DivWidget<$($wty),*> {
			type FinalData<'a> = Option<DivWidgetElement<$($wty::FinalData<'a>),*>> where Self: 'a;

			fn bubble_event<'a, R, F: FnOnce(Self::FinalData<'a>, crate::widgets::BubblingEvent) -> R>(
				&'a mut self,
				event: crate::widgets::BubblingEvent,
				callback: F,
			) -> R {
				if self.horizontal {
					if !(self.start_padding as i16..(self.size.width - self.end_padding) as i16)
						.contains(&(event.pos().column))
					{
						return callback(None, event);
					}
					let x_pos = self.start_padding;
					for (i, (t_padd, l_padd), _) in &self.lines_data {
						if (x_pos + self.widget_size(*i).width) as i16 > event.pos().column {
							if (*t_padd as i16..(self.size.height - l_padd) as i16)
								.contains(&event.pos().line)
							{
								match i {
									$($windex => {
										return self.widgets.$windex.bubble_event(
											event.bubble_at(Position { line: *t_padd as i16, column: x_pos as i16 }),
											|a, evt| callback(Some(DivWidgetElement::$wty(a)), evt),
										);
									})*
									_ => panic!("No widget of index {i}"),
								}
							} else {
								return callback(None, event);
							}
						}
					}
					callback(None, event)
				} else {
					if !(self.start_padding as i16..(self.size.width - self.end_padding) as i16)
						.contains(&(event.pos().line))
					{
						return callback(None, event);
					}
					let (i, padding, Some(widget_line)) =
						&self.lines_data[event.pos().line as usize - self.start_padding as usize]
					else {
						panic!("Internal error: horizontal Coll widget has no widget line number")
					};

					if (padding.0 as i16..(self.size.width - padding.1) as i16)
						.contains(&event.pos().column)
					{
						callback(None, event)
					} else {
						match i {
							$($windex => {
								let bubble_pos = Position {
									line: event.pos().line - *widget_line as i16,
									column: padding.0 as i16,
								};
								self.widgets.$windex.bubble_event(
									event.bubble_at(bubble_pos), |a, evt| callback(Some(DivWidgetElement::$wty(a)), evt)
								)
							})*
							_ => panic!("No widget of index {i}"),
						}

					}
				}
			}
		}
	}
}

mod div1 {
	use super::*;
	div!(Div1, (0: W0));
}
pub use div1::*;
mod div2 {
	use super::*;
	div!(Div2, (0: W0), (1: W1));
}
pub use div2::*;

mod div3 {
	use super::*;
	div!(Div3, (0: W0), (1: W1), (2: W2));
}
pub use div3::*;

mod div4 {
	use super::*;
	div!(Div4, (0: W0), (1: W1), (2: W2), (3: W3));
}
pub use div4::*;

mod div5 {
	use super::*;
	div!(Div5, (0: W0), (1: W1), (2: W2), (3: W3), (4: W4));
}
pub use div5::*;

mod div6 {
	use super::*;
	div!(Div6, (0: W0), (1: W1), (2: W2), (3: W3), (4: W4),(5: W5));
}
pub use div6::*;

mod div7 {
	use super::*;
	div!(Div7, (0: W0), (1: W1), (2: W2), (3: W3), (4: W4),(5: W5), (6: W6));
}
pub use div7::*;

mod div8 {
	use super::*;
	div!(Div8, (0: W0), (1: W1), (2: W2), (3: W3), (4: W4),(5: W5), (6: W6), (7: W7));
}
pub use div8::*;

mod div9 {
	use super::*;
	div!(Div9, (0: W0), (1: W1), (2: W2), (3: W3), (4: W4),(5: W5), (6: W6), (7: W7), (8: W8));
}
pub use div9::*;

mod div10 {
	use super::*;
	div!(Div10, (0: W0), (1: W1), (2: W2), (3: W3), (4: W4), (5: W5), (6: W6), (7: W7), (8: W8),
	            (9: W9));
}
pub use div10::*;

mod div11 {
	use super::*;
	div!(Div11, (0: W0), (1: W1), (2: W2), (3: W3), (4: W4), (5: W5), (6: W6), (7: W7), (8: W8),
	            (9: W9), (10: W10));
}
pub use div11::*;

mod div12 {
	use super::*;
	div!(Div12, (0: W0), (1: W1), (2: W2), (3: W3), (4: W4), (5: W5), (6: W6), (7: W7), (8: W8),
	            (9: W9), (10: W10), (11: W11));
}
pub use div12::*;

#[derive(Debug, Clone, Copy)]
pub struct CollDiv<Coll>
where
	Coll: AsIndexedIterator,
	for<'a> Coll::Value: AsWidget,
{
	collection: Coll,
	pub horizontal: bool,
	pub content_alignment: Positionning,
	pub content_pos: Positionning,
	pub min_height: Option<u16>,
	pub min_width: Option<u16>,
	pub max_height: Option<u16>,
	pub max_width: Option<u16>,
}

impl<Coll> CollDiv<Coll>
where
	Coll: AsIndexedIterator,
	for<'a> Coll::Value: AsWidget,
{
	pub fn new(horizontal: bool, coll: Coll) -> Self {
		Self {
			collection: coll,
			horizontal,
			content_alignment: Positionning::Start,
			content_pos: Positionning::Start,
			min_height: None,
			min_width: None,
			max_height: None,
			max_width: None,
		}
	}

	pub fn collection(&self) -> &Coll {
		&self.collection
	}

	pub fn collection_mut(&mut self) -> &mut Coll {
		&mut self.collection
	}

	setters_getters! {
		content_alignment: Positionning,
		content_pos: Positionning,
		min_height: Option<u16>,
		min_width: Option<u16>,
		max_height: Option<u16>,
		max_width: Option<u16>,
	}

	pub fn with_max_size(mut self, val: Size) -> Self {
		self.set_max_size(val);
		self
	}

	pub fn with_min_size(mut self, val: Size) -> Self {
		self.set_min_size(val);
		self
	}

	pub fn with_exact_size(mut self, val: Size) -> Self {
		self.set_exact_size(val);
		self
	}

	pub fn set_max_size(&mut self, val: Size) {
		self.max_width = Some(val.width);
		self.max_height = Some(val.height);
	}

	pub fn set_min_size(&mut self, val: Size) {
		self.min_width = Some(val.width);
		self.min_height = Some(val.height);
	}

	pub fn set_exact_size(&mut self, val: Size) {
		self.set_max_size(val);
		self.set_min_size(val);
	}
}

impl<Coll> Deref for CollDiv<Coll>
where
	Coll: AsIndexedIterator,
	for<'a> Coll::Value: AsWidget,
{
	// add code here
	type Target = Coll;
	fn deref(&self) -> &<Self as std::ops::Deref>::Target {
		&self.collection
	}
}

impl<Coll> DerefMut for CollDiv<Coll>
where
	Coll: AsIndexedIterator,
	for<'a> Coll::Value: AsWidget,
{
	// add code here
	fn deref_mut(&mut self) -> &mut <Self as std::ops::Deref>::Target {
		&mut self.collection
	}
}

impl<Coll> crate::widgets::AsWidget for CollDiv<Coll>
where
	Coll: AsIndexedIterator,
	for<'a> Coll::Value: AsWidget,
{
	type WidgetType<'a> =
		CollDivWidget<Coll::Index<'a>, <Coll::Value as AsWidget>::WidgetType<'a>> where Self: 'a;

	fn as_widget(&mut self) -> Self::WidgetType<'_> {
		let (lines_data, widgets, start_padding, end_padding, size) = if self.horizontal {
			let mut tot_width = 0;
			let mut max_height = 0;
			let widgets: Vec<_> = self
				.collection
				.as_iterator()
				.map(|(k, w)| {
					let w = (*w).as_widget();
					tot_width += w.size().width;
					max_height = max_height.max(w.size().height);
					(k, w)
				})
				.collect();
			let size = Size {
				width: tot_width
					.min(self.max_width.unwrap_or(tot_width))
					.max(self.min_width.unwrap_or(tot_width)),
				height: max_height
					.min(self.max_height.unwrap_or(max_height))
					.max(self.min_height.unwrap_or(max_height)),
			};
			let w_padding = size.width - tot_width;
			let (left_pad, right_pad) = match self.content_pos {
				Positionning::Start => (w_padding, 0),
				Positionning::Center => (w_padding / 2, w_padding - w_padding / 2),
				Positionning::End => (0, w_padding),
			};

			let (lines_data, widget_list) = widgets
				.into_iter()
				.enumerate()
				.map(|(i, (k, w))| {
					let padding = size.height - w.size().height;
					let (top_pad, bot_pad) = match self.content_alignment {
						Positionning::Start => (padding, 0),
						Positionning::Center => (padding / 2, padding - padding / 2),
						Positionning::End => (0, padding),
					};
					((i, (top_pad, bot_pad), None), (k, w))
				})
				.unzip();
			(lines_data, widget_list, left_pad, right_pad, size)
		} else {
			let mut max_width = 0;
			let mut tot_height = 0;
			let widgets: Vec<_> = self
				.collection
				.as_iterator()
				.map(|(k, w)| {
					let w = (*w).as_widget();
					max_width = max_width.max(w.size().width);
					tot_height += w.size().height;
					(k, w)
				})
				.collect();
			let size = Size {
				width: max_width
					.min(self.max_width.unwrap_or(max_width))
					.max(self.min_width.unwrap_or(max_width)),
				height: tot_height
					.min(self.max_height.unwrap_or(tot_height))
					.max(self.min_height.unwrap_or(tot_height)),
			};
			let h_padding = size.height - tot_height;
			let (top_pad, bot_pad) = match self.content_pos {
				Positionning::Start => (h_padding, 0),
				Positionning::Center => (h_padding / 2, h_padding - h_padding / 2),
				Positionning::End => (0, h_padding),
			};

			let mut widget_list = vec![];

			let lines_data = widgets
				.into_iter()
				.flat_map(|(k, w)| {
					let i = widget_list.len();
					let padding = size.width - w.size().width;
					let (left_pad, right_pad) = match self.content_alignment {
						Positionning::Start => (padding, 0),
						Positionning::Center => (padding / 2, padding - padding / 2),
						Positionning::End => (0, padding),
					};
					let widget_height = w.size().height;
					widget_list.push((k, w));
					(0..widget_height).map(move |l| (i, (left_pad, right_pad), Some(l)))
				})
				.collect();
			(lines_data, widget_list, top_pad, bot_pad, size)
		};
		CollDivWidget {
			lines_data,
			widgets,
			start_padding,
			end_padding,
			horizontal: self.horizontal,
			size,
		}
	}
}

pub struct CollDivWidget<K, W: Widget> {
	lines_data: Vec<(usize, (u16, u16), Option<u16>)>,
	widgets: Vec<(K, W)>,
	start_padding: u16,
	end_padding: u16,
	horizontal: bool,
	size: Size,
}

impl<K, W: Widget> Widget for CollDivWidget<K, W> {
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
		if self.horizontal {
			Spacing::line(self.start_padding).display_line(f, line)?;
			for (i, (top_pad, bot_pad), _) in &self.lines_data {
				let w = &self.widgets[*i].1;
				if line < *top_pad || line > self.size.height - bot_pad {
					Spacing::line(w.size().width).display_line(f, line)?;
				} else {
					w.display_line(f, line - top_pad)?;
				}
			}
			Spacing::line(self.end_padding).display_line(f, line)?;
		} else if line < self.start_padding || line > self.size.height - self.end_padding {
			Spacing::line(self.size.width).display_line(f, line)?;
		} else {
			let (i, (left_pad, right_pad), Some(w_line)) =
				&self.lines_data[line as usize - self.start_padding as usize]
			else {
				panic!("Internal error: vertical Div has no index value");
			};
			Spacing::line(*left_pad).display_line(f, line)?;
			self.widgets[*i].1.display_line(f, *w_line)?;
			Spacing::line(*right_pad).display_line(f, line)?;
		}
		Ok(())
	}

	fn size(&self) -> Size {
		self.size
	}
}

impl<K, W: Widget + EventBubbling> EventBubbling for CollDivWidget<K, W> {
	type FinalData<'a> = Option<(&'a K, W::FinalData<'a>)> where Self: 'a;

	fn bubble_event<'a, R, F: FnOnce(Self::FinalData<'a>, crate::widgets::BubblingEvent) -> R>(
		&'a mut self,
		event: crate::widgets::BubblingEvent,
		callback: F,
	) -> R {
		if self.horizontal {
			if !(self.start_padding as i16..(self.size.width - self.end_padding) as i16)
				.contains(&(event.pos().column))
			{
				return callback(None, event);
			}
			let x_pos = self.start_padding;
			for (i, (t_padd, l_padd), _) in &self.lines_data {
				let (_, w) = &self.widgets[*i];
				if (x_pos + w.size().width) as i16 > event.pos().column {
					let (k, w) = &mut self.widgets[*i];
					if (*t_padd as i16..(self.size.height - l_padd) as i16)
						.contains(&event.pos().line)
					{
						return w.bubble_event(
							event
								.bubble_at(Position { line: *t_padd as i16, column: x_pos as i16 }),
							|a, evt| callback(Some((k, a)), evt),
						);
					} else {
						return callback(None, event);
					}
				}
			}
			callback(None, event)
		} else {
			if !(self.start_padding as i16..(self.size.width - self.end_padding) as i16)
				.contains(&(event.pos().line))
			{
				return callback(None, event);
			}
			let (i, padding, Some(widget_line)) =
				&self.lines_data[event.pos().line as usize - self.start_padding as usize]
			else {
				panic!("Internal error: horizontal Coll widget has no widget line number")
			};

			if (padding.0 as i16..(self.size.width - padding.1) as i16)
				.contains(&event.pos().column)
			{
				callback(None, event)
			} else {
				let (k, w) = &mut self.widgets[*i];
				let bubble_pos = Position {
					line: event.pos().line - *widget_line as i16,
					column: padding.0 as i16,
				};

				w.bubble_event(event.bubble_at(bubble_pos), |a, evt| callback(Some((k, a)), evt))
			}
		}
	}
}
