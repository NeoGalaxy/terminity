use std::mem::size_of;

use proc_macro2::TokenStream;
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use syn::LitStr;

#[derive(Debug)]
pub struct LineData {
	pub pos: u16,
	pub width: u16,
}

pub fn run(input: LitStr) -> (TokenStream, Vec<Diagnostic>) {
	match build_str(input.value()) {
		None => (
			quote! {
				unsafe{
					terminity::widgets::WidgetStr::from_raw(&[])
				}
			},
			vec![],
		),
		Some((line_data, result, errors)) => (details_to_tokens(line_data, result), errors),
	}
}

pub fn build_str(input_val: String) -> Option<(Vec<LineData>, String, Vec<Diagnostic>)> {
	if input_val.is_empty() {
		return None;
	}
	let mut errors = vec![];
	let mut result = String::new();
	let mut line_data = vec![LineData { pos: 0, width: 0 }];

	let mut chars = input_val.chars();
	while let Some(c) = chars.next() {
		match c {
			'\r' => (),
			'\n' => line_data.push(LineData { pos: result.len() as u16, width: 0 }),
			'\x1b' => errors.push(Diagnostic::new(
				Level::Error,
				"ANSI escapes can't be used in WidgetStr literals.".to_owned(),
			)),
			'\\' => match chars.next() {
				Some('#') => errors.push(Diagnostic::new(
					Level::Error,
					"Escapes are not yet supported".to_owned(),
				)),
				Some(c) => {
					result.push('\\');
					result.push(c);
					line_data.last_mut().unwrap().width += 2;
				}
				None => {
					result.push(c);
					line_data.last_mut().unwrap().width += 1;
				}
			},
			c => {
				result.push(c);
				line_data.last_mut().unwrap().width += 1;
			}
		}
	}
	Some((line_data, result, errors))
}

pub fn details_to_tokens(line_data: Vec<LineData>, result: String) -> TokenStream {
	// write width + content
	let mut data: Vec<_> =
		(line_data.len() as u16).to_le_bytes().into_iter().chain(result.into_bytes()).collect();

	// write end pos
	data.extend((data.len() as u16).to_le_bytes());
	// write lines details
	for line in line_data.into_iter().rev() {
		let pos = (line.pos + size_of::<u16>() as u16).to_le_bytes();
		let len = line.width.to_le_bytes();
		data.extend([pos[0], pos[1], len[0], len[1]]);
	}
	let res = quote! {
		unsafe {
			terminity::widgets::WidgetStr::from_raw(&[#(#data,)*])
		}
	};
	res
}
