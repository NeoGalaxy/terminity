use std::{cell::RefCell, collections::HashMap};

use convert_case::{Case, Casing};
use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{Diagnostic, Level};
use quote::quote;

use syn::{
	braced, bracketed, parenthesized,
	parse::{Parse, ParseStream},
	parse2,
	punctuated::Punctuated,
	spanned::Spanned,
	token, DeriveInput, Ident, LitChar, LitStr, Member, Token,
};

use crate::frame::parse_frame_lines;
use unicode_segmentation::UnicodeSegmentation;

pub fn run(input: DeriveInput) -> (TokenStream, Vec<Diagnostic>) {
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

	(expanded, vec![])
}
