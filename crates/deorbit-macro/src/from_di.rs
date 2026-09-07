use crate::utils::resolve_crate;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Error, Expr, Fields, ItemStruct, Meta, Result, Token, Type, parse_quote,
    spanned::Spanned,
};

#[derive(Default)]
struct FromDiParams {
    postfix: Option<Expr>,
}

#[derive(Default, Clone)]
enum FieldBindingKind {
    #[default]
    ResolveOne,
    ResolveMany,
    Default,
    Clone,
    Init(Expr),
}

impl FieldBindingKind {
    pub fn is_resolved(&self) -> bool {
        match self {
            FieldBindingKind::ResolveOne => true,
            FieldBindingKind::ResolveMany => true,
            FieldBindingKind::Clone => true,
            _ => false,
        }
    }
}

struct FieldBinding {
    ident: Ident,
    ty: Type,
    kind: FieldBindingKind,
}

pub fn transform_from_di(meta: TokenStream, mut input: ItemStruct) -> Result<TokenStream> {
    let crate_name = resolve_crate();
    let fields = transform_and_collect_fields(&crate_name, &mut input)?;
    let params = parse_attrs(meta)?;

    let initializer = expand_initializer(&crate_name, &fields)?;
    let deps = expand_dependencies(&crate_name, &fields)?;
    let ident = &input.ident;

    let postfix = params
        .postfix
        .map(|x| {
            // Using a function with explicit types to allow passing lambdas without type specification
            quote! {{
                fn invoke_inferred(this: &mut #ident, f: impl Fn(&mut #ident)) {
                    f(this);
                }
                invoke_inferred(&mut this, #x);
            }}
        })
        .unwrap_or_default();

    let from_di_ts = quote! {
        impl #crate_name::FromDi for #ident {
            fn depends_on() -> &'static [#crate_name::TypeMeta] {
               #deps
            }

            fn produce(services: &#crate_name::Services) -> Result<Self, #crate_name::Error> {
                use std::ops::Deref;

                let mut this = #initializer;

                #postfix

                Ok(this)
            }
        }
    };

    Ok(quote! {
        #input
        #from_di_ts
    })
}

fn parse_attrs(tt: TokenStream) -> Result<FromDiParams> {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let args = parser.parse2(tt)?;

    let mut params = FromDiParams::default();

    for meta in args {
        let Meta::NameValue(nv) = meta else {
            continue;
        };

        if nv.path.is_ident("postfix") {
            match &nv.value {
                Expr::Path(_) | Expr::Closure(_) => {
                    params.postfix = Some(nv.value);
                }

                _ => {
                    return Err(Error::new_spanned(
                        &nv.value,
                        "Expected a function path (e.g., Self::postfix) or a closure",
                    ));
                }
            }
        } else {
            return Err(Error::new_spanned(&nv.path, "Unknown attribute key"));
        }
    }

    Ok(params)
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

fn expand_initializer(crate_name: &TokenStream, fields: &Vec<FieldBinding>) -> Result<TokenStream> {
    let field_inits = fields
        .iter()
        .map(|x| expand_field_initializer(crate_name, x))
        .collect::<Result<Vec<_>>>()?;

    let ts = quote! {
        Self {
            #(#field_inits),*
        }
    };

    Ok(ts)
}

fn expand_field_initializer(crate_name: &TokenStream, field: &FieldBinding) -> Result<TokenStream> {
    let ident = &field.ident;

    let ts = match field.kind {
        FieldBindingKind::Default => {
            quote! {
                #ident: Default::default()
            }
        }

        FieldBindingKind::Clone => {
            let field_type = field.ty.to_token_stream();

            quote! {
                #ident: services.resolve::<#field_type>().ok_or(#crate_name::Error::missing::<#field_type>())?.deref().clone()
            }
        }

        FieldBindingKind::ResolveOne => {
            let field_type = field.ty.to_token_stream();

            quote! {
                #ident: services.resolve().ok_or(#crate_name::Error::missing::<#field_type>())?
            }
        }

        FieldBindingKind::ResolveMany => {
            let field_type = field.ty.to_token_stream();

            quote! {
                #ident: services.resolve_all().ok_or(#crate_name::Error::missing::<#field_type>())?.collect::<Vec<_>>()
            }
        }

        FieldBindingKind::Init(ref expr) => {
            quote! {
                #ident: #expr
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
            FieldBindingKind::Default | FieldBindingKind::Clone | FieldBindingKind::Init(..) => {
                ty.clone()
            }
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

            if meta.path.is_ident("clone") {
                kind = Some(FieldBindingKind::Clone);
            };

            if meta.path.is_ident("one") {
                kind = Some(FieldBindingKind::ResolveOne);
            };

            if meta.path.is_ident("many") {
                kind = Some(FieldBindingKind::ResolveMany);
            };

            if meta.path.is_ident("init") {
                let value = meta.value().map_err(|x| {
                    Error::new(
                        di.span(),
                        "init must be on the left side of an assignment expression",
                    )
                })?;

                let expr = value.parse()?;

                kind = Some(FieldBindingKind::Init(expr));
            }

            Ok(())
        })?;

        attrs.remove(idx);
    };

    Ok(kind.unwrap_or_default())
}
