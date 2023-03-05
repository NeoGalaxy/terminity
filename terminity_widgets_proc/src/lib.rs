#![warn(missing_docs)]

/*! Crate defining the procedural macros for
	[terminity_widgets](https://docs.rs/terminity_widgets/latest/terminity_widgets/index.html).

	This crate defines two macros, [`frame!`] and [`derive(WidgetDisplay)`](widget_display),
	to help you write your apps with terminity's widgets. This is at a very early development stage,
	and breaking changes or new macros (as a StructFrame one) are to be foreseen.

*/

mod frame;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;

use syn::{parse_macro_input, DeriveInput};

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
	proc_macro::TokenStream::from(frame::run(parse_macro_input!(tokens as frame::FrameMacro)))
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
