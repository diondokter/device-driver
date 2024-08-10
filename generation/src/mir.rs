//! The MIR takes its data from HIR and makes sure all required data is there
//! and all optional data is filled in with defaults.

use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Device {
    pub global_config: GlobalConfig,
    pub objects: Vec<Object>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GlobalConfig {
    pub default_register_access: Access,
    pub default_field_access: Access,
    pub default_buffer_access: Access,
    pub default_byte_order: ByteOrder,
    pub default_bit_order: BitOrder,
    pub register_address_type: Option<Integer>,
    pub command_address_type: Option<Integer>,
    pub buffer_address_type: Option<Integer>,
    pub name_case: NameCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Integer {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Access {
    #[default]
    RW,
    RC,
    RO,
    WO,
    CO,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ByteOrder {
    LE,
    #[default]
    BE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BitOrder {
    #[default]
    LSB0,
    MSB0,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum NameCase {
    #[default]
    Varying,
    Pascal,
    Snake,
    ScreamingSnake,
    Camel,
    Kebab,
    Cobol,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Block(Block),
    Register(Register),
    Command(Command),
    Buffer(Buffer),
    Ref(RefObject),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub cfg_attrs: Vec<String>,
    pub description: String,
    pub name: String,
    pub address_offset: i64,
    pub repeat: Option<Repeat>,
    pub objects: Vec<Object>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repeat {
    pub count: u64,
    pub stride: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Register {
    pub cfg_attrs: Vec<String>,
    pub description: String,
    pub name: String,
    pub access: Access,
    pub byte_order: ByteOrder,
    pub bit_order: BitOrder,
    pub address: u64,
    pub size_bits: u64,
    pub reset_value: Vec<u8>,
    pub repeat: Option<Repeat>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub cfg_attrs: Vec<String>,
    pub description: String,
    pub name: String,
    pub access: Access,
    pub base_type: BaseType,
    pub field_conversion: Option<FieldConversion>,
    pub field_address: Range<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseType {
    Bool,
    Uint,
    Int,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldConversion {
    Direct(String),
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub cfg_attrs: Vec<String>,
    pub description: String,
    pub name: String,
    pub value: EnumValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumValue {
    Specified(u128),
    Default,
    CatchAll,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub cfg_attrs: Vec<String>,
    pub description: String,
    pub name: String,
    pub address: u64,
    pub byte_order: ByteOrder,
    pub bit_order: BitOrder,
    pub size_bits_in: u64,
    pub size_bits_out: u64,
    pub repeat: Option<Repeat>,
    pub in_fields: Vec<Field>,
    pub out_fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    pub cfg_attrs: Option<String>,
    pub description: String,
    pub name: String,
    pub access: Access,
    pub address: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefObject {
    pub cfg_attrs: Vec<String>,
    pub description: String,
    pub name: String,
    pub object: Box<Object>,
}