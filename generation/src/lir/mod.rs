use std::ops::Range;

use proc_macro2::{Ident, Literal, TokenStream};

use crate::mir::{self, Access, BitOrder, ByteOrder};

pub mod token_transform;

pub struct Device {
    pub blocks: Vec<Block>,
    pub field_sets: Vec<FieldSet>,
    pub enums: Vec<Enum>,
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
    },
    SimpleCommand {
        address_type: Ident,
    },
    Command {
        field_set_name_in: Ident,
        field_set_name_out: Ident,
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
    pub size_bits: usize,
    pub reset_value: Vec<u8>,
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
    Into(Ident),
    UnsafeInto(Ident),
    TryInto(Ident),
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
