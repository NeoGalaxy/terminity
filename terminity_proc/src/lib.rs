#![allow(clippy::tabs_in_doc_comments)]
#![warn(missing_docs)]

/*! Crate defining the procedural macros for
	[terminity](https://docs.rs/terminity/latest/terminity/index.html).

	This crate defines two macros, [`frame!`] and [`derive(WidgetDisplay)`](widget_display),
	to help you write your apps with terminity's widgets. This is at a very early development stage,
	and breaking changes or new macros (as a StructFrame one) are to be foreseen.

*/

// mod derive_event_bubbling_widget;
mod derive_struct_frame;
mod frame;
mod img;
mod wstr;

use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;
use quote::quote;

use syn::{parse_macro_input, DeriveInput, LitChar, LitStr};
use wstr::WStrMacro;

/*
seq: ESQ [ <val> m

val: 0x1B[<val>m | \e[<val>m | \033[<val>m

https://en.wikipedia.org/wiki/ANSI_escape_code

X0 black, gray
X1 red
X2 green
X3 yellow
X4 blue
X5 magenta
X6 cyan
X7 white

0 ~all
1 bold
2 dimmed
3 italic
4 underline
5 sblink
6 rblink/fblink
7 reverse
8 -
9 strike
10 primaryfont
11-19 alternativefont
20 fractur
21 -
22 ~1,2
23 ~3
24 ~4
25 ~5,6
26 -
27 ~7
28 ~8
29 ~9
30-38 fg color
39 ~30-38,90-97
40-48 bg color
49 ~40-48,100-107
50 ~26
51 framed (?)
52 encirled (?)
53 overline
54 ~51,52
55 ~53
58 - (underline color)
59 ~58
60 -
61 -
62 -
63 -
64 -
65 ~60-64
73 -
74 -
75 -
90-97 bright fg color
100-107 bright bg color
*/

#[proc_macro_error]
#[proc_macro_derive(Widget, attributes(/*widget_impl,*/ widget_layout))]
pub fn widget(tokens: TokenStream) -> TokenStream {
	let (tokens, errors) = derive_struct_frame::run(parse_macro_input!(tokens as DeriveInput));
	for e in errors {
		e.emit();
	}
	proc_macro::TokenStream::from(tokens)
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
/// use terminity::Widget;
/// use terminity_proc::WidgetDisplay;
/// #[derive(WidgetDisplay)]
/// struct MyWidget();
///
/// impl Widget for MyWidget {
/// 	fn size(&self) -> (usize, usize) {
/// 		(5, 2)
/// 	}
/// 	fn display_line(&self, f: &mut std::fmt::Formatter<'_>, mut line_nb: usize) -> std::fmt::Result {
/// 		match line_nb {
/// 			0 => f.write_str("Hello"),
/// 			1 => f.write_str("World"),
/// 			_ => panic!("Error: tried to print outside a widget")
/// 		}
/// 	}
/// }
///
/// # fn main() {
/// let foo = MyWidget();
/// println!("{}", foo); //"Hello\nWorld"
/// # }
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
				for i in 0..self.size().height {
					self.display_line(f, i)?;
					if i != self.size().height - 1 {
						f.write_str(&format!("{}\n\r",
							terminity::_reexport::Clear(terminity::_reexport::UntilNewLine)))?;
					}
				}
				Ok(())
			}
		}
	};
	proc_macro::TokenStream::from(expanded)
}

// #[proc_macro_error]
// #[proc_macro_derive(EventBubblingWidget)]
// pub fn event_bubbling_widget(tokens: TokenStream) -> TokenStream {
// 	let (tokens, errors) =
// 		derive_event_bubbling_widget::run(parse_macro_input!(tokens as DeriveInput));
// 	for e in errors {
// 		e.emit();
// 	}
// 	proc_macro::TokenStream::from(tokens)
// }

#[proc_macro_error]
#[proc_macro]
pub fn wchar(tokens: TokenStream) -> TokenStream {
	let (tokens, errors) = wstr::wchar(parse_macro_input!(tokens as LitChar));
	for e in errors {
		e.emit();
	}
	proc_macro::TokenStream::from(tokens)
}

#[proc_macro_error]
#[proc_macro]
pub fn wstr(tokens: TokenStream) -> TokenStream {
	let (tokens, errors) = wstr::wstr(parse_macro_input!(tokens as WStrMacro));
	for e in errors {
		e.emit();
	}
	proc_macro::TokenStream::from(tokens)
}

#[proc_macro_error]
#[proc_macro]
pub fn wline(tokens: TokenStream) -> TokenStream {
	let (tokens, errors) = wstr::wline(parse_macro_input!(tokens as LitStr));
	for e in errors {
		e.emit();
	}
	proc_macro::TokenStream::from(tokens)
}

#[proc_macro_error]
#[proc_macro]
pub fn img(tokens: TokenStream) -> TokenStream {
	let (tokens, errors) = img::run(parse_macro_input!(tokens as img::ImgMacro));
	for e in errors {
		e.emit();
	}
	proc_macro::TokenStream::from(tokens)
}
