//! The MIR takes its data from HIR and makes sure all required data is there
//! and all optional data is filled in with defaults.

use std::{fmt::Display, ops::Range};

use convert_case::Boundary;

pub mod kdl_transform;
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
            name_word_boundaries: convert_case::Boundary::defaults(),
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
    I8,
    I16,
    I32,
    I64,
}

impl Integer {
    pub fn min_value(&self) -> i64 {
        match self {
            Integer::U8 => u8::MIN as i64,
            Integer::U16 => u16::MIN as i64,
            Integer::U32 => u32::MIN as i64,
            Integer::I8 => i8::MIN as i64,
            Integer::I16 => i16::MIN as i64,
            Integer::I32 => i32::MIN as i64,
            Integer::I64 => i64::MIN,
        }
    }

    pub fn max_value(&self) -> i64 {
        match self {
            Integer::U8 => u8::MAX as i64,
            Integer::U16 => u16::MAX as i64,
            Integer::U32 => u32::MAX as i64,
            Integer::I8 => i8::MAX as i64,
            Integer::I16 => i16::MAX as i64,
            Integer::I32 => i32::MAX as i64,
            Integer::I64 => i64::MAX,
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
    Ref(RefObject),
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
            Object::Ref(val) => &mut val.name,
        }
    }

    /// Get a reference to the name of the specific object
    pub(self) fn name(&self) -> &str {
        match self {
            Object::Block(val) => &val.name,
            Object::Register(val) => &val.name,
            Object::Command(val) => &val.name,
            Object::Buffer(val) => &val.name,
            Object::Ref(val) => &val.name,
        }
    }

    /// Get a reference to the description of the specific object
    pub(self) fn description(&self) -> &str {
        match self {
            Object::Block(val) => &val.description,
            Object::Register(val) => &val.description,
            Object::Command(val) => &val.description,
            Object::Buffer(val) => &val.description,
            Object::Ref(val) => &val.description,
        }
    }

    /// Get a reference to the cfg of the specific object
    pub(self) fn cfg_attr_mut(&mut self) -> &mut Cfg {
        match self {
            Object::Block(val) => &mut val.cfg_attr,
            Object::Register(val) => &mut val.cfg_attr,
            Object::Command(val) => &mut val.cfg_attr,
            Object::Buffer(val) => &mut val.cfg_attr,
            Object::Ref(val) => &mut val.cfg_attr,
        }
    }

    /// Get an iterator over all the field sets in the object
    pub(self) fn field_sets_mut(&mut self) -> impl Iterator<Item = &mut [Field]> {
        match self {
            Object::Register(val) => vec![val.fields.as_mut_slice()].into_iter(),
            Object::Command(val) => {
                vec![val.in_fields.as_mut_slice(), val.out_fields.as_mut_slice()].into_iter()
            }
            Object::Block(_) | Object::Buffer(_) | Object::Ref(_) => Vec::new().into_iter(),
        }
    }

    /// Get an iterator over all the field sets in the object
    pub(self) fn field_sets(&self) -> impl Iterator<Item = &[Field]> {
        match self {
            Object::Register(val) => vec![val.fields.as_slice()].into_iter(),
            Object::Command(val) => {
                vec![val.in_fields.as_slice(), val.out_fields.as_slice()].into_iter()
            }
            Object::Block(_) | Object::Buffer(_) | Object::Ref(_) => Vec::new().into_iter(),
        }
    }

    pub fn as_block_mut(&mut self) -> Option<&mut Block> {
        if let Self::Block(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_register_mut(&mut self) -> Option<&mut Register> {
        if let Self::Register(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_register(&self) -> Option<&Register> {
        if let Self::Register(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_command_mut(&mut self) -> Option<&mut Command> {
        if let Self::Command(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Return the address if it is specified.
    /// It's only not specified in ref objects where the user hasn't overridden the address
    fn address(&self) -> Option<i64> {
        match self {
            Object::Block(block) => Some(block.address_offset),
            Object::Register(register) => Some(register.address),
            Object::Command(command) => Some(command.address),
            Object::Buffer(buffer) => Some(buffer.address),
            Object::Ref(ref_object) => match &ref_object.object_override {
                ObjectOverride::Block(block_override) => block_override.address_offset,
                ObjectOverride::Register(register_override) => register_override.address,
                ObjectOverride::Command(command_override) => command_override.address,
            },
        }
    }

    /// Return the repeat value if it exists
    fn repeat(&self) -> Option<Repeat> {
        match self {
            Object::Block(block) => block.repeat,
            Object::Register(register) => register.repeat,
            Object::Command(command) => command.repeat,
            Object::Buffer(_) => None,
            Object::Ref(ref_object) => match &ref_object.object_override {
                ObjectOverride::Block(block_override) => block_override.repeat,
                ObjectOverride::Register(register_override) => register_override.repeat,
                ObjectOverride::Command(command_override) => command_override.repeat,
            },
        }
    }

    pub fn as_ref_object_mut(&mut self) -> Option<&mut RefObject> {
        if let Self::Ref(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Block {
    pub cfg_attr: Cfg,
    pub description: String,
    pub name: String,
    pub address_offset: i64,
    pub repeat: Option<Repeat>,
    pub objects: Vec<Object>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Repeat {
    pub count: u64,
    pub stride: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Register {
    pub cfg_attr: Cfg,
    pub description: String,
    pub name: String,
    pub access: Access,
    pub byte_order: Option<ByteOrder>,
    pub bit_order: BitOrder,
    pub allow_bit_overlap: bool,
    pub allow_address_overlap: bool,
    pub address: i64,
    pub size_bits: u32,
    pub reset_value: Option<ResetValue>,
    pub repeat: Option<Repeat>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Field {
    pub cfg_attr: Cfg,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldConversion {
    Direct { type_name: String, use_try: bool },
    Enum { enum_value: Enum, use_try: bool },
}

impl FieldConversion {
    pub const fn use_try(&self) -> bool {
        match self {
            FieldConversion::Direct { use_try, .. } => *use_try,
            FieldConversion::Enum { use_try, .. } => *use_try,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            FieldConversion::Direct { type_name, .. } => type_name,
            FieldConversion::Enum { enum_value, .. } => &enum_value.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Enum {
    pub cfg_attr: Cfg,
    pub description: String,
    pub name: String,
    pub variants: Vec<EnumVariant>,
    generation_style: Option<EnumGenerationStyle>,
}

impl Enum {
    pub fn new(description: String, name: String, variants: Vec<EnumVariant>) -> Self {
        Self {
            cfg_attr: Cfg::default(),
            description,
            name,
            variants,
            generation_style: None,
        }
    }

    #[cfg(test)]
    pub fn new_with_style(
        description: String,
        name: String,
        variants: Vec<EnumVariant>,
        generation_style: EnumGenerationStyle,
    ) -> Self {
        Self {
            cfg_attr: Cfg::default(),
            description,
            name,
            variants,
            generation_style: Some(generation_style),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumGenerationStyle {
    Fallible,
    Infallible { bit_size: u32 },
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
    pub cfg_attr: Cfg,
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
    pub cfg_attr: Cfg,
    pub description: String,
    pub name: String,
    pub address: i64,
    pub byte_order: Option<ByteOrder>,
    pub bit_order: BitOrder,
    pub allow_bit_overlap: bool,
    pub allow_address_overlap: bool,
    pub size_bits_in: u32,
    pub size_bits_out: u32,
    pub repeat: Option<Repeat>,
    pub in_fields: Vec<Field>,
    pub out_fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Buffer {
    pub cfg_attr: Cfg,
    pub description: String,
    pub name: String,
    pub access: Access,
    pub address: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RefObject {
    pub cfg_attr: Cfg,
    pub description: String,
    pub name: String,
    pub object_override: ObjectOverride,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectOverride {
    Block(BlockOverride),
    Register(RegisterOverride),
    Command(CommandOverride),
}

impl Default for ObjectOverride {
    fn default() -> Self {
        Self::Register(Default::default())
    }
}

impl ObjectOverride {
    fn name(&self) -> &str {
        match self {
            ObjectOverride::Block(v) => &v.name,
            ObjectOverride::Register(v) => &v.name,
            ObjectOverride::Command(v) => &v.name,
        }
    }

    fn name_mut(&mut self) -> &mut String {
        match self {
            ObjectOverride::Block(v) => &mut v.name,
            ObjectOverride::Register(v) => &mut v.name,
            ObjectOverride::Command(v) => &mut v.name,
        }
    }

    pub fn as_register(&self) -> Option<&RegisterOverride> {
        if let Self::Register(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_register_mut(&mut self) -> Option<&mut RegisterOverride> {
        if let Self::Register(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockOverride {
    pub name: String,
    pub address_offset: Option<i64>,
    pub repeat: Option<Repeat>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegisterOverride {
    pub name: String,
    pub access: Option<Access>,
    pub address: Option<i64>,
    pub allow_address_overlap: bool,
    pub reset_value: Option<ResetValue>,
    pub repeat: Option<Repeat>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandOverride {
    pub name: String,
    pub address: Option<i64>,
    pub allow_address_overlap: bool,
    pub repeat: Option<Repeat>,
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

#[derive(Debug, Clone, Eq, PartialEq, Default, Hash)]
pub struct Cfg {
    value: Option<String>,
}

impl Cfg {
    pub fn new(value: Option<&str>) -> Self {
        Self {
            value: value.map(|v| v.into()),
        }
    }

    #[must_use]
    pub fn combine(&self, other: &Self) -> Self {
        match (&self.value, &other.value) {
            (None, None) => Self { value: None },
            (None, Some(val)) => Self {
                value: Some(val.clone()),
            },
            (Some(val), None) => Self {
                value: Some(val.clone()),
            },
            (Some(val1), Some(val2)) if val1 == val2 => Self {
                value: Some(val1.clone()),
            },
            (Some(val1), Some(val2)) => Self {
                value: Some(format!("all({val1}, {val2})")),
            },
        }
    }

    pub fn inner(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

impl Display for Cfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = self.inner() {
            write!(f, "#[cfg({value})]")?
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct UniqueId {
    object_name: String,
    object_cfg: Cfg,
}

impl Display for UniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.object_cfg.inner() {
            Some(cfg) => write!(f, "{}(cfg=`{}`)", self.object_name, cfg),
            None => write!(f, "{}", self.object_name),
        }
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
                    object_cfg: self.cfg_attr.clone(),
                }
            }
        }
    };
}

impl_unique!(Register);
impl_unique!(Command);
impl_unique!(Buffer);
impl_unique!(RefObject);
impl_unique!(Block);
impl_unique!(Enum);
impl_unique!(EnumVariant);

impl Unique for Object {
    fn id(&self) -> UniqueId {
        match self {
            Object::Block(val) => val.id(),
            Object::Register(val) => val.id(),
            Object::Command(val) => val.id(),
            Object::Buffer(val) => val.id(),
            Object::Ref(val) => val.id(),
        }
    }
}
