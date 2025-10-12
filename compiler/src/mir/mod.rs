//! The MIR takes its data from HIR and makes sure all required data is there
//! and all optional data is filled in with defaults.

use std::{fmt::Display, ops::Range};

use convert_case::Boundary;

pub mod lir_transform;
pub mod passes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub root_objects: Vec<Object>,
}

impl Manifest {
    pub fn iter_objects_mut(&mut self) -> ManifestIterMut<'_> {
        ManifestIterMut {
            children: &mut self.root_objects,
            parent: None,
            collection_object_returned: false,
        }
    }

    pub fn iter_objects(&self) -> ManifestIter<'_> {
        ManifestIter {
            children: &self.root_objects,
            parent: None,
            collection_object_returned: false,
        }
    }
}

#[derive(Default)]
struct ManifestIterMut<'a> {
    children: &'a mut [Object],
    parent: Option<Box<ManifestIterMut<'a>>>,
    collection_object_returned: bool,
}

impl<'a> Iterator for ManifestIterMut<'a> {
    type Item = &'a mut Object;

    fn next(&mut self) -> Option<Self::Item> {
        let children = std::mem::take(&mut self.children);

        match children.split_first_mut() {
            None => match self.parent.take() {
                Some(parent) => {
                    // continue with the parent node
                    *self = *parent;
                    self.next()
                }
                None => None,
            },
            Some((first, rest)) => {
                self.children = rest;

                if first.child_objects_mut().is_empty() {
                    Some(first)
                } else if !self.collection_object_returned {
                    self.collection_object_returned = true;
                    Some(first)
                } else {
                    self.collection_object_returned = false;
                    *self = ManifestIterMut {
                        children: first.child_objects_mut(),
                        parent: Some(Box::new(std::mem::take(self))),
                        ..Default::default()
                    };
                    self.next()
                }
            }
        }
    }
}

#[derive(Default)]
struct ManifestIter<'a> {
    children: &'a [Object],
    parent: Option<Box<ManifestIter<'a>>>,
    collection_object_returned: bool,
}

impl<'a> Iterator for ManifestIter<'a> {
    type Item = &'a Object;

    fn next(&mut self) -> Option<Self::Item> {
        let children = std::mem::take(&mut self.children);

        match children.split_first() {
            None => match self.parent.take() {
                Some(parent) => {
                    // continue with the parent node
                    *self = *parent;
                    self.next()
                }
                None => None,
            },
            Some((first, rest)) => {
                self.children = rest;

                if first.child_objects().is_empty() {
                    Some(first)
                } else if !self.collection_object_returned {
                    self.collection_object_returned = true;
                    Some(first)
                } else {
                    self.collection_object_returned = false;
                    *self = ManifestIter {
                        children: first.child_objects(),
                        parent: Some(Box::new(std::mem::take(self))),
                        ..Default::default()
                    };
                    self.next()
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Device {
    pub name: String,
    pub device_config: DeviceConfig,
    pub objects: Vec<Object>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceConfig {
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

impl Default for DeviceConfig {
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
    Device(Device),
    Block(Block),
    Register(Register),
    Command(Command),
    Buffer(Buffer),
    FieldSet(FieldSet),
    Enum(Enum),
    Extern(Extern),
}

impl Object {
    pub(self) fn child_objects_mut(&mut self) -> &mut [Object] {
        match self {
            Object::Device(device) => &mut device.objects,
            Object::Block(block) => &mut block.objects,
            _ => &mut [],
        }
    }

    pub(self) fn child_objects(&self) -> &[Object] {
        match self {
            Object::Device(device) => &device.objects,
            Object::Block(block) => &block.objects,
            _ => &[],
        }
    }

    /// Get a mutable reference to the name of the specific object
    pub(self) fn name_mut(&mut self) -> &mut String {
        match self {
            Object::Device(val) => &mut val.name,
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
            Object::Device(val) => &val.name,
            Object::Block(val) => &val.name,
            Object::Register(val) => &val.name,
            Object::Command(val) => &val.name,
            Object::Buffer(val) => &val.name,
            Object::FieldSet(val) => &val.name,
            Object::Enum(val) => &val.name,
            Object::Extern(val) => &val.name,
        }
    }

    pub(self) fn type_name(&self) -> &'static str {
        match self {
            Object::Device(_) => "device",
            Object::Block(_) => "block",
            Object::Register(_) => "register",
            Object::Command(_) => "command",
            Object::Buffer(_) => "buffer",
            Object::FieldSet(_) => "fieldset",
            Object::Enum(_) => "enum",
            Object::Extern(_) => "extern",
        }
    }

    pub(self) fn field_set_refs_mut(&mut self) -> Vec<&mut FieldSetRef> {
        match self {
            Object::Device(_) => Vec::new(),
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
            Object::Device(_) => None,
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
            Object::Device(_) => None,
            Object::Block(block) => block.repeat.clone(),
            Object::Register(register) => register.repeat.clone(),
            Object::Command(command) => command.repeat.clone(),
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

    pub(self) fn as_enum(&self) -> Option<&Enum> {
        if let Self::Enum(v) = self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repeat {
    pub source: RepeatSource,
    pub stride: i128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepeatSource {
    Count(u64),
    Enum(String),
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
    pub repeat: Option<Repeat>,
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

    /// Returns `true` if the base type is [`FixedSize`].
    ///
    /// [`FixedSize`]: BaseType::FixedSize
    #[must_use]
    pub fn is_fixed_size(&self) -> bool {
        matches!(self, Self::FixedSize(..))
    }
}

impl Display for BaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseType::Unspecified => write!(f, "unspecified"),
            BaseType::Bool => write!(f, "bool"),
            BaseType::Uint => write!(f, "uint"),
            BaseType::Int => write!(f, "int"),
            BaseType::FixedSize(integer) => write!(f, "{integer}"),
        }
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

    /// Get an iterator over the variants, but with an extra counter to get the specified discriminant for each.
    ///
    /// *Note:* The validity of this is checked in the [passes::enum_values_checked] pass. If this function is run
    /// before that pass, there might be weird results.
    pub fn iter_variants_with_discriminant(&self) -> impl Iterator<Item = (i128, &EnumVariant)> {
        let mut next_discriminant = 0;
        self.variants
            .iter()
            .map(move |variant| match variant.value {
                EnumValue::Specified(discriminant) => {
                    next_discriminant = discriminant + 1;
                    (discriminant, variant)
                }
                _ => {
                    let discriminant = next_discriminant;
                    next_discriminant += 1;
                    (discriminant, variant)
                }
            })
    }

    /// Get an iterator over the variants, but with an extra counter to get the specified discriminant for each.
    ///
    /// *Note:* The validity of this is checked in the [passes::enum_values_checked] pass. If this function is run
    /// before that pass, there might be weird results.
    pub fn iter_variants_with_discriminant_mut(
        &mut self,
    ) -> impl Iterator<Item = (i128, &mut EnumVariant)> {
        let mut next_discriminant = 0;
        self.variants
            .iter_mut()
            .map(move |variant| match variant.value {
                EnumValue::Specified(discriminant) => {
                    next_discriminant = discriminant + 1;
                    (discriminant, variant)
                }
                _ => {
                    let discriminant = next_discriminant;
                    next_discriminant += 1;
                    (discriminant, variant)
                }
            })
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

    /// Returns `true` if the enum value is [`Unspecified`].
    ///
    /// [`Unspecified`]: EnumValue::Unspecified
    #[must_use]
    pub fn is_unspecified(&self) -> bool {
        matches!(self, Self::Unspecified)
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
    /// If true, this extern can be converted infallibly too
    pub supports_infallible: bool,
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

impl_unique!(Device);
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
            Object::Device(val) => val.id(),
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
