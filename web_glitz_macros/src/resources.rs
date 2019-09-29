use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Field, Ident, Lit, Meta, NestedMeta, Type};

use crate::util::ErrorLog;
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

pub fn expand_derive_resources(input: &DeriveInput) -> Result<TokenStream, String> {
    if let Data::Struct(ref data) = input.data {
        let struct_name = &input.ident;
        let mod_path = quote!(_web_glitz::pipeline::resources);
        let mut log = ErrorLog::new();

        let mut buffer_resources: Vec<BufferResourceField> = Vec::new();
        let mut texture_resources: Vec<TextureResourceField> = Vec::new();

        for (position, field) in data.fields.iter().enumerate() {
            match ResourcesField::from_ast(field, position, &mut log) {
                ResourcesField::BufferResource(buffer_resource_field) => {
                    for field in buffer_resources.iter() {
                        if field.binding == buffer_resource_field.binding {
                            log.log_error(format!(
                                "Fields `{}` and `{}` cannot both use buffer binding `{}`.",
                                field.name, buffer_resource_field.name, field.binding
                            ));
                        }
                    }

                    buffer_resources.push(buffer_resource_field);
                }
                ResourcesField::TextureResource(texture_resource_field) => {
                    for field in texture_resources.iter() {
                        if field.binding == texture_resource_field.binding {
                            log.log_error(format!(
                                "Fields `{}` and `{}` cannot both use texture binding `{}`.",
                                field.name, texture_resource_field.name, field.binding
                            ));
                        }
                    }

                    texture_resources.push(texture_resource_field);
                }
                ResourcesField::Excluded => (),
            };
        }

        let buffer_slot_descriptors = buffer_resources.iter().map(|field| {
            let ty = &field.ty;
            let slot_identifier = &field.name;
            let slot_index = field.binding as u32;
            let span = field.span;

            quote_spanned! {span=>
                #mod_path::TypedResourceSlotDescriptor {
                    slot_identifier: #mod_path::ResourceSlotIdentifier::Static(#slot_identifier),
                    slot_index: #slot_index,
                    slot_type: #mod_path::ResourceSlotType::UniformBuffer(<#ty as #mod_path::BufferResource>::MEMORY_UNITS)
                }
            }
        });

        let texture_slot_descriptors = texture_resources.iter().map(|field| {
            let ty = &field.ty;
            let slot_identifier = &field.name;
            let slot_index = field.binding as u32;
            let span = field.span;

            quote_spanned! {span=>
                #mod_path::TypedResourceSlotDescriptor {
                    slot_identifier: #mod_path::ResourceSlotIdentifier::Static(#slot_identifier),
                    slot_index: #slot_index,
                    slot_type: #mod_path::ResourceSlotType::SampledTexture(<#ty as #mod_path::TextureResource>::SAMPLED_TEXTURE_TYPE)
                }
            }
        });

        let buffer_resource_encodings = buffer_resources.iter().map(|field| {
            let field_name = field
                .ident
                .clone()
                .map(|i| i.into_token_stream())
                .unwrap_or(field.position.into_token_stream());

            let binding = field.binding as u32;

            quote! {
                let encoder = self.#field_name.encode(#binding, encoder);
            }
        });

        let texture_resource_encodings = texture_resources.iter().map(|field| {
            let field_name = field
                .ident
                .clone()
                .map(|i| i.into_token_stream())
                .unwrap_or(field.position.into_token_stream());

            let binding = field.binding as u32;

            quote! {
                let encoder = self.#field_name.encode(#binding, encoder);
            }
        });

        let total_bindings = buffer_resources.len() + texture_resources.len();

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let impl_block = quote! {
            #[automatically_derived]
            unsafe impl #impl_generics #mod_path::Resources for #struct_name #ty_generics #where_clause {
                type Bindings = [#mod_path::BindingDescriptor;#total_bindings];

                const LAYOUT: &'static [#mod_path::TypedResourceSlotDescriptor] = &[
                    #(#buffer_slot_descriptors,)*
                    #(#texture_slot_descriptors,)*
                ];

                fn encode_bindings(
                    self,
                    context: &mut #mod_path::ResourceBindingsEncodingContext,
                ) -> #mod_path::ResourceBindingsEncoding<Self::Bindings> {
                    let encoder = #mod_path::StaticResourceBindingsEncoder::new(context);

                    #(#buffer_resource_encodings)*
                    #(#texture_resource_encodings)*

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

                use #mod_path::{BufferResource, TextureResource};

                #impl_block
            };
        };

        log.compile().map(|_| generated)
    } else {
        Err("`Resources` can only be derived for a struct.".into())
    }
}

enum ResourcesField {
    BufferResource(BufferResourceField),
    TextureResource(TextureResourceField),
    Excluded,
}

impl ResourcesField {
    pub fn from_ast(ast: &Field, position: usize, log: &mut ErrorLog) -> Self {
        let field_name = ast
            .ident
            .clone()
            .map(|i| i.to_string())
            .unwrap_or(position.to_string());

        let buffer_resource_attributes: Vec<&Attribute> = ast
            .attrs
            .iter()
            .filter(|a| is_buffer_resource_attribute(a))
            .collect();

        let texture_resource_attributes: Vec<&Attribute> = ast
            .attrs
            .iter()
            .filter(|a| is_texture_resource_attribute(a))
            .collect();

        if buffer_resource_attributes.len() == 1 && texture_resource_attributes.len() == 0 {
            let attr = buffer_resource_attributes[0];

            let meta_items: Vec<NestedMeta> = match attr.interpret_meta() {
                Some(Meta::List(ref meta)) => meta.nested.iter().cloned().collect(),
                Some(Meta::Word(ref i)) if i == "buffer_resource" => Vec::new(),
                _ => {
                    log.log_error(format!(
                        "Malformed #[buffer_resource] attribute for field `{}`.",
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
                                "Malformed #[buffer_resource] attribute for field `{}`: \
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
                                "Malformed #[buffer_resource] attribute for field `{}`: \
                                 expected `name` to be a string.",
                                field_name
                            ));
                        };
                    }
                    _ => log.log_error(format!(
                        "Malformed #[buffer_resource] attribute for field `{}`: unrecognized \
                         option `{}`.",
                        field_name,
                        meta_item.into_token_stream()
                    )),
                }
            }

            if binding.is_none() {
                log.log_error(format!(
                    "Field `{}` is marked a buffer resource, but does not declare a `binding` index.",
                    field_name
                ));
            }

            if name.is_none() {
                log.log_error(format!(
                    "Field `{}` is marked a buffer attribute, but does not declare a resource name.",
                    field_name
                ));
            }

            if binding.is_some() && name.is_some() {
                let binding = binding.unwrap();
                let name = name.unwrap();

                ResourcesField::BufferResource(BufferResourceField {
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
        } else if texture_resource_attributes.len() == 1 && buffer_resource_attributes.len() == 0 {
            let attr = texture_resource_attributes[0];

            let meta_items: Vec<NestedMeta> = match attr.interpret_meta() {
                Some(Meta::List(ref meta)) => meta.nested.iter().cloned().collect(),
                Some(Meta::Word(ref i)) if i == "texture_resource" => Vec::new(),
                _ => {
                    log.log_error(format!(
                        "Malformed #[texture_resource] attribute for field `{}`.",
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
                                "Malformed #[texture_resource] attribute for field `{}`: \
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
                                "Malformed #[texture_resource] attribute for field `{}`: \
                                 expected `name` to be a string.",
                                field_name
                            ));
                        };
                    }
                    _ => log.log_error(format!(
                        "Malformed #[texture_resource] attribute for field `{}`: unrecognized \
                         option `{}`.",
                        field_name,
                        meta_item.into_token_stream()
                    )),
                }
            }

            if binding.is_none() {
                log.log_error(format!(
                    "Field `{}` is marked a texture resource, but does not declare a `binding` index.",
                    field_name
                ));
            }

            if name.is_none() {
                log.log_error(format!(
                    "Field `{}` is marked a texture attribute, but does not declare a resource name.",
                    field_name
                ));
            }

            if binding.is_some() && name.is_some() {
                let binding = binding.unwrap();
                let name = name.unwrap();

                ResourcesField::TextureResource(TextureResourceField {
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
            log.log_error(format!(
                "Cannot declare multiple #[buffer_resource] and/or #[texture_resource] attributes on field `{}`.",
                field_name
            ));

            ResourcesField::Excluded
        }
    }
}

struct BufferResourceField {
    ident: Option<Ident>,
    ty: Type,
    position: usize,
    binding: u64,
    name: String,
    span: Span,
}

struct TextureResourceField {
    ident: Option<Ident>,
    ty: Type,
    position: usize,
    binding: u64,
    name: String,
    span: Span,
}

fn is_buffer_resource_attribute(attribute: &Attribute) -> bool {
    attribute.path.segments[0].ident == "buffer_resource"
}

fn is_texture_resource_attribute(attribute: &Attribute) -> bool {
    attribute.path.segments[0].ident == "texture_resource"
}
