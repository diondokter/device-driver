//! The MIR takes its data from HIR and makes sure all required data is there
//! and all optional data is filled in with defaults.

use std::{fmt::Display, ops::Range};

use convert_case::Boundary;

pub mod lir_transform;
pub mod passes;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Device {
    pub name: Option<String>,
    pub global_config: GlobalConfig,
    pub objects: Vec<Object>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalConfig {
    pub default_register_access: Access,
    pub default_field_access: Access,
    pub default_buffer_access: Access,
    pub default_byte_order: Option<ByteOrder>,
    pub default_bit_order: BitOrder,
    pub register_address_type: Option<Integer>,
    pub command_address_type: Option<Integer>,
    pub buffer_address_type: Option<Integer>,
    pub name_word_boundaries: Vec<Boundary>,
    pub defmt_feature: Option<String>,
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
            name_word_boundaries: convert_case::Boundary::defaults().to_vec(),
            defmt_feature: Default::default(),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::Display, strum::EnumString,
)]
#[strum(serialize_all = "lowercase")]
pub enum Integer {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl Integer {
    pub fn is_signed(&self) -> bool {
        self.min_value() != 0
    }

    pub fn min_value(&self) -> i128 {
        match self {
            Integer::U8 => u8::MIN as i128,
            Integer::U16 => u16::MIN as i128,
            Integer::U32 => u32::MIN as i128,
            Integer::U64 => u64::MIN as i128,
            Integer::I8 => i8::MIN as i128,
            Integer::I16 => i16::MIN as i128,
            Integer::I32 => i32::MIN as i128,
            Integer::I64 => i64::MIN as i128,
        }
    }

    pub fn max_value(&self) -> i128 {
        match self {
            Integer::U8 => u8::MAX as i128,
            Integer::U16 => u16::MAX as i128,
            Integer::U32 => u32::MAX as i128,
            Integer::U64 => u64::MAX as i128,
            Integer::I8 => i8::MAX as i128,
            Integer::I16 => i16::MAX as i128,
            Integer::I32 => i32::MAX as i128,
            Integer::I64 => i64::MAX as i128,
        }
    }

    pub fn size_bits(&self) -> u32 {
        match self {
            Integer::U8 => 8,
            Integer::U16 => 16,
            Integer::U32 => 32,
            Integer::U64 => 64,
            Integer::I8 => 8,
            Integer::I16 => 16,
            Integer::I32 => 32,
            Integer::I64 => 64,
        }
    }

    /// Find the smallest integer type that can fully contain the min and max
    /// and is equal or larger than the given size_bits.
    ///
    /// This function has a preference for unsigned integers.
    /// You can force a signed integer by making the min be negative (e.g. -1)
    pub fn find_smallest(min: i128, max: i128, size_bits: u32) -> Option<Integer> {
        Some(match (min, max, size_bits) {
            (0.., ..0x1_00, ..=8) => Integer::U8,
            (0.., ..0x1_0000, ..=16) => Integer::U16,
            (0.., ..0x1_0000_0000, ..=32) => Integer::U32,
            (0.., ..0x1_0000_0000_0000_0000, ..=64) => Integer::U64,
            (-0x80.., ..0x80, ..=8) => Integer::I8,
            (-0x8000.., ..0x8000, ..=16) => Integer::I16,
            (-0x8000_00000.., ..0x8000_0000, ..=32) => Integer::I32,
            (-0x8000_0000_0000_0000.., ..0x8000_0000_0000_0000, ..=32) => Integer::I64,
            _ => return None,
        })
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    strum::VariantNames,
    strum::Display,
    strum::EnumString,
)]
pub enum Access {
    #[default]
    RW,
    RO,
    WO,
}

impl Access {
    pub fn is_readable(&self) -> bool {
        match self {
            Access::RW => true,
            Access::RO => true,
            Access::WO => false,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::Display, strum::EnumString,
)]
pub enum ByteOrder {
    LE,
    BE,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    strum::VariantNames,
    strum::Display,
    strum::EnumString,
)]
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
    FieldSet(FieldSet),
    Enum(Enum),
    Extern(Extern),
}

impl Object {
    pub(self) fn get_block_object_list_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Object::Block(b) => Some(&mut b.objects),
            _ => None,
        }
    }

    pub(self) fn get_block_object_list(&self) -> Option<&[Self]> {
        match self {
            Object::Block(b) => Some(&b.objects),
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
            Object::FieldSet(val) => &mut val.name,
            Object::Enum(val) => &mut val.name,
            Object::Extern(val) => &mut val.name,
        }
    }

    /// Get a reference to the name of the specific object
    pub(self) fn name(&self) -> &str {
        match self {
            Object::Block(val) => &val.name,
            Object::Register(val) => &val.name,
            Object::Command(val) => &val.name,
            Object::Buffer(val) => &val.name,
            Object::FieldSet(val) => &val.name,
            Object::Enum(val) => &val.name,
            Object::Extern(val) => &val.name,
        }
    }

    pub(self) fn field_set_refs_mut(&mut self) -> Vec<&mut FieldSetRef> {
        match self {
            Object::Block(_) => Vec::new(),
            Object::Register(register) => vec![&mut register.field_set_ref],
            Object::Command(command) => {
                let mut buffer = Vec::new();

                if let Some(fs_in) = command.field_set_ref_in.as_mut() {
                    buffer.push(fs_in);
                }
                if let Some(fs_out) = command.field_set_ref_out.as_mut() {
                    buffer.push(fs_out);
                }

                buffer
            }
            Object::Buffer(_) => Vec::new(),
            Object::FieldSet(_) => Vec::new(),
            Object::Enum(_) => Vec::new(),
            Object::Extern(_) => Vec::new(),
        }
    }

    /// Return the address if it is specified.
    fn address(&self) -> Option<i128> {
        match self {
            Object::Block(block) => Some(block.address_offset),
            Object::Register(register) => Some(register.address),
            Object::Command(command) => Some(command.address),
            Object::Buffer(buffer) => Some(buffer.address),
            Object::FieldSet(_) => None,
            Object::Enum(_) => None,
            Object::Extern(_) => None,
        }
    }

    /// Return the repeat value if it exists
    fn repeat(&self) -> Option<Repeat> {
        match self {
            Object::Block(block) => block.repeat,
            Object::Register(register) => register.repeat,
            Object::Command(command) => command.repeat,
            Object::Buffer(_) => None,
            Object::FieldSet(_) => None,
            Object::Enum(_) => None,
            Object::Extern(_) => None,
        }
    }

    pub(self) fn as_field_set(&self) -> Option<&FieldSet> {
        if let Self::FieldSet(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(self) fn as_field_set_mut(&mut self) -> Option<&mut FieldSet> {
        if let Self::FieldSet(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Block {
    pub description: String,
    pub name: String,
    pub address_offset: i128,
    pub repeat: Option<Repeat>,
    pub objects: Vec<Object>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Repeat {
    pub count: u64,
    pub stride: i128,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Register {
    pub description: String,
    pub name: String,
    pub access: Access,
    pub allow_address_overlap: bool,
    pub address: i128,
    pub reset_value: Option<ResetValue>,
    pub repeat: Option<Repeat>,
    pub field_set_ref: FieldSetRef,
}

/// Here for DSL + Manifest codegen.
/// There fieldsets have functions for the reset value.
/// Once we hit 2.0, this can be removed
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyFieldSetInfo {
    pub reset_value: Option<Vec<u8>>,
    pub ref_reset_overrides: Vec<(String, Vec<u8>)>,
}

/// An externally defined fieldset. This is the name of that fieldset
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldSetRef(pub String);

impl From<String> for FieldSetRef {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a str> for FieldSetRef {
    fn from(value: &'a str) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldSet {
    pub description: String,
    pub name: String,
    pub size_bits: u32,
    pub byte_order: Option<ByteOrder>,
    pub bit_order: Option<BitOrder>,
    pub allow_bit_overlap: bool,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Field {
    pub description: String,
    pub name: String,
    pub access: Access,
    pub base_type: BaseType,
    pub field_conversion: Option<FieldConversion>,
    pub field_address: Range<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BaseType {
    Unspecified,
    Bool,
    #[default]
    Uint,
    Int,
    FixedSize(Integer),
}

impl BaseType {
    /// Returns `true` if the base type is [`Unspecified`].
    ///
    /// [`Unspecified`]: BaseType::Unspecified
    #[must_use]
    pub fn is_unspecified(&self) -> bool {
        matches!(self, Self::Unspecified)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldConversion {
    /// The name of the type we're converting to
    pub type_name: String,
    /// True when we want to use the fallible interface (like a Result<type, error>)
    pub use_try: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Enum {
    pub description: String,
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub base_type: BaseType,
    pub size_bits: Option<u32>,
    generation_style: Option<EnumGenerationStyle>,
}

impl Enum {
    pub fn new(
        description: String,
        name: String,
        variants: Vec<EnumVariant>,
        base_type: BaseType,
        size_bits: Option<u32>,
    ) -> Self {
        Self {
            description,
            name,
            variants,
            base_type,
            size_bits,
            generation_style: None,
        }
    }

    #[cfg(test)]
    pub fn new_with_style(
        description: String,
        name: String,
        variants: Vec<EnumVariant>,
        base_type: BaseType,
        size_bits: Option<u32>,
        generation_style: EnumGenerationStyle,
    ) -> Self {
        Self {
            description,
            name,
            variants,
            base_type,
            size_bits,
            generation_style: Some(generation_style),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumGenerationStyle {
    /// Not all basetype values can be converted to a variant
    Fallible,
    /// All bitpatterns within bits 0..size-bits are covered.
    /// The general interface is fallible, but this special knowledge can be used for safety guarantees
    InfallibleWithinRange,
    /// There's a fallback, so it's always safe
    Fallback,
}

impl EnumGenerationStyle {
    /// Returns `true` if the enum generation style is [`Fallible`].
    ///
    /// [`Fallible`]: EnumGenerationStyle::Fallible
    #[must_use]
    pub fn is_fallible(&self) -> bool {
        matches!(self, Self::Fallible)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnumVariant {
    pub description: String,
    pub name: String,
    pub value: EnumValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EnumValue {
    #[default]
    Unspecified,
    Specified(i128),
    Default,
    CatchAll,
}

impl EnumValue {
    /// Returns `true` if the enum value is [`Default`].
    ///
    /// [`Default`]: EnumValue::Default
    #[must_use]
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }

    /// Returns `true` if the enum value is [`CatchAll`].
    ///
    /// [`CatchAll`]: EnumValue::CatchAll
    #[must_use]
    pub fn is_catch_all(&self) -> bool {
        matches!(self, Self::CatchAll)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Command {
    pub description: String,
    pub name: String,
    pub address: i128,
    pub allow_address_overlap: bool,
    pub repeat: Option<Repeat>,

    pub field_set_ref_in: Option<FieldSetRef>,
    pub field_set_ref_out: Option<FieldSetRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Buffer {
    pub description: String,
    pub name: String,
    pub access: Access,
    pub address: i128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResetValue {
    Integer(u128),
    Array(Vec<u8>),
}

impl ResetValue {
    pub fn as_array(&self) -> Option<&Vec<u8>> {
        if let Self::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Extern {
    pub description: String,
    pub name: String,
    /// From/into what base type can this extern be converted?
    pub base_type: BaseType,
    /// We can convert safely if the base type value is under this limit
    /// If this is none, it's basically the size_bits of the base_type.
    pub size_bits: Option<u32>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UniqueId {
    object_name: String,
}

impl Display for UniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.object_name)
    }
}

pub trait Unique {
    fn id(&self) -> UniqueId;
}

impl Unique for Device {
    fn id(&self) -> UniqueId {
        UniqueId {
            object_name: self
                .name
                .clone()
                .expect("Can only get a device unique id when it's initialized with a name"),
        }
    }
}

macro_rules! impl_unique {
    ($t:ty) => {
        impl Unique for $t {
            fn id(&self) -> UniqueId {
                UniqueId {
                    object_name: self.name.clone(),
                }
            }
        }
    };
}

impl_unique!(Register);
impl_unique!(Command);
impl_unique!(Buffer);
impl_unique!(Block);
impl_unique!(Enum);
impl_unique!(EnumVariant);
impl_unique!(FieldSet);
impl_unique!(Extern);

impl Unique for Object {
    fn id(&self) -> UniqueId {
        match self {
            Object::Block(val) => val.id(),
            Object::Register(val) => val.id(),
            Object::Command(val) => val.id(),
            Object::Buffer(val) => val.id(),
            Object::FieldSet(val) => val.id(),
            Object::Enum(val) => val.id(),
            Object::Extern(val) => val.id(),
        }
    }
}
