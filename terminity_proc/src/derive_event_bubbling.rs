use proc_macro2::TokenStream;
use proc_macro_error::Diagnostic;
use quote::quote;

use syn::DeriveInput;

pub fn run(input: DeriveInput) -> (TokenStream, Vec<Diagnostic>) {
	let name = input.ident;

	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let expanded = quote! {
		impl #impl_generics terminity::widgets::EventBubbling for #name #ty_generics #where_clause {

			type FinalData<'__local_lifetime> = &'__local_lifetime mut Self where Self: '__local_lifetime;
			fn bubble_event<'__local_lifetime, R, F: FnOnce(Self::FinalData<'__local_lifetime>, terminity::widgets::BubblingEvent) -> R>(
				&'__local_lifetime mut self,
				event: terminity::widgets::BubblingEvent,
				callback: F,
			) -> R {
				callback(self, event)
			}
		}
	};

	(expanded, vec![])
}
