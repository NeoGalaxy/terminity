use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use std::{cmp::Ordering, iter};
use syn::{
	braced, bracketed,
	parse::{Parse, ParseStream},
	parse_quote,
	punctuated::{Pair, Punctuated},
	token::{self, Brace, Bracket},
	Expr, LitChar, LitStr, Token,
};

#[allow(dead_code)]
struct FrameWidget {
	name: LitChar,
	col: Token![:],
	expr: Expr,
}

#[allow(dead_code)]
struct FrameWidgetIndex {
	name: LitChar,
	col: Token![:],
	index: Expr,
}

#[allow(dead_code)]
enum FrameColl {
	Array {
		brackets: Bracket,
		values: Punctuated<FrameWidget, Token![,]>,
	},
	External {
		value: Expr,
		arrow: Token![=>],
		braces: Brace,
		values: Punctuated<FrameWidgetIndex, Token![,]>,
	},
}

impl FrameColl {
	fn widgets_names<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a LitChar, Expr)> + 'a> {
		match self {
			Self::Array { values, .. } => Box::new(
				values
					.iter()
					.enumerate()
					.map(|(i, w)| (&w.name, parse_quote!(#i))),
			),
			Self::External { values, .. } => {
				Box::new(values.iter().map(|w| (&w.name, w.index.clone())))
			}
		}
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
		let name = input.parse()?;
		let col = input.parse()?;
		let index = input.parse()?;
		Ok(Self { index, col, name })
	}
}

impl Parse for FrameMacro {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		if input.peek(token::Bracket) {
			let widgets;
			let brackets = bracketed!(widgets in input);
			let values = widgets.parse_terminated(FrameWidget::parse)?;
			let content: Vec<_> = iter::repeat(())
				.map_while(|()| {
					if input.is_empty() {
						None
					} else {
						Some(input.parse())
					}
				})
				.collect::<syn::Result<_>>()?;
			Ok(Self {
				collection: FrameColl::Array { brackets, values },
				content,
			})
		} else {
			let indexes;
			Ok(Self {
				collection: FrameColl::External {
					value: input.parse()?,
					arrow: input.parse()?,
					braces: braced!(indexes in input),
					values: indexes.parse_terminated(FrameWidgetIndex::parse)?,
				},
				content: iter::repeat(())
					.map_while(|()| {
						if input.is_empty() {
							None
						} else {
							Some(input.parse())
						}
					})
					.collect::<syn::Result<_>>()?,
			})
		}
	}
}

pub fn run(input: FrameMacro) -> TokenStream {
	let mut frame_width = None;
	//let mut content_width = None;
	//let mut content_height = None;

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
	for line in input.content {
		let line_content = line.value();
		match frame_width {
			Some(w) => {
				if w != line_content.chars().count() {
					emit_error!(
						line.span(),
						"Frame width is incoherent. Got {} earlier, found {} here.",
						w,
						line_content.chars().count()
					)
				}
			} // TODO: check width in graphemes
			None => frame_width = Some(line_content.chars().count()),
		}
		let mut indexes = input
			.collection
			.widgets_names()
			.flat_map(|(name, index)| {
				let mut res = vec![];
				let mut substr = &line_content[..];
				let mut substr_index = 0;
				while let Some(mut start_index) = substr.find(name.value()) {
					let mut end_index = substr[start_index..]
						.find(|ch| ch != name.value())
						.unwrap_or(line_content.len());
					end_index += start_index;
					//let len = end_index - start_index;
					/*match content_width {
						None => content_width = Some(len),
						Some(cw) => if cw != len {
							emit_error!(line.span(),
								"Content widget width is incoherent. Got {} earlier, found {} here.",
								cw, len)
						}
					}*/

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
					let height: usize = match above {
						None => 0,
						Some(((last_name, _), (start, end), y_index)) => {
							if last_name.value() != name.value()
								|| *start != start_index || *end != end_index
							{
								0
							} else {
								y_index + 1
							}
						}
					};

					// Prepare next iteration
					substr_index = end_index;
					substr = &line_content[substr_index..];
					res.push(((name, index.clone()), (start_index, end_index), height));
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

	quote!({
		let widgets = #widgets;
		terminity_widgets::widgets::frame::Frame::new(
			vec![#frame_lines], widgets
		)
	})
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
		assert_eq!(res.to_string(), expected.to_string());
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
		assert_eq!(res.to_string(), expected.to_string());
	}
}
