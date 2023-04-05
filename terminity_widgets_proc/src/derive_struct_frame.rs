use std::{cell::RefCell, collections::HashMap};

use convert_case::{Case, Casing};
use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use quote::quote;

use syn::{
	braced, bracketed, parenthesized,
	parse::{Parse, ParseStream},
	parse2,
	punctuated::Punctuated,
	spanned::Spanned,
	token, DeriveInput, Ident, LitChar, LitStr, Member, Token,
};

use crate::frame::parse_frame_lines;
use unicode_segmentation::UnicodeSegmentation;

struct LayoutArgs {
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
		Ok(LayoutArgs { args: content.parse_terminated(<LitStr as Parse>::parse)? })
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
	#[darling(rename = "MouseEventWidget")]
	mouse_event: Option<()>,
}

#[derive(FromMeta)]
struct AttributeLayoutArgs {
	name: LitChar,
	#[darling(default)]
	ignore_mouse_event: bool,
}

pub fn run(input: DeriveInput) -> (TokenStream, Vec<Diagnostic>) {
	let mut errors = vec![];
	let DeriveInput { attrs, ident, generics, data, .. } = input;
	let (layout, non_layout): (Vec<_>, Vec<_>) =
		attrs.into_iter().partition(|a| a.path.is_ident("sf_layout"));

	let (impls, _othet_attrs): (Vec<_>, Vec<_>) =
		non_layout.into_iter().partition(|a| a.path.is_ident("sf_impl"));

	let (all_impls, layout_content) = if layout.len() != 1 || impls.len() > 1 {
		if layout.len() != 1 {
			errors.push(Diagnostic::spanned(
				Span::call_site(),
				Level::Error,
				concat!(
					"Expecting ONE `#[sf_layout {{}}]` attribute on the struct ",
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
					"Expecting at most one `#[sf_impl (...)]` attribute ",
					"to indicate what widget traits to implement. Found {} of them."
				)
				.into(),
			));
		}
		return (quote!().into(), errors);
	} else {
		let layout_content: LayoutArgs = match parse2(layout[0].tokens.clone()) {
			Ok(v) => v,
			Err(e) => {
				errors.push(e.into());
				return (quote!().into(), errors);
			}
		};
		let all_impls = if impls.len() == 0 {
			None
		} else {
			match impls[0].parse_meta().map(|m| SFImplArgs::from_meta(&m)) {
				Ok(Ok(v)) => Some(v),
				Ok(Err(e)) => {
					errors.push(Diagnostic::spanned(
						e.span(),
						Level::Error,
						format!("{}", e).into(),
					));
					None
				}
				Err(e) => {
					errors.push(Diagnostic::spanned(
						e.span(),
						Level::Error,
						format!("{}", e).into(),
					));
					None
				}
			}
		}
		.unwrap_or(SFImplArgs { mouse_event: None });
		(all_impls, layout_content)
	};

	let widget_indexes = match data {
		syn::Data::Union(_) => {
			errors.push(Diagnostic::spanned(
				Span::call_site(),
				Level::Error,
				"Can't build a struct frame from a union.".into(),
			));
			todo!()
		}
		syn::Data::Enum(_) => {
			errors.push(Diagnostic::spanned(
				Span::call_site(),
				Level::Error,
				"Can't build a struct frame from an enum.".into(),
			));
			todo!()
		}
		syn::Data::Struct(content) => {
			let mut res = HashMap::new();
			for (field_index, f) in content.fields.into_iter().enumerate() {
				let (mut raw_attrs, others): (Vec<_>, Vec<_>) =
					f.attrs.iter().partition(|f| f.path.is_ident("sf_layout"));
				if raw_attrs.len() > 1 {
					errors.push(Diagnostic::spanned(
						f.span(),
						Level::Error,
						format!(
							"Expecting at most one attribute #[sf_layout(...)] per field. Found: {:?}",
							others
								.into_iter()
								.map(|a| a.path.get_ident().unwrap().to_string())
								.collect::<Vec<_>>()
						),
					));
					todo!();
				}
				let (_, attr_details) = match raw_attrs.pop() {
					Some(d) => (
						d.span(),
						d.parse_meta()
							.map(|m| AttributeLayoutArgs::from_meta(&m))
							.unwrap_or_else(|e| Err(e.into())),
					),
					None => continue,
				};
				// Extract attr_details
				let attr_details = match attr_details {
					Ok(d) => d,
					Err(e) => {
						errors.push(Diagnostic::spanned(f.span(), Level::Error, format!("{}", e)));
						continue;
					}
				};

				let details = (
					// Field name to access the field
					f.ident.as_ref().map(|i| Member::Named(i.clone())).unwrap_or(Member::Unnamed(
						syn::Index { index: field_index as u32, span: f.span() },
					)),
					// Corresponding enum variant
					if attr_details.ignore_mouse_event {
						None
					} else {
						Some(Ident::new(
							&f.ident
								.as_ref()
								.map(|i| i.to_string().to_case(Case::Pascal))
								.unwrap_or("_".to_owned() + &field_index.to_string()),
							f.ident.span(),
						))
					},
					// Type
					f.ty,
					// Size
					RefCell::new(None),
				);
				if let Some(_) = res.insert(attr_details.name.value(), details) {
					errors.push(Diagnostic::spanned(
						attr_details.name.span(),
						Level::Error,
						format!(
							"There are multiple fields of frame name {:?}.",
							attr_details.name.value()
						),
					));
				}
			}
			res
		}
	};

	let mut errors = vec![];
	let mut frame_width = None;

	let layout_body = parse_frame_lines(
		&mut frame_width,
		&mut errors,
		&layout_content.args.into_iter().collect::<Vec<_>>(),
		widget_indexes.iter().map(|(name, (_, _, _, size))| (*name, size)).collect(),
	);

	let frame_width = frame_width.expect("Error: Empty struct frame layout");
	let frame_height = layout_body.len();

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let disp_content =
		layout_body.iter().cloned().enumerate().map(|(line, (prefix, line_parts))| {
			let line_parts = line_parts.into_iter().map(|(((name, _), line_i), suffix)| {
				let (field, ..) = &widget_indexes[&name];
				quote! {
					self.#field.displ_line(f, #line_i)?;
					f.write_str(#suffix)?;
				}
			});
			quote!(#line => {
				f.write_str(#prefix)?;
				#(#line_parts)*
			})
		});

	let mut expanded = quote! {
		#(#errors)* // Give the errors
		impl #impl_generics terminity_widgets::Widget for #ident #ty_generics #where_clause {
			fn displ_line(&self, f: &mut core::fmt::Formatter<'_>, line: usize) -> std::fmt::Result {
				match line {
					#(#disp_content,)*
					_ => panic!("Displaying line out of struct frame"),
				}
				Ok(())
			}
			fn size(&self) -> (usize, usize) {
				(#frame_width, #frame_height)
			}
		}
	};

	if all_impls.mouse_event == Some(()) {
		let enum_name = Ident::new(&(ident.to_string() + "MouseEvents"), ident.span());

		let enum_variants =
			widget_indexes.values().map(|(_, variant, field_type, _)| match variant {
				Some(v) => quote! {
					#v(<#field_type as terminity_widgets::MouseEventWidget>::MouseHandlingResult),
				},
				None => quote!(),
			});

		let mouse_event_content =
			layout_body.iter().cloned().enumerate().map(|(line, (prefix, line_parts))| {
				let prefix_len =
					String::from_utf8(strip_ansi_escapes::strip(&prefix.value()).unwrap())
						.unwrap()
						.graphemes(true)
						.count();
				let line_parts = line_parts.into_iter().map(|(((name, _), _line_i), suffix)| {
					let suffix_len =
						String::from_utf8(strip_ansi_escapes::strip(&suffix.value()).unwrap())
							.unwrap()
							.graphemes(true)
							.count();
					let (field, enum_variant, _, _) = &widget_indexes[&name];
					match enum_variant {
						None => quote! {
							curr_col += self.#field.size().0 + #suffix_len;
						},
						Some(variant) => quote! {
							if curr_col > column as usize {
								return None;
							}
							if curr_col + self.#field.size().0 > column as usize {
								return Some(#enum_name::#variant(
										terminity_widgets::MouseEventWidget::mouse_event(
											&mut self.#field,
											crossterm::event::MouseEvent {
												column: column - curr_col as u16,
												row,
												kind,
												modifiers,
											}
										)
									)
								);
							}
							curr_col += self.#field.size().0 + #suffix_len;
						},
					}
				});
				quote!(#line => {
					let mut curr_col = #prefix_len;
					#(#line_parts)*
					None
				})
			});

		expanded.extend(quote! {
			#[derive(Clone, PartialEq, Eq, Debug)]
			enum #enum_name {
				#(#enum_variants)*
			}

			impl #impl_generics terminity_widgets::MouseEventWidget for #ident #ty_generics #where_clause {
				type MouseHandlingResult = Option<#enum_name>;
				fn mouse_event(&mut self, event: crossterm::event::MouseEvent) -> Self::MouseHandlingResult {
					let crossterm::event::MouseEvent { column, row, kind, modifiers } = event;
					match row as usize {
						#(#mouse_event_content)*
						_ => None,
					}
				}
			}
		});
	}

	(expanded, errors)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a() {
		let input = quote! {
			#[sf_impl(MouseEventWidget)]
			#[sf_layout {
				"*-------------*",
				"| HHHHHHHHHHH |",
				"|   ccccccc   |",
				"| l ccccccc r |",
				"|   ccccccc   |",
				"| FFFFFFFFFFF |",
				"*-------------*",
			}]
			struct MyFrame {
				#[sf_layout(name = 'c')]
				content: Img,
				#[sf_layout(name = 'H')]
				header: Img,
				#[sf_layout(name = 'l')]
				left: Img,
				#[sf_layout(name = 'r')]
				right: Img,
				#[sf_layout(name = 'F')]
				footer: Img,
			}
		};
		let (result, errors) = run(parse2(input).unwrap());
		println!("{}", result);
		println!("--------------------------");
		println!("{:?}", errors);
	}
}
