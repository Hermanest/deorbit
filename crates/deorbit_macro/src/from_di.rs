use crate::utils::resolve_crate;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    Attribute, Error, Field, Fields, ItemStruct, Result, Type, parse_quote, spanned::Spanned,
};

#[derive(Default, Copy, Clone, Eq, PartialEq)]
enum FieldBindingKind {
    #[default]
    ResolveOne,
    ResolveMany,
    Default,
}

impl FieldBindingKind {
    pub fn is_resolved(&self) -> bool {
        match self {
            FieldBindingKind::ResolveOne => true,
            FieldBindingKind::ResolveMany => true,
            _ => false,
        }
    }
}

struct FieldBinding {
    ident: Ident,
    ty: Type,
    kind: FieldBindingKind,
}

pub fn transform_from_di(mut input: ItemStruct) -> Result<TokenStream> {
    let crate_name = resolve_crate();
    let fields = transform_and_collect_fields(&crate_name, &mut input)?;

    let initializer = expand_initializer(&fields)?;
    let deps = expand_dependencies(&crate_name, &fields)?;
    let ident = &input.ident;

    let from_di_ts = quote! {
        impl FromDi for #ident {
            fn depends_on() -> &'static [#crate_name::TypeMeta] {
               #deps
            }

            fn produce(services: &#crate_name::Services) -> Self {
                #initializer
            }
        }
    };

    Ok(quote! {
        #input
        #from_di_ts
    })
}

fn expand_dependencies(
    crate_name: &TokenStream,
    fields: &Vec<FieldBinding>,
) -> Result<TokenStream> {
    let deps = fields.iter().filter(|x| x.kind.is_resolved()).map(|x| {
        let ty = &x.ty;
        quote! { #crate_name::TypeMeta::of::<#ty>() }
    });

    let ts = quote! {
        const { &[
            #(#deps),*
        ] }
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
    let ident = &field.ident;

    let ts = match field.kind {
        FieldBindingKind::Default => {
            quote! {
                #ident: Default::default()
            }
        }

        FieldBindingKind::ResolveOne => {
            let field_type = field.ty.to_token_stream();
            let err_msg = format!("Failed to resolve {}", field_type);

            quote! {
                #ident: services.resolve().expect(#err_msg)
            }
        }

        FieldBindingKind::ResolveMany => {
            let field_type = field.ty.to_token_stream();
            let err_msg = format!("Failed to resolve {}", field_type);

            quote! {
                #ident: services.resolve_all().expect(#err_msg).collect::<Vec<_>>()
            }
        }
    };

    Ok(ts)
}

fn transform_and_collect_fields(
    crate_name: &TokenStream,
    input: &mut ItemStruct,
) -> Result<Vec<FieldBinding>> {
    let ident = &input.ident;

    let fields = match &mut input.fields {
        Fields::Named(named) => Ok(named),
        Fields::Unnamed(_) => Err(Error::new(ident.span(), "Tuple structs are not supported")),
        Fields::Unit => Err(Error::new(ident.span(), "Unit structs are not supported")),
    }?;

    let mut bindings = vec![];

    for field in fields.named.iter_mut() {
        let kind = extract_field_attr(&mut field.attrs)?;
        let ty = field.ty.clone();

        // Wrapping type in Resolved or ResolvedMany
        field.ty = match &kind {
            FieldBindingKind::ResolveOne => {
                parse_quote!(#crate_name::Resolved<#ty>)
            }
            FieldBindingKind::ResolveMany => {
                parse_quote!(#crate_name::ResolvedMany<#ty>)
            }
            FieldBindingKind::Default => ty.clone(),
        };

        bindings.push(FieldBinding {
            // Ident can be None only on tuple and unit structs
            ident: field.ident.clone().unwrap(),
            ty,
            kind,
        });
    }

    Ok(bindings)
}

fn extract_field_attr(attrs: &mut Vec<Attribute>) -> Result<FieldBindingKind> {
    let mut kind = None;
    let idx = attrs.iter().position(|x| x.path().is_ident("di"));

    // Attribute is not presented, return default
    if let Some(idx) = idx {
        let di = &attrs[idx];
        di.parse_nested_meta(|meta| {
            // If kind was previously specified
            if kind.is_some() {
                return Err(Error::new(
                    di.span(),
                    "#[di()] can only define a single resolution kind",
                ));
            }

            if meta.path.is_ident("default") {
                kind = Some(FieldBindingKind::Default);
            };

            if meta.path.is_ident("one") {
                kind = Some(FieldBindingKind::ResolveOne);
            };

            if meta.path.is_ident("many") {
                kind = Some(FieldBindingKind::ResolveMany);
            };

            Ok(())
        })?;

        attrs.remove(idx);
    };

    Ok(kind.unwrap_or_default())
}
