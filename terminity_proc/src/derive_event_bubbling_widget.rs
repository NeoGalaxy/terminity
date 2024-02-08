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
	todo!()
}
