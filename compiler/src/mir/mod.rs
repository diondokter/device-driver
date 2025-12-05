//! The MIR takes its data from HIR and makes sure all required data is there
//! and all optional data is filled in with defaults.

use std::{fmt::Display, ops::Range, rc::Rc};

use convert_case::Boundary;
use miette::SourceSpan;

pub mod lir_transform;
pub mod passes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub root_objects: Vec<Object>,
    pub config: DeviceConfig,
}

impl Manifest {
    pub fn iter_objects_with_config_mut(&mut self) -> ObjectIterMut<'_> {
        ObjectIterMut {
            children: &mut self.root_objects,
            parent: None,
            collection_object_returned: false,
            current_device_config: Rc::new(self.config.clone()),
        }
    }

    pub fn iter_objects(&self) -> impl Iterator<Item = &Object> {
        ObjectIter {
            children: &self.root_objects,
            parent: None,
            collection_object_returned: false,
            current_device_config: Rc::new(self.config.clone()),
        }
        .map(|(object, _)| object)
    }

    pub fn iter_objects_with_config(&self) -> ObjectIter<'_> {
        ObjectIter {
            children: &self.root_objects,
            parent: None,
            collection_object_returned: false,
            current_device_config: Rc::new(self.config.clone()),
        }
    }

    pub fn iter_enums(&self) -> impl Iterator<Item = &'_ Enum> {
        self.iter_objects_with_config().filter_map(|(o, _)| {
            if let Object::Enum(e) = o {
                Some(e)
            } else {
                None
            }
        })
    }

    pub fn iter_enums_with_config(&self) -> impl Iterator<Item = (&'_ Enum, Rc<DeviceConfig>)> {
        self.iter_objects_with_config().filter_map(|(o, config)| {
            if let Object::Enum(e) = o {
                Some((e, config))
            } else {
                None
            }
        })
    }

    pub fn iter_devices_with_config(&self) -> impl Iterator<Item = (&'_ Device, Rc<DeviceConfig>)> {
        self.iter_objects_with_config().filter_map(|(o, config)| {
            if let Object::Device(d) = o {
                Some((d, config))
            } else {
                None
            }
        })
    }
}

#[derive(Default)]
pub struct ObjectIterMut<'a> {
    children: &'a mut [Object],
    parent: Option<Box<ObjectIterMut<'a>>>,
    collection_object_returned: bool,
    current_device_config: Rc<DeviceConfig>,
}

/// A GAT based lending iterator.
/// Can't do anything fancy with it yet though.
pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

impl<'a> LendingIterator for ObjectIterMut<'a> {
    type Item<'b>
        = (&'b mut Object, Rc<DeviceConfig>)
    where
        Self: 'b;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        match self.children.is_empty() {
            true => match self.parent.take() {
                Some(parent) => {
                    // continue with the parent node
                    *self = *parent;
                    self.next()
                }
                None => None,
            },
            false => {
                if self.children[0].child_objects_mut().is_empty() {
                    let (first, rest) = std::mem::take(&mut self.children)
                        .split_first_mut()
                        .expect("Already checked not empty");
                    self.children = rest;
                    Some((first, self.current_device_config.clone()))
                } else if !self.collection_object_returned {
                    self.collection_object_returned = true;

                    let next_device_config =
                        if let Some(new_config) = self.children[0].device_config() {
                            Rc::new(self.current_device_config.override_with(new_config))
                        } else {
                            self.current_device_config.clone()
                        };

                    Some((&mut self.children[0], next_device_config))
                } else {
                    self.collection_object_returned = false;

                    let next_device_config =
                        if let Some(new_config) = self.children[0].device_config() {
                            Rc::new(self.current_device_config.override_with(new_config))
                        } else {
                            self.current_device_config.clone()
                        };

                    let (first, rest) = std::mem::take(&mut self.children)
                        .split_first_mut()
                        .expect("Already checked not empty");
                    self.children = rest;

                    *self = ObjectIterMut {
                        children: first.child_objects_mut(),
                        parent: Some(Box::new(std::mem::take(self))),
                        collection_object_returned: false,
                        current_device_config: next_device_config,
                    };
                    self.next()
                }
            }
        }
    }
}

#[derive(Default)]
pub struct ObjectIter<'a> {
    children: &'a [Object],
    parent: Option<Box<ObjectIter<'a>>>,
    collection_object_returned: bool,
    current_device_config: Rc<DeviceConfig>,
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = (&'a Object, Rc<DeviceConfig>);

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
                    Some((first, self.current_device_config.clone()))
                } else if !self.collection_object_returned {
                    self.collection_object_returned = true;

                    let next_device_config = if let Some(new_config) = first.device_config() {
                        Rc::new(self.current_device_config.override_with(new_config))
                    } else {
                        self.current_device_config.clone()
                    };

                    self.children = children;

                    Some((&children[0], next_device_config))
                } else {
                    self.collection_object_returned = false;

                    let next_device_config = if let Some(new_config) = first.device_config() {
                        Rc::new(self.current_device_config.override_with(new_config))
                    } else {
                        self.current_device_config.clone()
                    };

                    *self = ObjectIter {
                        children: first.child_objects(),
                        parent: Some(Box::new(std::mem::take(self))),
                        collection_object_returned: false,
                        current_device_config: next_device_config,
                    };
                    self.next()
                }
            }
        }
    }
}

/// Implementation meant for testing to easily create a manifest with just one device
impl From<Device> for Manifest {
    fn from(value: Device) -> Self {
        Self {
            root_objects: vec![Object::Device(value)],
            config: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Device {
    pub description: String,
    pub name: Spanned<String>,
    pub device_config: DeviceConfig,
    pub objects: Vec<Object>,
}

impl Device {
    pub fn iter_objects(&self) -> impl Iterator<Item = &Object> {
        ObjectIter {
            children: &self.objects,
            parent: None,
            collection_object_returned: false,
            current_device_config: Rc::new(Default::default()),
        }
        .map(|(object, _)| object)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct DeviceConfig {
    pub register_access: Option<Access>,
    pub field_access: Option<Access>,
    pub buffer_access: Option<Access>,
    pub byte_order: Option<ByteOrder>,
    pub bit_order: Option<BitOrder>,
    pub register_address_type: Option<Integer>,
    pub command_address_type: Option<Integer>,
    pub buffer_address_type: Option<Integer>,
    pub name_word_boundaries: Option<Vec<Boundary>>,
    pub defmt_feature: Option<String>,
}

impl DeviceConfig {
    pub fn override_with(&self, other: &Self) -> DeviceConfig {
        Self {
            register_access: other.register_access.or(self.register_access),
            field_access: other.field_access.or(self.field_access),
            buffer_access: other.buffer_access.or(self.buffer_access),
            byte_order: other.byte_order.or(self.byte_order),
            bit_order: other.bit_order.or(self.bit_order),
            register_address_type: other.register_address_type.or(self.register_address_type),
            command_address_type: other.command_address_type.or(self.command_address_type),
            buffer_address_type: other.buffer_address_type.or(self.buffer_address_type),
            name_word_boundaries: other
                .name_word_boundaries
                .as_ref()
                .or(self.name_word_boundaries.as_ref())
                .cloned(),
            defmt_feature: other
                .defmt_feature
                .as_ref()
                .or(self.defmt_feature.as_ref())
                .cloned(),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::Display, strum::EnumString, Hash,
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
    pub const fn is_signed(&self) -> bool {
        self.min_value() != 0
    }

    pub const fn min_value(&self) -> i128 {
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

    pub const fn max_value(&self) -> i128 {
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

    pub const fn size_bits(&self) -> u32 {
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
    pub const fn find_smallest(min: i128, max: i128, size_bits: u32) -> Option<Integer> {
        Some(match (min, max, size_bits) {
            (0.., ..0x1_00, ..=8) => Integer::U8,
            (0.., ..0x1_0000, ..=16) => Integer::U16,
            (0.., ..0x1_0000_0000, ..=32) => Integer::U32,
            (0.., ..0x1_0000_0000_0000_0000, ..=64) => Integer::U64,
            (-0x80.., ..0x80, ..=8) => Integer::I8,
            (-0x8000.., ..0x8000, ..=16) => Integer::I16,
            (-0x8000_00000.., ..0x8000_0000, ..=32) => Integer::I32,
            (-0x8000_0000_0000_0000.., ..0x8000_0000_0000_0000, ..=64) => Integer::I64,
            _ => return None,
        })
    }

    /// Given the min and the max and the sign of the integer,
    /// how many bits are required to fit the min and max? (inclusive)
    pub const fn bits_required(&self, min: i128, max: i128) -> u32 {
        assert!(max >= min);

        if self.is_signed() {
            let min_bits = if min.is_negative() {
                i128::BITS - (min.abs() - 1).leading_zeros() + 1
            } else {
                0
            };
            let max_bits = if max.is_positive() {
                i128::BITS - max.leading_zeros() + 1
            } else {
                0
            };

            if min_bits > max_bits {
                min_bits
            } else {
                max_bits
            }
        } else {
            assert!(min >= 0);
            i128::BITS - max.leading_zeros()
        }
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
    Hash,
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
    Debug, Clone, Copy, PartialEq, Eq, strum::VariantNames, strum::Display, strum::EnumString, Hash,
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
    Hash,
)]
pub enum BitOrder {
    #[default]
    LSB0,
    MSB0,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    pub(self) fn device_config(&self) -> Option<&DeviceConfig> {
        match self {
            Object::Device(device) => Some(&device.device_config),
            _ => None,
        }
    }

    pub(self) fn child_objects_mut(&mut self) -> &mut [Object] {
        match self {
            Object::Device(device) => &mut device.objects,
            Object::Block(block) => &mut block.objects,
            _ => &mut [],
        }
    }

    pub(self) fn child_objects_vec(&mut self) -> Option<&mut Vec<Object>> {
        match self {
            Object::Device(device) => Some(&mut device.objects),
            Object::Block(block) => Some(&mut block.objects),
            _ => None,
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

    /// Get the span of the name of the object
    pub(self) fn name_span(&self) -> SourceSpan {
        match self {
            Object::Device(val) => val.name.span,
            Object::Block(val) => val.name.span,
            Object::Register(val) => val.name.span,
            Object::Command(val) => val.name.span,
            Object::Buffer(val) => val.name.span,
            Object::FieldSet(val) => val.name.span,
            Object::Enum(val) => val.name.span,
            Object::Extern(val) => val.name.span,
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
    fn repeat(&self) -> Option<&Repeat> {
        match self {
            Object::Device(_) => None,
            Object::Block(block) => block.repeat.as_ref(),
            Object::Register(register) => register.repeat.as_ref(),
            Object::Command(command) => command.repeat.as_ref(),
            Object::Buffer(_) => None,
            Object::FieldSet(_) => None,
            Object::Enum(_) => None,
            Object::Extern(_) => None,
        }
    }

    /// Return the repeat value if it exists
    fn repeat_mut(&mut self) -> Option<&mut Repeat> {
        match self {
            Object::Device(_) => None,
            Object::Block(block) => block.repeat.as_mut(),
            Object::Register(register) => register.repeat.as_mut(),
            Object::Command(command) => command.repeat.as_mut(),
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

    pub fn as_device(&self) -> Option<&Device> {
        if let Self::Device(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(self) fn object_type_name(&self) -> &'static str {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Block {
    pub description: String,
    pub name: Spanned<String>,
    pub address_offset: i128,
    pub repeat: Option<Repeat>,
    pub objects: Vec<Object>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Repeat {
    pub source: RepeatSource,
    pub stride: i128,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RepeatSource {
    Count(u64),
    Enum(Spanned<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Register {
    pub description: String,
    pub name: Spanned<String>,
    pub access: Access,
    pub allow_address_overlap: bool,
    pub address: i128,
    pub reset_value: Option<Spanned<ResetValue>>,
    pub repeat: Option<Repeat>,
    pub field_set_ref: FieldSetRef,
}

/// An externally defined fieldset. This is the name of that fieldset
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct FieldSet {
    pub description: String,
    pub name: Spanned<String>,
    pub size_bits: Spanned<u32>,
    pub byte_order: Option<ByteOrder>,
    pub bit_order: Option<BitOrder>,
    pub allow_bit_overlap: bool,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Field {
    pub description: String,
    pub name: Spanned<String>,
    pub access: Access,
    pub base_type: Spanned<BaseType>,
    pub field_conversion: Option<FieldConversion>,
    pub field_address: Spanned<Range<u32>>,
    pub repeat: Option<Repeat>,
}

impl Field {
    pub fn get_type_specifier_string(&self) -> String {
        match &self.field_conversion {
            Some(fc) => {
                format!(
                    "{}:{}{}",
                    self.base_type,
                    fc.type_name,
                    if fc.use_try { "?" } else { "" }
                )
            }
            None => self.base_type.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldConversion {
    /// The name of the type we're converting to
    pub type_name: Spanned<String>,
    /// True when we want to use the fallible interface (like a Result<type, error>)
    pub use_try: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Enum {
    pub description: String,
    pub name: Spanned<String>,
    pub variants: Vec<EnumVariant>,
    pub base_type: Spanned<BaseType>,
    pub size_bits: Option<u32>,
    generation_style: Option<EnumGenerationStyle>,
}

impl Enum {
    pub fn new(
        description: String,
        name: Spanned<String>,
        variants: Vec<EnumVariant>,
        base_type: Spanned<BaseType>,
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
        name: Spanned<String>,
        variants: Vec<EnumVariant>,
        base_type: Spanned<BaseType>,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct EnumVariant {
    pub description: String,
    pub name: Spanned<String>,
    pub value: EnumValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Command {
    pub description: String,
    pub name: Spanned<String>,
    pub address: i128,
    pub allow_address_overlap: bool,
    pub repeat: Option<Repeat>,

    pub field_set_ref_in: Option<FieldSetRef>,
    pub field_set_ref_out: Option<FieldSetRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Buffer {
    pub description: String,
    pub name: Spanned<String>,
    pub access: Access,
    pub address: i128,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Extern {
    pub description: String,
    pub name: Spanned<String>,
    /// From/into what base type can this extern be converted?
    pub base_type: Spanned<BaseType>,
    /// If true, this extern can be converted infallibly too
    pub supports_infallible: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum UniqueId {
    Object {
        object_name: Spanned<String>,
    },
    Field {
        parent_id: Box<UniqueId>,
        field_name: Spanned<String>,
    },
}

impl UniqueId {
    pub fn span(&self) -> SourceSpan {
        match self {
            UniqueId::Object { object_name } => object_name.span,
            UniqueId::Field { field_name, .. } => field_name.span,
        }
    }

    /// *Only for tests:* Create a new instance with a dummy span.
    #[cfg(test)]
    pub fn new_test(object_name: impl Into<String>) -> Self {
        Self::Object {
            object_name: object_name.into().with_dummy_span(),
        }
    }
}

impl Display for UniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniqueId::Object { object_name } => write!(f, "{}", object_name),
            UniqueId::Field {
                parent_id,
                field_name,
            } => write!(f, "{} {{ {} }}", parent_id, field_name),
        }
    }
}

pub trait Unique {
    type Metadata;

    fn id(&self) -> UniqueId
    where
        Self::Metadata: Empty;
    fn id_with(&self, meta: Self::Metadata) -> UniqueId;

    fn has_id(&self, id: &UniqueId) -> bool
    where
        Self::Metadata: Empty;
    fn has_id_with(&self, meta: Self::Metadata, id: &UniqueId) -> bool {
        self.id_with(meta) == *id
    }
}

pub trait Empty {}
impl Empty for () {}

macro_rules! impl_unique_object {
    ($t:ty) => {
        impl Unique for $t {
            type Metadata = ();

            fn id(&self) -> UniqueId {
                UniqueId::Object {
                    object_name: self.name.clone(),
                }
            }

            fn id_with(&self, _: Self::Metadata) -> UniqueId {
                self.id()
            }

            fn has_id(&self, id: &UniqueId) -> bool {
                match id {
                    UniqueId::Object { object_name } => &self.name == object_name,
                    _ => false,
                }
            }
        }
    };
}

impl_unique_object!(Device);
impl_unique_object!(Register);
impl_unique_object!(Command);
impl_unique_object!(Buffer);
impl_unique_object!(Block);
impl_unique_object!(Enum);
impl_unique_object!(EnumVariant);
impl_unique_object!(FieldSet);
impl_unique_object!(Extern);

impl Unique for Field {
    type Metadata = UniqueId;

    fn id(&self) -> UniqueId {
        unreachable!()
    }

    fn id_with(&self, parent: Self::Metadata) -> UniqueId {
        UniqueId::Field {
            parent_id: Box::new(parent),
            field_name: self.name.clone(),
        }
    }

    fn has_id(&self, _id: &UniqueId) -> bool {
        unreachable!()
    }
}

impl Unique for Object {
    type Metadata = ();

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

    fn id_with(&self, _: Self::Metadata) -> UniqueId {
        self.id()
    }

    fn has_id(&self, id: &UniqueId) -> bool {
        match self {
            Object::Device(val) => val.has_id(id),
            Object::Block(val) => val.has_id(id),
            Object::Register(val) => val.has_id(id),
            Object::Command(val) => val.has_id(id),
            Object::Buffer(val) => val.has_id(id),
            Object::FieldSet(val) => val.has_id(id),
            Object::Enum(val) => val.has_id(id),
            Object::Extern(val) => val.has_id(id),
        }
    }
}

#[derive(Debug, Clone, Eq, Copy)]
pub struct Spanned<T> {
    pub span: SourceSpan,
    pub value: T,
}

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        // Only compare value. The span is transparent
        self.value == other.value
    }
}

impl<T: PartialEq> PartialEq<T> for Spanned<T> {
    fn eq(&self, other: &T) -> bool {
        // Only compare value. The span is transparent
        &self.value == other
    }
}

impl<T: std::hash::Hash> std::hash::Hash for Spanned<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
        // Only hash value. The span is transparent
    }
}

impl<T: Default> Default for Spanned<T> {
    fn default() -> Self {
        Self {
            span: (0, 0).into(),
            value: Default::default(),
        }
    }
}

impl<T: Display> Display for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Spanned<T> {
    pub fn new(span: SourceSpan, value: T) -> Self {
        Self { span, value }
    }
}

impl<T> From<(T, SourceSpan)> for Spanned<T> {
    fn from((value, span): (T, SourceSpan)) -> Self {
        Self { span, value }
    }
}

impl<T: PartialOrd> PartialOrd<T> for Spanned<T> {
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(other)
    }
}

pub trait Span {
    fn with_span(self, span: impl Into<SourceSpan>) -> Spanned<Self>
    where
        Self: Sized,
    {
        Spanned::new(span.into(), self)
    }

    fn with_dummy_span(self) -> Spanned<Self>
    where
        Self: Sized,
    {
        self.with_span((0, 0))
    }
}
impl<T> Span for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter_works() {
        const NAME_ORDER: &[&str] = &["a", "b", "c", "d"];

        let mut manifest = Manifest {
            root_objects: vec![
                Object::Device(Device {
                    description: String::new(),
                    name: "a".to_owned().with_dummy_span(),
                    device_config: DeviceConfig {
                        register_access: Some(Access::RW),
                        ..Default::default()
                    },
                    objects: vec![
                        Object::Extern(Extern {
                            name: "b".to_owned().with_dummy_span(),
                            ..Default::default()
                        }),
                        Object::Extern(Extern {
                            name: "c".to_owned().with_dummy_span(),
                            ..Default::default()
                        }),
                    ],
                }),
                Object::Extern(Extern {
                    name: "d".to_owned().with_dummy_span(),
                    ..Default::default()
                }),
            ],
            config: Default::default(),
        };

        let names: Vec<_> = manifest.iter_objects().map(|o| o.name()).collect();
        assert_eq!(&names, NAME_ORDER);

        let mut names = Vec::new();
        let mut lender = manifest.iter_objects_with_config_mut();
        while let Some((object, _)) = lender.next() {
            names.push(object.name().to_string());
        }
        assert_eq!(&names, NAME_ORDER);
    }

    #[test]
    fn correct_integer_size_bits() {
        assert_eq!(Integer::U8.bits_required(0, 0), 0);
        assert_eq!(Integer::U8.bits_required(0, 1), 1);
        assert_eq!(Integer::U8.bits_required(0, 2), 2);
        assert_eq!(Integer::U8.bits_required(0, 3), 2);
        assert_eq!(Integer::U8.bits_required(0, 4), 3);

        assert_eq!(Integer::I8.bits_required(0, 0), 0);
        assert_eq!(Integer::I8.bits_required(-1, 0), 1);
        assert_eq!(Integer::I8.bits_required(-1, 1), 2);
        assert_eq!(Integer::I8.bits_required(0, 1), 2);
        assert_eq!(Integer::I8.bits_required(-2, 1), 2);
        assert_eq!(Integer::I8.bits_required(0, 2), 3);
        assert_eq!(Integer::I8.bits_required(-128, 0), 8);
        assert_eq!(Integer::I8.bits_required(-129, 0), 9);
        assert_eq!(Integer::I8.bits_required(0, 127), 8);
        assert_eq!(Integer::I8.bits_required(0, 128), 9);
        assert_eq!(Integer::I8.bits_required(-16, 15), 5);
        assert_eq!(Integer::I8.bits_required(-16, 16), 6);
        assert_eq!(Integer::I8.bits_required(-17, 15), 6);
    }
}
