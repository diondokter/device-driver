#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use std::iter::FromIterator;

use convert_case::Casing;
use deserialization::{BufferCollection, CommandCollection, FieldCollection, RegisterCollection};
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};

pub use deserialization::ResetValue;

pub mod deserialization;
mod generation;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub register_address_type: Option<BaseType>,
    pub registers: Option<RegisterCollection>,
    pub commands: Option<CommandCollection>,
    pub buffers: Option<BufferCollection>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Register {
    #[serde(skip)]
    pub name: String,
    pub description: Option<String>,
    #[serde(skip)]
    pub cfg_attributes: Vec<syn::Attribute>,
    #[serde(flatten)]
    pub kind: RegisterKind,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(untagged)]
pub enum RegisterKind {
    Standalone(StandaloneRegister),
    Block(RegisterBlock),
    Ref(RegisterRef),
}

impl RegisterKind {
    fn address(&self) -> u64 {
        match self {
            RegisterKind::Standalone(StandaloneRegister { address, .. }) => *address,
            RegisterKind::Block(RegisterBlock { base_address, .. }) => base_address.unwrap_or(0),
            RegisterKind::Ref(RegisterRef::Standalone(StandaloneRegisterRef {
                address, ..
            })) => *address,
            RegisterKind::Ref(RegisterRef::Block(RegisterBlockRef { base_address, .. })) => {
                *base_address
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct StandaloneRegister {
    pub address: u64,
    pub rw_type: RWType,
    pub size_bits: u64,
    /// BE by default
    pub byte_order: Option<ByteOrder>,
    pub reset_value: Option<ResetValue>,
    pub fields: FieldCollection,
}

impl From<StandaloneRegister> for RegisterKind {
    fn from(value: StandaloneRegister) -> Self {
        Self::Standalone(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct RegisterBlock {
    pub base_address: Option<u64>,
    pub repeat: Option<RegisterRepeat>,
    pub registers: RegisterCollection,
}

impl From<RegisterBlock> for RegisterKind {
    fn from(value: RegisterBlock) -> Self {
        Self::Block(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(untagged)]
pub enum RegisterRef {
    Standalone(StandaloneRegisterRef),
    Block(RegisterBlockRef),
}

impl From<RegisterRef> for RegisterKind {
    fn from(value: RegisterRef) -> Self {
        Self::Ref(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct StandaloneRegisterRef {
    pub copy_of: String,
    pub address: u64,
    pub rw_type: Option<RWType>,
}

impl From<StandaloneRegisterRef> for RegisterKind {
    fn from(value: StandaloneRegisterRef) -> Self {
        Self::Ref(RegisterRef::Standalone(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct RegisterBlockRef {
    pub copy_of: String,
    pub base_address: u64,
    pub repeat: Option<RegisterRepeat>,
}

impl From<RegisterBlockRef> for RegisterKind {
    fn from(value: RegisterBlockRef) -> Self {
        Self::Ref(RegisterRef::Block(value))
    }
}

impl RegisterRef {
    pub fn copy_of(&self) -> &str {
        match self {
            RegisterRef::Standalone(StandaloneRegisterRef { copy_of, .. })
            | RegisterRef::Block(RegisterBlockRef { copy_of, .. }) => copy_of,
        }
    }

    pub fn resolve<'a>(&self, registers: &'a [Register]) -> Option<&'a Register> {
        match registers.iter().find(|x| x.name == self.copy_of()) {
            Some(Register {
                kind: RegisterKind::Ref(r),
                ..
            }) => r.resolve(registers),
            Some(r) => Some(r),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterKindError;

impl StandaloneRegisterRef {
    pub fn apply(&self, register: &Register) -> Result<StandaloneRegister, RegisterKindError> {
        match &register.kind {
            RegisterKind::Standalone(r) => {
                let mut r = r.clone();
                r.address = self.address;
                r.rw_type = self.rw_type.unwrap_or(r.rw_type);
                Ok(r)
            }
            _ => Err(RegisterKindError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterRepeat {
    pub count: u64,
    pub stride: u64,
}

impl PartialOrd for Register {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Register {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.kind
            .address()
            .cmp(&other.kind.address())
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
    pub conversion: Option<TypePathOrEnum>,
    pub strict_conversion: Option<TypePathOrEnum>,
    pub start: u32,
    pub end: Option<u32>,
    #[serde(skip)]
    pub cfg_attributes: Vec<syn::Attribute>,
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    #[serde(skip)]
    pub name: String,
    pub id: u32,
    pub description: Option<String>,
    #[serde(skip)]
    pub cfg_attributes: Vec<syn::Attribute>,
}

impl PartialOrd for Command {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Command {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id
            .cmp(&other.id)
            .then_with(|| self.name.cmp(&other.name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Buffer {
    #[serde(skip)]
    pub name: String,
    pub id: u32,
    pub description: Option<String>,
    pub rw_type: RWType,
    #[serde(skip)]
    pub cfg_attributes: Vec<syn::Attribute>,
}

impl PartialOrd for Buffer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Buffer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id
            .cmp(&other.id)
            .then_with(|| self.name.cmp(&other.name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypePathOrEnum {
    TypePath(TypePath),
    Enum(IndexMap<String, EnumVariant>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub description: Option<String>,
    pub value: EnumVariantValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EnumVariantValue {
    #[default]
    None,
    Specified(i128),
    Default,
    CatchAll,
}

impl TryFrom<&str> for EnumVariantValue {
    type Error = serde::de::value::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use serde::Deserialize;
        Self::deserialize(serde::de::value::StrDeserializer::new(value))
    }
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
        field_description: &Option<String>,
        strict_conversion: bool,
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
                    let doc = value
                        .description
                        .as_ref()
                        .map(|description| quote!(#[doc = #description]));
                    let (value_specifier, num_enum_attr, data) = match value.value {
                        EnumVariantValue::Specified(value) => {
                            let value = proc_macro2::Literal::i128_unsuffixed(value);
                            (Some(quote!(= #value)), None, None)
                        }
                        EnumVariantValue::None => (None, None, None),
                        EnumVariantValue::Default => {
                            (None, Some(quote!(#[num_enum(default)])), None)
                        }
                        EnumVariantValue::CatchAll => (
                            None,
                            Some(quote!(#[num_enum(catch_all)])),
                            Some(quote!((#register_type))),
                        ),
                    };
                    quote! {
                        #doc
                        #num_enum_attr
                        #variant #data #value_specifier,
                    }
                }));

                let from_primitive = if strict_conversion {
                    quote!(device_driver::num_enum::FromPrimitive)
                } else {
                    quote!(device_driver::num_enum::TryFromPrimitive)
                };

                let doc = field_description
                    .as_ref()
                    .map(|description| quote!(#[doc = #description]));

                Some(
                    quote::quote! {
                        #doc
                        #[derive(#from_primitive, device_driver::num_enum::IntoPrimitive, Debug, Copy, Clone, PartialEq, Eq)]
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
                segments: syn::punctuated::Punctuated::from_iter(
                    self.0
                        .split("::")
                        .map(|seg| syn::parse_str::<syn::PathSegment>(seg).unwrap()),
                ),
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
    #[serde(alias = "co", alias = "CO", alias = "c", alias = "C")]
    ClearOnly,
}

impl RWType {
    pub fn into_type(&self) -> syn::Type {
        match self {
            RWType::ReadOnly => syn::parse_quote!(device_driver::ReadOnly),
            RWType::WriteOnly => syn::parse_quote!(device_driver::WriteOnly),
            RWType::ReadWrite => syn::parse_quote!(device_driver::ReadWrite),
            RWType::ReadClear => syn::parse_quote!(device_driver::ReadClear),
            RWType::ClearOnly => syn::parse_quote!(device_driver::ClearOnly),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum ByteOrder {
    /// Little endian
    #[serde(alias = "le", alias = "LE")]
    LE,
    /// Big endian
    #[serde(alias = "be", alias = "BE")]
    BE,
}

impl TryFrom<&str> for ByteOrder {
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

        pretty_assertions::assert_eq!(from_json_value, from_yaml_value);
    }

    #[test]
    fn generate() {
        let definitions = include_str!("../../test-files/json_syntax.json");
        let device = serde_json::from_str::<Device>(definitions).unwrap();

        let mut stream = TokenStream::new();

        let existing_impl = syn::parse_quote! {
            impl<FOO> MyRegisterDevice<FOO> {}
        };
        device
            .generate_device_impl(existing_impl)
            .to_tokens(&mut stream);
        device.generate_definitions().to_tokens(&mut stream);

        let output = prettyplease::unparse(&syn::parse2(stream).unwrap());
        println!("{output}");
    }
}
