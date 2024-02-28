use proc_macro2::TokenStream;
use proc_macro_error::{Diagnostic, Level};
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, LitChar, LitStr, Token};

#[derive(Debug)]
pub struct LineData {
	pub pos: u16,
	pub width: u16,
}

pub struct WStrMacro(Punctuated<LitStr, Token![,]>);

impl Parse for WStrMacro {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(WStrMacro(Punctuated::parse_terminated(input)?))
	}
}

pub fn wchar(input: LitChar) -> (TokenStream, Vec<Diagnostic>) {
	let (out, e) = if input.value().is_control() {
		(
			char::REPLACEMENT_CHARACTER,
			vec![Diagnostic::spanned(
				input.span(),
				Level::Error,
				"Control characters are not allowed".into(),
			)],
		)
	} else {
		(input.value(), vec![])
	};

	(quote!(unsafe{ terminity::widgets::wchar::WChar::from_char_unchecked(#out) }), e)
}

pub fn wstr(input: WStrMacro) -> (TokenStream, Vec<Diagnostic>) {
	if input.0.is_empty() {
		return (
			quote!(unsafe { terminity::widget_string::WidgetStr::from_content_unchecked("", &[]) }),
			vec![Diagnostic::new(
				Level::Error,
				"Empty widget strings are not yet allowed.\n\
				Try instead a widget string with a single empty line: wstr![\"\"]"
					.to_string(),
			)],
		);
	}

	let mut errors = vec![];
	let mut content = String::new();
	let mut lines = Vec::new();

	for line in input.0 {
		let (w, c, mut errs) = parse_line(&line);
		errors.append(&mut errs);
		let pos = content.len() as u16;
		lines.push(quote!(terminity::widget_string::LineInfo {pos: #pos, width: #w}));
		content.push_str(&c);
	}

	(
		quote!(unsafe{ terminity::widget_string::WidgetStr::from_content_unchecked(
			#content,
			&[#(#lines)*]
		) }),
		errors,
	)
}

pub fn wline(input: LitStr) -> (TokenStream, Vec<Diagnostic>) {
	let (w, content, errs) = parse_line(&input);

	(
		quote!(unsafe{ terminity::widget_string::line::WidgetLine::from_parts_unchecked(
			#content,
			#w
		) }),
		errs,
	)
}

pub fn parse_line(input: &LitStr) -> (u16, String, Vec<Diagnostic>) {
	let input_val = input.value();
	let mut errors = vec![];
	let mut result = String::new();
	let mut width = 0;

	let chars = input_val.chars();
	let mut newlines = vec![];
	for c in chars {
		match c {
			'\r' => (),
			'\n' => newlines.push(result.len()),
			'\x1b' => errors.push(Diagnostic::spanned(
				input.span(),
				Level::Error,
				"Escape codes like ANSI escapes ('\\x1b') can't be used in WidgetStr literals."
					.to_owned(),
			)),
			c => {
				if c.is_control() {
					errors.push(Diagnostic::spanned(
						input.span(),
						Level::Error,
						format!(
							"Character {:?} is a control character \
							(Widget strings don't allow these)",
							c
						),
					))
				} else {
					use unicode_width::UnicodeWidthChar;
					result.push(c);
					width += c.width().unwrap() as u16;
				}
			}
		}
	}
	if !newlines.is_empty() {
		// Take the indexes in reverse order (to avoid indexes changing)
		let (segments, prefix) =
			newlines.iter().rev().fold((vec![], result.as_str()), |(mut segs, remain), idx| {
				let parts = remain.split_at(*idx);
				segs.push(parts.1);
				(segs, parts.0)
			});
		let segments: Vec<_> = [prefix].into_iter().chain(segments.into_iter().rev()).collect();

		errors.push(Diagnostic::spanned(
			input.span(),
			Level::Error,
			format!(
				"Newlines controls are not allowed in wstr. \
				Try instead breaking your string into multiple lines:\n\
				wstr!{segments:#?}"
			),
		));
	}
	(width, result, errors)
}
