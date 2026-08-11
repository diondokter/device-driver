use std::num::NonZeroU32;

use crate::DeviceDriverV1Format;
use dd_v1_convert_case::Boundary;
use device_driver_common::{
    span::{Span, SpanExt},
    specifiers::{Access, BaseType, ByteOrder, Integer},
};
use device_driver_diagnostics::{DynError, ResultExt};
use device_driver_generation::mir::{
    Access as V1Access, BaseType as V1BaseType, BitOrder, Block, Buffer, ByteOrder as V1ByteOrder,
    Command, Enum, Field, FieldConversion, Integer as V1Integer, Object, ObjectOverride, RefObject,
    Register, Repeat as V1Repeat,
};
use device_driver_parser::{
    Expression, Ident, Node, Property, Repeat, RepeatSource, TypeConversion, TypeSpecifier,
};
use itertools::Itertools;

pub fn convert(source: &str, sub_format: DeviceDriverV1Format) -> Result<String, DynError> {
    let device_mir = match sub_format {
        DeviceDriverV1Format::DSL => device_driver_generation::_private_transform_dsl_mir(
            source
                .parse()
                .map_err(DynError::new)
                .with_message(|| "parsing source into tokenstream")?,
        )
        .into_dyn_result(),
        DeviceDriverV1Format::YAML => {
            device_driver_generation::_private_transform_yaml_mir(source).map_err(DynError::new)
        }
        DeviceDriverV1Format::JSON => {
            device_driver_generation::_private_transform_json_mir(source).map_err(DynError::new)
        }
        DeviceDriverV1Format::TOML => {
            device_driver_generation::_private_transform_toml_mir(source).map_err(DynError::new)
        }
    }
    .with_message(|| "transforming source into v1 mir")?;

    let ddsl_root = Node {
        doc_comments: Vec::new(),
        node_type: Ident::new_no_span("device"),
        name: Ident::new_no_span(device_mir.name.as_deref().unwrap_or("ConvertedDevice")),
        repeat: None,
        type_specifier: None,
        short_properties: Vec::new(),
        properties: [
            device_mir
                .global_config
                .default_byte_order
                .map(|val| Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("byte-order"),
                    expression: Expression::ByteOrder(convert_byte_order(val)).with_dummy_span(),
                }),
            device_mir
                .global_config
                .register_address_type
                .map(|val| Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("register-address-type"),
                    expression: Expression::Integer(convert_integer(val)).with_dummy_span(),
                }),
            device_mir
                .global_config
                .command_address_type
                .map(|val| Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("command-address-type"),
                    expression: Expression::Integer(convert_integer(val)).with_dummy_span(),
                }),
            device_mir
                .global_config
                .buffer_address_type
                .map(|val| Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("buffer-address-type"),
                    expression: Expression::Integer(convert_integer(val)).with_dummy_span(),
                }),
            Some(Property {
                doc_comments: Vec::new(),
                name: Ident::new_no_span("word-boundaries"),
                expression: Expression::String(
                    device_mir
                        .global_config
                        .name_word_boundaries
                        .iter()
                        .map(convert_boundary)
                        .join(":")
                        .leak(),
                )
                .with_dummy_span(),
            }),
        ]
        .into_iter()
        .flatten()
        .map(SpanExt::with_dummy_span)
        .collect(),
        sub_nodes: device_mir
            .objects
            .iter()
            .map(|o| {
                convert_object(o, &device_mir.objects)
                    .with_message(|| format!("converting object `{}`", object_name(o)))
            })
            .collect::<Result<Vec<_>, _>>()?,
        span: Span::empty(),
    };

    Ok(ddsl_root.to_string())
}

fn convert_integer(value: V1Integer) -> Integer {
    match value {
        V1Integer::U8 => Integer::U8,
        V1Integer::U16 => Integer::U16,
        V1Integer::U32 => Integer::U32,
        V1Integer::I8 => Integer::I8,
        V1Integer::I16 => Integer::I16,
        V1Integer::I32 => Integer::I32,
        V1Integer::I64 => Integer::I64,
    }
}

fn convert_byte_order(value: V1ByteOrder) -> ByteOrder {
    match value {
        V1ByteOrder::LE => ByteOrder::LE,
        V1ByteOrder::BE => ByteOrder::BE,
    }
}

fn convert_access(value: V1Access) -> Access {
    match value {
        V1Access::RW => Access::RW,
        V1Access::RO => Access::RO,
        V1Access::WO => Access::WO,
    }
}

fn convert_boundary(value: &Boundary) -> &'static str {
    match value {
        dd_v1_convert_case::Boundary::Hyphen => "-",
        dd_v1_convert_case::Boundary::Underscore => "_",
        dd_v1_convert_case::Boundary::Space => " ",
        dd_v1_convert_case::Boundary::UpperLower => "Aa",
        dd_v1_convert_case::Boundary::LowerUpper => "bB",
        dd_v1_convert_case::Boundary::DigitUpper => "1A",
        dd_v1_convert_case::Boundary::UpperDigit => "A1",
        dd_v1_convert_case::Boundary::DigitLower => "1a",
        dd_v1_convert_case::Boundary::LowerDigit => "a1",
        dd_v1_convert_case::Boundary::Acronym => "AAa",
    }
}

fn convert_object(object: &Object, all_objects: &[Object]) -> Result<Node<'static>, DynError> {
    let node = match object {
        Object::Block(block) => convert_block(block, all_objects)?,
        Object::Register(register) => convert_register(register, None)?,
        Object::Command(command) => convert_command(command, None, None)?,
        Object::Buffer(buffer) => convert_buffer(buffer)?,
        Object::Ref(ref_object) => convert_ref_object(ref_object, all_objects)?,
    };

    Ok(node)
}

fn convert_ref_object(
    ref_object: &RefObject,
    all_objects: &[Object],
) -> Result<Node<'static>, DynError> {
    match &ref_object.object_override {
        ObjectOverride::Block(block_override) => {
            let Some(Object::Block(original_block)) = all_objects
                .iter()
                .find(|o| object_name(o) == block_override.name)
            else {
                return Err(DynError::new("could not find the original block"));
            };

            let overridden_block = Block {
                cfg_attr: original_block.cfg_attr.clone(),
                description: ref_object.description.clone(),
                name: ref_object.name.clone(),
                address_offset: block_override
                    .address_offset
                    .unwrap_or(original_block.address_offset),
                repeat: block_override.repeat.or(original_block.repeat),
                objects: original_block.objects.clone(),
            };

            convert_block(&overridden_block, all_objects)
        }
        ObjectOverride::Register(register_override) => {
            let Some(Object::Register(original_register)) = all_objects
                .iter()
                .find(|o| object_name(o) == register_override.name)
            else {
                return Err(DynError::new("could not find the original register"));
            };

            let overridden_register = Register {
                cfg_attr: original_register.cfg_attr.clone(),
                description: ref_object.description.clone(),
                name: ref_object.name.clone(),
                access: register_override.access.unwrap_or(original_register.access),
                byte_order: Default::default(),
                bit_order: Default::default(),
                allow_bit_overlap: Default::default(),
                allow_address_overlap: register_override.allow_address_overlap,
                address: register_override
                    .address
                    .unwrap_or(original_register.address),
                size_bits: Default::default(),
                reset_value: register_override
                    .reset_value
                    .clone()
                    .or(original_register.reset_value.clone()),
                repeat: register_override.repeat.or(original_register.repeat),
                fields: Default::default(),
            };

            convert_register(&overridden_register, Some(register_override.name.clone()))
        }
        ObjectOverride::Command(command_override) => {
            let Some(Object::Command(original_command)) = all_objects
                .iter()
                .find(|o| object_name(o) == command_override.name)
            else {
                return Err(DynError::new("could not find the original command"));
            };

            let overridden_command = Command {
                cfg_attr: original_command.cfg_attr.clone(),
                description: ref_object.description.clone(),
                name: ref_object.name.clone(),
                byte_order: Default::default(),
                bit_order: Default::default(),
                allow_bit_overlap: Default::default(),
                allow_address_overlap: command_override.allow_address_overlap,
                address: command_override.address.unwrap_or(original_command.address),
                size_bits_in: Default::default(),
                size_bits_out: Default::default(),
                repeat: command_override.repeat.or(original_command.repeat),
                in_fields: Default::default(),
                out_fields: Default::default(),
            };

            convert_command(
                &overridden_command,
                (!original_command.in_fields.is_empty())
                    .then(|| format!("{}FieldsIn", original_command.name)),
                (!original_command.out_fields.is_empty())
                    .then(|| format!("{}FieldsOut", original_command.name)),
            )
        }
    }
}

fn convert_register(
    register: &Register,
    fieldset_override: Option<String>,
) -> Result<Node<'static>, DynError> {
    Ok(Node {
        doc_comments: register
            .description
            .lines()
            .map(|l| (l.to_owned().leak() as &str).with_dummy_span())
            .collect(),
        node_type: Ident::new_no_span("register"),
        name: Ident::new_no_span(register.name.clone().leak()),
        repeat: register
            .repeat
            .map(convert_repeat)
            .transpose()?
            .map(SpanExt::with_dummy_span),
        type_specifier: None,
        short_properties: Vec::new(),
        properties: [
            Some(
                Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("address"),
                    expression: Expression::Number(register.address.into()).with_dummy_span(),
                }
                .with_dummy_span(),
            ),
            register.allow_address_overlap.then(|| {
                Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("address-overlap"),
                    expression: Expression::Allow.with_dummy_span(),
                }
                .with_dummy_span()
            }),
            register.reset_value.as_ref().map(|reset_value| {
                Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("reset"),
                    expression: match reset_value {
                        device_driver_generation::mir::ResetValue::Integer(num) => {
                            Expression::Number(*num as i128)
                        }
                        device_driver_generation::mir::ResetValue::Array(items) => {
                            Expression::ByteArray(items.clone())
                        }
                    }
                    .with_dummy_span(),
                }
                .with_dummy_span()
            }),
            Some(
                Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("fields"),
                    expression: if let Some(fieldset_override) = fieldset_override {
                        Expression::TypeReference(Ident::new_no_span(fieldset_override.leak()))
                    } else {
                        Expression::SubNode(Box::new(
                            convert_fieldset(
                                None,
                                register.byte_order,
                                register.bit_order,
                                register.allow_bit_overlap,
                                register.size_bits,
                                &register.fields,
                            )
                            .with_message(|| "converting fieldset")?,
                        ))
                    }
                    .with_dummy_span(),
                }
                .with_dummy_span(),
            ),
        ]
        .into_iter()
        .flatten()
        .collect(),
        sub_nodes: Vec::new(),
        span: Span::empty(),
    })
}

fn convert_command(
    command: &Command,
    fieldset_in_override: Option<String>,
    fieldset_out_override: Option<String>,
) -> Result<Node<'static>, DynError> {
    Ok(Node {
        doc_comments: command
            .description
            .lines()
            .map(|l| (l.to_owned().leak() as &str).with_dummy_span())
            .collect(),
        node_type: Ident::new_no_span("command"),
        name: Ident::new_no_span(command.name.clone().leak()),
        repeat: command
            .repeat
            .map(convert_repeat)
            .transpose()?
            .map(SpanExt::with_dummy_span),
        type_specifier: None,
        short_properties: Vec::new(),
        properties: [
            Some(
                Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("address"),
                    expression: Expression::Number(command.address.into()).with_dummy_span(),
                }
                .with_dummy_span(),
            ),
            command.allow_address_overlap.then(|| {
                Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("address-overlap"),
                    expression: Expression::Allow.with_dummy_span(),
                }
                .with_dummy_span()
            }),
            (!command.in_fields.is_empty() || fieldset_in_override.is_some())
                .then(|| {
                    Ok(Property {
                        doc_comments: Vec::new(),
                        name: Ident::new_no_span("fields-in"),
                        expression: if let Some(fieldset_in_override) = fieldset_in_override {
                            Expression::TypeReference(Ident::new_no_span(
                                fieldset_in_override.leak(),
                            ))
                        } else {
                            Expression::SubNode(Box::new(
                                convert_fieldset(
                                    Some(format!("{}FieldsIn", command.name)),
                                    command.byte_order,
                                    command.bit_order,
                                    command.allow_bit_overlap,
                                    command.size_bits_in,
                                    &command.in_fields,
                                )
                                .with_message(|| "converting in fieldset")?,
                            ))
                        }
                        .with_dummy_span(),
                    }
                    .with_dummy_span())
                })
                .transpose()?,
            (!command.out_fields.is_empty() || fieldset_out_override.is_some())
                .then(|| {
                    Ok(Property {
                        doc_comments: Vec::new(),
                        name: Ident::new_no_span("fields-out"),
                        expression: if let Some(fieldset_out_override) = fieldset_out_override {
                            Expression::TypeReference(Ident::new_no_span(
                                fieldset_out_override.leak(),
                            ))
                        } else {
                            Expression::SubNode(Box::new(
                                convert_fieldset(
                                    Some(format!("{}FieldsOut", command.name)),
                                    command.byte_order,
                                    command.bit_order,
                                    command.allow_bit_overlap,
                                    command.size_bits_out,
                                    &command.out_fields,
                                )
                                .with_message(|| "converting out fieldset")?,
                            ))
                        }
                        .with_dummy_span(),
                    }
                    .with_dummy_span())
                })
                .transpose()?,
        ]
        .into_iter()
        .flatten()
        .collect(),
        sub_nodes: Vec::new(),
        span: Span::empty(),
    })
}

fn convert_fieldset(
    name: Option<String>,
    byte_order: Option<V1ByteOrder>,
    bit_order: BitOrder,
    allow_bit_overlap: bool,
    size_bits: u32,
    fields: &[Field],
) -> Result<Node<'static>, DynError> {
    if !size_bits.is_multiple_of(8) {
        return Err(DynError::new(
            "size-bits is not a multiple of 8. This is no longer supported in v2",
        ));
    }
    if matches!(bit_order, BitOrder::MSB0) {
        return Err(DynError::new(
            "bitorder is MSB0. This is no longer supported in v2 which only supports LSB0",
        ));
    }

    Ok(Node {
        doc_comments: Vec::new(),
        node_type: Ident::new_no_span("fieldset"),
        name: Ident::new_no_span(name.map(|name| name.clone().leak() as &str).unwrap_or("_")),
        repeat: None,
        type_specifier: None,
        short_properties: Vec::new(),
        properties: [
            Some(
                Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("size-bytes"),
                    expression: Expression::Number((size_bits / 8).into()).with_dummy_span(),
                }
                .with_dummy_span(),
            ),
            byte_order.map(|byte_order| {
                Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("byte-order"),
                    expression: Expression::ByteOrder(convert_byte_order(byte_order))
                        .with_dummy_span(),
                }
                .with_dummy_span()
            }),
            allow_bit_overlap.then(|| {
                Property {
                    doc_comments: Vec::new(),
                    name: Ident::new_no_span("bit-overlap"),
                    expression: Expression::Allow.with_dummy_span(),
                }
                .with_dummy_span()
            }),
        ]
        .into_iter()
        .flatten()
        .collect(),
        sub_nodes: fields
            .iter()
            .map(|f| convert_field(f).with_message(|| format!("converting field {}", f.name)))
            .collect::<Result<Vec<_>, _>>()?,
        span: Span::empty(),
    })
}

fn convert_field(field: &Field) -> Result<Node<'static>, DynError> {
    Ok(Node {
        doc_comments: field
            .description
            .lines()
            .map(|l| (l.to_owned().leak() as &str).with_dummy_span())
            .collect(),
        node_type: Ident::new_no_span("field"),
        name: Ident::new_no_span(field.name.clone().leak()),
        repeat: None,
        type_specifier: Some(
            TypeSpecifier {
                base_type: convert_base_type(field.base_type).with_dummy_span(),
                use_try: field
                    .field_conversion
                    .as_ref()
                    .is_some_and(|fc| fc.use_try()),
                conversion: field
                    .field_conversion
                    .as_ref()
                    .map(convert_field_conversion),
            }
            .with_dummy_span(),
        ),
        short_properties: [
            if field.field_address.len() == 1 {
                Expression::Number(field.field_address.start.into())
            } else {
                Expression::AddressRange {
                    end: (field.field_address.end - 1).into(),
                    start: field.field_address.start.into(),
                }
            }
            .with_dummy_span(),
            Expression::Access(convert_access(field.access)).with_dummy_span(),
        ]
        .into_iter()
        .collect(),
        properties: Vec::new(),
        sub_nodes: Vec::new(),
        span: Span::empty(),
    })
}

fn convert_buffer(buffer: &Buffer) -> Result<Node<'static>, DynError> {
    Ok(Node {
        doc_comments: buffer
            .description
            .lines()
            .map(|l| (l.to_owned().leak() as &str).with_dummy_span())
            .collect(),
        node_type: Ident::new_no_span("buffer"),
        name: Ident::new_no_span(buffer.name.clone().leak()),
        repeat: None,
        type_specifier: None,
        short_properties: Vec::new(),
        properties: vec![
            Property {
                doc_comments: Vec::new(),
                name: Ident::new_no_span("access"),
                expression: Expression::Access(convert_access(buffer.access)).with_dummy_span(),
            }
            .with_dummy_span(),
            Property {
                doc_comments: Vec::new(),
                name: Ident::new_no_span("address"),
                expression: Expression::Number(buffer.address.into()).with_dummy_span(),
            }
            .with_dummy_span(),
        ],
        sub_nodes: Vec::new(),
        span: Span::empty(),
    })
}

fn convert_block(block: &Block, all_objects: &[Object]) -> Result<Node<'static>, DynError> {
    Ok(Node {
        doc_comments: block
            .description
            .lines()
            .map(|l| (l.to_owned().leak() as &str).with_dummy_span())
            .collect(),
        node_type: Ident::new_no_span("block"),
        name: Ident::new_no_span(block.name.clone().leak()),
        repeat: block
            .repeat
            .map(convert_repeat)
            .transpose()?
            .map(SpanExt::with_dummy_span),
        type_specifier: None,
        short_properties: Vec::new(),
        properties: vec![
            Property {
                doc_comments: Vec::new(),
                name: Ident::new_no_span("address-offset"),
                expression: Expression::Number(block.address_offset.into()).with_dummy_span(),
            }
            .with_dummy_span(),
        ],
        sub_nodes: block
            .objects
            .iter()
            .map(|o| {
                convert_object(o, all_objects)
                    .with_message(|| format!("converting object {}", object_name(o)))
            })
            .collect::<Result<Vec<_>, _>>()?,
        span: Span::empty(),
    })
}

fn convert_repeat(repeat: V1Repeat) -> Result<Repeat<'static>, DynError> {
    Ok(Repeat {
        source: RepeatSource::Count(
            NonZeroU32::new(
                u32::try_from(repeat.count).with_message(|| "converting repeat count")?,
            )
            .ok_or_else(|| DynError::new("converting repeat count"))?,
        )
        .with_dummy_span(),
        stride: i32::try_from(repeat.stride)
            .with_message(|| "converting repeat stride")?
            .with_dummy_span(),
    })
}

fn object_name(object: &Object) -> &str {
    match object {
        Object::Block(block) => &block.name,
        Object::Register(register) => &register.name,
        Object::Command(command) => &command.name,
        Object::Buffer(buffer) => &buffer.name,
        Object::Ref(ref_object) => &ref_object.name,
    }
}

fn convert_base_type(value: V1BaseType) -> BaseType {
    match value {
        V1BaseType::Unspecified => BaseType::Unspecified,
        V1BaseType::Bool => BaseType::Bool,
        V1BaseType::Uint => BaseType::Uint,
        V1BaseType::Int => BaseType::Int,
        V1BaseType::FixedSize(integer) => BaseType::FixedSize(convert_integer(integer)),
    }
}

fn convert_field_conversion(fs: &FieldConversion) -> TypeConversion<'static> {
    match fs {
        FieldConversion::Direct { type_name, .. } => {
            TypeConversion::Reference(Ident::new_no_span(type_name.clone().leak()))
        }
        FieldConversion::Enum { enum_value, .. } => {
            TypeConversion::Subnode(Box::new(convert_enum(enum_value)))
        }
    }
}

fn convert_enum(enum_value: &Enum) -> Node<'static> {
    Node {
        doc_comments: enum_value
            .description
            .lines()
            .map(|l| (l.to_owned().leak() as &str).with_dummy_span())
            .collect(),
        node_type: Ident::new_no_span("enum"),
        name: Ident::new_no_span(enum_value.name.clone().leak()),
        repeat: None,
        type_specifier: None,
        short_properties: Vec::new(),
        properties: enum_value
            .variants
            .iter()
            .map(|variant| {
                Property {
                    doc_comments: variant
                        .description
                        .lines()
                        .map(|l| (l.to_owned().leak() as &str).with_dummy_span())
                        .collect(),
                    name: Ident::new_no_span(variant.name.clone().leak()),
                    expression: match variant.value {
                        device_driver_generation::mir::EnumValue::Unspecified => Expression::Auto,
                        device_driver_generation::mir::EnumValue::Specified(num) => {
                            Expression::Number(num)
                        }
                        device_driver_generation::mir::EnumValue::Default => {
                            Expression::DefaultNumber(None)
                        }
                        device_driver_generation::mir::EnumValue::CatchAll => {
                            Expression::CatchAllNumber(None)
                        }
                    }
                    .with_dummy_span(),
                }
                .with_dummy_span()
            })
            .collect(),
        sub_nodes: Vec::new(),
        span: Span::empty(),
    }
}
