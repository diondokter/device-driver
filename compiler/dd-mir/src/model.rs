use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    marker::PhantomData,
    ops::{Index, IndexMut, Not},
};

use convert_case::Boundary;
use device_driver_common::{
    bitset::BitSet,
    identifier::{
        Global, Identifier, IdentifierRef, Local, Namespace, Operation, RuntimeNamespace, Type,
    },
    interner::Istr,
    span::{Span, Spanned},
    specifiers::{
        Access, AddressMode, AddressRange, BaseType, ByteOrder, Integer, NodeType, Repeat,
        ResetValue,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectType {
    Device,
    Block,
    Register,
    Command,
    Buffer,
    FieldSet,
    Field,
    Enum,
    EnumVariant,
    Extern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectIdConversionError {
    source: ObjectType,
    target: ObjectType,
}
impl Display for ObjectIdConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot convert from an ObjectId with type {:?} to {:?}",
            self.source, self.target
        )
    }
}
impl Error for ObjectIdConversionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(ObjectType, u32);

impl ObjectId {
    pub fn object_type(self) -> ObjectType {
        self.0
    }

    pub fn get(self, manifest: &Manifest) -> Option<Object<'_>> {
        manifest.object(self)
    }

    pub fn get_mut(self, manifest: &mut Manifest) -> Option<ObjectMut<'_>> {
        manifest.object_mut(self)
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self(ObjectType::Device, u32::MAX)
    }
}

pub trait Id: Copy {
    fn index(&self) -> usize;
    fn create(val: usize) -> Self;
}

macro_rules! create_id {
    ($name:ident, $object_type:ident, $manifest_collection:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            pub fn get(self, manifest: &Manifest) -> Option<&$object_type> {
                manifest.$manifest_collection.get(self)
            }

            pub fn get_mut(self, manifest: &mut Manifest) -> Option<&mut $object_type> {
                manifest.$manifest_collection.get_mut(self)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(u32::MAX)
            }
        }

        impl From<$name> for ObjectId {
            fn from(value: $name) -> Self {
                Self(ObjectType::$object_type, value.0)
            }
        }

        impl TryFrom<ObjectId> for $name {
            type Error = ObjectIdConversionError;
            fn try_from(value: ObjectId) -> Result<Self, Self::Error> {
                if value.0 == ObjectType::$object_type {
                    Ok(Self(value.1))
                } else {
                    Err(ObjectIdConversionError {
                        source: value.0,
                        target: ObjectType::$object_type,
                    })
                }
            }
        }

        impl Id for $name {
            fn index(&self) -> usize {
                self.0 as usize
            }

            fn create(val: usize) -> Self {
                debug_assert!(u32::try_from(val).is_ok());
                Self(val as u32)
            }
        }
    };
}

create_id!(DeviceId, Device, devices);
create_id!(BlockId, Block, blocks);
create_id!(RegisterId, Register, registers);
create_id!(CommandId, Command, commands);
create_id!(BufferId, Buffer, buffers);
create_id!(FieldSetId, FieldSet, fieldsets);
create_id!(FieldId, Field, fields);
create_id!(EnumId, Enum, enums);
create_id!(EnumVariantId, EnumVariant, enum_variants);
create_id!(ExternId, Extern, externs);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectArena<O, ID: Id> {
    inner: Vec<O>,
    removed: BitSet,
    _phantom: PhantomData<ID>,
}

impl<O: Default, ID: Id> Default for ObjectArena<O, ID> {
    fn default() -> Self {
        Self::new()
    }
}

impl<O, ID: Id> ObjectArena<O, ID> {
    pub const fn new() -> Self {
        Self {
            inner: Vec::new(),
            removed: BitSet::new(),
            _phantom: PhantomData,
        }
    }

    pub fn get(&self, id: ID) -> Option<&O> {
        if !self.removed.get(id.index())? {
            self.inner.get(id.index())
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: ID) -> Option<&mut O> {
        if !self.removed.get(id.index())? {
            self.inner.get_mut(id.index())
        } else {
            None
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &O> {
        self.inner
            .iter()
            .enumerate()
            .filter_map(|(i, obj)| self.removed.get(i)?.not().then_some(obj))
    }

    pub fn iter_enumerated(&self) -> impl Iterator<Item = (ID, &O)> {
        self.inner
            .iter()
            .enumerate()
            .filter_map(|(i, obj)| self.removed.get(i)?.not().then_some((ID::create(i), obj)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut O> {
        self.inner
            .iter_mut()
            .enumerate()
            .filter_map(|(i, obj)| self.removed.get(i)?.not().then_some(obj))
    }

    pub fn iter_enumerated_mut(&mut self) -> impl Iterator<Item = (ID, &mut O)> {
        self.inner
            .iter_mut()
            .enumerate()
            .filter_map(|(i, obj)| self.removed.get(i)?.not().then_some((ID::create(i), obj)))
    }

    pub fn ids(&self) -> impl Iterator<Item = ID> {
        (0..self.inner.len()).filter_map(|i| self.removed.get(i)?.not().then_some(ID::create(i)))
    }

    pub fn push(&mut self, obj: O) -> ID {
        let new_id = ID::create(self.inner.len());
        self.inner.push(obj);
        self.removed.push(false);
        new_id
    }

    pub fn remove(&mut self, id: ID) {
        self.removed.set(id.index(), true);
    }
}

impl<O, ID: Id> Index<ID> for ObjectArena<O, ID> {
    type Output = O;

    fn index(&self, index: ID) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<O, ID: Id> IndexMut<ID> for ObjectArena<O, ID> {
    fn index_mut(&mut self, index: ID) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Manifest {
    pub description: Istr,
    pub name: Spanned<Identifier<Global>>,
    pub default_access: Option<Access>,
    pub config: DeviceConfig,

    pub devices: ObjectArena<Device, DeviceId>,
    pub blocks: ObjectArena<Block, BlockId>,
    pub registers: ObjectArena<Register, RegisterId>,
    pub commands: ObjectArena<Command, CommandId>,
    pub buffers: ObjectArena<Buffer, BufferId>,
    pub fieldsets: ObjectArena<FieldSet, FieldSetId>,
    pub fields: ObjectArena<Field, FieldId>,
    pub enums: ObjectArena<Enum, EnumId>,
    pub enum_variants: ObjectArena<EnumVariant, EnumVariantId>,
    pub externs: ObjectArena<Extern, ExternId>,

    pub parent_map: HashMap<ObjectId, Box<[ObjectId]>>,

    pub short_properties_span: Span,
    pub properties_span: Option<Span>,
    pub span: Span,
}

impl Manifest {
    pub fn objects(&self) -> impl Iterator<Item = Object<'_>> {
        self.devices
            .iter()
            .map(Object::Device)
            .chain(self.blocks.iter().map(Object::Block))
            .chain(self.registers.iter().map(Object::Register))
            .chain(self.commands.iter().map(Object::Command))
            .chain(self.buffers.iter().map(Object::Buffer))
            .chain(self.fieldsets.iter().map(Object::FieldSet))
            .chain(self.fields.iter().map(Object::Field))
            .chain(self.enums.iter().map(Object::Enum))
            .chain(self.enum_variants.iter().map(Object::EnumVariant))
            .chain(self.externs.iter().map(Object::Extern))
    }

    pub fn objects_enumerated(&self) -> impl Iterator<Item = (ObjectId, Object<'_>)> {
        self.devices
            .iter_enumerated()
            .map(|(id, obj)| (id.into(), Object::Device(obj)))
            .chain(
                self.blocks
                    .iter_enumerated()
                    .map(|(id, obj)| (id.into(), Object::Block(obj))),
            )
            .chain(
                self.registers
                    .iter_enumerated()
                    .map(|(id, obj)| (id.into(), Object::Register(obj))),
            )
            .chain(
                self.commands
                    .iter_enumerated()
                    .map(|(id, obj)| (id.into(), Object::Command(obj))),
            )
            .chain(
                self.buffers
                    .iter_enumerated()
                    .map(|(id, obj)| (id.into(), Object::Buffer(obj))),
            )
            .chain(
                self.fieldsets
                    .iter_enumerated()
                    .map(|(id, obj)| (id.into(), Object::FieldSet(obj))),
            )
            .chain(
                self.fields
                    .iter_enumerated()
                    .map(|(id, obj)| (id.into(), Object::Field(obj))),
            )
            .chain(
                self.enums
                    .iter_enumerated()
                    .map(|(id, obj)| (id.into(), Object::Enum(obj))),
            )
            .chain(
                self.enum_variants
                    .iter_enumerated()
                    .map(|(id, obj)| (id.into(), Object::EnumVariant(obj))),
            )
            .chain(
                self.externs
                    .iter_enumerated()
                    .map(|(id, obj)| (id.into(), Object::Extern(obj))),
            )
    }

    pub fn objects_mut(&mut self) -> impl Iterator<Item = ObjectMut<'_>> {
        self.devices
            .iter_mut()
            .map(ObjectMut::Device)
            .chain(self.blocks.iter_mut().map(ObjectMut::Block))
            .chain(self.registers.iter_mut().map(ObjectMut::Register))
            .chain(self.commands.iter_mut().map(ObjectMut::Command))
            .chain(self.buffers.iter_mut().map(ObjectMut::Buffer))
            .chain(self.fieldsets.iter_mut().map(ObjectMut::FieldSet))
            .chain(self.fields.iter_mut().map(ObjectMut::Field))
            .chain(self.enums.iter_mut().map(ObjectMut::Enum))
            .chain(self.enum_variants.iter_mut().map(ObjectMut::EnumVariant))
            .chain(self.externs.iter_mut().map(ObjectMut::Extern))
    }

    pub fn objects_enumerated_mut(&mut self) -> impl Iterator<Item = (ObjectId, ObjectMut<'_>)> {
        self.devices
            .iter_enumerated_mut()
            .map(|(id, obj)| (id.into(), ObjectMut::Device(obj)))
            .chain(
                self.blocks
                    .iter_enumerated_mut()
                    .map(|(id, obj)| (id.into(), ObjectMut::Block(obj))),
            )
            .chain(
                self.registers
                    .iter_enumerated_mut()
                    .map(|(id, obj)| (id.into(), ObjectMut::Register(obj))),
            )
            .chain(
                self.commands
                    .iter_enumerated_mut()
                    .map(|(id, obj)| (id.into(), ObjectMut::Command(obj))),
            )
            .chain(
                self.buffers
                    .iter_enumerated_mut()
                    .map(|(id, obj)| (id.into(), ObjectMut::Buffer(obj))),
            )
            .chain(
                self.fieldsets
                    .iter_enumerated_mut()
                    .map(|(id, obj)| (id.into(), ObjectMut::FieldSet(obj))),
            )
            .chain(
                self.fields
                    .iter_enumerated_mut()
                    .map(|(id, obj)| (id.into(), ObjectMut::Field(obj))),
            )
            .chain(
                self.enums
                    .iter_enumerated_mut()
                    .map(|(id, obj)| (id.into(), ObjectMut::Enum(obj))),
            )
            .chain(
                self.enum_variants
                    .iter_enumerated_mut()
                    .map(|(id, obj)| (id.into(), ObjectMut::EnumVariant(obj))),
            )
            .chain(
                self.externs
                    .iter_enumerated_mut()
                    .map(|(id, obj)| (id.into(), ObjectMut::Extern(obj))),
            )
    }

    pub fn object_ids(&self) -> impl Iterator<Item = ObjectId> {
        self.devices
            .ids()
            .map(ObjectId::from)
            .chain(self.blocks.ids().map(ObjectId::from))
            .chain(self.registers.ids().map(ObjectId::from))
            .chain(self.commands.ids().map(ObjectId::from))
            .chain(self.buffers.ids().map(ObjectId::from))
            .chain(self.fieldsets.ids().map(ObjectId::from))
            .chain(self.fields.ids().map(ObjectId::from))
            .chain(self.enums.ids().map(ObjectId::from))
            .chain(self.enum_variants.ids().map(ObjectId::from))
            .chain(self.externs.ids().map(ObjectId::from))
    }

    pub fn object(&self, id: impl Into<ObjectId>) -> Option<Object<'_>> {
        let id = id.into();
        match id.0 {
            ObjectType::Device => self.devices.get(DeviceId(id.1)).map(Object::Device),
            ObjectType::Block => self.blocks.get(BlockId(id.1)).map(Object::Block),
            ObjectType::Register => self.registers.get(RegisterId(id.1)).map(Object::Register),
            ObjectType::Command => self.commands.get(CommandId(id.1)).map(Object::Command),
            ObjectType::Buffer => self.buffers.get(BufferId(id.1)).map(Object::Buffer),
            ObjectType::FieldSet => self.fieldsets.get(FieldSetId(id.1)).map(Object::FieldSet),
            ObjectType::Field => self.fields.get(FieldId(id.1)).map(Object::Field),
            ObjectType::Enum => self.enums.get(EnumId(id.1)).map(Object::Enum),
            ObjectType::EnumVariant => self
                .enum_variants
                .get(EnumVariantId(id.1))
                .map(Object::EnumVariant),
            ObjectType::Extern => self.externs.get(ExternId(id.1)).map(Object::Extern),
        }
    }

    pub fn object_mut(&mut self, id: impl Into<ObjectId>) -> Option<ObjectMut<'_>> {
        let id = id.into();
        match id.0 {
            ObjectType::Device => self.devices.get_mut(DeviceId(id.1)).map(ObjectMut::Device),
            ObjectType::Block => self.blocks.get_mut(BlockId(id.1)).map(ObjectMut::Block),
            ObjectType::Register => self
                .registers
                .get_mut(RegisterId(id.1))
                .map(ObjectMut::Register),
            ObjectType::Command => self
                .commands
                .get_mut(CommandId(id.1))
                .map(ObjectMut::Command),
            ObjectType::Buffer => self.buffers.get_mut(BufferId(id.1)).map(ObjectMut::Buffer),
            ObjectType::FieldSet => self
                .fieldsets
                .get_mut(FieldSetId(id.1))
                .map(ObjectMut::FieldSet),
            ObjectType::Field => self.fields.get_mut(FieldId(id.1)).map(ObjectMut::Field),
            ObjectType::Enum => self.enums.get_mut(EnumId(id.1)).map(ObjectMut::Enum),
            ObjectType::EnumVariant => self
                .enum_variants
                .get_mut(EnumVariantId(id.1))
                .map(ObjectMut::EnumVariant),
            ObjectType::Extern => self.externs.get_mut(ExternId(id.1)).map(ObjectMut::Extern),
        }
    }

    /// This assumes [crate::passes::Assumption::NamesUnique]
    pub fn search_object<T: Namespace>(&self, name: &IdentifierRef<T>) -> Option<Object<'_>> {
        self.objects().find(|o| name.is_ref_to(o.name()))
    }

    /// This assumes [crate::passes::Assumption::NamesUnique]
    pub fn search_fieldset(&self, ref_val: &FieldsetRef) -> Option<&FieldSet> {
        match ref_val {
            FieldsetRef::Identifier(identifier_ref) => self
                .fieldsets
                .iter()
                .find(|fs| identifier_ref.is_ref_to(&fs.name.value)),
            FieldsetRef::Id(id) => self.fieldsets.get(*id),
        }
    }

    /// This assumes [crate::passes::Assumption::NamesUnique]
    pub fn search_type(&self, ref_val: &TypeRef) -> Option<Object<'_>> {
        match ref_val {
            TypeRef::Identifier(identifier_ref) => self.search_object(identifier_ref),
            TypeRef::Id(id) => self.object(*id),
        }
    }

    pub fn remove_object(&mut self, id: impl Into<ObjectId>) {
        let id = id.into();

        fn remove(manifest: &mut Manifest, id: ObjectId) {
            match id.0 {
                ObjectType::Device => manifest.devices.remove(DeviceId(id.1)),
                ObjectType::Block => manifest.blocks.remove(BlockId(id.1)),
                ObjectType::Register => manifest.registers.remove(RegisterId(id.1)),
                ObjectType::Command => manifest.commands.remove(CommandId(id.1)),
                ObjectType::Buffer => manifest.buffers.remove(BufferId(id.1)),
                ObjectType::FieldSet => manifest.fieldsets.remove(FieldSetId(id.1)),
                ObjectType::Field => manifest.fields.remove(FieldId(id.1)),
                ObjectType::Enum => manifest.enums.remove(EnumId(id.1)),
                ObjectType::EnumVariant => manifest.enum_variants.remove(EnumVariantId(id.1)),
                ObjectType::Extern => manifest.externs.remove(ExternId(id.1)),
            }
        }

        remove(self, id);

        // Remove the children
        let mut children = Vec::new();
        for object_id in self.object_ids() {
            let parents = self.object_parents(object_id);
            if parents.contains(&id) {
                children.push(object_id);
            }
        }
        for child in children {
            remove(self, child);
        }

        // Remove from parent
        if let Some(parent) = self.object_parents(id).last() {
            self.object_mut(*parent).unwrap().remove_child(id);
        }
    }

    pub(crate) fn populate_parent_map(&mut self) {
        fn populate(
            parent_map: &mut HashMap<ObjectId, Box<[ObjectId]>>,
            manifest: &Manifest,
            parent_list: Vec<ObjectId>,
            children: impl Iterator<Item = ObjectId>,
        ) {
            for child in children {
                parent_map.insert(child, parent_list.clone().into());

                let child_object = manifest.object(child).unwrap();
                if let Some(grand_children) = child_object.child_objects() {
                    let mut grand_parent_list = parent_list.clone();
                    grand_parent_list.push(child);
                    populate(parent_map, manifest, grand_parent_list, grand_children);
                }
            }
        }

        let mut parent_map: HashMap<ObjectId, Box<[ObjectId]>> = Default::default();

        // We know all devices are children of the manifest and nothing else
        for root_object_id in self
            .object_ids()
            .filter(|object_id| self.parent_map.contains_key(object_id))
        {
            let object = self.object(root_object_id).unwrap();

            if let Some(children) = object.child_objects() {
                let parent_list = vec![root_object_id];
                populate(&mut parent_map, self, parent_list, children);
            }
        }

        self.parent_map = parent_map;
    }

    /// Get the parent list for this object.
    /// The last parent is the most direct parent.
    ///
    /// It is possible one of the parents has been removed.
    pub fn object_parents(&self, object: impl Into<ObjectId>) -> &[ObjectId] {
        self.parent_map
            .get(&object.into())
            .map(|parent_list| &parent_list[..])
            .unwrap_or(&[])
    }

    /// Get the device config that applies to the given object
    pub fn object_config(&self, object: impl Into<ObjectId>) -> &DeviceConfig {
        let parents = self.object_parents(object);

        for object_id in parents.iter().rev() {
            if let Some(object) = self.object(*object_id)
                && let Some(config) = object.device_config()
            {
                return config;
            }
        }

        &self.config
    }

    /// Get the device config that applies to the given object
    pub fn object_default_access(&self, object: impl Into<ObjectId>) -> Option<Access> {
        let parents = self.object_parents(object);

        for object_id in parents.iter().rev() {
            if let Some(object) = self.object(*object_id) {
                match object {
                    Object::Device(val) => {
                        if val.default_access.is_some() {
                            return val.default_access;
                        }
                    }
                    Object::Block(val) => {
                        if val.default_access.is_some() {
                            return val.default_access;
                        }
                    }
                    Object::FieldSet(val) => {
                        if val.default_access.is_some() {
                            return val.default_access;
                        }
                    }
                    Object::Register(_) => {}
                    Object::Command(_) => {}
                    Object::Buffer(_) => {}
                    Object::Enum(_) => {}
                    Object::Extern(_) => {}
                    Object::Field(_) => {}
                    Object::EnumVariant(_) => {}
                }
            }
        }

        self.default_access
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Device {
    pub description: Istr,
    pub name: Spanned<Identifier<Type>>,
    pub default_access: Option<Access>,
    pub address_offset: Spanned<i128>,
    pub device_config: DeviceConfig,
    pub children: Vec<ObjectId>,

    pub short_properties_span: Span,
    pub properties_span: Option<Span>,
    /// Span of the whole object
    pub span: Span,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeviceConfig {
    /// The id of the device that owns this config. If None, then this is a manifest config
    pub owner: Option<DeviceId>,
    pub byte_order: Option<ByteOrder>,
    pub register_address_type: Option<Spanned<Integer>>,
    pub command_address_type: Option<Spanned<Integer>>,
    pub buffer_address_type: Option<Spanned<Integer>>,
    pub name_word_boundaries: Option<Vec<Boundary>>,
    pub register_address_mode: Option<Spanned<AddressMode>>,
}

impl DeviceConfig {
    #[must_use]
    pub fn override_with(&self, other: &Self) -> DeviceConfig {
        Self {
            owner: other.owner.or(self.owner),
            byte_order: other.byte_order.or(self.byte_order),
            register_address_type: other.register_address_type.or(self.register_address_type),
            command_address_type: other.command_address_type.or(self.command_address_type),
            buffer_address_type: other.buffer_address_type.or(self.buffer_address_type),
            name_word_boundaries: other
                .name_word_boundaries
                .as_ref()
                .or(self.name_word_boundaries.as_ref())
                .cloned(),
            register_address_mode: other.register_address_mode.or(self.register_address_mode),
        }
    }

    pub fn name_word_boundaries_or_defaults(&self) -> Vec<Boundary> {
        self.name_word_boundaries
            .as_deref()
            .unwrap_or(&const { convert_case::Boundary::defaults() })
            .to_vec()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Object<'a> {
    Device(&'a Device),
    Block(&'a Block),
    Register(&'a Register),
    Command(&'a Command),
    Buffer(&'a Buffer),
    FieldSet(&'a FieldSet),
    Enum(&'a Enum),
    Extern(&'a Extern),
    Field(&'a Field),
    EnumVariant(&'a EnumVariant),
}

#[derive(Debug, PartialEq)]
pub enum ObjectMut<'a> {
    Device(&'a mut Device),
    Block(&'a mut Block),
    Register(&'a mut Register),
    Command(&'a mut Command),
    Buffer(&'a mut Buffer),
    FieldSet(&'a mut FieldSet),
    Enum(&'a mut Enum),
    Extern(&'a mut Extern),
    Field(&'a mut Field),
    EnumVariant(&'a mut EnumVariant),
}

impl<'a> ObjectMut<'a> {
    pub fn as_ref(&self) -> Object<'_> {
        match self {
            ObjectMut::Device(device) => Object::Device(device),
            ObjectMut::Block(block) => Object::Block(block),
            ObjectMut::Register(register) => Object::Register(register),
            ObjectMut::Command(command) => Object::Command(command),
            ObjectMut::Buffer(buffer) => Object::Buffer(buffer),
            ObjectMut::FieldSet(field_set) => Object::FieldSet(field_set),
            ObjectMut::Enum(e) => Object::Enum(e),
            ObjectMut::Extern(e) => Object::Extern(e),
            ObjectMut::Field(field) => Object::Field(field),
            ObjectMut::EnumVariant(field) => Object::EnumVariant(field),
        }
    }

    /// Get a mutable reference to the name of the specific object
    pub fn name_mut(&mut self) -> &mut Identifier<RuntimeNamespace> {
        match self {
            ObjectMut::Device(val) => val.name.as_runtime_namespace_mut(),
            ObjectMut::Block(val) => val.name.as_runtime_namespace_mut(),
            ObjectMut::Register(val) => val.name.as_runtime_namespace_mut(),
            ObjectMut::Command(val) => val.name.as_runtime_namespace_mut(),
            ObjectMut::Buffer(val) => val.name.as_runtime_namespace_mut(),
            ObjectMut::FieldSet(val) => val.name.as_runtime_namespace_mut(),
            ObjectMut::Enum(val) => val.name.as_runtime_namespace_mut(),
            ObjectMut::Extern(val) => val.name.as_runtime_namespace_mut(),
            ObjectMut::Field(val) => val.name.as_runtime_namespace_mut(),
            ObjectMut::EnumVariant(val) => val.name.as_runtime_namespace_mut(),
        }
    }

    /// Return the repeat value if it exists
    pub fn repeat_mut(&mut self) -> Option<&mut Repeat> {
        match self {
            ObjectMut::Device(_) => None,
            ObjectMut::Block(block) => block.repeat.as_mut(),
            ObjectMut::Register(register) => register.repeat.as_mut(),
            ObjectMut::Command(command) => command.repeat.as_mut(),
            ObjectMut::Buffer(_) => None,
            ObjectMut::FieldSet(_) => None,
            ObjectMut::Enum(_) => None,
            ObjectMut::Extern(_) => None,
            ObjectMut::Field(field) => field.repeat.as_mut(),
            ObjectMut::EnumVariant(_) => None,
        }
    }

    pub fn as_field_set_mut(&mut self) -> Option<&mut FieldSet> {
        if let Self::FieldSet(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_enum_mut(&mut self) -> Option<&mut Enum> {
        if let Self::Enum(v) = self {
            Some(v)
        } else {
            None
        }
    }

    fn remove_child(&mut self, id: ObjectId) {
        match self {
            ObjectMut::Device(device) => {
                if let Some(pos) = device.children.iter().position(|child| *child == id) {
                    device.children.remove(pos);
                }
            }
            ObjectMut::Block(block) => {
                if let Some(pos) = block.children.iter().position(|child| *child == id) {
                    block.children.remove(pos);
                }
            }
            ObjectMut::Register(_) => {}
            ObjectMut::Command(_) => {}
            ObjectMut::Buffer(_) => {}
            ObjectMut::FieldSet(field_set) => {
                if let Some(pos) = field_set
                    .fields
                    .iter()
                    .position(|child| ObjectId::from(*child) == id)
                {
                    field_set.fields.remove(pos);
                }
            }
            ObjectMut::Enum(enum_value) => {
                if let Some(pos) = enum_value
                    .variants
                    .iter()
                    .position(|child| ObjectId::from(*child) == id)
                {
                    enum_value.variants.remove(pos);
                }
            }
            ObjectMut::Extern(_) => {}
            ObjectMut::Field(_) => {}
            ObjectMut::EnumVariant(_) => {}
        }
    }
}

impl<'a> Object<'a> {
    pub fn device_config(&self) -> Option<&'a DeviceConfig> {
        match self {
            Object::Device(device) => Some(&device.device_config),
            _ => None,
        }
    }

    pub fn child_objects(&self) -> Option<Box<dyn Iterator<Item = ObjectId> + 'a>> {
        match self {
            Object::Device(device) => Some(Box::new(device.children.iter().copied())),
            Object::Block(block) => Some(Box::new(block.children.iter().copied())),
            Object::FieldSet(fs) => Some(Box::new(fs.fields.iter().copied().map(ObjectId::from))),
            Object::Enum(e) => Some(Box::new(e.variants.iter().copied().map(ObjectId::from))),
            _ => None,
        }
    }

    /// Get a reference to the name of the specific object
    pub fn name(&self) -> &'a Identifier<RuntimeNamespace> {
        match self {
            Object::Device(val) => val.name.as_runtime_namespace(),
            Object::Block(val) => val.name.as_runtime_namespace(),
            Object::Register(val) => val.name.as_runtime_namespace(),
            Object::Command(val) => val.name.as_runtime_namespace(),
            Object::Buffer(val) => val.name.as_runtime_namespace(),
            Object::FieldSet(val) => val.name.as_runtime_namespace(),
            Object::Enum(val) => val.name.as_runtime_namespace(),
            Object::Extern(val) => val.name.as_runtime_namespace(),
            Object::Field(val) => val.name.as_runtime_namespace(),
            Object::EnumVariant(val) => val.name.as_runtime_namespace(),
        }
    }

    /// Get the span of the name of the object
    pub fn name_span(&self) -> Span {
        match self {
            Object::Device(val) => val.name.span,
            Object::Block(val) => val.name.span,
            Object::Register(val) => val.name.span,
            Object::Command(val) => val.name.span,
            Object::Buffer(val) => val.name.span,
            Object::FieldSet(val) => val.name.span,
            Object::Enum(val) => val.name.span,
            Object::Extern(val) => val.name.span,
            Object::Field(val) => val.name.span,
            Object::EnumVariant(val) => val.name.span,
        }
    }

    /// Return the address if it is specified.
    pub fn address(&self) -> Option<Spanned<i128>> {
        match self {
            Object::Device(device) => Some(device.address_offset),
            Object::Block(block) => Some(block.address_offset),
            Object::Register(register) => Some(register.address),
            Object::Command(command) => Some(command.address),
            Object::Buffer(buffer) => Some(buffer.address),
            Object::FieldSet(_) => None,
            Object::Enum(_) => None,
            Object::Extern(_) => None,
            Object::Field(_) => None,
            Object::EnumVariant(_) => None,
        }
    }

    /// Return the repeat value if it exists
    pub fn repeat(&self) -> Option<&'a Repeat> {
        match self {
            Object::Device(_) => None,
            Object::Block(block) => block.repeat.as_ref(),
            Object::Register(register) => register.repeat.as_ref(),
            Object::Command(command) => command.repeat.as_ref(),
            Object::Buffer(_) => None,
            Object::FieldSet(_) => None,
            Object::Enum(_) => None,
            Object::Extern(_) => None,
            Object::Field(field) => field.repeat.as_ref(),
            Object::EnumVariant(_) => None,
        }
    }

    /// Return the type conversion value if it exists
    pub fn type_conversion(&self) -> Option<&'a TypeConversion> {
        match self {
            Object::Device(_) => None,
            Object::Block(_) => None,
            Object::Register(_) => None,
            Object::Command(_) => None,
            Object::Buffer(_) => None,
            Object::FieldSet(_) => None,
            Object::Enum(_) => None,
            Object::Extern(_) => None,
            Object::Field(field) => field.field_conversion.as_ref(),
            Object::EnumVariant(_) => None,
        }
    }

    pub fn as_field_set(&self) -> Option<&'a FieldSet> {
        if let Self::FieldSet(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_enum(&self) -> Option<&'a Enum> {
        if let Self::Enum(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn base_type(&self) -> Option<&'a Spanned<BaseType>> {
        match self {
            Object::Device(_) => None,
            Object::Block(_) => None,
            Object::Register(_) => None,
            Object::Command(_) => None,
            Object::Buffer(_) => None,
            Object::FieldSet(_) => None,
            Object::Enum(enum_value) => Some(&enum_value.base_type),
            Object::Extern(extern_value) => Some(&extern_value.base_type),
            Object::Field(field) => Some(&field.base_type),
            Object::EnumVariant(_) => None,
        }
    }

    pub fn allow_address_overlap(&self) -> bool {
        match self {
            Object::Device(_) => false,
            Object::Block(_) => false,
            Object::Register(register) => register.allow_address_overlap,
            Object::Command(command) => command.allow_address_overlap,
            Object::Buffer(_) => false,
            Object::FieldSet(_) => false,
            Object::Enum(_) => false,
            Object::Extern(_) => false,
            Object::Field(_) => false,
            Object::EnumVariant(_) => false,
        }
    }

    /// The span of the entire object
    pub fn span(&self) -> Span {
        match self {
            Object::Device(val) => val.span,
            Object::Block(val) => val.span,
            Object::Register(val) => val.span,
            Object::Command(val) => val.span,
            Object::Buffer(val) => val.span,
            Object::FieldSet(val) => val.span,
            Object::Enum(val) => val.span,
            Object::Extern(val) => val.span,
            Object::Field(val) => val.span,
            Object::EnumVariant(val) => val.span,
        }
    }

    pub fn short_properties_span(&self) -> Span {
        match self {
            Object::Device(device) => device.short_properties_span,
            Object::Block(block) => block.short_properties_span,
            Object::Register(register) => register.short_properties_span,
            Object::Command(command) => command.short_properties_span,
            Object::Buffer(buffer) => buffer.short_properties_span,
            Object::FieldSet(field_set) => field_set.short_properties_span,
            Object::Enum(enum_value) => enum_value.short_properties_span,
            Object::Extern(extern_value) => extern_value.short_properties_span,
            Object::Field(field) => field.short_properties_span,
            Object::EnumVariant(_) => Span::empty(),
        }
    }

    pub fn node_type(&self) -> NodeType {
        match self {
            Object::Device(_) => NodeType::Device,
            Object::Block(_) => NodeType::Block,
            Object::Register(_) => NodeType::Register,
            Object::Command(_) => NodeType::Command,
            Object::Buffer(_) => NodeType::Buffer,
            Object::FieldSet(_) => NodeType::FieldSet,
            Object::Enum(_) => NodeType::Enum,
            Object::Extern(_) => NodeType::Extern,
            Object::Field(_) => NodeType::Field,
            Object::EnumVariant(_) => panic!("Object has no valid node type"),
        }
    }

    /// Get the fieldset refs of the object. Only returns non-zero for registers and commands
    pub fn fieldset_refs(&self) -> Vec<Spanned<FieldsetRef>> {
        match self {
            Object::Device(_) => Vec::new(),
            Object::Block(_) => Vec::new(),
            Object::Register(r) => vec![r.field_set_ref.clone()],
            Object::Command(c) => [c.field_set_ref_in.clone(), c.field_set_ref_out.clone()]
                .into_iter()
                .flatten()
                .collect(),
            Object::Buffer(_) => Vec::new(),
            Object::FieldSet(_) => Vec::new(),
            Object::Enum(_) => Vec::new(),
            Object::Extern(_) => Vec::new(),
            Object::Field(_) => Vec::new(),
            Object::EnumVariant(_) => Vec::new(),
        }
    }

    pub fn properties_span(&self) -> Option<Span> {
        match self {
            Object::Device(val) => val.properties_span,
            Object::Block(val) => val.properties_span,
            Object::Register(val) => val.properties_span,
            Object::Command(val) => val.properties_span,
            Object::Buffer(val) => val.properties_span,
            Object::FieldSet(val) => val.properties_span,
            Object::Enum(val) => val.properties_span,
            Object::Extern(val) => val.properties_span,
            Object::Field(val) => val.properties_span,
            Object::EnumVariant(_) => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Block {
    pub description: Istr,
    pub name: Spanned<Identifier<Global>>,
    pub address_offset: Spanned<i128>,
    pub repeat: Option<Repeat>,
    pub children: Vec<ObjectId>,
    pub default_access: Option<Access>,

    pub short_properties_span: Span,
    pub properties_span: Option<Span>,
    /// Span of the whole object
    pub span: Span,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Register {
    pub description: Istr,
    pub name: Spanned<Identifier<Operation>>,
    pub access: Option<Access>,
    pub allow_address_overlap: bool,
    pub address: Spanned<i128>,
    pub reset_value: Option<Spanned<ResetValue>>,
    pub repeat: Option<Repeat>,
    pub field_set_ref: Spanned<FieldsetRef>,

    pub short_properties_span: Span,
    pub properties_span: Option<Span>,
    /// Span of the whole object
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldsetRef {
    Identifier(IdentifierRef<Type>),
    Id(FieldSetId),
}

impl Default for FieldsetRef {
    fn default() -> Self {
        Self::Id(FieldSetId::default())
    }
}

impl Display for FieldsetRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldsetRef::Identifier(identifier_ref) => write!(f, "{}", identifier_ref.original()),
            FieldsetRef::Id(field_set_id) => write!(f, "{field_set_id:?}"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FieldSet {
    pub description: Istr,
    pub name: Spanned<Identifier<Type>>,
    pub size_bytes: Spanned<u32>,
    pub byte_order: Option<ByteOrder>,
    pub allow_bit_overlap: bool,
    pub default_access: Option<Access>,
    pub fields: Vec<FieldId>,

    pub short_properties_span: Span,
    pub properties_span: Option<Span>,
    /// Span of the whole object
    pub span: Span,
}

impl FieldSet {
    pub fn size_bits(&self) -> u32 {
        self.size_bytes.value * 8
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeConversion {
    /// The name of the type we're converting to
    pub type_ref: Spanned<TypeRef>,
    /// True when we want to use the fallible interface (like a Result<type, error>)
    pub fallible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    Identifier(IdentifierRef<Type>),
    Id(ObjectId),
}

impl Default for TypeRef {
    fn default() -> Self {
        Self::Id(ObjectId::default())
    }
}

impl Display for TypeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeRef::Identifier(identifier_ref) => write!(f, "{}", identifier_ref.original()),
            TypeRef::Id(field_set_id) => write!(f, "{field_set_id:?}"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Field {
    pub description: Istr,
    pub name: Spanned<Identifier<Local>>,
    pub access: Option<Access>,
    pub base_type: Spanned<BaseType>,
    pub field_conversion: Option<TypeConversion>,
    pub field_address: Spanned<AddressRange>,
    pub repeat: Option<Repeat>,

    pub short_properties_span: Span,
    pub properties_span: Option<Span>,
    /// Span of the whole object
    pub span: Span,
}

impl Field {
    #[must_use]
    pub fn get_type_specifier_string(&self, manifest: &Manifest) -> String {
        match &self.field_conversion {
            Some(fc) => {
                format!(
                    "{}:{}{}",
                    self.base_type,
                    match &fc.type_ref.value {
                        TypeRef::Identifier(identifier_ref) => identifier_ref.original(),
                        TypeRef::Id(object_id) =>
                            manifest.object(*object_id).unwrap().name().original(),
                    },
                    if fc.fallible { "?" } else { "" }
                )
            }
            None => self.base_type.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Enum {
    pub description: Istr,
    pub name: Spanned<Identifier<Type>>,
    pub variants: Vec<EnumVariantId>,
    pub base_type: Spanned<BaseType>,
    pub size_bits: Option<u32>,
    pub generation_style: Option<EnumGenerationStyle>,

    pub short_properties_span: Span,
    pub properties_span: Option<Span>,
    /// Span of the whole object
    pub span: Span,
}

impl Enum {
    #[cfg(test)]
    pub fn new(
        description: Istr,
        name: Spanned<Identifier<Type>>,
        variants: Vec<EnumVariantId>,
        base_type: Spanned<BaseType>,
        size_bits: Option<u32>,
        span: Span,
    ) -> Self {
        Self {
            description,
            name,
            variants,
            base_type,
            size_bits,
            generation_style: None,
            short_properties_span: Span::empty(),
            properties_span: None,
            span,
        }
    }

    #[cfg(test)]
    pub fn new_with_style(
        description: Istr,
        name: Spanned<Identifier<Type>>,
        variants: Vec<EnumVariantId>,
        base_type: Spanned<BaseType>,
        size_bits: Option<u32>,
        generation_style: EnumGenerationStyle,
        span: Span,
    ) -> Self {
        Self {
            description,
            name,
            variants,
            base_type,
            size_bits,
            generation_style: Some(generation_style),
            short_properties_span: Span::empty(),
            properties_span: None,
            span,
        }
    }

    /// Get an iterator over the variants, but with an extra counter to get the specified discriminant for each.
    ///
    /// *Note:* The validity of this is checked in the [`passes::enum_values_checked`] pass. If this function is run
    /// before that pass, there might be weird results.
    pub fn iter_variants_with_discriminant(
        &self,
        enum_variants: &ObjectArena<EnumVariant, EnumVariantId>,
    ) -> impl Iterator<Item = (i128, EnumVariantId)> {
        let mut next_discriminant = 0;
        self.variants.iter().map(move |variant| {
            if let Some(discriminant) = enum_variants[*variant].value.specified_discriminant() {
                next_discriminant = discriminant + 1;
                (discriminant, *variant)
            } else {
                let discriminant = next_discriminant;
                next_discriminant += 1;
                (discriminant, *variant)
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

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnumVariant {
    pub description: Istr,
    pub name: Spanned<Identifier<Local>>,
    pub value: EnumValue,
    /// Span of the whole object
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub enum EnumValue {
    #[default]
    Unspecified,
    Specified(i128),
    Default(i128),
    UnspecifiedDefault,
    CatchAll(i128),
    UnspecifiedCatchAll,
}

impl EnumValue {
    #[must_use]
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default(_) | Self::UnspecifiedDefault)
    }

    #[must_use]
    pub fn is_catch_all(&self) -> bool {
        matches!(self, Self::CatchAll(_) | Self::UnspecifiedCatchAll)
    }

    pub fn specified_discriminant(&self) -> Option<i128> {
        match self {
            Self::Unspecified | Self::UnspecifiedDefault | Self::UnspecifiedCatchAll => None,
            Self::Specified(val) | EnumValue::Default(val) | EnumValue::CatchAll(val) => Some(*val),
        }
    }

    pub fn specify(&mut self, num: i128) {
        *self = match self {
            EnumValue::Unspecified | EnumValue::Specified(_) => Self::Specified(num),
            EnumValue::Default(_) | EnumValue::UnspecifiedDefault => Self::Default(num),
            EnumValue::CatchAll(_) | EnumValue::UnspecifiedCatchAll => Self::CatchAll(num),
        };
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Command {
    pub description: Istr,
    pub name: Spanned<Identifier<Operation>>,
    pub address: Spanned<i128>,
    pub allow_address_overlap: bool,
    pub repeat: Option<Repeat>,

    pub field_set_ref_in: Option<Spanned<FieldsetRef>>,
    pub field_set_ref_out: Option<Spanned<FieldsetRef>>,

    pub short_properties_span: Span,
    pub properties_span: Option<Span>,
    /// Span of the whole object
    pub span: Span,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Buffer {
    pub description: Istr,
    pub name: Spanned<Identifier<Operation>>,
    pub access: Option<Access>,
    pub address: Spanned<i128>,

    pub short_properties_span: Span,
    pub properties_span: Option<Span>,
    /// Span of the whole object
    pub span: Span,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Extern {
    pub description: Istr,
    pub name: Spanned<Identifier<Type>>,
    /// From/into what base type can this extern be converted?
    pub base_type: Spanned<BaseType>,
    /// If true, this extern can be converted infallibly too
    pub supports_infallible: bool,
    /// The user-specified size of the max value of the base type that should be expected
    pub size_bits: Option<Spanned<u64>>,

    pub short_properties_span: Span,
    pub properties_span: Option<Span>,
    /// Span of the whole object
    pub span: Span,
}
