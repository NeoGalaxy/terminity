use proc_macro2::TokenStream;
use proc_macro_error::{Diagnostic, Level};
use quote::quote;

use syn::{
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	LitStr, Token,
};

use crate::wstr;

#[allow(dead_code)]
pub struct ImgMacro {
	lines: Punctuated<LitStr, Token![,]>,
}

impl Parse for ImgMacro {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(Self { lines: Punctuated::parse_terminated(input)? })
	}
}

pub fn run(input: ImgMacro) -> (TokenStream, Vec<Diagnostic>) {
	let mut errors = vec![];
	let mut content = String::new();
	let mut lines = Vec::new();
	let mut width = None;

	for line in input.lines {
		let (l_width, l_content, mut l_errors) = wstr::parse_line(&line);
		if let Some(width) = width {
			if l_width != width {
				errors.push(Diagnostic::spanned(
					line.span(),
					Level::Error,
					"All the lines are not of the same length".to_owned(),
				))
			}
		} else {
			width = Some(l_width)
		}

		errors.append(&mut l_errors);
		let pos = content.len() as u16;
		lines.push(quote!(terminity::widget_string::LineInfo {pos: #pos, width: #l_width}));
		content.push_str(&l_content);
	}

	let height = lines.len() as u16;
	let width = width.unwrap_or(0);

	(
		quote!(
			unsafe { terminity::widgets::content::Img::from_raw_parts(
				terminity::widget_string::WidgetStr::from_content_unchecked(
					#content,
					&[#(#lines),*]
				),
				terminity::Size {
					width: #width,
					height: #height
				}
		) }),
		errors,
	)
}
