use proc_macro::TokenStream;
use syn::ItemStruct;
use syn::parse_macro_input;

mod from_di;
mod utils;

#[proc_macro_attribute]
pub fn from_di(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);

    match from_di::transform_from_di(input.into()) {
        Ok(ts) => ts.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
