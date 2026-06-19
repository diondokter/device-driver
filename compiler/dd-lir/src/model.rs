use device_driver_common::{
    identifier::{All, Identifier, Operation, Type},
    span::Spanned,
    specifiers::{Access, AddressMode, AddressRange, ByteOrder, Integer},
};

pub struct Driver {
    pub devices: Vec<Device>,
    pub field_sets: Vec<FieldSet>,
    pub enums: Vec<Enum>,
}

pub struct Device {
    pub internal_address_type: Integer,
    pub blocks: Vec<Block>,
}

pub struct Block {
    pub description: String,
    /// True for the root (top-level) block
    pub root: bool,
    pub name: Identifier<Type>,
    pub register_address_type: Integer,
    pub command_address_type: Integer,
    pub buffer_address_type: Integer,
    pub register_address_mode: Option<AddressMode>,
    pub methods: Vec<BlockMethod>,
}

pub struct BlockMethod {
    pub description: String,
    pub name: Identifier<Operation>,
    pub address: i128,
    pub repeat: Repeat,
    pub method_type: BlockMethodType,
}

pub enum Repeat {
    None,
    Count {
        count: u32,
        stride: i128,
    },
    Range {
        end: i128,
        start: i128,
        stride: i128,
    },
    Enum {
        enum_name: Identifier<Type>,
        enum_variants: Vec<Identifier<All>>,
        stride: i128,
    },
}

pub enum BlockMethodType {
    Block {
        name: Identifier<Type>,
    },
    Register {
        field_set_name: Identifier<Type>,
        access: Access,
        reset_value: Option<Spanned<Vec<u8>>>,
    },
    Command {
        field_set_name_in: Option<Identifier<Type>>,
        field_set_name_out: Option<Identifier<Type>>,
    },
    Buffer {
        access: Access,
    },
}

/// A set of fields, like a register or command in/out
pub struct FieldSet {
    pub description: String,
    pub name: Identifier<Type>,
    pub byte_order: ByteOrder,
    pub size_bytes: u32,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub description: String,
    pub name: Identifier<All>,
    pub address: AddressRange,
    pub base_type: String,
    pub conversion_method: FieldConversionMethod,
    pub access: Access,
    pub repeat: Repeat,
}

impl Field {
    pub fn address_text(&self) -> String {
        if self.address.len() <= 1 {
            format!("bit {}", self.address.start)
        } else {
            format!("{}:{}", self.address.end, self.address.start)
        }
    }
}

pub enum FieldConversionMethod {
    None,
    Into(Identifier<Type>),
    UnsafeInto(Identifier<Type>),
    TryInto(Identifier<Type>),
    Bool,
}

impl FieldConversionMethod {
    pub fn conversion_type(&self) -> Option<&Identifier<Type>> {
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
    pub name: Identifier<Type>,
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
    pub description: String,
    pub name: Identifier<All>,
    pub discriminant: i128,
    pub default: bool,
    pub catch_all: bool,
}
