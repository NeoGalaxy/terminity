use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;
use quote::quote;

use syn::DeriveInput;

pub fn run(input: DeriveInput) -> (TokenStream, Vec<Diagnostic>) {
	let name = input.ident;

	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let expanded = quote! {
		impl #impl_generics terminity::widgets::EventBubbling for #name #ty_generics #where_clause {

			type FinalData<'a> = &'a mut Self;
			fn bubble_event<'a, R, F: FnOnce(Self::FinalData<'a>, terminity::widgets::BubblingEvent) -> R>(
				&'a mut self,
				event: terminity::widgets::BubblingEvent,
				callback: F,
			) -> R {
				callback(self, event)
			}
		}
	};

	(expanded, vec![])
}
