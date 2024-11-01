use std::ops::Range;

use proc_macro2::{Ident, Literal, TokenStream};

use crate::mir::{self, Access, BitOrder, ByteOrder};

pub mod passes;
pub mod token_transform;

pub struct Device {
    pub internal_address_type: Ident,
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
    pub name: Ident,
    pub methods: Vec<BlockMethod>,
}

pub struct BlockMethod {
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
    pub name: Ident,
    pub address: Literal,
    // Only used for LIR passes, not codegen
    pub allow_address_overlap: bool,
    pub kind: BlockMethodKind,
    pub method_type: BlockMethodType,
}

pub enum BlockMethodKind {
    Normal,
    Repeated { count: Literal, stride: Literal },
}

pub enum BlockMethodType {
    Block {
        name: Ident,
    },
    Register {
        field_set_name: Ident,
        access: Access,
        address_type: Ident,
        reset_value_function: Ident,
    },
    Command {
        field_set_name_in: Option<Ident>,
        field_set_name_out: Option<Ident>,
        address_type: Ident,
    },
    Buffer {
        access: Access,
        address_type: Ident,
    },
}

/// A set of fields, like a register or command in/out
pub struct FieldSet {
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
    pub name: Ident,
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
    pub name: Ident,
    pub address: Range<Literal>,
    pub base_type: Ident,
    pub conversion_method: FieldConversionMethod,
    pub access: mir::Access,
}

pub enum FieldConversionMethod {
    None,
    Into(TokenStream),
    UnsafeInto(TokenStream),
    TryInto(TokenStream),
    Bool,
}

pub struct Enum {
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
    pub name: Ident,
    pub base_type: Ident,
    pub variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
    pub name: Ident,
    pub number: Literal,
    pub default: bool,
    pub catch_all: bool,
}
