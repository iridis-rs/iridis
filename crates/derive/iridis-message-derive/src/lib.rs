//! This module contains the macro `ArrowMessage`.
//! It's used to generate the necessary boilerplate code for creating an Arrow message.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    DeriveInput, Field, Fields, Ident, Token, Variant, parse_macro_input, punctuated::Punctuated,
    token::Comma,
};

/// Apply this macro to a struct or enum to implement the `ArrowMessage` trait.
///
/// All fields of the struct must implement the `ArrowMessage` trait. This is the only
/// constraint for the struct.
///
/// Rust primitives like `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64` and
/// Optionals of these types already implement the `ArrowMessage` trait, as well as all `arrow::array` types.
#[proc_macro_derive(ArrowMessage)]
pub fn from_into_arrow_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    match input.data {
        syn::Data::Struct(s) => match s.fields {
            Fields::Named(ref fields) => struct_derive(name, fields.named.clone()),
            _ => panic!("Only structs with named fields are supported"),
        },
        syn::Data::Enum(e) => enum_derive(name, e.variants.clone()),
        _ => panic!("Only structs and enums are supported"),
    }
}

fn struct_derive(name: Ident, fields: Punctuated<Field, Comma>) -> TokenStream {
    let field_attributes = fields
        .iter()
        .map(|field| (&field.ident, &field.ty))
        .collect::<Vec<_>>();

    let fields_fill = field_attributes.iter().map(|&(field, ty)| {
        quote! {
            <#ty>::field(stringify!(#field)),
        }
    });

    let union_data_fill = field_attributes.iter().map(|&(field, _)| {
        quote! {
            #field: extract_union_data(stringify!(#field), &map, &children)?,
        }
    });

    let arrow_data_fill = field_attributes.iter().map(|&(field, _)| {
        quote! {
            self.#field.try_into_arrow()?,
        }
    });

    let expanded = quote! {
        impl ArrowMessage for #name {
            fn field(name: impl Into<String>) -> iridis_message::prelude::thirdparty::arrow_schema::Field {
                make_union_fields(
                    name,
                    vec![
                        #(#fields_fill)*
                    ],
                )
            }

            fn try_from_arrow(data: iridis_message::prelude::thirdparty::arrow_data::ArrayData) -> iridis_message::prelude::thirdparty::eyre::Result<Self>
            where
                Self: Sized,
            {
                let (map, children) = unpack_union(data);

                Ok(Self {
                    #(#union_data_fill)*
                })
            }

            fn try_into_arrow(self) -> iridis_message::prelude::thirdparty::eyre::Result<iridis_message::prelude::thirdparty::arrow_array::ArrayRef> {
                let union_fields = get_union_fields::<Self>()?;

                make_union_array(
                    union_fields,
                    vec![
                        #(#arrow_data_fill)*
                    ],
                )
            }
        }

        impl TryFrom<iridis_message::prelude::thirdparty::arrow_data::ArrayData> for #name {
            type Error = iridis_message::prelude::thirdparty::eyre::Report;

            fn try_from(data: iridis_message::prelude::thirdparty::arrow_data::ArrayData) -> iridis_message::prelude::thirdparty::eyre::Result<Self> {
                #name::try_from_arrow(data)
            }
        }

        impl TryFrom<#name> for iridis_message::prelude::thirdparty::arrow_data::ArrayData {
            type Error = iridis_message::prelude::thirdparty::eyre::Report;

            fn try_from(item: #name) -> iridis_message::prelude::thirdparty::eyre::Result<Self> {
                use iridis_message::prelude::thirdparty::arrow_array::Array;

                item.try_into_arrow().map(|array| array.into_data())
            }
        }
    };

    TokenStream::from(expanded)
}

fn enum_derive(name: Ident, variants: Punctuated<Variant, Token![,]>) -> TokenStream {
    let variants: Vec<_> = variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let variant_str = variant_name.to_string().to_lowercase(); // Exemple : `Foo` -> "foo"
            (variant_name, variant_str)
        })
        .collect();

    let into_string_arms = variants.iter().map(|(variant_name, variant_str)| {
        quote! {
            #name::#variant_name => #variant_str.to_string(),
        }
    });

    let try_from_string_arms = variants.iter().map(|(variant_name, variant_str)| {
        quote! {
            #variant_str => Ok(#name::#variant_name),
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn into_string(self) -> String {
                match self {
                    #(#into_string_arms)*
                }
            }

            pub fn try_from_string(s: String) -> iridis_message::prelude::thirdparty::eyre::Result<Self> {
                match s.as_str() {
                    #(#try_from_string_arms)*
                    _ => Err(iridis_message::prelude::thirdparty::eyre::eyre!("Invalid value for {}: {}", stringify!(#name), s)),
                }
            }
        }

        impl ArrowMessage for #name {
            fn field(name: impl Into<String>) -> iridis_message::prelude::thirdparty::arrow_schema::Field {
                String::field(name)
            }

            fn try_from_arrow(data: iridis_message::prelude::thirdparty::arrow_data::ArrayData) -> iridis_message::prelude::thirdparty::eyre::Result<Self>
            where
                Self: Sized,
            {
                Encoding::try_from_string(String::try_from_arrow(data)?)
            }

            fn try_into_arrow(self) -> iridis_message::prelude::thirdparty::eyre::Result<iridis_message::prelude::thirdparty::arrow_array::ArrayRef> {
                String::try_into_arrow(self.into_string())
            }
        }


        impl TryFrom<iridis_message::prelude::thirdparty::arrow_data::ArrayData> for #name {
            type Error = iridis_message::prelude::thirdparty::eyre::Report;

            fn try_from(data: iridis_message::prelude::thirdparty::arrow_data::ArrayData) -> iridis_message::prelude::thirdparty::eyre::Result<Self> {
                #name::try_from_arrow(data)
            }
        }

        impl TryFrom<#name> for iridis_message::prelude::thirdparty::arrow_data::ArrayData {
            type Error = iridis_message::prelude::thirdparty::eyre::Report;

            fn try_from(item: #name) -> iridis_message::prelude::thirdparty::eyre::Result<Self> {
                item.try_into_arrow().map(|array| array.into_data())
            }
        }
    };

    TokenStream::from(expanded)
}
