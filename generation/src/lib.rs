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

impl Device {
    /// Turns all refs into real copied registers
    pub fn resolve(self) -> Result<ResolvedDevice, String> {
        if self.registers.is_some() && self.register_address_type.is_none() {
            return Err("`register_address_type` is not specified".into());
        }

        let mut registers = Vec::new();

        if let Some(unresolved_registers) = self.registers {
            for unresolved_register in unresolved_registers.0.iter() {
                registers.push(unresolved_register.resolve(&unresolved_registers.0)?);
            }
        }

        Ok(ResolvedDevice {
            register_address_type: self.register_address_type.unwrap_or_default(),
            registers,
            commands: self.commands.map(|v| v.0).unwrap_or_default(),
            buffers: self.buffers.map(|v| v.0).unwrap_or_default(),
        })
    }
}

pub struct ResolvedDevice {
    pub register_address_type: BaseType,
    pub registers: Vec<ResolvedRegisterItem>,
    pub commands: Vec<Command>,
    pub buffers: Vec<Buffer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRegisterItem {
    pub name: String,
    pub description: Option<String>,
    pub cfg_attributes: Vec<syn::Attribute>,
    pub kind: ResolvedRegisterKind,
}

impl PartialOrd for ResolvedRegisterItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResolvedRegisterItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.address()
            .cmp(&other.address())
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl ResolvedRegisterItem {
    fn address(&self) -> u64 {
        match &self.kind {
            ResolvedRegisterKind::Register(Register { address, .. }) => *address,
            ResolvedRegisterKind::Block(ResolvedRegisterBlock { base_address, .. }) => {
                base_address.unwrap_or(0)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct RegisterItem {
    #[serde(skip)]
    pub name: String,
    pub description: Option<String>,
    #[serde(skip)]
    pub cfg_attributes: Vec<syn::Attribute>,
    #[serde(flatten)]
    pub kind: RegisterKind,
}

impl PartialOrd for RegisterItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RegisterItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.address()
            .cmp(&other.address())
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl RegisterItem {
    fn address(&self) -> u64 {
        match &self.kind {
            RegisterKind::Register(Register { address, .. }) => *address,
            RegisterKind::Block(RegisterBlock { base_address, .. }) => base_address.unwrap_or(0),
            RegisterKind::RegisterRef(RegisterRef { address, .. }) => *address,
            RegisterKind::BlockRef(RegisterBlockRef { base_address, .. }) => *base_address,
        }
    }

    /// Returns Some with the name of the register item this copies
    fn copy_of(&self) -> Option<&str> {
        match &self.kind {
            RegisterKind::RegisterRef(RegisterRef { copy_of, .. })
            | RegisterKind::BlockRef(RegisterBlockRef { copy_of, .. }) => Some(copy_of),
            _ => None,
        }
    }

    /// Get the resolved register item from this unresolved item
    fn resolve(&self, registers: &[RegisterItem]) -> Result<ResolvedRegisterItem, String> {
        match &self.kind {
            RegisterKind::Register(r) => Ok(ResolvedRegisterItem {
                name: self.name.clone(),
                description: self.description.clone(),
                cfg_attributes: self.cfg_attributes.clone(),
                kind: ResolvedRegisterKind::Register(r.clone()),
            }),
            RegisterKind::Block(b) => Ok(ResolvedRegisterItem {
                name: self.name.clone(),
                description: self.description.clone(),
                cfg_attributes: self.cfg_attributes.clone(),
                kind: ResolvedRegisterKind::Block(ResolvedRegisterBlock {
                    base_address: b.base_address,
                    repeat: b.repeat,
                    registers: b
                        .registers
                        .0
                        .iter()
                        .map(|unresolved_register| unresolved_register.resolve(registers))
                        .collect::<Result<_, _>>()?,
                }),
            }),
            RegisterKind::RegisterRef(_) | RegisterKind::BlockRef(_) => {
                match registers
                    .iter()
                    .find(|x| self.copy_of() == Some(x.name.as_str()))
                {
                    Some(copy_source_item) => {
                        let mut resolved_copy_item = copy_source_item.resolve(registers)?;
                        resolved_copy_item.name = self.name.clone();
                        resolved_copy_item.cfg_attributes = self.cfg_attributes.clone();
                        resolved_copy_item.description = self.description.clone();

                        match (&self.kind, &mut resolved_copy_item.kind) {
                            (RegisterKind::RegisterRef(refreg), ResolvedRegisterKind::Register(copy_reg)) => {
                                copy_reg.address = refreg.address;
                                copy_reg.rw_type = refreg.rw_type.unwrap_or(copy_reg.rw_type);
                            },
                            (RegisterKind::BlockRef(refblock), ResolvedRegisterKind::Block(copy_block)) => {
                                copy_block.base_address = Some(refblock.base_address);
                                copy_block.repeat = refblock.repeat;
                            },
                            (RegisterKind::RegisterRef(_), ResolvedRegisterKind::Block(_)) => return Err(format!("Cannot point at block '{}' from register definition '{}'", self.copy_of().unwrap(), self.name)),
                            (RegisterKind::BlockRef(_), ResolvedRegisterKind::Register(_)) => return Err(format!("Cannot point at register '{}' from block definition '{}'", self.copy_of().unwrap(), self.name)),
                            _ => unreachable!()
                        }

                        Ok(resolved_copy_item)
                    },
                    _ => Err(format!("Could not find register/block '{}', which is linked by register/block '{}'", self.copy_of().unwrap(), self.name)),
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedRegisterKind {
    Register(Register),
    Block(ResolvedRegisterBlock),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(untagged)]
pub enum RegisterKind {
    Register(Register),
    Block(RegisterBlock),
    RegisterRef(RegisterRef),
    BlockRef(RegisterBlockRef),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Register {
    pub address: u64,
    pub rw_type: RWType,
    pub size_bits: u64,
    /// BE by default
    pub byte_order: Option<ByteOrder>,
    pub reset_value: Option<ResetValue>,
    pub fields: FieldCollection,
}

impl From<Register> for RegisterKind {
    fn from(value: Register) -> Self {
        Self::Register(value)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRegisterBlock {
    pub base_address: Option<u64>,
    pub repeat: Option<RegisterRepeat>,
    pub registers: Vec<ResolvedRegisterItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct RegisterRef {
    pub copy_of: String,
    pub address: u64,
    pub rw_type: Option<RWType>,
}

impl From<RegisterRef> for RegisterKind {
    fn from(value: RegisterRef) -> Self {
        Self::RegisterRef(value)
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
        Self::BlockRef(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterKindError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterRepeat {
    pub count: u64,
    pub stride: u64,
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
