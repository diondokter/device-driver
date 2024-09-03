use std::ops::Range;

use proc_macro2::{Ident, Literal, TokenStream};

use crate::mir::{self, BitOrder, ByteOrder};

pub mod lir_transform;

pub struct Device {
    pub register_address_type: Ident,
    pub command_address_type: Ident,
    pub buffer_address_type: Ident,

    pub blocks: Vec<Block>,
    pub field_sets: Vec<FieldSet>,
    pub enums: Vec<Enum>,
}

pub struct Block {
    pub name: Ident,
    pub address_offset: Literal,
    pub methods: Vec<BlockMethod>,
}

pub struct BlockMethod {
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
    pub name: Ident,
    pub address: Literal,
    pub kind: BlockMethodKind,
    pub return_type: Ident,
}

pub enum BlockMethodKind {
    Block,
    BlockRepeated { count: Literal, stride: Literal },
    Register,
    RegisterRepeated { count: Literal, stride: Literal },
    Command,
    CommandRepeated { count: Literal, stride: Literal },
    SimpleCommand,
    SimpleCommandRepeated { count: Literal, stride: Literal },
    Buffer,
}

/// A set of fields, like a register or command in/out
pub struct FieldSet {
    pub cfg_attr: TokenStream,
    pub doc_attr: TokenStream,
    pub name: Ident,
    pub byte_order: ByteOrder,
    pub bit_order: BitOrder,
    pub size_bits: Literal,
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
