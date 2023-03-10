#![warn(missing_docs)]

/*! Crate defining the procedural macros for
	[terminity_widgets](https://docs.rs/terminity_widgets/latest/terminity_widgets/index.html).

	This crate defines two macros, [`frame!`] and [`derive(WidgetDisplay)`](widget_display),
	to help you write your apps with terminity's widgets. This is at a very early development stage,
	and breaking changes or new macros (as a StructFrame one) are to be foreseen.

*/

mod frame;

use std::{cell::RefCell, collections::HashMap};

use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, emit_call_site_error, emit_error, proc_macro_error};
use quote::quote;

use syn::{
	braced,
	parse::{Nothing, Parse, ParseStream},
	parse2, parse_macro_input,
	punctuated::Punctuated,
	spanned::Spanned,
	DeriveInput, LitStr, Meta, NestedMeta, PathArguments, Token,
};

use crate::frame::parse_frame_lines;

/// Utility macro to build collection frames, aka. Frames.
///
/// The internal representation and the constructor of Frame may be a bit opaque. This macro
/// aims to create a frame in a very visual and explicit way. There are currently two syntaxes.
/// The move syntax allows to wrap any collection into a frame (like a HashMap for instance),
/// while the array syntax allows to be a tiny bit more explicit on which widget goes where.
///
/// # The Move syntax
///
/// TL;DR: check the example.
///
/// The move syntax is defined as followed (with non-terminals in italic, terminals as code and
/// full caps using rust's syntax) :
///
/// > *MoveSyntax* : &#9;*collection* `=>` `{` *IndexMap* `}` *FrameContent*
/// >
/// > *collection* : EXPRESSION
/// >
/// > *IndexMap* : (CHAR `:` EXPRESSION`,`)*
/// >
/// > *FrameContent* : STRING_LITERAL*
/// >
///
/// It takes ownership of the expression designated by *collection*, and uses the the *IndexMap*
/// to know the index of each widget and to be able to recognise the widgets in the *FrameContent*.
/// Any occurrence of a character used in the *IndexMap* will be replaced with the widget at the
/// index corresponding to the associated expression. For instance, in the following example, the
/// box described with `H`s will be replaced by the lines of `texts[0]`, and the one with `W`s by
/// the lines of `texts[1]`. See the doc of
/// [`Frame`](https://docs.rs/terminity_widgets/latest/terminity_widgets/widgets/frame/struct.Frame.html)
/// for more details on how to use a frame.
///
/// Example:
/// ```
/// use terminity_widgets_proc::frame;
/// use terminity_widgets::widgets::text::Text;
/// let texts = vec![
/// 	Text::new(["Hello".into(), "-----".into()], 5),
/// 	Text::new(["World".into(), "".into()], 5),
/// ];
/// let mut framed_texts = frame!(texts => { 'H': 0, 'W': 1 }
/// 	"*~~~~~~~~~~~~~~*"
/// 	"| HHHHH WWWWW! |"
/// 	"| HHHHH-WWWWW- |"
/// 	"*~~~~~~~~~~~~~~*"
/// );
/// framed_texts[1][1] = String::from("-----");
///
/// println!("{}", framed_texts);
/// // *~~~~~~~~~~~~~~*
/// // | Hello World! |
/// // | ------------ |
/// // *~~~~~~~~~~~~~~*
/// ```
///
/// # The Array syntax
///
/// TL;DR: check the example.
///
/// The array syntax is defined as followed :
///
/// > *MoveSyntax* : `[`*IndexedArray*`]` *FrameContent*
/// >
/// > *IndexedArray* : (CHAR `:` EXPRESSION`,`)*
/// >
/// > *FrameContent* : STRING_LITERAL*
/// >
///
/// This creates an array on the fly containing the values of the expressions given in
/// *IndexedArray* and makes it into a frame. It then uses the keys of *IndexedArray*
/// to know which is the char in the *FrameContent* to replace by the corresponding expression.
///
/// Here is an example equivalent to the previous one, using the array syntax:
/// ```
/// use terminity_widgets_proc::frame;
/// use terminity_widgets::widgets::text::Text;
/// let mut framed_texts = frame!(
/// 	[
/// 		'H': Text::new(["Hello".into(), "-----".into()], 5),
/// 		'W': Text::new(["World".into(), "".into()], 5),
/// 	]
/// 	"*~~~~~~~~~~~~~~*"
/// 	"| HHHHH WWWWW! |"
/// 	"| HHHHH-WWWWW- |"
/// 	"*~~~~~~~~~~~~~~*"
/// );
/// framed_texts[1][1] = String::from("-----");
///
/// println!("{}", framed_texts);
/// // *~~~~~~~~~~~~~~*
/// // | Hello World! |
/// // | ------------ |
/// // *~~~~~~~~~~~~~~*
/// ```
#[proc_macro_error]
#[proc_macro]
pub fn frame(tokens: TokenStream) -> TokenStream {
	let (tokens, errors) = frame::run(parse_macro_input!(tokens as frame::FrameMacro));
	for e in errors {
		e.emit();
	}
	proc_macro::TokenStream::from(tokens)
}

struct LayoutArgs {
	args: Punctuated<LitStr, Token![,]>,
}

impl Parse for LayoutArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let content;
		braced!(content in input);
		Ok(LayoutArgs { args: content.parse_terminated(<LitStr as Parse>::parse)? })
	}
}

fn parse_attr_content(content: Meta) -> Result<Vec<(syn::Path, syn::Lit)>, syn::Error> {
	match content {
		Meta::Path(_) => Ok(vec![]),
		Meta::NameValue(nv) => Ok(vec![(nv.path, nv.lit)]),
		Meta::List(l) => {
			let r = l
				.nested
				.into_iter()
				.map(|meta| match meta {
					NestedMeta::Meta(m) => match m {
						Meta::NameValue(nv) => Ok((nv.path, nv.lit)),
						_ => Err(syn::Error::new(m.span(), "Expected 'key = value'")),
					},
					NestedMeta::Lit(_) => todo!(),
				})
				.collect();
			r
		}
	}
}

///
#[proc_macro_error]
#[proc_macro_derive(StructFrame, attributes(layout))]
pub fn struct_frame(tokens: TokenStream) -> TokenStream {
	let input = parse_macro_input!(tokens as DeriveInput);

	let DeriveInput { attrs, ident, generics, data, .. } = input;
	let (layout, non_layout): (Vec<_>, Vec<_>) =
		attrs.into_iter().partition(|a| a.path.is_ident("layout"));

	let widget_indexes = match data {
		syn::Data::Union(_) => {
			abort!(Span::call_site(), "Can't build a struct frame from a union.")
		}
		syn::Data::Enum(_) => abort!(Span::call_site(), "Can't build a struct frame from an enum."),
		syn::Data::Struct(content) => {
			let mut res = HashMap::new();
			for (field_index, f) in content.fields.into_iter().enumerate() {
				let (mut attr_details, others): (Vec<_>, Vec<_>) =
					f.attrs.iter().partition(|f| f.path.is_ident("layout"));
				if attr_details.len() != 1 {
					abort!(
						f.span(),
						"Expecting ONE attribute #[layout(...)] per field. Found: {:?}",
						others
							.into_iter()
							.map(|a| a.path.get_ident().unwrap().to_string())
							.collect::<Vec<_>>()
					);
				}
				let attr_details = match attr_details.pop() {
					None => Ok(vec![]),
					Some(d) => d.parse_meta().map(parse_attr_content).unwrap_or_else(|e| Err(e)),
				};
				// Extract attr_details
				let name = attr_details
					.map(|d| {
						d.into_iter().try_fold(None, |name, (p, val)| {
							if p.is_ident("name") {
								if name.is_some() {
									Err(syn::Error::new(
										p.span(),
										"Multiple 'name' fields unexpected",
									))
								} else {
									match val {
										syn::Lit::Char(c) => Ok(Some(c)),
										_ => Err(syn::Error::new(val.span(), "Expected a char")),
									}
								}
							} else {
								Err(syn::Error::new(p.span(), "Unexpected key. Expected 'name'"))
							}
						})
					})
					.unwrap_or_else(|e| Err(e));
				let name = match name {
					Ok(Some(d)) => d,
					Ok(None) => abort!(f.span(), "Missing name for frame layout"),
					Err(e) => return e.to_compile_error().into(),
				};

				let details = (
					// How to access the field
					f.ident.map(|i| quote!(#i)).unwrap_or(quote!(#field_index)),
					// Size
					RefCell::new(None),
				);
				if let Some(_) = res.insert(name.value(), details) {
					abort!(
						name.span(),
						"There are multiple fields of frame name {:?}.",
						name.value()
					);
				}
			}
			res
		}
	};

	let mut errors = vec![];
	let mut frame_width = None;
	let layout_body = if layout.len() != 1 {
		abort!(
			Span::call_site(),
			"Expecting ONE `#[layout {{}}]` attribute to indicate the frame's layout. Found: {:?}",
			non_layout
				.into_iter()
				.map(|a| a.path.get_ident().unwrap().to_string())
				.collect::<Vec<_>>()
		);
	} else {
		let layout_raw: LayoutArgs = parse2(layout[0].tokens.clone()).unwrap();
		parse_frame_lines(
			&mut frame_width,
			&mut errors,
			&layout_raw.args.into_iter().collect::<Vec<_>>(),
			widget_indexes.iter().map(|(name, (_, size))| (*name, size)).collect(),
		)
	};
	let frame_width = frame_width.expect("Error: Empty layout on struct frame");
	let frame_height = layout_body.len();

	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let disp_content = layout_body.into_iter().enumerate().map(|(line, (prefix, line_parts))| {
		let line_parts = line_parts.into_iter().map(|(((name, _), line_i), suffix)| {
			let (field, _size) = &widget_indexes[&name];
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

	let expanded = quote! {
		#(#errors)* // Give the errors
		impl #impl_generics Widget for #ident #ty_generics #where_clause {
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
	proc_macro::TokenStream::from(expanded)
}

/// Derive macro to automatically implement [`Display`](std::fmt::Display) on widgets.
///
/// It iterates through the lines of the designated widget and writes them one after the other.
/// Doesn't write a newline on the last line.
///
/// Due to the way raw mode works and since this derive is mainly to display in console in terminity
/// (that uses raw mode internally), current implementation actually prints 3 things at each newline.
/// First, it prints the (ANSI) escape sequence to clear the line, then `"\n\r"`, since raw mode
/// doesn't automatically make a `\r` on new line.
/// This might be suspect to change and even removal and replaced by an addition in terminity's api.
///
/// Example:
/// ```
/// use terminity_widgets::Widget;
/// use terminity_widgets_proc::WidgetDisplay;
/// #[derive(WidgetDisplay)]
/// struct MyWidget();
///
/// impl Widget for MyWidget {
/// 	fn size(&self) -> (usize, usize) {
/// 		(5, 2)
/// 	}
/// 	fn displ_line(&self, f: &mut std::fmt::Formatter<'_>, mut line_nb: usize) -> std::fmt::Result {
/// 		match line_nb {
/// 			0 => f.write_str("Hello"),
/// 			1 => f.write_str("World"),
/// 			_ => panic!("Error: tried to print outside a widget")
/// 		}
/// 	}
/// }
///
/// fn main() {
/// 	let foo = MyWidget();
/// 	println!("{}", foo); //"Hello\nWorld"
/// }
/// ```
///
#[proc_macro_derive(WidgetDisplay)]
pub fn widget_display(tokens: TokenStream) -> TokenStream {
	let input = parse_macro_input!(tokens as DeriveInput);

	let name = input.ident;

	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let expanded = quote! {
		impl #impl_generics std::fmt::Display for #name #ty_generics #where_clause {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				for i in 0..self.size().1 {
					self.displ_line(f, i)?;
					if i != self.size().1 - 1 {
						f.write_str(&format!("{}\n\r",
							terminity_widgets::_reexport::Clear(terminity_widgets::_reexport::UntilNewLine)))?;
					}
				}
				Ok(())
			}
		}
	};
	proc_macro::TokenStream::from(expanded)
}
