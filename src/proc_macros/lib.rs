#![feature(proc_macro_diagnostic)]

use proc_macro::{TokenStream, Span};

#[proc_macro]
pub fn compile_warning(input: TokenStream) -> TokenStream {
    Span::call_site().warning(input.to_string()).emit();
    TokenStream::new()
}