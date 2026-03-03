use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{Attribute, DeriveInput, Error, Field, Fields, Result, spanned::Spanned};

#[derive(Default)]
enum FieldBindingKind {
    FromDefault,
    #[default]
    Resolved,
}

pub fn expand_from_di(derive: DeriveInput) -> Result<TokenStream> {
    let ident = &derive.ident;

    let fields = match derive.data {
        syn::Data::Struct(ref data) => &data.fields,
        _ => {
            return Err(Error::new(
                ident.span(),
                "FromDi can only be applied to structs",
            ));
        }
    };

    let initializer = expand_initializer(ident, fields)?;

    let ts = quote! {
        impl FromDi for #ident {
            fn inject(this: &mut MaybeUninit<Self>, services: &Services) -> Result<(), String> {
                this.write(#initializer);
                Ok(())
            }
        }
    }
    .into();

    Ok(ts)
}

fn expand_initializer(ident: &Ident, fields: &Fields) -> Result<TokenStream> {
    let field_inits = fields
        .iter()
        .map(|field| expand_field_initializer(ident, field))
        .collect::<Result<Vec<_>>>()?;

    let ts = quote! {
        Self {
            #(#field_inits),*
        }
    };

    Ok(ts)
}

fn expand_field_initializer(ty: &Ident, field: &Field) -> Result<TokenStream> {
    // Ident can be None only on tuple structs
    let ident = field
        .ident
        .as_ref()
        .ok_or_else(|| Error::new(ty.span(), "Tuple structs are not supported"))?;

    let ts = match parse_field_attr(&field.attrs)? {
        FieldBindingKind::FromDefault => {
            quote! {
                #ident: Default::default()
            }
        }

        FieldBindingKind::Resolved => {
            let field_type = field.ty.to_token_stream();
            let err_msg = format!("Failed to resolve {}", field_type);

            quote! {
                #ident: services.resolve().ok_or(#err_msg)?
            }
        }
    };

    Ok(ts)
}

fn parse_field_attr(attrs: &[Attribute]) -> Result<FieldBindingKind> {
    let mut kind = None;

    // Attribute is not presented, return default
    if let Some(di) = attrs.iter().find(|x| x.path().is_ident("di")) {
        di.parse_nested_meta(|meta| {
            // If kind was previously specified
            if kind.is_some() {
                return Err(Error::new(
                    di.span(),
                    "#[di()] can only define a single resolution kind",
                ));
            }

            if meta.path.is_ident("default") {
                kind = Some(FieldBindingKind::FromDefault);
            };

            Ok(())
        })?;
    };

    Ok(kind.unwrap_or_default())
}
