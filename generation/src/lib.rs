use std::iter::FromIterator;

use convert_case::Casing;
use deserialization::{FieldCollection, RegisterCollection, DefaultValue};
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

mod deserialization;
mod generation;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub address_type: BaseType,
    pub registers: RegisterCollection,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Register {
    #[serde(skip)]
    pub name: String,
    pub rw_type: RWType,
    pub address: u64,
    pub size_bits: u64,
    pub description: Option<String>,
    pub default: Option<DefaultValue>,
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
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub register_type: BaseType,
    #[serde(rename = "conversion")]
    pub conversion_type: Option<TypePathOrEnum>,
    pub start: u32,
    pub end: Option<u32>,
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
                        #[derive(device_driver::num_enum::TryFromPrimitive, device_driver::num_enum::IntoPrimitive, Debug, Copy, Clone, PartialEq, Eq)]
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
pub struct TypePath(pub String);

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
pub enum RWType {
    #[serde(alias = "ro", alias = "RO", alias = "r", alias = "R")]
    ReadOnly,
    #[serde(alias = "wo", alias = "WO", alias = "w", alias = "W")]
    WriteOnly,
    #[serde(alias = "rw", alias = "RW")]
    ReadWrite,
    #[serde(alias = "rc", alias = "RC")]
    ReadClear,
}

impl RWType {
    pub fn into_type(&self) -> syn::Type {
        match self {
            RWType::ReadOnly => syn::parse_quote!(device_driver::ReadOnly),
            RWType::WriteOnly => syn::parse_quote!(device_driver::WriteOnly),
            RWType::ReadWrite => syn::parse_quote!(device_driver::ReadWrite),
            RWType::ReadClear => syn::parse_quote!(device_driver::ReadClear),
        }
    }
}

impl TryFrom<&str> for RWType {
    type Error = serde::de::value::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use serde::Deserialize;
        Self::deserialize(serde::de::value::StrDeserializer::new(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BaseType {
    #[serde(alias = "bool", alias = "boolean")]
    Bool,
    #[default]
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

impl BaseType {
    pub fn into_type(&self) -> syn::Type {
        match self {
            BaseType::Bool => syn::parse_quote!(bool),
            BaseType::U8 => syn::parse_quote!(u8),
            BaseType::U16 => syn::parse_quote!(u16),
            BaseType::U32 => syn::parse_quote!(u32),
            BaseType::U64 => syn::parse_quote!(u64),
            BaseType::U128 => syn::parse_quote!(u128),
            BaseType::Usize => syn::parse_quote!(usize),
            BaseType::I8 => syn::parse_quote!(i8),
            BaseType::I16 => syn::parse_quote!(i16),
            BaseType::I32 => syn::parse_quote!(i32),
            BaseType::I64 => syn::parse_quote!(i64),
            BaseType::I128 => syn::parse_quote!(i128),
            BaseType::Isize => syn::parse_quote!(isize),
        }
    }
}

impl TryFrom<&str> for BaseType {
    type Error = serde::de::value::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use serde::Deserialize;
        Self::deserialize(serde::de::value::StrDeserializer::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_file_formats() {
        let json_string = include_str!("../../test-files/json_syntax.json");
        let from_json_value = serde_json::from_str::<Device>(json_string).unwrap();
        let yaml_string = include_str!("../../test-files/yaml_syntax.yaml");
        let from_yaml_value = serde_yaml::from_str::<Device>(yaml_string).unwrap();

        println!("From json: {from_json_value:#?}");
        println!("From yaml: {from_yaml_value:#?}");

        assert_eq!(from_json_value, from_yaml_value);
    }

    #[test]
    fn generate() {
        let definitions = include_str!("../../test-files/json_syntax.json");
        let device = serde_json::from_str::<Device>(definitions).unwrap();

        let mut stream = TokenStream::new();

        device
            .generate_device_impl(syn::parse_quote! {
                impl<FOO> MyRegisterDevice<FOO> {}
            })
            .to_tokens(&mut stream);
        device.generate_definitions().to_tokens(&mut stream);

        let output = prettyplease::unparse(&syn::parse2(stream).unwrap());
        println!("{output}");
    }
}
