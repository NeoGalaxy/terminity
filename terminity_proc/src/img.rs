use proc_macro2::TokenStream;
use proc_macro_error::{Diagnostic, Level};
use quote::quote;

use syn::{
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	LitStr, Token,
};

use crate::wstr::{self, LineData};

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
	let mut line_data = vec![];
	let mut content = String::new();
	let mut width = None;

	for line in input.lines {
		let (mut l_lines, l_content, l_errors) = wstr::build_str(line.value()).unwrap_or((
			vec![LineData { pos: 0, width: 0 }],
			String::new(),
			vec![],
		));

		for l in &l_lines {
			if let Some(w) = width {
				if l.width != w {
					errors.push(Diagnostic::spanned(
						line.span(),
						Level::Error,
						"All the lines are not of the same length".to_owned(),
					))
				}
			} else {
				width = Some(l.width)
			}
		}

		let start_pos = content.len() as u16;
		content.push_str(&l_content);
		for data in &mut l_lines {
			data.pos += start_pos;
		}
		line_data.extend(l_lines);
		errors.extend(l_errors);
	}

	let height = line_data.len() as u16;
	let width = width.unwrap_or(0);
	let content = wstr::details_to_tokens(line_data, content);

	(
		quote! {
			unsafe { terminity::widgets::content::Img::from_raw_parts(
				#content,
				terminity::Size {
					width: #width,
					height: #height
				}
			) }
		},
		errors,
	)
}
