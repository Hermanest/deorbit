use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};

pub fn resolve_crate() -> TokenStream {
    let found = crate_name("deorbit").expect("deorbit must be present");

    match found {
        // This is the fix! If we are inside 'deorbit', use 'crate'
        FoundCrate::Itself => quote!(crate),
        // If we are an external user, use the name (usually '::deorbit')
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name);
            quote!(::#ident)
        }
    }
}