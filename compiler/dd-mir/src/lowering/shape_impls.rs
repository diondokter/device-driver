use std::{borrow::Cow, sync::LazyLock};

use crate::{
    lowering::{LowerResult, PropertyInfo, PropertyName, SetterArgs, Shape, lower_node},
    model::{
        Block, Buffer, Command, Device, Enum, EnumValue, EnumVariant, Extern, Field, FieldSet,
        Manifest, Object, Register,
    },
};
use convert_case::Boundary;
use device_driver_common::{
    identifier::{All, Identifier, IdentifierRef, Operation, Type},
    span::{Span, SpanExt, Spanned},
    specifiers::{
        Access, AddressMode, AddressRange, BaseType, ByteOrder, Integer, NodeType, Repeat,
        ResetValue, TypeConversion,
    },
};
use device_driver_diagnostics::errors::{
    ExternInvalidSizeBits, FieldAddressOutOfRange, FieldAddressWrongOrder, InvalidIdentifier,
    ResetValueNegative, SizeBytesTooLarge,
};
use device_driver_parser::{Expression, Ident, Node};
use itertools::Itertools;

const FIELD_SET_EXAMPLE: Node<'static> = Node {
    doc_comments: Vec::new(),
    node_type: device_driver_parser::Ident::new_no_span("fieldset"),
    name: device_driver_parser::Ident::new_no_span("MyFieldSet"),
    repeat: None,
    type_specifier: None,
    properties: Vec::new(),
    short_properties: Vec::new(),
    sub_nodes: Vec::new(),
    span: Span::empty(),
};

impl Shape for Manifest {
    const NODE_TYPE: NodeType = NodeType::Manifest;
    type NameIdentifierType = All;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Manifest>] = &[
            PropertyInfo {
                name: PropertyName::Exact("byte-order"),
                description: "Sets the global default byte order used by fieldsets. This can be overridden per device and fieldset.",
                allowed_expression_types: Cow::Borrowed(&[Expression::ByteOrder(ByteOrder::LE)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: manifest,
                             property,
                             ..
                         }| {
                    manifest.config.byte_order = Some(property.expression.as_byte_order().unwrap());
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("register-address-type"),
                description: "Sets the global type used to address the registers for all devices. This can be overridden per device.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: manifest,
                             property,
                             ..
                         }| {
                    manifest.config.register_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("command-address-type"),
                description: "Sets the global type used to address the commands for all devices. This can be overridden per device.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: manifest,
                             property,
                             ..
                         }| {
                    manifest.config.command_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("buffer-address-type"),
                description: "Sets the global type used to address the buffers for all devices. This can be overridden per device.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: manifest,
                             property,
                             ..
                         }| {
                    manifest.config.buffer_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("word-boundaries"),
                description: "\
Sets the global word splitting rules for all objects. This can be overridden per device.

This option exists to aid in copying names from the datasheet. Those names are often not proper names for types and operations.
So by setting the rules, the compiler can split identifiers into good proper words and then convert them to the required casing.
The splitting is done with `convert_case` using their [`string representation`](https://docs.rs/convert_case/0.10.0/convert_case/enum.Boundary.html#method.defaults_from) for boundaries.

In short, place a colon (`:`) between every boundary. Then each boundary follows the expressed pattern.
For example `aB` will split words when a lower case letter is followed by an upper case letter.
Some symbols are also allowed as boundary, like `-` & `_`.

If not specified, this uses a reasonable default for splitting.",
                allowed_expression_types: Cow::Borrowed(&[Expression::String("bD:0B:_")]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: manifest,
                             property,
                             ..
                         }| {
                    manifest.config.name_word_boundaries = Some(Boundary::defaults_from(
                        property.expression.as_string().unwrap(),
                    ));
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("register-address-mode"),
                description: "\
Sets the global address mode for registers. This can be overridden per device.

When specified, the registers are assumed to share an address space:
- With the `mapped` option, that address space is a memory-mapped space where if register `A` has address `X` and is `Y` bytes big, then register `B` (if it exists) will have the address `X+Y`.
- With the `indexed` option, that address space has one register per number where if object `A` has address `X`, then object `B` (if it exists) will have the address `X+1`.

If this value is specified, then it permits bulk register reads and writes.",
                allowed_expression_types: Cow::Borrowed(&[Expression::AddressMode(
                    AddressMode::Mapped,
                )]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: manifest,
                             property,
                             ..
                         }| {
                    manifest.config.register_address_mode = Some(
                        property
                            .expression
                            .as_address_mode()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
        ];
        MAP
    }

    fn supported_subnodes() -> Option<&'static [NodeType]> {
        Some(&[
            NodeType::Device,
            NodeType::FieldSet,
            NodeType::Enum,
            NodeType::Extern,
        ])
    }

    fn push_subnode(&mut self, object: Object) {
        self.objects.push(object);
    }

    fn span(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl Shape for Device {
    const NODE_TYPE: NodeType = NodeType::Device;
    type NameIdentifierType = Type;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Device>] = &[
            PropertyInfo {
                name: PropertyName::Exact("byte-order"),
                description: "Sets the default byte order used by fieldsets in this device. This can be overridden per fieldset.",
                allowed_expression_types: Cow::Borrowed(&[Expression::ByteOrder(ByteOrder::LE)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: dev,
                             property,
                             ..
                         }| {
                    dev.device_config.byte_order =
                        Some(property.expression.as_byte_order().unwrap());
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("register-address-type"),
                description: "Sets the type used to address the registers in this device.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: dev,
                             property,
                             ..
                         }| {
                    dev.device_config.register_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("command-address-type"),
                description: "Sets the type used to address the commands in this device.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: dev,
                             property,
                             ..
                         }| {
                    dev.device_config.command_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("buffer-address-type"),
                description: "Sets the type used to address the buffers in this device.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Integer(Integer::I32)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: dev,
                             property,
                             ..
                         }| {
                    dev.device_config.buffer_address_type = Some(
                        property
                            .expression
                            .as_integer()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("word-boundaries"),
                description: "\
Sets the word splitting rules for all objects defined in the device.

This option exists to aid in copying names from the datasheet. Those names are often not proper names for types and operations.
So by setting the rules, the compiler can split identifiers into good proper words and then convert them to the required casing.
The splitting is done with `convert_case` using their [`string representation`](https://docs.rs/convert_case/0.10.0/convert_case/enum.Boundary.html#method.defaults_from) for boundaries.

In short, place a colon (`:`) between every boundary. Then each boundary follows the expressed pattern.
For example `aB` will split words when a lower case letter is followed by an upper case letter.
Some symbols are also allowed as boundary, like `-` & `_`.

If not specified, this uses a reasonable default for splitting.",
                allowed_expression_types: Cow::Borrowed(&[Expression::String("bD:0B:_")]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: dev,
                             property,
                             ..
                         }| {
                    dev.device_config.name_word_boundaries = Some(Boundary::defaults_from(
                        property.expression.as_string().unwrap(),
                    ));
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("register-address-mode"),
                description: "\
Sets the address mode for registers in this device.

When specified, the registers are assumed to share an address space:
- With the `mapped` option, that address space is a memory-mapped space where if register `A` has address `X` and is `Y` bytes big, then register `B` (if it exists) will have the address `X+Y`.
- With the `indexed` option, that address space has one register per number where if object `A` has address `X`, then object `B` (if it exists) will have the address `X+1`.

If this value is specified, then it permits bulk register reads and writes.",
                allowed_expression_types: Cow::Borrowed(&[Expression::AddressMode(
                    AddressMode::Mapped,
                )]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs {
                             target_object: device,
                             property,
                             ..
                         }| {
                    device.device_config.register_address_mode = Some(
                        property
                            .expression
                            .as_address_mode()
                            .unwrap()
                            .with_span(property.expression.span),
                    );
                    false
                },
            },
        ];
        MAP
    }

    fn supported_subnodes() -> Option<&'static [NodeType]> {
        Some(&[
            NodeType::Block,
            NodeType::Register,
            NodeType::Command,
            NodeType::Buffer,
            NodeType::FieldSet,
            NodeType::Enum,
            NodeType::Extern,
        ])
    }

    fn push_subnode(&mut self, object: Object) {
        self.objects.push(object);
    }

    fn span(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl Shape for Block {
    const NODE_TYPE: NodeType = NodeType::Block;
    type NameIdentifierType = All;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Block>] = &[PropertyInfo {
            name: PropertyName::Exact("address-offset"),
            description: "\
Defines the address offset of this block. All objects in the block are relative to the block.
For example, a block with an address offset of 10 which has a register at address 5, will have defined the register at address 15.
If this is not desired, then keep the address offset at 0.",
            allowed_expression_types: Cow::Borrowed(&[Expression::Number(0)]),
            multiple_allowed: false,
            required: true,
            supports_doc_comments: false,
            setter: |SetterArgs {
                         target_object: block,
                         property,
                         ..
                     }| {
                block.address_offset = property
                    .expression
                    .as_number()
                    .unwrap()
                    .with_span(property.expression.span);
                false
            },
        }];
        MAP
    }

    fn supported_subnodes() -> Option<&'static [NodeType]> {
        Some(&[
            NodeType::Block,
            NodeType::Register,
            NodeType::Command,
            NodeType::Buffer,
            NodeType::FieldSet,
            NodeType::Enum,
            NodeType::Extern,
        ])
    }

    fn push_subnode(&mut self, object: Object) {
        self.objects.push(object);
    }

    fn repeat(&mut self) -> Option<&mut Option<Repeat>> {
        Some(&mut self.repeat)
    }

    fn span(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl Shape for Register {
    const NODE_TYPE: NodeType = NodeType::Register;
    type NameIdentifierType = Operation;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: LazyLock<Vec<PropertyInfo<Register>>> = LazyLock::new(|| {
            [
                PropertyInfo {
                    name: PropertyName::Exact("address"),
                    description: "The address of the register.",
                    allowed_expression_types: Cow::Borrowed(&[Expression::Number(0)]),
                    multiple_allowed: false,
                    required: true,
                    supports_doc_comments: false,
                    setter: |SetterArgs::<Register> {
                                 target_object: r,
                                 property,
                                 ..
                             }| {
                        r.address = property
                            .expression
                            .as_number()
                            .unwrap()
                            .with_span(property.expression.span);
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("access"),
                    description: "Limits how the register can be accessed. If not specified, the access is `RW`.",
                    allowed_expression_types: Cow::Borrowed(&[Expression::Access(Access::RW)]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |SetterArgs::<Register> {
                                 target_object: r,
                                 property,
                                 ..
                             }| {
                        r.access = property.expression.as_access().unwrap();
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("address-overlap"),
                    description: "Allows addresses to overlap with other registers. This is not allowed by default to prevent copy-paste mistakes.",
                    allowed_expression_types: Cow::Borrowed(&[Expression::Allow]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |SetterArgs::<Register> {
                                 target_object: r, ..
                             }| {
                        r.allow_address_overlap = true;
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("reset"),
                    description: "\
Defines the reset value of the register. When performing a write operation, this value loaded in by default.

The value can be expressed in two ways:
- Byte array: No byte order changes are done. The array will be loaded into the fieldset as is.
- Integer: Will be converted to a byte array with the specified byte order.",
                    allowed_expression_types: Cow::Owned(vec![
                        Expression::ByteArray(vec![12, 34]),
                        Expression::Number(1234),
                    ]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |SetterArgs::<Register> {
                                 target_object: r,
                                 property,
                                 diagnostics,
                                 ..
                             }| match &property.expression.value {
                        Expression::Number(num) => match u128::try_from(*num) {
                            Ok(num) => {
                                r.reset_value = Some(
                                    ResetValue::Integer(num).with_span(property.expression.span),
                                );
                                false
                            }
                            Err(_) => {
                                diagnostics.add(ResetValueNegative {
                                    reset_value: property.expression.span,
                                });
                                true
                            }
                        },
                        Expression::ByteArray(bytes) => {
                            r.reset_value = Some(
                                ResetValue::Array(bytes.to_vec())
                                    .with_span(property.expression.span),
                            );
                            false
                        }
                        _ => unreachable!(),
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("fields"),
                    description: "The fieldset that represents the data of the register. This can be a reference to an existing fieldset or a completely new inline fieldset.",
                    allowed_expression_types: Cow::Owned(vec![
                        Expression::TypeReference(device_driver_parser::Ident::new_no_span(
                            "MyFieldset",
                        )),
                        Expression::SubNode(Box::new(FIELD_SET_EXAMPLE)),
                    ]),
                    multiple_allowed: false,
                    required: true,
                    supports_doc_comments: false,
                    setter: |SetterArgs::<Register> {
                                 target_object: r,
                                 property,
                                 node,
                                 diagnostics,
                                 sibling_objects,
                             }| {
                        match &property.expression.value {
                            Expression::TypeReference(ident) => {
                                r.field_set_ref =
                                    IdentifierRef::new(ident.val.into()).with_span(ident.span);
                                false
                            }
                            Expression::SubNode(sub_node) => {
                                let result = lower_node(
                                    sub_node,
                                    Some(NodeType::Register.with_span(node.node_type.span)),
                                    Some(Ident::new(r.name.original(), r.name.span)),
                                    &[NodeType::FieldSet],
                                    diagnostics,
                                );

                                match result {
                                    LowerResult::Objects(fs, fs_siblings) => {
                                        r.field_set_ref = fs
                                            .name()
                                            .clone()
                                            // This should always be a fieldset is a Type identifier
                                            .cast_assert()
                                            .take_ref()
                                            .with_span(fs.name_span());
                                        sibling_objects.push(fs);
                                        sibling_objects.extend(fs_siblings);
                                        false
                                    }
                                    LowerResult::Error(fs_siblings) => {
                                        sibling_objects.extend(fs_siblings);
                                        true
                                    }
                                    LowerResult::Manifest(_) => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
                        }
                    },
                },
            ]
            .into()
        });
        &MAP
    }

    fn repeat(&mut self) -> Option<&mut Option<Repeat>> {
        Some(&mut self.repeat)
    }

    fn span(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl Shape for FieldSet {
    const NODE_TYPE: NodeType = NodeType::FieldSet;
    type NameIdentifierType = Type;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<FieldSet>] = &[
            PropertyInfo {
                name: PropertyName::Exact("size-bytes"),
                description: "The size of the fieldset in number of bytes.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Number(8)]),
                multiple_allowed: false,
                required: true,
                supports_doc_comments: false,
                setter: |SetterArgs::<FieldSet> {
                             target_object: fs,
                             property,
                             node: fs_node,
                             diagnostics,
                             ..
                         }| match u32::try_from(
                    property.expression.as_number().unwrap(),
                ) {
                    Ok(size_bytes) if size_bytes <= 0x10_0000 => {
                        fs.size_bytes = size_bytes.with_span(property.expression.span);
                        false
                    }
                    _ => {
                        diagnostics.add(SizeBytesTooLarge {
                            value: property.expression.span,
                            field_set: fs_node.span,
                        });
                        true
                    }
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("byte-order"),
                description: "The byte order of the fieldset data.",
                allowed_expression_types: Cow::Borrowed(&[Expression::ByteOrder(ByteOrder::LE)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs::<FieldSet> {
                             target_object: fs,
                             property,
                             ..
                         }| {
                    fs.byte_order = Some(property.expression.as_byte_order().unwrap());
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("bit-overlap"),
                description: "Allows fields to overlap. This is not allowed by default to prevent copy-paste mistakes.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Allow]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs::<FieldSet> {
                             target_object: fs, ..
                         }| {
                    fs.allow_bit_overlap = true;
                    false
                },
            },
        ];
        MAP
    }

    fn supported_subnodes() -> Option<&'static [NodeType]> {
        Some(&[NodeType::Field])
    }

    fn push_subnode(&mut self, object: Object) {
        let Object::Field(field) = object else {
            unreachable!("{object:?}")
        };
        self.fields.push(field);
    }

    fn span(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl Shape for Extern {
    const NODE_TYPE: NodeType = NodeType::Extern;
    type NameIdentifierType = Type;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Extern>] = &[
            PropertyInfo {
                name: PropertyName::Exact("infallible"),
                description: "Allows this type to be infallably converted to.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Allow]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs::<Extern> {
                             target_object: ext, ..
                         }| {
                    ext.supports_infallible = true;
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("size-bits"),
                description: "The size of the type in bits.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Number(8)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs::<Extern> {
                             target_object: ext,
                             property,
                             diagnostics,
                             node,
                             ..
                         }| match u64::try_from(
                    property.expression.as_number().unwrap(),
                ) {
                    Ok(size_bits) => {
                        ext.size_bits = Some(size_bits.with_span(property.expression.span));
                        false
                    }
                    _ => {
                        diagnostics.add(ExternInvalidSizeBits {
                            extern_name: node.name.span,
                            size_bits: property.expression.span,
                            reason: "value must be in the range 0..2^64".into(),
                        });
                        true
                    }
                },
            },
        ];
        MAP
    }

    fn base_type(&mut self) -> Option<&mut Spanned<BaseType>> {
        Some(&mut self.base_type)
    }

    fn span(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl Shape for Buffer {
    const NODE_TYPE: NodeType = NodeType::Buffer;
    type NameIdentifierType = Operation;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Buffer>] = &[
            PropertyInfo {
                name: PropertyName::Exact("access"),
                description: "Limits how the buffer can be accessed. If not specified, the access is `RW`.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Access(Access::RW)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs::<Buffer> {
                             target_object: buf,
                             property,
                             ..
                         }| {
                    buf.access = property.expression.as_access().unwrap();
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Exact("address"),
                description: "The address of the buffer",
                allowed_expression_types: Cow::Borrowed(&[Expression::Number(0)]),
                multiple_allowed: false,
                required: true,
                supports_doc_comments: false,
                setter: |SetterArgs::<Buffer> {
                             target_object: buf,
                             property,
                             ..
                         }| {
                    buf.address = property
                        .expression
                        .as_number()
                        .unwrap()
                        .with_span(property.expression.span);
                    false
                },
            },
        ];
        MAP
    }

    fn span(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl Shape for Enum {
    const NODE_TYPE: NodeType = NodeType::Enum;
    type NameIdentifierType = Type;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Enum>] = &[PropertyInfo {
            name: PropertyName::Any,
            description: "Defines a variant for the enum. The name of the property becomes the variant name.",
            allowed_expression_types: Cow::Borrowed(&[
                Expression::Auto,
                Expression::Number(0),
                Expression::DefaultNumber(Some(0)),
                Expression::DefaultNumber(None),
                Expression::CatchAllNumber(Some(0)),
                Expression::CatchAllNumber(None),
            ]),
            multiple_allowed: true,
            required: false,
            supports_doc_comments: true,
            setter: |SetterArgs::<Enum> {
                         target_object: enum_value,
                         property,
                         diagnostics,
                         ..
                     }| {
                let identifier = match Identifier::try_parse(property.name.val) {
                    Ok(identifier) => identifier,
                    Err(e) => {
                        diagnostics.add(InvalidIdentifier {
                            error: e,
                            identifier: property.name.span,
                        });
                        return true;
                    }
                };

                enum_value.variants.push(EnumVariant {
                    description: property.doc_comments.iter().map(|c| c.value).join("\n"),
                    name: identifier.with_span(property.name.span),
                    value: match &property.expression.value {
                        Expression::Number(num) => EnumValue::Specified(*num),
                        Expression::DefaultNumber(Some(num)) => EnumValue::Default(*num),
                        Expression::DefaultNumber(None) => EnumValue::UnspecifiedDefault,
                        Expression::CatchAllNumber(Some(num)) => EnumValue::CatchAll(*num),
                        Expression::CatchAllNumber(None) => EnumValue::UnspecifiedCatchAll,
                        Expression::Auto => EnumValue::Unspecified,
                        _ => unreachable!(),
                    },
                    span: property.span,
                });
                false
            },
        }];
        MAP
    }

    fn base_type(&mut self) -> Option<&mut Spanned<BaseType>> {
        Some(&mut self.base_type)
    }

    fn span(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl Shape for Command {
    const NODE_TYPE: NodeType = NodeType::Command;
    type NameIdentifierType = Operation;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: LazyLock<Vec<PropertyInfo<Command>>> = LazyLock::new(|| {
            [
                PropertyInfo {
                    name: PropertyName::Exact("address"),
                    description: "The address of the command",
                    allowed_expression_types: Cow::Borrowed(&[Expression::Number(0)]),
                    multiple_allowed: false,
                    required: true,
                    supports_doc_comments: false,
                    setter: |SetterArgs::<Command> {
                                 target_object: command,
                                 property,
                                 ..
                             }| {
                        command.address = property
                            .expression
                            .as_number()
                            .unwrap()
                            .with_span(property.expression.span);
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("address-overlap"),
                    description: "Allows addresses to overlap with other commands. This is not allowed by default to prevent copy-paste mistakes.",
                    allowed_expression_types: Cow::Borrowed(&[Expression::Allow]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |SetterArgs::<Command> {
                                 target_object: command,
                                 ..
                             }| {
                        command.allow_address_overlap = true;
                        false
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("fields-in"),
                    description: "The fieldset that represents the input data of the command. This can be a reference to an existing fieldset or a completely new inline fieldset.",
                    allowed_expression_types: Cow::Owned(vec![
                        Expression::TypeReference(device_driver_parser::Ident::new_no_span(
                            "MyFieldset",
                        )),
                        Expression::SubNode(Box::new(FIELD_SET_EXAMPLE)),
                    ]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |SetterArgs::<Command> {
                                 target_object: command,
                                 property,
                                 node,
                                 diagnostics,
                                 sibling_objects,
                             }| {
                        match &property.expression.value {
                            Expression::TypeReference(ident) => {
                                command.field_set_ref_in = Some(
                                    IdentifierRef::new(ident.val.into()).with_span(ident.span),
                                );
                                false
                            }
                            Expression::SubNode(sub_node) => {
                                let result = lower_node(
                                    sub_node,
                                    Some(NodeType::Register.with_span(node.node_type.span)),
                                    Some(Ident::new(command.name.original(), command.name.span)),
                                    &[NodeType::FieldSet],
                                    diagnostics,
                                );

                                match result {
                                    LowerResult::Objects(fs, fs_siblings) => {
                                        command.field_set_ref_in = Some(
                                            fs.name()
                                                .clone()
                                                // Always a fieldset, so should be fine
                                                .cast_assert()
                                                .take_ref()
                                                .with_span(fs.name_span()),
                                        );
                                        sibling_objects.push(fs);
                                        sibling_objects.extend(fs_siblings);
                                        false
                                    }
                                    LowerResult::Error(fs_siblings) => {
                                        sibling_objects.extend(fs_siblings);
                                        true
                                    }
                                    LowerResult::Manifest(_) => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
                        }
                    },
                },
                PropertyInfo {
                    name: PropertyName::Exact("fields-out"),
                    description: "The fieldset that represents the output data of the command. This can be a reference to an existing fieldset or a completely new inline fieldset.",
                    allowed_expression_types: Cow::Owned(vec![
                        Expression::TypeReference(device_driver_parser::Ident::new_no_span(
                            "MyFieldset",
                        )),
                        Expression::SubNode(Box::new(FIELD_SET_EXAMPLE)),
                    ]),
                    multiple_allowed: false,
                    required: false,
                    supports_doc_comments: false,
                    setter: |SetterArgs::<Command> {
                                 target_object: command,
                                 property,
                                 node,
                                 diagnostics,
                                 sibling_objects,
                             }| {
                        match &property.expression.value {
                            Expression::TypeReference(ident) => {
                                command.field_set_ref_out = Some(
                                    IdentifierRef::new(ident.val.into()).with_span(ident.span),
                                );
                                false
                            }
                            Expression::SubNode(sub_node) => {
                                let result = lower_node(
                                    sub_node,
                                    Some(NodeType::Register.with_span(node.node_type.span)),
                                    Some(Ident::new(command.name.original(), command.name.span)),
                                    &[NodeType::FieldSet],
                                    diagnostics,
                                );

                                match result {
                                    LowerResult::Objects(fs, fs_siblings) => {
                                        command.field_set_ref_out = Some(
                                            fs.name()
                                                .clone()
                                                // Always a fieldset, so should be fine
                                                .cast_assert()
                                                .take_ref()
                                                .with_span(fs.name_span()),
                                        );
                                        sibling_objects.push(fs);
                                        sibling_objects.extend(fs_siblings);
                                        false
                                    }
                                    LowerResult::Error(fs_siblings) => {
                                        sibling_objects.extend(fs_siblings);
                                        true
                                    }
                                    LowerResult::Manifest(_) => unreachable!(),
                                }
                            }
                            _ => unreachable!(),
                        }
                    },
                },
            ]
            .into()
        });
        &MAP
    }

    fn repeat(&mut self) -> Option<&mut Option<Repeat>> {
        Some(&mut self.repeat)
    }

    fn span(&mut self) -> &mut Span {
        &mut self.span
    }
}

impl Shape for Field {
    const NODE_TYPE: NodeType = NodeType::Field;
    type NameIdentifierType = All;

    fn doc_comments(&mut self) -> &mut String {
        &mut self.description
    }

    fn name(&mut self) -> &mut Spanned<Identifier<Self::NameIdentifierType>> {
        &mut self.name
    }

    fn supported_properties() -> &'static [PropertyInfo<Self>] {
        static MAP: &[PropertyInfo<Field>] = &[
            PropertyInfo {
                name: PropertyName::Short("address"),
                description: "The bit address of the field within the fieldset",
                allowed_expression_types: Cow::Borrowed(&[
                    Expression::AddressRange { end: 8, start: 0 },
                    Expression::Number(0),
                ]),
                multiple_allowed: false,
                required: true,
                supports_doc_comments: false,
                setter: |SetterArgs::<Field> {
                             target_object: field,
                             property,
                             diagnostics,
                             ..
                         }| {
                    let u32_range = 0..=u32::MAX as i128;

                    field.field_address = match property.expression.value {
                        Expression::AddressRange { end, start }
                            if u32_range.contains(&end) && u32_range.contains(&start) =>
                        {
                            if end < start {
                                diagnostics.add(FieldAddressWrongOrder {
                                    address: property.expression.span,
                                    end,
                                    start,
                                });
                                return true;
                            }

                            AddressRange {
                                start: start.try_into().unwrap(),
                                end: end.try_into().unwrap(),
                            }
                        }
                        Expression::Number(num) if u32_range.contains(&num) => AddressRange {
                            start: num.try_into().unwrap(),
                            end: num.try_into().unwrap(),
                        },
                        Expression::AddressRange { .. } | Expression::Number(_) => {
                            diagnostics.add(FieldAddressOutOfRange {
                                field_address: property.expression.span,
                            });
                            return true;
                        }
                        _ => unreachable!(),
                    }
                    .with_span(property.expression.span);
                    false
                },
            },
            PropertyInfo {
                name: PropertyName::Short("access"),
                description: "Limits how the field can be accessed. If not specified, the access is `RW`.",
                allowed_expression_types: Cow::Borrowed(&[Expression::Access(Access::RW)]),
                multiple_allowed: false,
                required: false,
                supports_doc_comments: false,
                setter: |SetterArgs::<Field> {
                             target_object: field,
                             property,
                             ..
                         }| {
                    field.access = property.expression.as_access().unwrap();
                    false
                },
            },
        ];
        MAP
    }

    fn base_type(&mut self) -> Option<&mut Spanned<BaseType>> {
        Some(&mut self.base_type)
    }

    fn conversion_type(&mut self) -> Option<&mut Option<TypeConversion>> {
        Some(&mut self.field_conversion)
    }

    fn repeat(&mut self) -> Option<&mut Option<Repeat>> {
        Some(&mut self.repeat)
    }

    fn span(&mut self) -> &mut Span {
        &mut self.span
    }
}
