use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Field, Ident, Lit, Meta, NestedMeta, Type};

use crate::util::ErrorLog;

pub fn expand_derive_resources(input: &DeriveInput) -> Result<TokenStream, String> {
    if let Data::Struct(ref data) = input.data {
        let struct_name = &input.ident;
        let mod_path = quote!(_web_glitz::pipeline::resources);
        let mut log = ErrorLog::new();

        let mut resource_fields: Vec<ResourceField> = Vec::new();

        for (position, field) in data.fields.iter().enumerate() {
            match ResourcesField::from_ast(field, position, &mut log) {
                ResourcesField::Resource(resource_field) => {
                    for field in resource_fields.iter() {
                        if field.binding == resource_field.binding {
                            log.log_error(format!(
                                "Fields `{}` and `{}` cannot both use binding `{}`.",
                                field.name, resource_field.name, field.binding
                            ));
                        }
                    }

                    resource_fields.push(resource_field);
                }
                ResourcesField::Excluded => (),
            };
        }

        let resource_slot_descriptors = resource_fields.iter().map(|field| {
            let ty = &field.ty;
            let slot_identifier = &field.name;
            let slot_index = field.binding as u32;
            let span = field.span;

            quote_spanned! {span=>
                #mod_path::TypedResourceSlotDescriptor {
                    slot_identifier: #mod_path::ResourceSlotIdentifier::Static(#slot_identifier),
                    slot_index: #slot_index,
                    slot_type: <#ty as #mod_path::Resource>::TYPE
                }
            }
        });

        let resource_encodings = resource_fields.iter().map(|field| {
            let field_name = field
                .ident
                .clone()
                .map(|i| i.into_token_stream())
                .unwrap_or(field.position.into_token_stream());

            let binding = field.binding as u32;

            quote! {
                self.#field_name.encode(#binding, &mut encoder);
            }
        });

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
        let len = resource_fields.len();

        let impl_block = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::Resources for #struct_name #ty_generics #where_clause {
                const LAYOUT: &'static [#mod_path::TypedResourceSlotDescriptor] = &[
                    #(#resource_slot_descriptors,)*
                ];

                fn encode_bind_group(
                    self,
                    context: &mut #mod_path::BindGroupEncodingContext,
                ) -> #mod_path::BindGroupEncoding {
                    let mut encoder = #mod_path::BindGroupEncoder::new(context, Some(#len));

                    #(#resource_encodings)*

                    encoder.finish()
                }
            }
        };

        let suffix = struct_name.to_string().trim_start_matches("r#").to_owned();
        let dummy_const = Ident::new(
            &format!("_IMPL_RESOURCES_FOR_{}", suffix),
            Span::call_site(),
        );

        let generated = quote! {
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const #dummy_const: () = {
                #[allow(unknown_lints)]
                #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
                #[allow(rust_2018_idioms)]
                extern crate web_glitz as _web_glitz;

                use #mod_path::Resource;

                #impl_block
            };
        };

        log.compile().map(|_| generated)
    } else {
        Err("`Resources` can only be derived for a struct.".into())
    }
}

enum ResourcesField {
    Resource(ResourceField),
    Excluded,
}

impl ResourcesField {
    pub fn from_ast(ast: &Field, position: usize, log: &mut ErrorLog) -> Self {
        let field_name = ast
            .ident
            .clone()
            .map(|i| i.to_string())
            .unwrap_or(position.to_string());

        let mut iter = ast.attrs.iter().filter(|a| is_resource_attribute(a));

        if let Some(attr) = iter.next() {
            if iter.next().is_some() {
                log.log_error(format!(
                    "Cannot add more than 1 #[resource] attribute to field `{}`.",
                    field_name
                ));

                return ResourcesField::Excluded;
            }

            let meta_items: Vec<NestedMeta> = match attr.interpret_meta() {
                Some(Meta::List(ref meta)) => meta.nested.iter().cloned().collect(),
                Some(Meta::Word(ref i)) if i == "resource" => Vec::new(),
                _ => {
                    log.log_error(format!(
                        "Malformed #[resource] attribute for field `{}`.",
                        field_name
                    ));

                    Vec::new()
                }
            };

            let mut binding = None;
            let mut name = ast.ident.clone().map(|i| i.to_string());

            for meta_item in meta_items.into_iter() {
                match meta_item {
                    NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "binding" => {
                        if let Lit::Int(ref i) = m.lit {
                            binding = Some(i.value());
                        } else {
                            log.log_error(format!(
                                "Malformed #[resource] attribute for field `{}`: \
                                 expected `binding` to be a positive integer.",
                                field_name
                            ));
                        };
                    }
                    NestedMeta::Meta(Meta::NameValue(ref m)) if m.ident == "name" => {
                        if let Lit::Str(ref f) = m.lit {
                            name = Some(f.value());
                        } else {
                            log.log_error(format!(
                                "Malformed #[resource] attribute for field `{}`: \
                                 expected `name` to be a string.",
                                field_name
                            ));
                        };
                    }
                    _ => log.log_error(format!(
                        "Malformed #[resource] attribute for field `{}`: unrecognized \
                         option `{}`.",
                        field_name,
                        meta_item.into_token_stream()
                    )),
                }
            }

            if binding.is_none() {
                log.log_error(format!(
                    "Field `{}` is marked with #[resource], but does not declare a `binding` \
                     index.",
                    field_name
                ));
            }

            if name.is_none() {
                log.log_error(format!(
                    "Field `{}` is marked with #[resource], but does not declare a slot name.",
                    field_name
                ));
            }

            if binding.is_some() && name.is_some() {
                let binding = binding.unwrap();
                let name = name.unwrap();

                ResourcesField::Resource(ResourceField {
                    ident: ast.ident.clone(),
                    ty: ast.ty.clone(),
                    position,
                    binding,
                    name,
                    span: ast.span(),
                })
            } else {
                ResourcesField::Excluded
            }
        } else {
            ResourcesField::Excluded
        }
    }
}

struct ResourceField {
    ident: Option<Ident>,
    ty: Type,
    position: usize,
    binding: u64,
    name: String,
    span: Span,
}

fn is_resource_attribute(attribute: &Attribute) -> bool {
    attribute.path.segments[0].ident == "resource"
}
