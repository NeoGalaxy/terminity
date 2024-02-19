#![allow(clippy::tabs_in_doc_comments)]
#![warn(missing_docs)]

/*! Crate defining the procedural macros for
	[terminity](https://docs.rs/terminity/latest/terminity/index.html).

	This crate defines two macros, [`frame!`] and [`derive(WidgetDisplay)`](widget_display),
	to help you write your apps with terminity's widgets. This is at a very early development stage,
	and breaking changes or new macros (as a StructFrame one) are to be foreseen.

*/

mod derive_struct_frame;
mod frame;
mod img;
mod wstr;

use proc_macro::TokenStream;

use proc_macro_error::proc_macro_error;
use quote::quote;

use syn::{parse_macro_input, DeriveInput, LitStr};

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
/// [`Frame`](https://docs.rs/terminity/latest/terminity/widgets/frame/struct.Frame.html)
/// for more details on how to use a frame.
///
/// Example:
/// ```
/// use terminity_proc::frame;
/// use terminity::widgets::text::Text;
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
/// use terminity_proc::frame;
/// use terminity::widgets::text::Text;
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

#[proc_macro_error]
#[proc_macro_derive(StructFrame, attributes(sf_impl, sf_layout))]
pub fn struct_frame(tokens: TokenStream) -> TokenStream {
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

#[proc_macro_derive(EventBubblingWidget)]
pub fn event_bubbling_widget(tokens: TokenStream) -> TokenStream {
	let input = parse_macro_input!(tokens as DeriveInput);

	let name = input.ident;

	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let expanded = quote! {
		impl #impl_generics terminity::widgets::EventBubblingWidget for #name #ty_generics #where_clause {

			type FinalWidgetData<'a> = &'a mut Self;
			fn bubble_event<'a, R, F: FnOnce(Self::FinalWidgetData<'a>, BubblingEvent) -> R>(
				&'a mut self,
				event: BubblingEvent,
				callback: F,
			) -> R {
				callback(self, event)
			}
		}
	};
	proc_macro::TokenStream::from(expanded)
}

#[proc_macro_error]
#[proc_macro]
pub fn wstr(tokens: TokenStream) -> TokenStream {
	let (tokens, errors) = wstr::run(parse_macro_input!(tokens as LitStr));
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
