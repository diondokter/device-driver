use std::ops::Range;

use crate::mir;

pub struct Device {
    pub register_address_type: &'static str,
    pub command_address_type: &'static str,
    pub buffer_address_type: &'static str,

    pub blocks: Vec<Block>,
    pub field_sets: Vec<FieldSet>,
    pub enums: Vec<Enum>,
}

pub struct Block {
    pub name: String,
    pub address_offset: u64,
    pub methods: Vec<BlockMethod>,
}

pub struct BlockMethod {
    pub cfg_attr: String,
    pub doc_attr: String,
    pub name: String,
    pub address: u64,
    pub kind: BlockMethodKind,
    pub return_type: String,
}

pub enum BlockMethodKind {
    Block,
    BlockRepeated { count: u64, stride: u64 },
    Register,
    RegisterRepeated { count: u64, stride: u64 },
    Command,
    CommandRepeated { count: u64, stride: u64 },
    SimpleCommand,
    SimpleCommandRepeated { count: u64, stride: u64 },
    Buffer,
}

/// A set of fields, like a register or command in/out
pub struct FieldSet {
    pub cfg_attr: String,
    pub doc_attr: String,
    pub name: String,
    pub size_bits: u64,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub cfg_attr: String,
    pub doc_attr: String,
    pub name: String,
    pub address: Range<u64>,
    pub base_type: String,
    pub conversion_method: FieldConversionMethod,
    pub access: mir::Access,
}

pub enum FieldConversionMethod {
    None,
    Into(String),
    UnsafeInto(String),
    TryInto(String),
}

pub struct Enum {
    pub cfg_attr: String,
    pub doc_attr: String,
    pub name: String,
    pub base_type: String,
    pub variants: Vec<EnumVariant>
}

pub struct EnumVariant {
    pub cfg_attr: String,
    pub description: String,
    pub name: String,
    pub value: EnumValue,
}

pub enum EnumValue {
    Specified(u64),
    Default,
    CatchAll,
}
