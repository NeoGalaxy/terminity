use crate::widgets::positionning::{Position, Spacing};
use std::ops::{Deref, DerefMut};

use crate::Size;

use super::Widget;

pub struct DivGuard<'a, W: Widget> {
	widget: &'a mut W,
	old_w_size: Size,
	div_size: &'a mut Size,
	horizontal: bool,
}

impl<W: Widget> Deref for DivGuard<'_, W> {
	type Target = W;

	fn deref(&self) -> &Self::Target {
		self.widget
	}
}

impl<W: Widget> DerefMut for DivGuard<'_, W> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.widget
	}
}

impl<W: Widget> Drop for DivGuard<'_, W> {
	fn drop(&mut self) {
		let new_w_size = self.widget.size();
		if self.horizontal {
			self.div_size.width =
				(self.div_size.width + new_w_size.width).saturating_sub(self.old_w_size.width);
			self.div_size.height = self.div_size.height.max(new_w_size.height);
		} else {
			self.div_size.height =
				(self.div_size.height + new_w_size.height).saturating_sub(self.old_w_size.height);
			self.div_size.width = self.div_size.width.max(new_w_size.width);
		}
	}
}

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
	($name:ident $(,($wfield:ident: $wty:ident))* $(,)?) => {
		#[derive(Debug, Clone, Copy)]
		pub struct $name<$($wty),*> {
			$($wfield: $wty,)*
			horizontal: bool,
			content_alignment: Position,
			content_pos: Position,
			min_height: Option<u16>,
			min_width: Option<u16>,
			max_height: Option<u16>,
			max_width: Option<u16>,
			content_size: Size,
		}
		impl<$($wty: crate::widgets::Widget),*> $name<$($wty),*> {

			#[allow(clippy::too_many_arguments)]
			pub fn new(horizontal: bool, $($wfield: $wty,)*) -> Self {
				let mut res = Self {
					$($wfield,)*
					horizontal,
					content_alignment: Position::Start,
					content_pos: Position::Start,
					min_height: None,
					min_width: None,
					max_height: None,
					max_width: None,
					content_size: Size{width: 0, height: 0},
				};
				res.content_size = res.compute_size();
				res
			}

			fn compute_size(&self) -> Size {
				let mut size = Size { width: 0, height: 0 };
				$(
					let Size { width: w_width, height: w_height } = self.$wfield.size();
					if self.horizontal {
						size.width += w_width;
						size.height = size.height.max(w_height);
					} else {
						size.width = size.width.max(w_width);
						size.height += w_height;
					}
				)*
				size
			}

			setters_getters!{
				content_pos: Position,
				content_alignment: Position,
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

			$(
				pub fn $wfield(&self) -> &$wty {
					&self.$wfield
				}

				concat_idents::concat_idents!(wfield_mut = $wfield, _mut {
					pub fn wfield_mut(&mut self) -> DivGuard<$wty> {
						let old_w_size = self.$wfield.size();
						DivGuard {
							widget: &mut self.$wfield,
							old_w_size,
							div_size: &mut self.content_size,
							horizontal: self.horizontal,
						}
					}
				});

			)*
		}


		impl<$($wty: crate::widgets::Widget),*> crate::widgets::Widget for $name<$($wty),*> {
			fn display_line(&self, f: &mut std::fmt::Formatter<'_>, mut line: u16) -> std::fmt::Result {
				let content_height = self.content_size.height;
				let padding = self.size().height.saturating_sub(content_height);
				let top_pad = match self.content_pos {
					Position::Start => 0,
					Position::Center => padding / 2,
					Position::End => padding,
				};
				if line < top_pad {
					return Spacing::line(self.size().width).display_line(f, 0);
				}
				line -= top_pad;
				if line >= content_height {
					return Spacing::line(self.size().width).display_line(f, 0);
				}

				#[allow(unused_assignments)]
				if self.horizontal {
					let content_width = self.content_size.width;
					let padding = self.size().width.saturating_sub(content_width);
					let l_pad = match self.content_pos {
						Position::Start => 0,
						Position::Center => padding / 2,
						Position::End => padding,
					};
					let r_pad = padding.saturating_sub(l_pad);
					Spacing::line(r_pad).display_line(f, 0)?;
					$(
						self.$wfield.display_line(f, line)?;
					)*
					Spacing::line(l_pad).display_line(f, 0)?;
				} else {
					$(
						if line < self.$wfield.size().height {
							let w_pad = self.size().width.saturating_sub(self.$wfield.size().width);
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
				}
				Ok(())
			}

			fn size(&self) -> Size {
				let mut width = self.content_size.width;
				let mut height = self.content_size.height;

				if let Some(min_width) = self.min_width {
					width = width.max(min_width);
				}
				if let Some(max_width) = self.max_width {
					width = width.min(max_width);
				}
				if let Some(min_height) = self.min_height {
					height = height.max(min_height);
				}
				if let Some(max_height) = self.max_height {
					height = height.min(max_height);
				}

				Size{
					width,
					height,
				}
			}
			fn resize(&mut self, val: Size) -> Size {
				self.set_exact_size(val);
				val
			}
		}
	}
}

div!(Div1, (widget0: W0));
div!(Div2, (widget0: W0), (widget1: W1));
div!(Div3, (widget0: W0), (widget1: W1), (widget2: W2));
div!(Div4, (widget0: W0), (widget1: W1), (widget2: W2), (widget3: W3));
div!(Div5, (widget0: W0), (widget1: W1), (widget2: W2), (widget3: W3), (widget4: W4));

div!(Div6, (widget0: W0), (widget1: W1), (widget2: W2), (widget3: W3), (widget4: W4),
           (widget5: W5));

div!(Div7, (widget0: W0), (widget1: W1), (widget2: W2), (widget3: W3), (widget4: W4),
           (widget5: W5), (widget6: W6));

div!(Div8, (widget0: W0), (widget1: W1), (widget2: W2), (widget3: W3), (widget4: W4),
           (widget5: W5), (widget6: W6), (widget7: W7));

div!(Div9, (widget0: W0), (widget1: W1), (widget2: W2), (widget3: W3), (widget4: W4),
           (widget5: W5), (widget6: W6), (widget7: W7), (widget8: W8));

div!(Div10, (widget0: W0), (widget1: W1), (widget2: W2), (widget3: W3), (widget4: W4),
            (widget5: W5), (widget6: W6), (widget7: W7), (widget8: W8), (widget9: W9));

#[derive(Debug, Clone, Copy)]
pub struct CollDiv<Coll, W>
where
	for<'a> &'a Coll: IntoIterator<Item = &'a W>,
	W: Widget,
{
	collection: Coll,
	pub horizontal: bool,
	pub content_alignment: Position,
	pub content_pos: Position,
	pub min_height: Option<u16>,
	pub min_width: Option<u16>,
	pub max_height: Option<u16>,
	pub max_width: Option<u16>,
	content_size: Size,
}

impl<Coll, W> CollDiv<Coll, W>
where
	for<'a> &'a Coll: IntoIterator<Item = &'a W>,
	W: Widget,
{
	pub fn new(horizontal: bool, coll: Coll) -> Self {
		let content_size = Self::compute_content_size(&coll, horizontal);
		Self {
			collection: coll,
			horizontal,
			content_alignment: Position::Start,
			content_pos: Position::Start,
			min_height: None,
			min_width: None,
			max_height: None,
			max_width: None,
			content_size,
		}
	}

	pub fn collection(&self) -> &Coll {
		&self.collection
	}

	pub fn collection_mut(&mut self) -> CollContentGuard<Coll, W> {
		CollContentGuard {
			content: &mut self.collection,
			size: &mut self.content_size,
			horizontal: self.horizontal,
		}
	}

	fn compute_content_size(content: &Coll, horizontal: bool) -> Size {
		let mut size = Size { width: 0, height: 0 };
		for w in content {
			let Size { width: w_width, height: w_height } = w.size();
			if horizontal {
				size.width += w_width;
				size.height = size.height.max(w_height);
			} else {
				size.width = size.width.max(w_width);
				size.height += w_height;
			}
		}
		size
	}

	setters_getters! {
		content_alignment: Position,
		content_pos: Position,
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

impl<Coll, W> Deref for CollDiv<Coll, W>
where
	for<'a> &'a Coll: IntoIterator<Item = &'a W>,
	W: Widget,
{
	// add code here
	type Target = Coll;
	fn deref(&self) -> &<Self as std::ops::Deref>::Target {
		&self.collection
	}
}

impl<Coll, W> crate::widgets::Widget for CollDiv<Coll, W>
where
	for<'a> &'a Coll: IntoIterator<Item = &'a W>,
	W: Widget,
{
	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, mut line: u16) -> std::fmt::Result {
		let content_height = self.content_size.height;
		let padding = self.size().height.saturating_sub(content_height);
		let top_pad = match self.content_pos {
			Position::Start => 0,
			Position::Center => padding / 2,
			Position::End => padding,
		};
		if line < top_pad {
			return Spacing::line(self.size().width).display_line(f, 0);
		}
		line -= top_pad;
		if line >= content_height {
			return Spacing::line(self.size().width).display_line(f, 0);
		}

		if self.horizontal {
			let content_width = self.content_size.width;
			let padding = self.size().width.saturating_sub(content_width);
			let l_pad = match self.content_pos {
				Position::Start => 0,
				Position::Center => padding / 2,
				Position::End => padding,
			};
			let r_pad = padding.saturating_sub(l_pad);
			Spacing::line(r_pad).display_line(f, 0)?;
			for w in &self.collection {
				w.display_line(f, line)?;
			}
			Spacing::line(l_pad).display_line(f, 0)?;
		} else {
			for w in &self.collection {
				if line < w.size().height {
					let w_pad = self.size().width.saturating_sub(w.size().width);
					let (l_pad, r_pad) = match self.content_alignment {
						Position::Start => (0, w_pad),
						Position::Center => {
							let tmp = w_pad / 2;
							(tmp, w_pad - tmp)
						}
						Position::End => (w_pad, 0),
					};
					Spacing::line(l_pad).display_line(f, 0)?;
					w.display_line(f, line)?;
					Spacing::line(r_pad).display_line(f, 0)?;
					return Ok(());
				}

				line -= w.size().height;
			}

			Spacing::line(self.size().width).display_line(f, 0)?;
		}
		Ok(())
	}

	fn size(&self) -> Size {
		let mut width = self.content_size.width;
		let mut height = self.content_size.height;

		if let Some(min_width) = self.min_width {
			width = width.max(min_width);
		}
		if let Some(max_width) = self.max_width {
			width = width.min(max_width);
		}
		if let Some(min_height) = self.min_height {
			height = height.max(min_height);
		}
		if let Some(max_height) = self.max_height {
			height = height.min(max_height);
		}

		Size { width, height }
	}

	fn resize(&mut self, val: Size) -> Size {
		self.set_exact_size(val);
		val
	}
}
pub struct CollContentGuard<'g, Coll, W>
where
	for<'a> &'a Coll: IntoIterator<Item = &'a W>,
	W: Widget,
{
	content: &'g mut Coll,
	size: &'g mut Size,
	horizontal: bool,
}

impl<'g, Coll, W> Deref for CollContentGuard<'g, Coll, W>
where
	for<'a> &'a Coll: IntoIterator<Item = &'a W>,
	W: Widget,
{
	// add code here
	type Target = Coll;
	fn deref(&self) -> &<Self as std::ops::Deref>::Target {
		self.content
	}
}

impl<'g, Coll, W> DerefMut for CollContentGuard<'g, Coll, W>
where
	for<'a> &'a Coll: IntoIterator<Item = &'a W>,
	W: Widget,
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.content
	}
}

impl<Coll, W> Drop for CollContentGuard<'_, Coll, W>
where
	for<'a> &'a Coll: IntoIterator<Item = &'a W>,
	W: Widget,
{
	fn drop(&mut self) {
		*self.size = CollDiv::compute_content_size(self.content, self.horizontal)
	}
}
