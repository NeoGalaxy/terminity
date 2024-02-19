use proc_macro2::TokenStream;
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use std::{
	cell::{Cell, RefCell},
	cmp::Ordering,
	collections::HashMap,
	iter,
};
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
pub enum IndexKind<'a> {
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
			if repeat != "repeat" {
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
				if of != "of" {
					return Err(syn::Error::new(
						of.span(),
						"Expecting a size description ('of size<w, h>') or the token '=>'",
					));
				}
				let size: Ident = input.parse()?;
				if size != "size" {
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

#[derive(Debug, Clone)]
pub struct WidgetLine {
	pub widget_char: char,
	pub uid: usize,
	pub line_index: u16,
}

#[derive(Debug, Clone)]
pub struct FrameLine {
	pub prefix: LitStr,
	pub line_content: Vec<(WidgetLine, LitStr)>,
}

pub fn parse_frame_lines(
	frame_width: &mut Option<usize>,
	errors: &mut Vec<Diagnostic>,
	content: &[LitStr],
	widgets_names: Vec<(char, &RefCell<Option<(usize, usize)>>)>,
) -> Vec<FrameLine> {
	let mut res_lines = vec![];

	let mut next_uid = 0;
	// char to match, uid, start, end, current line
	let mut last_indexes: Vec<(
		char,
		usize,
		(usize, usize),
		usize,
		(&RefCell<Option<(usize, usize)>>, bool),
	)> = vec![];

	for line in content {
		let line_content = line.value().chars().collect::<Vec<_>>();
		// Check full width of frame
		match frame_width {
			Some(w) => {
				if *w != line_content.len() {
					errors.push(Diagnostic::spanned(
						line.span(),
						Level::Error,
						format!(
							"Frame width is inconsistant. Got {} earlier, found {} here.",
							w,
							line_content.len()
						),
					));
				}
			} // TODO: check width in graphemes
			None => *frame_width = Some(line_content.len()),
		}
		// Get the list of index of widget on current line
		let mut indexes = widgets_names
			.iter()
			.flat_map(|(name, widget_size)| {
				let name = *name;
				// List of all the places in the string where there is the widget of char `name`
				let mut res: Vec<(char, usize, (usize, usize), usize, _)> = vec![];
				// Substring that hasn't been checked yet
				let mut substr = &line_content[..];
				// The index at which the above substring starts in `line_content`
				let mut substr_index = 0;

				// Find next occurence of the char to match
				while let Some(mut start_index) = substr.iter().position(|c| *c == name) {
					// getting `end_index` relatively to `start_index`
					let mut end_index = match *widget_size.borrow_mut() {
						// If current widget has pre-defined width, then use it
						Some((w, _)) => {
							let widget_section = substr
								.get(start_index..(start_index + w))
								.or_else(|| substr.get(start_index..))
								.unwrap_or(&[]);
							if widget_section.len() != w
								|| !widget_section.iter().all(|c| *c == name)
							{
								// Create error and skip until a char is different
								errors.push(Diagnostic::spanned(
									line.span(),
									Level::Error,
									format!("Widget of character {} too short", name),
								));
								// Relatively to line_content
								start_index += substr_index;
								// Prepare next iter
								substr_index = start_index
									+ line_content.get(start_index..).map_or(0, |s| {
										s.iter().position(|ch| *ch != name).unwrap_or(s.len())
									});
								substr = line_content.get(substr_index..).unwrap_or(&[]);
								continue;
							} else {
								w
							}
						}
						None => substr
							.get(start_index..)
							.map_or(0, |s| s.iter().position(|ch| *ch != name).unwrap_or(s.len())),
					};
					// Relatively to substr
					end_index += start_index;

					// Relatively to line_content
					start_index += substr_index;
					end_index += substr_index;

					// Get details of this same span on the line above
					let above = last_indexes
						.binary_search_by(|(_, _, (start, end), _, _)| {
							if (start_index..end_index).contains(start)
								|| (start_index..end_index).contains(&(end - 1))
								|| (*start..*end).contains(&start_index)
								|| (*start..*end).contains(&(end_index - 1))
							{
								Ordering::Equal
							} else {
								start.cmp(&start_index)
							}
						})
						.ok()
						.map(|v| &mut last_indexes[v]);
					// Check if the line above matches and generate the accurate vertical index.
					let (widget_y_index, uid) = match above {
						None => {
							next_uid += 1;
							(0, next_uid - 1)
						}
						Some((last_name, last_uid, (start, end), y_index, (_, matched))) => {
							// if there's some kind of issue, then we start a brand new display
							if widget_size.borrow().map_or(false, |(_, h)| h == *y_index + 1)
								|| *last_name != name || *start != start_index
								|| *end != end_index
							{
								next_uid += 1;
								(0, next_uid - 1)
							} else {
								*matched = true;
								(*y_index + 1, *last_uid)
							}
						}
					};

					// Prepare next iteration
					substr_index = end_index;
					substr = line_content.get(substr_index..).unwrap_or(&[]);
					res.push((
						name,
						uid,
						(start_index, end_index),
						widget_y_index,
						(*widget_size, false),
					));
				}
				res
			})
			.collect::<Vec<_>>();
		indexes.sort_unstable_by_key(|(_, _, i, _, _)| *i);

		// check size
		for (l_name, _, (start, end), y_index, (size, matched)) in last_indexes {
			if matched {
				continue;
			}
			let borrowed = size.borrow();
			if let Some((w, h)) = *borrowed {
				if y_index + 1 != h {
					errors.push(Diagnostic::spanned(
						line.span(),
						Level::Error,
						format!("Invalid height for widget of layout name {l_name:?}: expected {}, got {}.",
						h,
						y_index + 1)
					));
				}
				if w != end - start {
					errors.push(Diagnostic::spanned(
						line.span(),
						Level::Error,
						format!("Invalid width for widget of layout name {l_name:?}: expected {}, got {}.",
						w,
						end - start)
					));
				}
			} else {
				drop(borrowed);
				*size.borrow_mut() = Some((end - start, y_index + 1))
			}
		}

		//let _ = check_heights(&last_indexes, &indexes, &mut content_height, &last_line.unwrap());

		// Make (widget, suffix) pairs from the end of the line
		let mut last_index = line_content.len();
		let mut line_res = vec![];
		for (widget, uid, (line_index, line_end), line_height, _) in indexes.iter().rev() {
			//let width = content_width.unwrap();
			line_res.push((
				WidgetLine { widget_char: *widget, uid: *uid, line_index: *line_height as u16 },
				LitStr::new(
					&line_content
						.get(*line_end..last_index)
						.unwrap_or(&[])
						.iter()
						.collect::<String>(),
					line.span(),
				),
			));
			last_index = *line_index;
		}
		// Reorder line to have it in appropriate order
		line_res.reverse();

		res_lines.push(FrameLine {
			prefix: LitStr::new(
				&line_content[0..last_index].iter().collect::<String>(),
				line.span(),
			),
			line_content: line_res,
		});

		// Prepare next iteration
		last_indexes = indexes;
		//last_line = Some(line);
	}

	for (l_name, _, _, y_index, (size, matched)) in last_indexes {
		if matched {
			continue;
		}
		if let Some((_, h)) = *size.borrow() {
			if y_index + 1 != h {
				errors.push(Diagnostic::spanned(
					content.last().expect("impossible: content can't be empty").span(),
					Level::Error,
					format!(
						"Invalid height for widget of layout name {l_name:?}: expected {}, got {}.",
						h,
						y_index + 1
					),
				));
			}
		}
	}
	// TODO: check height of last_indexes
	//let _ = check_heights(&last_indexes, &[], &mut content_height, &last_line.unwrap());
	/*match widget_size {
		None => (),
		Some(s) => {
			for i in last_indexes {
				if i.2 + 1 != s.1 {
					errors.push(Diagnostic::spanned(
						content.last().unwrap().span(),
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
	}*/
	res_lines
}

pub fn run(input: FrameMacro) -> (TokenStream, Vec<Diagnostic>) {
	let mut errors = vec![];
	//let mut content_width = None;
	//let mut content_height = None;

	let widgets_size = RefCell::new(match input.collection {
		FrameColl::Array { .. } => None,
		FrameColl::External { size, .. } => size,
	});

	let widgets = match &input.collection {
		FrameColl::Array { values, .. } => {
			let mut res = Punctuated::new();
			for pair in values.pairs() {
				let (wi, punct) = match pair {
					Pair::Punctuated(w, p) => (w, Some(p)),
					Pair::End(w) => (w, None),
				};
				res.push_value(&wi.expr);
				if let Some(p) = punct {
					res.push_punct(p)
				}
			}
			quote!([#res])
		}
		FrameColl::External { value, .. } => {
			quote!(#value)
		}
	};

	let widgets_indexes: HashMap<_, _> =
		input.collection.widgets_names().map(|wi_data| (wi_data.0.value(), wi_data)).collect();

	let mut frame_width = None;
	let frame_layout = parse_frame_lines(
		&mut frame_width,
		&mut errors,
		&input.content,
		widgets_indexes.keys().map(|k| (*k, &widgets_size)).collect::<Vec<_>>(),
	);

	let mut frame_lines: Punctuated<_, Token![,]> = Punctuated::new();

	let mut uid_indexes: HashMap<usize, Expr> = HashMap::new();
	for FrameLine { prefix, line_content } in frame_layout.into_iter() {
		let line_res = line_content.into_iter().map(|(line_details, suffix)| {
			let index_expr = match uid_indexes.get(&line_details.uid) {
				Some(i) => i.clone(),
				None => {
					let (_, index) = &widgets_indexes[&line_details.widget_char];
					match index {
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
							let res: Expr = parse_quote!(#i);
							let _ = uid_indexes.insert(line_details.uid, res.clone());
							res
						}
					}
				}
			};
			let line_i = line_details.line_index;
			quote!(((#index_expr, #line_i), #suffix.to_owned()))
		});
		frame_lines.push(quote!((#prefix.to_owned(), vec![#(#line_res),*])));
	}

	// Check number of repetition of Repeat indexes
	errors.append(&mut input.collection.check_repeat());

	(
		quote!({
			let widgets = #widgets;
			terminity::widgets::frame::Frame::new(
				  vec![#frame_lines], widgets
			)
		}),
		errors,
	)
}

#[cfg(test)]
mod tests {
	use proc_macro2::Span;

	use super::*;

	#[test]
	fn frame_layout() {
		let mut width = None;
		let mut errors = vec![];
		let content = [
			LitStr::new("0 ─────── 1 ──────────── 2 ─────── 3", Span::call_site()),
			LitStr::new("0 Library 1 Install Game 2 Options 3", Span::call_site()),
			LitStr::new("0 ─────── 1 ──────────── 2 ─────── 3", Span::call_site()),
		];
		let widgets = vec![
			('0', Default::default()),
			('1', Default::default()),
			('2', Default::default()),
			('3', Default::default()),
		];
		let res = parse_frame_lines(
			&mut width,
			&mut errors,
			&content,
			widgets.iter().map(|(c, cell)| (*c, cell)).collect(),
		);
		println!("{width:?}");
		println!("{errors:?}");
		println!("{:?}", widgets.into_iter().map(|(c, cell)| (c, cell.take().unwrap())));
		println!("{res:#?}");
		panic!();
	}

	#[test]
	fn array_frame() {
		let frame_def: proc_macro2::TokenStream = quote!(
			['H': img1, 'W': img2]
			r"/===================\"
			r"| * HHHHHH WWWWWW * |"
			r"| * HHHHHH WWWWWW * |"
			r"\===================/"
		);
		let res = run(syn::parse2(frame_def).unwrap());
		#[rustfmt::skip]
		let expected: proc_macro2::TokenStream = quote!({
			let widgets = [img1, img2];
			terminity::widgets::frame::Frame::new(
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
		);
		let res = run(syn::parse2(frame_def).unwrap());
		#[rustfmt::skip]
		let expected: proc_macro2::TokenStream = quote!({
			let widgets = values;
			terminity::widgets::frame::Frame::new(
				vec![
					("/=============\\".to_owned(), vec![]),
					(
						"| ".to_owned(),
						vec![
							((0, 0u16), " ".to_owned()),
							((1, 0u16), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((0, 1u16), " ".to_owned()),
							((1, 1u16), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((0, 2u16), " ".to_owned()),
							((1, 2u16), " |".to_owned())
						]
					),
					("*=============*".to_owned(), vec![]),
					(
						"| ".to_owned(),
						vec![
							((2, 0u16), " ".to_owned()),
							((3, 0u16), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((2, 1u16), " ".to_owned()),
							((3, 1u16), " |".to_owned())
						]
					),
					(
						"| ".to_owned(),
						vec![
							((2, 2u16), " ".to_owned()),
							((3, 2u16), " |".to_owned())
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
		);
		let res = run(syn::parse2(frame_def).unwrap());
		#[rustfmt::skip]
		let expected: proc_macro2::TokenStream = quote!({
			let widgets = values;
			terminity::widgets::frame::Frame::new(
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
		);
		let res = run(syn::parse2(frame_def).unwrap());
		#[rustfmt::skip]
		let expected: proc_macro2::TokenStream = quote!({
			let widgets = values;
			terminity::widgets::frame::Frame::new(
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
		);
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
		);
		let res = run(syn::parse2(frame_def).unwrap());
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 1);
	}

	#[test]
	fn wrong_width() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values of size<3, 2> => {repeat 'a': 0..}
			r"/======\"
			r"| aaaa |"
			r"| aaaa |"
			r"\======/"
		);
		let res = run(syn::parse2(frame_def).unwrap());
		println!("{:?}", res.1);
		assert_ne!(res.1.len(), 0);
	}

	#[test]
	fn wrong_height() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values of size<2, 3> => {repeat 'a': 0..}
			r"/====\"
			r"| aa |"
			r"| aa |"
		);
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
		);
		let res = run(syn::parse2(frame_def).unwrap());
		println!("{:?}", res.1);
		assert_eq!(res.1.len(), 1);
	}

	#[test]
	fn consistant_sizes() {
		let frame_def: proc_macro2::TokenStream = quote!(
			values => {repeat 'a': 0..}
			r"/=========\"
			r"| aa  aaa |"
			r"| aa  aaa |"
			r"\=========/"
		);
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
		);
		let res = run(syn::parse2(frame_def).unwrap());
		#[rustfmt::skip]
		let expected: proc_macro2::TokenStream = quote!({
			let widgets = values;
			terminity::widgets::frame::Frame::new(
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
