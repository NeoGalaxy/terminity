use std::{cell::RefCell, collections::HashMap};

use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use quote::quote;

use syn::{
	braced, bracketed, parenthesized,
	parse::{Parse, ParseStream},
	parse2,
	punctuated::Punctuated,
	token, DeriveInput, Lit, LitChar, LitStr, Member, Token,
};

use crate::frame::{parse_frame_lines, FrameLine, WidgetLine};

enum Content {
	Bracked(Lit),
	Dot(Member),
}

impl Parse for Content {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		if input.peek(Token![.]) {
			input.parse::<Token![.]>()?;
			Ok(Self::Dot(input.parse()?))
		} else {
			let content;
			bracketed!(content in input);
			Ok(Self::Bracked(content.parse()?))
		}
	}
}

struct ContentMap {
	c: LitChar,
	_arrow: Token![=>],
	index: Vec<Content>,
}

impl Parse for ContentMap {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self {
			c: input.parse()?,
			_arrow: input.parse()?,
			index: {
				let mut res = vec![];
				while !input.peek(Token![,]) {
					res.push(input.parse()?);
				}
				res
			},
		})
	}
}

struct LayoutArgs {
	indices: Punctuated<ContentMap, Token![,]>,
	_comma: Token![,],
	args: Punctuated<LitStr, Token![,]>,
}

impl Parse for LayoutArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		if input.peek(token::Paren) {
			parenthesized!(content in input);
		} else if input.peek(token::Bracket) {
			bracketed!(content in input);
		} else {
			braced!(content in input);
		}
		let indices_content;
		braced!(indices_content in content);
		Ok(LayoutArgs {
			indices: indices_content.parse_terminated(<ContentMap as Parse>::parse)?,
			_comma: content.parse()?,
			args: content.parse_terminated(<LitStr as Parse>::parse)?,
		})
	}
}

/*struct SFImplArgs {
	args: Punctuated<syn::Ident, Token![,]>,
}

impl Parse for SFImplArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		parenthesized!(content in input);
		Ok(SFImplArgs { args: content.parse_terminated(<syn::Ident as Parse>::parse)? })
	}
}*/

#[derive(FromMeta)]
struct SFImplArgs {
	#[darling(rename = "EventBubblingWidget")]
	bubble_event: Option<()>,
}

#[derive(FromMeta, Debug)]
struct AttributeLayoutArgs {
	name: LitChar,
	#[darling(default)]
	ignore_mouse_event: bool,
}

#[derive(Debug)]
struct FieldDetails<FieldWidget> {
	access_field: FieldWidget, // It's this way because of `use_parent`
	// enum_variant: Option<Ident>,
	// field_ty: syn::Type,
	// field_widget_ty: TokenStream, // Exists because it was needed by `use_parent`
	size: RefCell<Option<(usize, usize)>>,
}

pub fn run(input: DeriveInput) -> (TokenStream, Vec<Diagnostic>) {
	let mut errors = vec![];
	let DeriveInput { attrs, ident, generics, .. } = input;
	let (layout, non_layout): (Vec<_>, Vec<_>) =
		attrs.into_iter().partition(|a| a.path.is_ident("widget_layout"));

	let (impls, _othet_attrs): (Vec<_>, Vec<_>) =
		non_layout.into_iter().partition(|a| a.path.is_ident("widget_impl"));

	let (all_impls, layout_content) = if layout.len() != 1 || impls.len() > 1 {
		if layout.len() != 1 {
			errors.push(Diagnostic::spanned(
				Span::call_site(),
				Level::Error,
				concat!(
					"Expecting ONE `#[widget_layout (...)]` attribute on the struct ",
					"to indicate the frame's layout."
				)
				.into(),
			));
		}
		if impls.len() > 1 {
			errors.push(Diagnostic::spanned(
				Span::call_site(),
				Level::Error,
				concat!(
					"Expecting at most one `#[widget_impl (...)]` attribute ",
					"to indicate what widget traits to implement. Found {} of them."
				)
				.into(),
			));
		}
		return (quote!(), errors);
	} else {
		let layout_content: LayoutArgs = match parse2(layout[0].tokens.clone()) {
			Ok(v) => v,
			Err(e) => {
				errors.push(e.into());
				return (quote!(), errors);
			}
		};
		let all_impls = if impls.is_empty() {
			None
		} else {
			match impls[0].parse_meta().map(|m| SFImplArgs::from_meta(&m)) {
				Ok(Ok(v)) => Some(v),
				Ok(Err(e)) => {
					errors.push(Diagnostic::spanned(e.span(), Level::Error, format!("{}", e)));
					None
				}
				Err(e) => {
					errors.push(Diagnostic::spanned(e.span(), Level::Error, format!("{}", e)));
					None
				}
			}
		}
		.unwrap_or(SFImplArgs { bubble_event: None });
		(all_impls, layout_content)
	};

	let widget_indexes: HashMap<_, _> = layout_content
		.indices
		.iter()
		.map(|ContentMap { c, index, .. }| {
			(
				c.value(),
				FieldDetails {
					access_field: |this| {
						let parts = index.iter().map(|i| match i {
							Content::Bracked(b) => quote!([#b]),
							Content::Dot(s) => quote!(.#s),
						});
						quote!(#this #(#parts)*)
					},
					size: None.into(),
				},
			)
		})
		.collect();

	let mut frame_width = None;

	let layout_body = parse_frame_lines(
		&mut frame_width,
		&mut errors,
		&layout_content.args.into_iter().collect::<Vec<_>>(),
		widget_indexes.iter().map(|(name, details)| (*name, &details.size)).collect::<Vec<_>>(),
	);

	let frame_width = frame_width.expect("Error: Empty struct frame layout") as u16;
	let frame_height = layout_body.len() as u16;

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let disp_content = layout_body.iter().cloned().enumerate().map(
		|(line, FrameLine { prefix, line_content })| {
			let line = line as u16;
			let line_parts = line_content.into_iter().map(
				|(WidgetLine { widget_char, line_index, .. }, suffix)| {
					let FieldDetails { access_field, .. } = &widget_indexes[&widget_char];
					let w = access_field(quote!(self));
					quote! {
						#w.display_line(f, #line_index)?;
						f.write_str(#suffix)?;
					}
				},
			);
			quote!(#line => {
				f.write_str(#prefix)?;
				#(#line_parts)*
			})
		},
	);

	let expanded = quote! {
		impl #impl_generics terminity::widgets::Widget for #ident #ty_generics #where_clause {
			fn display_line(&self, f: &mut core::fmt::Formatter<'_>, line: u16) -> std::fmt::Result {
				match line {
					#(#disp_content,)*
					_ => panic!("Displaying line out of struct frame"),
				}
				Ok(())
			}
			fn size(&self) -> terminity::Size {
				terminity::Size{
					width: #frame_width,
					height: #frame_height,
				}
			}
		}
	};

	// TODO: support mouse events
	// if all_impls.bubble_event == Some(()) {
	// 	let enum_name = Ident::new(&(ident.to_string() + "MouseEvents"), ident.span());

	// 	let enum_variants = widget_indexes.values().map(
	// 		|FieldDetails { enum_variant, field_widget_ty, .. }| match enum_variant {
	// 			Some(v) => quote! {
	// 				#v(<#field_widget_ty as terminity::widgets::EventBubblingWidget>::HandledEvent),
	// 			},
	// 			None => quote!(),
	// 		},
	// 	);

	// 	let mouse_event_content =
	// 		layout_body.iter().cloned().enumerate().map(|(line, (prefix, line_parts))| {
	// 			let prefix_len =
	// 				String::from_utf8(strip_ansi_escapes::strip(prefix.value()).unwrap())
	// 					.unwrap()
	// 					.graphemes(true)
	// 					.count() as u16;
	// 			let line_parts = line_parts.into_iter().map(|(((name, _), _line_i), suffix)| {
	// 				let suffix_len =
	// 					String::from_utf8(strip_ansi_escapes::strip(suffix.value()).unwrap())
	// 						.unwrap()
	// 						.graphemes(true)
	// 						.count();
	// 				let details = &widget_indexes[&name];
	// 				let w = (details.field_widget)(quote!(self), false);
	// 				match &details.enum_variant {
	// 					None => quote! {
	// 						curr_col += #w.size().0 + #suffix_len;
	// 					},
	// 					Some(variant) => quote! {
	// 						if curr_col > column {
	// 							return None;
	// 						}
	// 						if curr_col + #w.size().0 > column {
	// 							return Some(#enum_name::#variant(
	// 									terminity::widgets::EventBubblingWidget::bubble_event(
	// 										&mut #w,
	// 										crossterm::event::MouseEvent {
	// 											column: column - curr_col,
	// 											row,
	// 											kind,
	// 											modifiers,
	// 										}
	// 									)
	// 								)
	// 							);
	// 						}
	// 						curr_col += #w.size().0 + #suffix_len;
	// 					},
	// 				}
	// 			});
	// 			quote!(#line => {
	// 				let mut curr_col = #prefix_len;
	// 				#(#line_parts)*
	// 				None
	// 			})
	// 		});

	// 	 expanded.extend(quote! {
	// 	 	#[derive(Clone, PartialEq, Eq, Debug)]
	// 	 	enum #enum_name {
	// 	 		#(#enum_variants)*
	// 	 	}

	// 	 	impl #impl_generics terminity::widgets::EventBubblingWidget for #ident #ty_generics #where_clause {
	// 	 		type FinalWidgetData<'a> = ();
	// 	 		/// Handles a mouse event. see the [trait](Self)'s doc for more details.
	// 	 		fn bubble_event<'a, R, F: FnOnce(Self::FinalWidgetData<'a>) -> R>(
	// 	 			&'a mut self,
	// 	 			event: crossterm::event::MouseEvent,
	// 	 			widget_pos: Position,
	// 	 			callback: F,
	// 	 		) -> R {
	// 	 			todo!()
	// 	 			// let crossterm::event::MouseEvent { column, row, kind, modifiers } = event;
	// 	 			// match row as usize {
	// 	 			// 	#(#mouse_event_content)*
	// 	 			// 	_ => None,
	// 	 			// }
	// 	 		}
	// 	 	}
	// 	 });
	// }

	(expanded, errors)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a() {
		let input = quote! {
			#[widget_impl(EventBubblingWidget)]
			#[widget_layout (
				{
					'c' => .content,
					'H' => .header,
					'l' => .left,
					'r' => .right,
					'F' => .footer,
				},
				"*-------------*",
				"| HHHHHHHHHHH |",
				"|   ccccccc   |",
				"| l ccccccc r |",
				"|   ccccccc   |",
				"| FFFFFFFFFFF |",
				"*-------------*",
			)]
			struct MyFrame {
				content: Img,
				header: Img,
				left: Img,
				right: Img,
				footer: Img,
			}
		};
		let (result, errors) = run(parse2(input).unwrap());
		println!("{}", result);
		println!("--------------------------");
		println!("{:#?}", errors);
		assert!(errors.is_empty());
	}

	#[test]
	fn c() {
		let input = quote! {
			#[widget_layout(
				{
					'0' => .left_border,
					'1' => .left_center_border,
					'2' => .right_center_border,
					'3' => .right_border,
				},
				"0 1 2 3",
				"0 1 2 3",
				"0 1 2 3",
			)]
			pub struct TabSelect {
				left_border: Border,
				left_center_border: Border,
				right_center_border: Border,
				right_border: Border,

				selected: u8,
			}

		};

		let (result, errors) = run(parse2(input).unwrap());
		println!("{}", result);
		println!("--------------------------");
		println!("{:#?}", errors);
		assert!(errors.is_empty());
	}
}
