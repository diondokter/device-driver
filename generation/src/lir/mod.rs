use std::ops::Range;

use proc_macro2::{Literal, TokenStream};

use crate::mir::{self, Access, BitOrder, ByteOrder, Integer};

pub mod passes;
pub mod token_transform;

pub struct Device {
    pub internal_address_type: Integer,
    pub register_address_type: Integer,
    pub blocks: Vec<Block>,
    pub field_sets: Vec<FieldSet>,
    pub enums: Vec<Enum>,
    pub defmt_feature: Option<String>,
}

pub struct Block {
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
    /// True for the root (top-level) block
    pub root: bool,
    pub name: String,
    pub methods: Vec<BlockMethod>,
}

pub struct BlockMethod {
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
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
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
    pub name: String,
    pub byte_order: ByteOrder,
    pub bit_order: BitOrder,
    pub size_bits: u32,
    pub reset_value: Vec<u8>,
    pub ref_reset_overrides: Vec<(String, Vec<u8>)>,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
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
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
    pub name: String,
    pub base_type: String,
    pub variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
    pub name: String,
    pub number: Literal,
    pub default: bool,
    pub catch_all: bool,
}
