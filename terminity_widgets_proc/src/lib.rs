mod frame;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;

use syn::{parse_macro_input, DeriveInput};

#[proc_macro_error]
#[proc_macro]
pub fn frame(tokens: TokenStream) -> TokenStream {
	proc_macro::TokenStream::from(frame::run(parse_macro_input!(tokens as frame::FrameMacro)))
}

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
					f.write_str(&format!("{}\n\r",
						terminity_widgets::_reexport::Clear(terminity_widgets::_reexport::UntilNewLine)))?;
				}
				Ok(())
			}
		}
	};
	proc_macro::TokenStream::from(expanded)
}
