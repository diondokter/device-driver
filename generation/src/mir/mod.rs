//! The MIR takes its data from HIR and makes sure all required data is there
//! and all optional data is filled in with defaults.

use std::ops::Range;

use convert_case::Boundary;

pub mod passes;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Device {
    pub global_config: GlobalConfig,
    pub objects: Vec<Object>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalConfig {
    pub default_register_access: Access,
    pub default_field_access: Access,
    pub default_buffer_access: Access,
    pub default_byte_order: ByteOrder,
    pub default_bit_order: BitOrder,
    pub register_address_type: Option<Integer>,
    pub command_address_type: Option<Integer>,
    pub buffer_address_type: Option<Integer>,
    pub name_word_boundaries: Vec<Boundary>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_register_access: Default::default(),
            default_field_access: Default::default(),
            default_buffer_access: Default::default(),
            default_byte_order: Default::default(),
            default_bit_order: Default::default(),
            register_address_type: Default::default(),
            command_address_type: Default::default(),
            buffer_address_type: Default::default(),
            name_word_boundaries: convert_case::Boundary::defaults(),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Block(Block),
    Register(Register),
    Command(Command),
    Buffer(Buffer),
    Ref(RefObject),
}

impl Object {
    pub(self) fn get_block_object_list(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Object::Block(b) => Some(&mut b.objects),
            _ => None,
        }
    }

    /// Get a mutable reference to the name of the specific object
    pub(self) fn name_mut(&mut self) -> &mut String {
        match self {
            Object::Block(val) => &mut val.name,
            Object::Register(val) => &mut val.name,
            Object::Command(val) => &mut val.name,
            Object::Buffer(val) => &mut val.name,
            Object::Ref(val) => &mut val.name,
        }
    }

    /// Get a reference to the name of the specific object
    pub(self) fn name(&self) -> &String {
        match self {
            Object::Block(val) => &val.name,
            Object::Register(val) => &val.name,
            Object::Command(val) => &val.name,
            Object::Buffer(val) => &val.name,
            Object::Ref(val) => &val.name,
        }
    }

    /// If the object has fields, get an iterator over all of them
    pub(self) fn fields_mut(&mut self) -> Option<Box<dyn Iterator<Item = &mut Field> + '_>> {
        match self {
            Object::Block(_) => None,
            Object::Register(val) => Some(Box::new(val.fields.iter_mut())),
            Object::Command(val) => Some(Box::new(
                val.in_fields.iter_mut().chain(val.out_fields.iter_mut()),
            )),
            Object::Buffer(_) => None,
            Object::Ref(_) => None,
        }
    }

    /// If the object has fields, get an iterator over all of them
    pub(self) fn fields(&self) -> Option<Box<dyn Iterator<Item = &Field> + '_>> {
        match self {
            Object::Block(_) => None,
            Object::Register(val) => Some(Box::new(val.fields.iter())),
            Object::Command(val) => {
                Some(Box::new(val.in_fields.iter().chain(val.out_fields.iter())))
            }
            Object::Buffer(_) => None,
            Object::Ref(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub cfg_attr: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Register {
    pub cfg_attr: Option<String>,
    pub description: String,
    pub name: String,
    pub access: Access,
    pub byte_order: ByteOrder,
    pub bit_order: BitOrder,
    pub address: u64,
    pub size_bits: u64,
    pub reset_value: Option<ResetValue>,
    pub repeat: Option<Repeat>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Field {
    pub cfg_attr: Option<String>,
    pub description: String,
    pub name: String,
    pub access: Access,
    pub base_type: BaseType,
    pub field_conversion: Option<FieldConversion>,
    pub field_address: Range<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BaseType {
    Bool,
    #[default]
    Uint,
    Int,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldConversion {
    Direct(String),
    Enum(Enum),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    generation_style: Option<EnumGenerationStyle>,
}

impl Enum {
    pub fn new(name: String, variants: Vec<EnumVariant>) -> Self {
        Self {
            name,
            variants,
            generation_style: None,
        }
    }

    fn new_with_style(
        name: String,
        variants: Vec<EnumVariant>,
        generation_style: EnumGenerationStyle,
    ) -> Self {
        Self {
            name,
            variants,
            generation_style: Some(generation_style),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumGenerationStyle {
    Fallible,
    Infallible,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnumVariant {
    pub cfg_attr: Option<String>,
    pub description: String,
    pub name: String,
    pub value: EnumValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EnumValue {
    #[default]
    Unspecified,
    Specified(u64),
    Default,
    CatchAll,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Command {
    pub cfg_attr: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Buffer {
    pub cfg_attr: Option<String>,
    pub description: String,
    pub name: String,
    pub access: Access,
    pub address: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefObject {
    pub cfg_attr: Option<String>,
    pub description: String,
    pub name: String,
    pub object: ObjectOverride,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectOverride {
    Block(BlockOverride),
    Register(RegisterOverride),
    Command(CommandOverride),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockOverride {
    pub name: String,
    pub address_offset: Option<i64>,
    pub repeat: Option<Repeat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterOverride {
    pub name: String,
    pub access: Option<Access>,
    pub address: Option<u64>,
    pub reset_value: Option<ResetValue>,
    pub repeat: Option<Repeat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOverride {
    pub name: String,
    pub address: Option<u64>,
    pub repeat: Option<Repeat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResetValue {
    Integer(u128),
    Array(Vec<u8>),
}
