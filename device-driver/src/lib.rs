use std::iter::FromIterator;

use convert_case::Casing;
use deserialization::{FieldCollection, RegisterCollection};
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

mod deserialization;
mod generation;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub address_type: IntegerType,
    pub registers: RegisterCollection,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Register {
    #[serde(skip)]
    pub name: String,
    pub rw_capability: RWCapability,
    pub address: u64,
    pub size_bytes: u64,
    pub fields: FieldCollection,
}

impl PartialOrd for Register {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Register {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.address
            .cmp(&other.address)
            .then_with(|| self.name.cmp(&other.name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    #[serde(skip)]
    pub name: String,
    #[serde(rename = "type")]
    pub register_type: IntegerType,
    #[serde(rename = "conversion")]
    pub conversion_type: Option<TypePathOrEnum>,
    pub start: u32,
    pub end: u32,
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Field {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start
            .cmp(&other.start)
            .then_with(|| self.name.cmp(&other.name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypePathOrEnum {
    TypePath(TypePath),
    Enum(IndexMap<String, Option<i128>>),
}

impl TypePathOrEnum {
    pub fn into_type(&self, field_name: &str) -> syn::Type {
        match self {
            TypePathOrEnum::TypePath(type_path) => type_path.into_type(),
            TypePathOrEnum::Enum(_) => {
                let name = syn::Ident::new(
                    &field_name.to_case(convert_case::Case::Pascal),
                    proc_macro2::Span::call_site(),
                );

                let mut segments = syn::punctuated::Punctuated::new();
                segments.push(name.into());

                syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: syn::Path {
                        leading_colon: None,
                        segments,
                    },
                })
            }
        }
    }

    pub fn generate_type_definition(
        &self,
        register_type: syn::Type,
        field_name: &str,
    ) -> Option<TokenStream> {
        match self {
            TypePathOrEnum::TypePath(_) => None,
            TypePathOrEnum::Enum(map) => {
                let name = syn::Ident::new(
                    &field_name.to_case(convert_case::Case::Pascal),
                    proc_macro2::Span::call_site(),
                );
                let mut variants = TokenStream::new();
                variants.append_all(map.iter().map(|(name, value)| {
                    let variant = syn::Ident::new(
                        &name.to_case(convert_case::Case::Pascal),
                        proc_macro2::Span::call_site(),
                    );
                    let value_specifier = value.map(|value| {
                        let value = proc_macro2::Literal::i128_unsuffixed(value);
                        quote!(= #value)
                    });
                    quote! {
                        #variant #value_specifier,
                    }
                }));
                Some(
                    quote::quote! {
                        #[derive(device_driver_core::num_enum::TryFromPrimitive, device_driver_core::num_enum::IntoPrimitive, Debug, Copy, Clone, PartialEq, Eq)]
                        #[repr(#register_type)]
                        pub enum #name {
                            #variants
                        }
                    }.into_token_stream()
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(transparent)]
pub struct TypePath(String);

impl TypePath {
    pub fn into_type(&self) -> syn::Type {
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: syn::punctuated::Punctuated::from_iter(self.0.split("::").map(|seg| {
                    syn::PathSegment::from(syn::Ident::new(seg, proc_macro2::Span::call_site()))
                })),
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum RWCapability {
    #[serde(alias = "ro", alias = "RO")]
    ReadOnly,
    #[serde(alias = "wo", alias = "WO")]
    WriteOnly,
    #[serde(alias = "rw", alias = "RW")]
    ReadWrite,
}

impl RWCapability {
    pub fn into_type(&self) -> syn::Type {
        match self {
            RWCapability::ReadOnly => syn::parse_quote!(device_driver_core::ReadOnly),
            RWCapability::WriteOnly => syn::parse_quote!(device_driver_core::WriteOnly),
            RWCapability::ReadWrite => syn::parse_quote!(device_driver_core::ReadWrite),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IntegerType {
    #[serde(alias = "unsigned char", alias = "byte")]
    U8,
    #[serde(alias = "unsigned short")]
    U16,
    #[serde(alias = "unsigned int")]
    U32,
    #[serde(alias = "unsigned long")]
    U64,
    #[serde(alias = "unsigned long long")]
    U128,
    #[serde(alias = "unsigned size")]
    Usize,
    #[serde(alias = "char")]
    I8,
    #[serde(alias = "short")]
    I16,
    #[serde(alias = "int")]
    I32,
    #[serde(alias = "long")]
    I64,
    #[serde(alias = "long long")]
    I128,
    #[serde(alias = "size")]
    Isize,
}

impl IntegerType {
    pub fn into_type(&self) -> syn::Type {
        match self {
            IntegerType::U8 => syn::parse_quote!(u8),
            IntegerType::U16 => syn::parse_quote!(u16),
            IntegerType::U32 => syn::parse_quote!(u32),
            IntegerType::U64 => syn::parse_quote!(u64),
            IntegerType::U128 => syn::parse_quote!(u128),
            IntegerType::Usize => syn::parse_quote!(usize),
            IntegerType::I8 => syn::parse_quote!(i8),
            IntegerType::I16 => syn::parse_quote!(i16),
            IntegerType::I32 => syn::parse_quote!(i32),
            IntegerType::I64 => syn::parse_quote!(i64),
            IntegerType::I128 => syn::parse_quote!(i128),
            IntegerType::Isize => syn::parse_quote!(isize),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_file_formats() {
        let json_string = include_str!("../json_syntax.json");
        let from_json_value = serde_json::from_str::<Device>(json_string).unwrap();
        let yaml_string = include_str!("../yaml_syntax.yaml");
        let from_yaml_value = serde_yaml::from_str::<Device>(yaml_string).unwrap();

        println!("From json: {from_json_value:#?}");
        println!("From yaml: {from_yaml_value:#?}");

        assert_eq!(from_json_value, from_yaml_value);
    }

    #[test]
    fn generate() {
        let definitions = include_str!("../json_syntax.json");
        let device = serde_json::from_str::<Device>(definitions).unwrap();

        let output = prettyplease::unparse(&syn::parse2(device.generate_definitions()).unwrap());
        println!("{output}");
    }
}
