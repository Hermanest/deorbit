use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

mod from_di;

#[proc_macro_derive(FromDi, attributes(di))]
pub fn derive_from_di(input: TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match from_di::expand_from_di(input.into()) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
