use proc_macro2::TokenStream;
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use std::{cell::Cell, cmp::Ordering, iter};
use syn::{
	braced, bracketed,
	parse::{Parse, ParseStream},
	parse_quote,
	punctuated::{Pair, Punctuated},
	token::{self, Brace, Bracket},
	Expr, Ident, LitChar, LitInt, LitStr, Token,
};

#[allow(dead_code)]
struct FrameWidget {
	name: LitChar,
	col: Token![:],
	expr: Expr,
}

/*#[allow(dead_code)]
struct FrameWidgetIndex {
	name: LitChar,
	col: Token![:],
	index: Expr,
}*/
#[allow(dead_code)]
enum FrameWidgetIndex {
	Simple {
		name: LitChar,
		col: Token![:],
		index: Expr,
	},
	Repeat {
		name: LitChar,
		col: Token![:],
		start: usize,
		range: Token![..],
		end: Option<LitInt>,
		current: Cell<usize>,
	},
}

#[allow(dead_code)]
enum FrameColl {
	Array {
		brackets: Bracket,
		values: Punctuated<FrameWidget, Token![,]>,
	},
	External {
		value: Expr,
		size: Option<(usize, usize)>,
		arrow: Token![=>],
		braces: Brace,
		values: Punctuated<FrameWidgetIndex, Token![,]>,
	},
}
enum IndexKind<'a> {
	Expr(Expr),
	Range((usize, Option<LitInt>, &'a Cell<usize>)),
}

impl FrameColl {
	fn widgets_names<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a LitChar, IndexKind<'a>)> + 'a> {
		match self {
			Self::Array { values, .. } => Box::new(
				values
					.iter()
					.enumerate()
					.map(|(i, w)| (&w.name, IndexKind::Expr(parse_quote!(#i)))),
			),
			Self::External { values, .. } => Box::new(values.iter().map(|w| match w {
				FrameWidgetIndex::Simple { name, index, .. } => {
					(name, IndexKind::Expr(index.clone()))
				}
				FrameWidgetIndex::Repeat { name, start, end, current, .. } => {
					(name, IndexKind::Range((*start, end.clone(), current)))
				}
			})),
		}
	}

	#[must_use]
	fn check_repeat(&self) -> Vec<Diagnostic> {
		let mut diag = vec![];
		match self {
			Self::Array { .. } => (),
			Self::External { values, .. } => {
				for w_i in values {
					match w_i {
						FrameWidgetIndex::Simple { .. } => continue,
						FrameWidgetIndex::Repeat { end, current, .. } => {
							if let Some(end) = end {
								if current.get() < end.base10_parse().unwrap() {
									let d =
										Diagnostic::spanned(
											end.span(),
											Level::Error,
											format!(
												"Error: Upper bound of {} hasn't been reached (got {})",
												end, current.get()),
										);
									diag.push(d);
								}
							}
						}
					}
				}
			}
		}
		diag
	}
}

#[allow(dead_code)]
pub struct FrameMacro {
	collection: FrameColl,
	content: Vec<LitStr>,
}

impl Parse for FrameWidget {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let name = input.parse()?;
		let col = input.parse()?;
		let expr = input.parse()?;
		Ok(Self { expr, col, name })
	}
}

impl Parse for FrameWidgetIndex {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		if input.peek(Ident) {
			let repeat: Ident = input.parse()?;
			if repeat.to_string() != "repeat" {
				return Err(syn::Error::new(repeat.span(), "Expected a char or 'repreat'"));
			}
			let name = input.parse()?;
			let col = input.parse()?;
			let start: LitInt = input.parse()?;
			let range = input.parse()?;
			let end: Option<LitInt> = input.parse()?;
			let start = start.base10_parse()?;
			Ok(Self::Repeat { name, col, start, range, end, current: Cell::new(start) })
		} else {
			let name = input.parse()?;
			let col = input.parse()?;
			let index = input.parse()?;
			Ok(Self::Simple { index, col, name })
		}
	}
}

impl Parse for FrameMacro {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		if input.peek(token::Bracket) {
			let widgets;
			let brackets = bracketed!(widgets in input);
			let values = widgets.parse_terminated(FrameWidget::parse)?;
			let content: Vec<_> = iter::repeat(())
				.map_while(|()| if input.is_empty() { None } else { Some(input.parse()) })
				.collect::<syn::Result<_>>()?;
			Ok(Self { collection: FrameColl::Array { brackets, values }, content })
		} else {
			let indexes;
			let value = input.parse()?;
			let size = if input.peek(Ident) {
				let of: Ident = input.parse()?;
				if of.to_string() != "of" {
					return Err(syn::Error::new(
						of.span(),
						"Expecting a size description ('of size<w, h>') or the token '=>'",
					));
				}
				let size: Ident = input.parse()?;
				if size.to_string() != "size" {
					return Err(syn::Error::new(
						size.span(),
						"Expecting a size description ('of size<w, h>')",
					));
				}
				let _: Token![<] = input.parse()?;
				let left: LitInt = input.parse()?;
				let _: Token![,] = input.parse()?;
				let right: LitInt = input.parse()?;
				let _: Token![>] = input.parse()?;
				Some((left.base10_parse()?, right.base10_parse()?))
			} else {
				None
			};
			Ok(Self {
				collection: FrameColl::External {
					value,
					size,
					arrow: input.parse()?,
					braces: braced!(indexes in input),
					values: indexes.parse_terminated(FrameWidgetIndex::parse)?,
				},
				content: iter::repeat(())
					.map_while(|()| if input.is_empty() { None } else { Some(input.parse()) })
					.collect::<syn::Result<_>>()?,
			})
		}
	}
}

pub fn run(input: FrameMacro) -> (TokenStream, Vec<Diagnostic>) {
	let mut frame_width = None;
	let mut errors = vec![];
	//let mut content_width = None;
	//let mut content_height = None;

	let widgets_size = match input.collection {
		FrameColl::Array { .. } => None,
		FrameColl::External { size, .. } => size,
	};

	let widgets = match &input.collection {
		FrameColl::Array { values, .. } => {
			let mut res = Punctuated::new();
			for pair in values.pairs() {
				let (wi, punct) = match pair {
					Pair::Punctuated(w, p) => (w, Some(p)),
					Pair::End(w) => (w, None),
				};
				res.push_value(&wi.expr);
				punct.map(|p| res.push_punct(p));
			}
			quote!([#res])
		}
		FrameColl::External { value, .. } => {
			quote!(#value)
		}
	};

	let mut frame_lines: Punctuated<_, Token![,]> = Punctuated::new();

	let mut last_indexes: Vec<((&LitChar, Expr), (usize, usize), usize)> = vec![];
	//let mut last_line = None;
	for line in &input.content {
		let line_content = line.value();
		match frame_width {
			Some(w) => {
				if w != line_content.chars().count() {
					errors.push(Diagnostic::spanned(
						line.span(),
						Level::Error,
						format!(
							"Frame width is inconsistant. Got {} earlier, found {} here.",
							w,
							line_content.chars().count()
						),
					));
				}
			} // TODO: check width in graphemes
			None => frame_width = Some(line_content.chars().count()),
		}
		let mut indexes = input
			.collection
			.widgets_names()
			.flat_map(|(name, index)| {
				let index = &index;
				let mut res: Vec<((&LitChar, Expr), (usize, usize), usize)> = vec![];
				let mut substr = &line_content[..];
				let mut substr_index = 0;
				while let Some(mut start_index) = substr.find(name.value()) {
					// Relatively to start_index
					let mut end_index = match widgets_size {
						Some((w, _)) => w,
						None => substr[start_index..]
							.find(|ch| ch != name.value())
							.unwrap_or(line_content.len()),
					};
					// Relatively to substr
					end_index += start_index;

					// From substr index to line_content index
					start_index += substr_index;
					end_index += substr_index;

					// Get line index
					let above = last_indexes
						.binary_search_by(|(_, (start, end), _)| {
							if (start_index..end_index).contains(start)
								|| (start_index..end_index).contains(end)
								|| (*start..*end).contains(&start_index)
								|| (*start..*end).contains(&end_index)
							{
								Ordering::Equal
							} else {
								start.cmp(&start_index)
							}
						})
						.ok()
						.map(|v| &last_indexes[v]);
					let (height, widget_index) = match above {
						None => (0, None),
						Some(((last_name, w_i), (start, end), y_index)) => {
							if widgets_size.map_or(false, |(_, h)| h == y_index + 1)
								|| last_name.value() != name.value()
								|| *start != start_index || *end != end_index
							{
								(0, None)
							} else {
								(y_index + 1, Some(w_i.clone()))
							}
						}
					};
					let widget_index = widget_index.unwrap_or_else(|| match index {
						IndexKind::Expr(e) => e.clone(),
						IndexKind::Range((_, end, current)) => {
							let i = current.get();
							if let Some(end) = end {
								let end_val = end.base10_parse().expect("TODO: better error");
								if i == end_val {
									let d = Diagnostic::spanned(
										end.span(),
										Level::Error,
										format!("Upper bound of {:?} repetition exceeded", end_val),
									);
									errors.push(d);
									//d.emit();
									/*emit_error!(
										end.span(),
										"Upper bound on the number of repetition exceeded"
									);*/
								}
							}
							current.set(i + 1);
							parse_quote!(#i)
						}
					});

					// Prepare next iteration
					substr_index = end_index;
					substr = &line_content[substr_index..];
					res.push(((name, widget_index), (start_index, end_index), height));
				}
				res
			})
			.collect::<Vec<_>>();
		indexes.sort_unstable_by_key(|(_, i, _)| *i);

		//let _ = check_heights(&last_indexes, &indexes, &mut content_height, &last_line.unwrap());

		// Make (widget, suffix) pairs from the end of the line
		let mut last_index = line_content.len();
		let mut line_res = vec![];
		for (widget, (line_index, line_end), line_height) in indexes.iter().rev() {
			//let width = content_width.unwrap();
			line_res.push(((widget, line_height), &line_content[*line_end..last_index]));
			last_index = *line_index;
		}
		// Reorder line
		line_res.reverse();

		let result_line: Punctuated<_, Token![,]> = line_res
			.into_iter()
			.map(|(((_, index), i), suffix)| quote!(((#index, #i), #suffix.to_owned())))
			.collect();

		// Add Frame argument
		let prefix = &line_content[0..last_index];
		frame_lines.push(quote!((#prefix.to_owned(), vec![#result_line])));

		// Prepare next iteration
		last_indexes = indexes;
		//last_line = Some(line);
	}
	// TODO: check height of last_indexes
	//let _ = check_heights(&last_indexes, &[], &mut content_height, &last_line.unwrap());
	match widgets_size {
		None => (),
		Some(s) => {
			for i in last_indexes {
				if i.2 + 1 != s.1 {
					errors.push(Diagnostic::spanned(
						input.content.last().unwrap().span(),
						Level::Error,
						format!(
							"Lines of {:?} missing (at {} out of {})",
							i.0 .0.value(),
							i.2,
							s.1
						),
					))
				}
			}
		}
	}

	// Check number of repetition of Repeat indexes
	errors.append(&mut input.collection.check_repeat());

	(
		quote!({
			let widgets = #widgets;
			terminity_widgets::widgets::frame::Frame::new(
				vec![#frame_lines], widgets
			)
		}),
		errors,
	)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn array_frame() {
		let frame_def: proc_macro2::TokenStream = quote!(
			['H': img1, 'W': img2]
			r"/===================\"
			r"| * HHHHHH WWWWWW * |"
			r"| * HHHHHH WWWWWW * |"
			r"\===================/"
		)
		.into();
		let res = run(syn::parse2(frame_def).unwrap());
		#[rustfmt::skip]
		let expected: proc_macro2::TokenStream = quote!({
			let widgets = [img1, img2];
			terminity_widgets::widgets::frame::Frame::new(
				vec![
					("/===================\\".to_owned(), vec![]),
					(
						"| * ".to_owned(),
						vec![
							((0usize, 0usize), " ".to_owned()),
							((1usize, 0usize), " * |".to_owned())
						]
					),
					(
						"| * ".to_owned(),
						vec![
							((0usize, 1usize), " ".to_owned()),
							((1usize, 1usize), " * |".to_owned())
						]
					),
					("\\===================/".to_owned(), vec![])
				],
				widgets
			)
		});
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 0);
		assert_eq!(res.0.to_string(), expected.to_string());
	}

	#[test]
	fn coll_frame() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values => {'a': 0, 'b': 1, 'c': 2, 'd': 3}
			r"/=============\"
			r"| aaaaa bbbbb |"
			r"| aaaaa bbbbb |"
			r"| aaaaa bbbbb |"
			r"*=============*"
			r"| ccccc ddddd |"
			r"| ccccc ddddd |"
			r"| ccccc ddddd |"
			r"\=============/"
		)
		.into();
		let res = run(syn::parse2(frame_def).unwrap());
		#[rustfmt::skip]
		let expected: proc_macro2::TokenStream = quote!({
			let widgets = values;
			terminity_widgets::widgets::frame::Frame::new(
				vec![
					("/=============\\".to_owned(), vec![]),
					(
						"| ".to_owned(),
						vec![
							((0, 0usize), " ".to_owned()),
							((1, 0usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((0, 1usize), " ".to_owned()),
							((1, 1usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((0, 2usize), " ".to_owned()),
							((1, 2usize), " |".to_owned())
						]
					),
					("*=============*".to_owned(), vec![]),
					(
						"| ".to_owned(),
						vec![
							((2, 0usize), " ".to_owned()),
							((3, 0usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((2, 1usize), " ".to_owned()),
							((3, 1usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((2, 2usize), " ".to_owned()),
							((3, 2usize), " |".to_owned())
						]
					),
					("\\=============/".to_owned(), vec![])
				],
				widgets
			)
		});
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 0);
		assert_eq!(res.0.to_string(), expected.to_string());
	}

	#[test]
	fn repeat_one_frame() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values => {repeat 'a': 0..4}
			r"/=============\"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"*=============*"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"\=============/"
		)
		.into();
		let res = run(syn::parse2(frame_def).unwrap());
		#[rustfmt::skip]
		let expected: proc_macro2::TokenStream = quote!({
			let widgets = values;
			terminity_widgets::widgets::frame::Frame::new(
				vec![
					("/=============\\".to_owned(), vec![]),
					(
						"| ".to_owned(),
						vec![
							((0usize, 0usize), " ".to_owned()),
							((1usize, 0usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((0usize, 1usize), " ".to_owned()),
							((1usize, 1usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((0usize, 2usize), " ".to_owned()),
							((1usize, 2usize), " |".to_owned())
						]
					),
					("*=============*".to_owned(), vec![]),
					(
						"| ".to_owned(),
						vec![
							((2usize, 0usize), " ".to_owned()),
							((3usize, 0usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((2usize, 1usize), " ".to_owned()),
							((3usize, 1usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((2usize, 2usize), " ".to_owned()),
							((3usize, 2usize), " |".to_owned())
						]
					),
					("\\=============/".to_owned(), vec![])
				],
				widgets
			)
		});
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 0);
		assert_eq!(res.0.to_string(), expected.to_string());
	}

	#[test]
	fn repeat_one_frame_noend() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values => {repeat 'a': 0..}
			r"/=============\"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"*=============*"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"\=============/"
		)
		.into();
		let res = run(syn::parse2(frame_def).unwrap());
		#[rustfmt::skip]
		let expected: proc_macro2::TokenStream = quote!({
			let widgets = values;
			terminity_widgets::widgets::frame::Frame::new(
				vec![
					("/=============\\".to_owned(), vec![]),
					(
						"| ".to_owned(),
						vec![
							((0usize, 0usize), " ".to_owned()),
							((1usize, 0usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((0usize, 1usize), " ".to_owned()),
							((1usize, 1usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((0usize, 2usize), " ".to_owned()),
							((1usize, 2usize), " |".to_owned())
						]
					),
					("*=============*".to_owned(), vec![]),
					(
						"| ".to_owned(),
						vec![
							((2usize, 0usize), " ".to_owned()),
							((3usize, 0usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((2usize, 1usize), " ".to_owned()),
							((3usize, 1usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((2usize, 2usize), " ".to_owned()),
							((3usize, 2usize), " |".to_owned())
						]
					),
					("\\=============/".to_owned(), vec![])
				],
				widgets
			)
		});
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 0);
		assert_eq!(res.0.to_string(), expected.to_string());
	}

	#[test]
	fn repeat_not_enough() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values => {repeat 'a': 0..5}
			r"/=============\"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"*=============*"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"\=============/"
		)
		.into();
		let res = run(syn::parse2(frame_def).unwrap());
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 1);
	}

	#[test]
	fn repeat_too_much() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values => {repeat 'a': 0..3}
			r"/=============\"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"*=============*"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"| aaaaa aaaaa |"
			r"\=============/"
		)
		.into();
		let res = run(syn::parse2(frame_def).unwrap());
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 1);
	}

	#[test]
	fn wrong_width() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values of size<3, 2> => {repeat 'a': 0..}
			r"/====\"
			r"| aa |"
			r"| aa |"
			r"\====/"
		)
		.into();
		let res = run(syn::parse2(frame_def).unwrap());
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 1);
	}

	#[test]
	fn wrong_height() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values of size<2, 3> => {repeat 'a': 0..}
			r"/====\"
			r"| aa |"
			r"| aa |"
		)
		.into();
		let res = run(syn::parse2(frame_def).unwrap());
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 1);
	}

	#[test]
	fn wrong_height2() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values of size<2, 3> => {repeat 'a': 0..}
			r"/====\"
			r"| aa |"
			r"| aa |"
			r"\====/"
		)
		.into();
		let res = run(syn::parse2(frame_def).unwrap());
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 1);
	}

	#[test]
	fn repeat_frame() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values of size<1, 1> => {repeat 'x': 0..12}
			r"/======\"
			r"| xxxx |"
			r"| xxxx |"
			r"| xxxx |"
			r"\======/"
		)
		.into();
		let res = run(syn::parse2(frame_def).unwrap());
		#[rustfmt::skip]
		let expected: proc_macro2::TokenStream = quote!({
			let widgets = values;
			terminity_widgets::widgets::frame::Frame::new(
				vec![
					("/======\\".to_owned(), vec![]),
					(
						"| ".to_owned(),
						vec![
							((0usize, 0usize), "".to_owned()),
							((1usize, 0usize), "".to_owned()),
							((2usize, 0usize), "".to_owned()),
							((3usize, 0usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((4usize, 0usize), "".to_owned()),
							((5usize, 0usize), "".to_owned()),
							((6usize, 0usize), "".to_owned()),
							((7usize, 0usize), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((8usize, 0usize), "".to_owned()),
							((9usize, 0usize), "".to_owned()),
							((10usize, 0usize), "".to_owned()),
							((11usize, 0usize), " |".to_owned())
						]
					),
					("\\======/".to_owned(), vec![])
				],
				widgets
			)
		});
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 0);
		assert_eq!(res.0.to_string(), expected.to_string());
	}
}
