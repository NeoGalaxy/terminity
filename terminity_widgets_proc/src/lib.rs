use std::iter::Extend;
use proc_macro::{TokenStream, TokenTree, Ident, Punct, Spacing, Group, Literal};
use proc_macro_error::{emit_error, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

struct Widget {
	id: char,
	//var: Vec<TokenTree>,
	i: Option<u16>,
	//index: u16
	location: Vec<TokenTree>
}

#[proc_macro_error]
#[proc_macro]
pub fn frame(tokens: TokenStream) -> TokenStream {
	let tokens = tokens.into_iter();
	let mut chunks = vec![vec![]];
	for t in tokens {
		if t.to_string() == ",".to_owned() {
			chunks.push(vec![]);
		} else {
			chunks.last_mut().unwrap().push(t);
		}
	}
	let start_span = chunks[0][0].span();
	let content = chunks.pop().unwrap();

	let arrow_index = if chunks.len() == 1 {
		chunks[0].iter().enumerate().scan(0, |state, (i, t)| {
			match state {
				0 => {
					if let TokenTree::Punct(p) = t {
						if p.as_char() == '=' && p.spacing() == Spacing::Joint { *state = 1 };
						Some(None)
					} else {
						Some(None)
					}
				},
				1 => {
					if let TokenTree::Punct(p) = t {
						if p.as_char() == '>' && p.spacing() == Spacing::Alone {
							*state = 2;
							Some(Some(i-1))
						} else {
							*state = 0;
							Some(None)
						}
					} else {
						*state = 0;
						Some(None)
					}
				},
				_ => None
			}
		}).last().flatten()
	} else {
		None
	};
	let (collection, widgets) = if let Some(index) = arrow_index {
		let (mut coll, content) = chunks[0].split_at(index + 2);
		coll = &coll[0..coll.len() - 2];
		match &content[0] {
		    TokenTree::Group(c) => {
		    	let mut decls = vec![vec![]];
		    	for t in c.stream() {
					if t.to_string() == ",".to_owned() {
						decls.push(vec![]);
					} else {
						decls.last_mut().unwrap().push(t)
					}
				}
				(coll.to_vec(), decls.iter().map(|d| {
					if d[1].to_string() != ":" {
						emit_error!(d[1].span(), "Syntax error: expected column (':').");
						None
					} else {
						let id: Vec<_> = d[0].to_string().chars().collect();
						if id.len() != 3 || id[0] != '\'' || id[2] != '\'' {
							emit_error!(d[0].span(), "Syntax error: expected single char.");
							return None;
						}
						let id = id[1];
						if d.len() > 3 {
							emit_error!(d[3].span(), "Syntax error: expected an index surrounded by brackets");
							return None;
						}
						if let TokenTree::Group(g) = &d[2] {
							Some(Widget { id, location: g.stream().into_iter().collect(), i: Some(0) })
						} else {
							None
						}
					}
				}).collect::<Vec<_>>())
		    },
		    c => {
		    	emit_error!(c.span(), "Collection indexes details should be between braces");
		    	return TokenStream::new();
		    }
		}
	} else {
		let (c, w): (Vec<_>, Vec<_>) = chunks.iter().enumerate().map(|(i, c)| {
			if let Some(TokenTree::Literal(lit)) = c.last() {
				if c.last().unwrap().to_string().as_bytes()[0] == '"' as u8 {
					let error = "Syntax error: expected to have widget identifyer".to_owned()
						+ " followed by a column and the widget itself."
						+ "\nString litteral found, maybe you put a comma between string litterals?";
					emit_error!(lit.span(), error);
					return (vec![], None);
				}
			}
			if c[1].to_string() != ":" {
				emit_error!(c[1].span(), "Syntax error: expected column (':').");
				(vec![], None)
			} else {
				let id: Vec<_> = c[0].to_string().chars().collect();
				if id.len() != 3 || id[0] != '\'' || id[2] != '\'' {
					emit_error!(c[0].span(), "Syntax error: expected single char.");
					return (vec![], None);
				}
				let id = id[1];
				let mut tokens = c[2..c.len()].to_vec();
				tokens.push(TokenTree::Punct(Punct::new(',', Spacing::Alone)));
				(tokens, Some(Widget {
					id,
					location: vec![TokenTree::Literal(Literal::usize_suffixed(i))],
					i: Some(0)
				}))
			}
		}).unzip();
		let mut coll_content = TokenStream::new();
		coll_content.extend(c.into_iter().flatten());
			/*TokenTree::Ident(Ident::new("vec", start_span)),
			TokenTree::Punct(Punct::new('!', Spacing::Alone)),*/
			
		(vec![TokenTree::Group(Group::new(proc_macro::Delimiter::Bracket, coll_content))], w)
	};

	if widgets.iter().any(|e| match e { None => true, Some(_) => false }) {
		return TokenStream::new();
	}

	let mut lines = vec![];

	let mut widgets: Vec<_> = widgets.into_iter().map(|w| w.unwrap()).collect();
	for line_token in content {
		let line = line_token.to_string();
		if let TokenTree::Literal(_) = line_token {
			if line.contains("\"") {
				let string_prefix = line[0..=line.find("\"").unwrap()].to_owned();
				let string_suffix = line[line.rfind("\"").unwrap()..line.len()].to_owned();
				let line_content = &line[string_prefix.len()..line.len() - string_suffix.len()]; /* Hide it so we don't use it anymore */
				let mut indexes = widgets.iter_mut().filter_map(|w| {
					match line_content.find(w.id) {
						Some(i) => Some((w, i)),
						None => {
							if w.i != Some(0) {
								w.i = None;
							}
							None
						},
					}
				}).collect::<Vec<_>>();
				indexes.sort_unstable_by_key(|(_, i)| *i);
				let mut last_index = line_content.len();
				let mut line_res = vec![];
				for (w, i) in indexes.into_iter().rev() {
					let end_index = line_content.rfind(w.id).unwrap();
					if !line_content.as_bytes()[i..(end_index + 1)].iter().all(|c| *c as char == w.id) {
						emit_error!(line_token.span(), format!(
							"Syntax error: The same widget ('{}') is used twice on the same line.",
							w.id));
						break;
					}
					match w.i {
						None => todo!(),
						Some(w_line_i) => {
							line_res.push(((w.location.clone(), w_line_i), 
								string_prefix.clone() + &line_content[(end_index + 1)..last_index] + &string_suffix));
							w.i = Some(w_line_i + 1);
						}
					}
						last_index = i;
				}
				line_res.reverse();
				lines.push((string_prefix.clone() + &line_content[0..last_index] + &string_suffix, line_res));
				continue;
			}
		}
		/*else*/
		emit_error!(line_token.span(),
			"Syntax error: expected string litteral.".to_owned()
			+ "Maybe you're missing commas somewhere.");
		break;
	}

	// First arg: vec of line datas
	let mut text_arg = TokenStream::new();
	let tmp = lines.into_iter().map(|(prefix, line_data)| {
		let mut interior_text = TokenStream::new();
		interior_text.extend(line_data.into_iter().map(|((w_index, w_line_index), postfix)| {
			// (Idx, u16): the index of the widget and the line it has to display
			let mut tuple2_arg = TokenStream::new();
			tuple2_arg.extend(w_index.into_iter());
			tuple2_arg.extend([
				TokenTree::Punct(Punct::new(',', Spacing::Alone)),
				TokenTree::Literal(Literal::u16_unsuffixed(w_line_index)),
			].into_iter());
			// (<tuple2>, String): the Widget line display data + its suffix
			let mut tuple1_arg = TokenStream::new();
			tuple1_arg.extend([
				TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, tuple2_arg)),
				TokenTree::Punct(Punct::new(',', Spacing::Alone)),
				TokenTree::Literal(postfix.parse().unwrap()),
				TokenTree::Punct(Punct::new('.', Spacing::Alone)),
			TokenTree::Ident(Ident::new("to_string", start_span)),
			TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, TokenStream::new())),
			].into_iter());
			[TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, tuple1_arg)),
			TokenTree::Punct(Punct::new(',', Spacing::Alone))].into_iter()
		}).flatten());
		// (String, <interior_text>): the line prefix + all the widgets on the line with suffix
		let mut tuple0_arg = TokenStream::new();
		tuple0_arg.extend([
			TokenTree::Literal(prefix.parse().unwrap()),
			TokenTree::Punct(Punct::new('.', Spacing::Alone)),
			TokenTree::Ident(Ident::new("to_string", start_span)),
			TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, TokenStream::new())),
			TokenTree::Punct(Punct::new(',', Spacing::Alone)),
			TokenTree::Ident(Ident::new("vec", start_span)),
			TokenTree::Punct(Punct::new('!', Spacing::Alone)),
			TokenTree::Group(Group::new(proc_macro::Delimiter::Bracket, interior_text)),
		].into_iter());
		[TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, tuple0_arg)),
		TokenTree::Punct(Punct::new(',', Spacing::Alone))].into_iter()
	}).flatten().collect::<Vec<_>>();
	text_arg.extend(tmp.into_iter());

	let mut args = TokenStream::new();
	args.extend([
		TokenTree::Ident(Ident::new("vec", start_span)),
		TokenTree::Punct(Punct::new('!', Spacing::Alone)),
		TokenTree::Group(Group::new(proc_macro::Delimiter::Bracket, text_arg)),
		TokenTree::Punct(Punct::new(',', Spacing::Alone)),
	].into_iter());
	args.extend(collection.into_iter());

	let mut res = TokenStream::new();
	res.extend([
		TokenTree::Ident(Ident::new("terminity_widgets", start_span)),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("widgets", start_span)),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("frame", start_span)),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("Frame", start_span)),
		TokenTree::Punct(Punct::new(':', Spacing::Joint)),
		TokenTree::Punct(Punct::new(':', Spacing::Alone)),
		TokenTree::Ident(Ident::new("new", start_span)),
		TokenTree::Group(Group::new(proc_macro::Delimiter::Parenthesis, args)),
	].into_iter());
	res
}

#[proc_macro_error]
#[proc_macro]
pub fn tokens(tokens: TokenStream) -> TokenStream {
	for token in tokens {
		println!(">> {:?} -> {:?}", token.to_string(), token);
	}
	TokenStream::new()
}

#[proc_macro_derive(WidgetDisplay)]
pub fn widget_display(tokens: TokenStream) -> TokenStream {
	let input = parse_macro_input!(tokens as DeriveInput);

	let name = input.ident;

	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let expanded = quote! {
		impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
			fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
				for i in 0..self.size().1 {
					self.displ_line(f, i)?;
					f.write_str(&format!("{}\n\r",
						crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)))?;
				}
				Ok(())
			}
		}
	};
	proc_macro::TokenStream::from(expanded)
}
