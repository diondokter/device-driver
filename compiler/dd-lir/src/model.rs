use std::ops::Range;

use device_driver_common::{
    identifier::Identifier,
    specifiers::{Access, ByteOrder, Integer},
};

pub struct Driver {
    pub devices: Vec<Device>,
    pub field_sets: Vec<FieldSet>,
    pub enums: Vec<Enum>,
}

pub struct Device {
    pub internal_address_type: Integer,
    pub blocks: Vec<Block>,
    pub defmt_feature: Option<String>,
}

pub struct Block {
    pub description: String,
    /// True for the root (top-level) block
    pub root: bool,
    pub name: Identifier,
    pub methods: Vec<BlockMethod>,
}

pub struct BlockMethod {
    pub description: String,
    pub name: Identifier,
    pub address: i128,
    pub repeat: Repeat,
    pub method_type: BlockMethodType,
}

pub enum Repeat {
    None,
    Count {
        count: u64,
        stride: i128,
    },
    Enum {
        enum_name: Identifier,
        enum_variants: Vec<Identifier>,
        stride: i128,
    },
}

pub enum BlockMethodType {
    Block {
        name: Identifier,
    },
    Register {
        field_set_name: Identifier,
        access: Access,
        address_type: Integer,
        reset_value: Option<Vec<u8>>,
    },
    Command {
        field_set_name_in: Option<Identifier>,
        field_set_name_out: Option<Identifier>,
        address_type: Integer,
    },
    Buffer {
        access: Access,
        address_type: Integer,
    },
}

/// A set of fields, like a register or command in/out
pub struct FieldSet {
    pub description: String,
    pub name: Identifier,
    pub byte_order: ByteOrder,
    pub size_bits: u32,
    pub fields: Vec<Field>,
    pub defmt_feature: Option<String>,
}

impl FieldSet {
    pub fn size_bytes(&self) -> u32 {
        self.size_bits.div_ceil(8)
    }
}

pub struct Field {
    pub description: String,
    pub name: Identifier,
    pub address: Range<u32>,
    pub base_type: String,
    pub conversion_method: FieldConversionMethod,
    pub access: Access,
    pub repeat: Repeat,
}

impl Field {
    pub fn address_text(&self) -> String {
        if self.address.len() <= 1 {
            format!("@{}", self.address.start)
        } else {
            format!("@{}:{}", self.address.end - 1, self.address.start)
        }
    }
}

pub enum FieldConversionMethod {
    None,
    Into(Identifier),
    UnsafeInto(Identifier),
    TryInto(Identifier),
    Bool,
}

impl FieldConversionMethod {
    pub fn conversion_type(&self) -> Option<&Identifier> {
        match self {
            FieldConversionMethod::None => None,
            FieldConversionMethod::Into(type_path) => Some(type_path),
            FieldConversionMethod::UnsafeInto(type_path) => Some(type_path),
            FieldConversionMethod::TryInto(type_path) => Some(type_path),
            FieldConversionMethod::Bool => None,
        }
    }
}

pub struct Enum {
    pub description: String,
    pub name: Identifier,
    pub base_type: String,
    pub variants: Vec<EnumVariant>,
    pub defmt_feature: Option<String>,
}

impl Enum {
    pub fn default_variant(&self) -> Option<&EnumVariant> {
        self.variants.iter().find(|v| v.default)
    }

    pub fn catch_all_variant(&self) -> Option<&EnumVariant> {
        self.variants.iter().find(|v| v.catch_all)
    }
}

pub struct EnumVariant {
    pub description: String,
    pub name: Identifier,
    pub discriminant: i128,
    pub default: bool,
    pub catch_all: bool,
}
