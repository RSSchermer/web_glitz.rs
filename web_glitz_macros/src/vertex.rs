use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Field, Ident, Lit, Meta, NestedMeta, Type};

use crate::util::ErrorLog;

pub fn expand_derive_vertex(input: &DeriveInput) -> Result<TokenStream, String> {
    if let Data::Struct(ref data) = input.data {
        let struct_name = &input.ident;
        let mod_path = quote!(web_glitz::pipeline::graphics);
        let mut log = ErrorLog::new();

        let mut position = 0;
        let mut vertex_attributes = Vec::new();

        for field in data.fields.iter() {
            if let VertexField::Attribute(attr) = VertexField::from_ast(field, position, &mut log) {
                vertex_attributes.push(attr);
            }

            position += 1;
        }

        let recurse = vertex_attributes.iter().map(|a| {
            let field_name = a
                .ident
                .clone()
                .map(|i| i.into_token_stream())
                .unwrap_or(a.position.into_token_stream());
            let location = a.location as u32;
            let ty = &a.ty;
            let span = a.span;
            let format_kind = {
                let ident = Ident::new(a.format.as_str(), Span::call_site()).into_token_stream();

                quote_spanned!(span=> {
                    assert_format_compatible::<#ty, #ident>();

                    <#ident as VertexAttributeFormatIdentifier>::FORMAT
                })
            };

            quote! {
                #mod_path::VertexAttributeDescriptor {
                    location: #location,
                    format: #format_kind,
                    offset_in_bytes: web_glitz::offset_of!(#struct_name, #field_name) as u8
                }
            }
        });

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {

            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::Vertex for #struct_name #ty_generics #where_clause {
                const ATTRIBUTE_DESCRIPTORS: &'static [#mod_path::VertexAttributeDescriptor] =
                    &[
                        #(#recurse),*
                    ];
            }
        };

        let suffix = struct_name.to_string().trim_start_matches("r#").to_owned();
        let dummy_const = Ident::new(&format!("_IMPL_VERTEX_FOR_{}", suffix), Span::call_site());

        let generated = quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const #dummy_const: () = {
                #[allow(unknown_lints)]
                #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
                #[allow(rust_2018_idioms)]
                use #mod_path::attribute_format::*;

                const fn assert_format_compatible<T, F>()
                where
                    T: #mod_path::attribute_format::VertexAttributeFormatCompatible<F>,
                    F: #mod_path::attribute_format::VertexAttributeFormatIdentifier
                {}

                #impl_block
            };
        };

        log.compile().map(|_| generated)
    } else {
        Err("`Vertex` can only be derived for a struct.".into())
    }
}

enum VertexField {
    Attribute(AttributeField),
    Excluded,
}

impl VertexField {
    pub fn from_ast(ast: &Field, position: usize, log: &mut ErrorLog) -> Self {
        let vertex_attributes: Vec<&Attribute> = ast
            .attrs
            .iter()
            .filter(|a| is_vertex_attribute(a))
            .collect();
        let field_name = ast
            .ident
            .clone()
            .map(|i| i.to_string())
            .unwrap_or(position.to_string());

        match vertex_attributes.len() {
            0 => VertexField::Excluded,
            1 => {
                let attr = vertex_attributes[0];

                let meta_items: Vec<NestedMeta> = match attr.parse_meta() {
                    Ok(Meta::List(meta)) => meta.nested.iter().cloned().collect(),
                    Ok(Meta::Path(path)) if path.is_ident("vertex_attribute") => Vec::new(),
                    _ => {
                        log.log_error(format!(
                            "Malformed #[vertex_attribute] attribute for field `{}`.",
                            field_name
                        ));

                        Vec::new()
                    }
                };

                let mut location = None;
                let mut format = None;

                for meta_item in meta_items.into_iter() {
                    match meta_item {
                        NestedMeta::Meta(Meta::NameValue(m)) if m.path.is_ident("location") => {
                            if let Lit::Int(i) = &m.lit {
                                if let Ok(value) = i.base10_parse::<u32>() {
                                    location = Some(value);
                                } else {
                                    log.log_error(format!(
                                        "Malformed #[vertex_attribute] attribute for field `{}`: \
                                        expected `location` to be representable as a u32.",
                                        field_name
                                    ));
                                }
                            } else {
                                log.log_error(format!(
                                    "Malformed #[vertex_attribute] attribute for field `{}`: \
                                     expected `location` to be a positive integer.",
                                    field_name
                                ));
                            };
                        }
                        NestedMeta::Meta(Meta::NameValue(m)) if m.path.is_ident("format") => {
                            if let Lit::Str(f) = &m.lit {
                                format = Some(f.value());
                            } else {
                                log.log_error(format!(
                                    "Malformed #[vertex_attribute] attribute for field `{}`: \
                                     expected `format` to be a string.",
                                    field_name
                                ));
                            };
                        }
                        _ => log.log_error(format!(
                            "Malformed #[vertex_attribute] attribute for field `{}`: unrecognized \
                             option `{}`.",
                            field_name,
                            meta_item.into_token_stream()
                        )),
                    }
                }

                if location.is_none() {
                    log.log_error(format!(
                        "Field `{}` is marked a vertex attribute, but does not declare a binding \
                         location.",
                        field_name
                    ));
                }

                if format.is_none() {
                    log.log_error(format!(
                        "Field `{}` is marked a vertex attribute, but does not declare a format.",
                        field_name
                    ));
                }

                if location.is_some() && format.is_some() {
                    let location = location.unwrap();
                    let format = format.unwrap();

                    VertexField::Attribute(AttributeField {
                        ident: ast.ident.clone(),
                        ty: ast.ty.clone(),
                        position,
                        location,
                        format,
                        span: ast.span(),
                    })
                } else {
                    VertexField::Excluded
                }
            }
            _ => {
                log.log_error(format!(
                    "#[vertex_attribute] must not be defined more than once for field `{}`.",
                    field_name
                ));

                VertexField::Excluded
            }
        }
    }
}

struct AttributeField {
    ident: Option<Ident>,
    ty: Type,
    position: usize,
    location: u32,
    format: String,
    span: Span,
}

fn is_vertex_attribute(attribute: &Attribute) -> bool {
    attribute.path.segments[0].ident == "vertex_attribute"
}
