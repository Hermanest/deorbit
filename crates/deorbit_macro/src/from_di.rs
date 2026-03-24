use crate::utils::resolve_crate;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, Attribute, DeriveInput, Error, Field, Fields, Result, Type};

#[derive(Default, Copy, Clone, Eq, PartialEq)]
enum FieldBindingKind {
    FromDefault,
    #[default]
    Resolved,
}

struct FieldBinding<'a> {
    field: &'a Field,
    actual_type: Type,
    kind: FieldBindingKind,
}

pub fn expand_from_di(derive: DeriveInput) -> Result<TokenStream> {
    let crate_name = resolve_crate();
    let fields = extract_fields(&derive)?;

    let initializer = expand_initializer(&fields)?;
    let deps = expand_dependencies(&crate_name, &fields)?;
    let ident = &derive.ident;

    let ts = quote! {
        impl FromDi for #ident {
            fn depends_on() -> &'static [#crate_name::TypeMeta] {
               #deps
            }

            fn produce(services: &#crate_name::Services) -> Self {
                #initializer
            }
        }
    }
    .into();

    Ok(ts)
}

fn expand_dependencies(crate_name: &TokenStream, fields: &Vec<FieldBinding>) -> Result<TokenStream> {
    let deps = fields
        .iter()
        .filter(|x| x.kind == FieldBindingKind::Resolved)
        .map(|x| {
            let ty = &x.actual_type;
            quote! { #crate_name::TypeMeta::of::<#ty>() }
        });

    let ts = quote! {
        const DEPS: &'static [#crate_name::TypeMeta] = &[
            #(#deps),*
        ];

        DEPS
    };

    Ok(ts)
}

fn expand_initializer(fields: &Vec<FieldBinding>) -> Result<TokenStream> {
    let field_inits = fields
        .iter()
        .map(expand_field_initializer)
        .collect::<Result<Vec<_>>>()?;

    let ts = quote! {
        Self {
            #(#field_inits),*
        }
    };

    Ok(ts)
}

fn expand_field_initializer(field: &FieldBinding) -> Result<TokenStream> {
    // Ident can be None only on tuple and unit structs
    let ident = field.field.ident.as_ref().unwrap();

    let ts = match field.kind {
        FieldBindingKind::FromDefault => {
            quote! {
                #ident: Default::default()
            }
        }

        FieldBindingKind::Resolved => {
            let field_type = field.actual_type.to_token_stream();
            let err_msg = format!("Failed to resolve {}", field_type);

            quote! {
                #ident: services.resolve().expect(#err_msg)
            }
        }
    };

    Ok(ts)
}

fn extract_fields(derive: &'_ DeriveInput) -> Result<Vec<FieldBinding<'_>>> {
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

    let fields = match fields {
        Fields::Named(named) => Ok(named),
        Fields::Unnamed(_) => Err(Error::new(ident.span(), "Tuple structs are not supported")),
        Fields::Unit => Err(Error::new(ident.span(), "Unit structs are not supported")),
    }?;

    fields
        .named
        .iter()
        .map(|field| {
            let kind = parse_field_attr(&field.attrs)?;
            let ty = extract_service_type(&field.ty);

            Ok(FieldBinding {
                field,
                actual_type: ty.clone(),
                kind,
            })
        })
        .collect()
}

fn extract_service_type(ty: &Type) -> &Type {
    if let Type::Path(type_path) = ty {
        let segment = type_path.path.segments.last().unwrap();

        if segment.ident == "Service" {
            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                    return inner_ty;
                }
            }
        }
    }

    ty
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
