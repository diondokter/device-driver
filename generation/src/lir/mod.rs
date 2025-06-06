use std::ops::Range;

use crate::mir::{self, Access, BitOrder, ByteOrder, Integer};

pub mod code_transform;
pub mod passes;

pub struct Device {
    pub internal_address_type: Integer,
    pub register_address_type: Integer,
    pub blocks: Vec<Block>,
    pub field_sets: Vec<FieldSet>,
    pub enums: Vec<Enum>,
    pub defmt_feature: Option<String>,
}

pub struct Block {
    pub cfg_attr: String,
    pub description: String,
    /// True for the root (top-level) block
    pub root: bool,
    pub name: String,
    pub methods: Vec<BlockMethod>,
}

pub struct BlockMethod {
    pub cfg_attr: String,
    pub description: String,
    pub name: String,
    pub address: i64,
    // Only used for LIR passes, not codegen
    pub allow_address_overlap: bool,
    pub kind: BlockMethodKind,
    pub method_type: BlockMethodType,
}

pub enum BlockMethodKind {
    Normal,
    Repeated { count: u64, stride: i64 },
}

pub enum BlockMethodType {
    Block {
        name: String,
    },
    Register {
        field_set_name: String,
        access: Access,
        address_type: Integer,
        reset_value_function: String,
    },
    Command {
        field_set_name_in: Option<String>,
        field_set_name_out: Option<String>,
        address_type: Integer,
    },
    Buffer {
        access: Access,
        address_type: Integer,
    },
}

/// A set of fields, like a register or command in/out
pub struct FieldSet {
    pub cfg_attr: String,
    pub description: String,
    pub name: String,
    pub byte_order: ByteOrder,
    pub bit_order: BitOrder,
    pub size_bits: u32,
    pub reset_value: Vec<u8>,
    pub ref_reset_overrides: Vec<(String, Vec<u8>)>,
    pub fields: Vec<Field>,
}

impl FieldSet {
    pub fn size_bytes(&self) -> u32 {
        self.size_bits.div_ceil(8)
    }
}

pub struct Field {
    pub cfg_attr: String,
    pub description: String,
    pub name: String,
    pub address: Range<u32>,
    pub base_type: String,
    pub conversion_method: FieldConversionMethod,
    pub access: mir::Access,
}

pub enum FieldConversionMethod {
    None,
    Into(String),
    UnsafeInto(String),
    TryInto(String),
    Bool,
}

impl FieldConversionMethod {
    pub fn conversion_type(&self) -> Option<&String> {
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
    pub cfg_attr: String,
    pub description: String,
    pub name: String,
    pub base_type: String,
    pub variants: Vec<EnumVariant>,
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
    pub cfg_attr: String,
    pub description: String,
    pub name: String,
    pub number: i128,
    pub default: bool,
    pub catch_all: bool,
}
