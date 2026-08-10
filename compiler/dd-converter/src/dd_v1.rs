use std::num::NonZeroU32;

use crate::DeviceDriverV1Format;
use dd_v1_convert_case::Boundary;
use device_driver_common::{
    span::{Span, SpanExt},
    specifiers::{Access, ByteOrder, Integer},
};
use device_driver_diagnostics::{DynError, ResultExt};
use device_driver_generation::mir::{
    Access as V1Access, BitOrder, Block, Buffer, ByteOrder as V1ByteOrder, Command, Field,
    Integer as V1Integer, Object, Register, Repeat as V1Repeat,
};
use device_driver_parser::{Expression, Ident, Node, Property, Repeat, RepeatSource};
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
                convert_object(o).with_message(|| format!("converting object `{}`", object_name(o)))
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

fn convert_object(object: &Object) -> Result<Node<'static>, DynError> {
    let node = match object {
        Object::Block(block) => convert_block(block)?,
        Object::Register(register) => convert_register(register)?,
        Object::Command(command) => convert_command(command)?,
        Object::Buffer(buffer) => convert_buffer(buffer)?,
        Object::Ref(ref_object) => Node {
            doc_comments: Vec::new(),
            node_type: Ident::new_no_span("todo-ref"),
            name: Ident::new_no_span(ref_object.name.clone().leak()),
            repeat: None,
            type_specifier: None,
            short_properties: Vec::new(),
            properties: Vec::new(),
            sub_nodes: Vec::new(),
            span: Span::empty(),
        },
    };

    Ok(node)
}

fn convert_register(register: &Register) -> Result<Node<'static>, DynError> {
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
                    expression: Expression::SubNode(Box::new(
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

fn convert_command(command: &Command) -> Result<Node<'static>, DynError> {
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
            (!command.in_fields.is_empty())
                .then(|| {
                    Ok(Property {
                        doc_comments: Vec::new(),
                        name: Ident::new_no_span("fields-in"),
                        expression: Expression::SubNode(Box::new(
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
                        .with_dummy_span(),
                    }
                    .with_dummy_span())
                })
                .transpose()?,
            (!command.out_fields.is_empty())
                .then(|| {
                    Ok(Property {
                        doc_comments: Vec::new(),
                        name: Ident::new_no_span("fields-out"),
                        expression: Expression::SubNode(Box::new(
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
        doc_comments: Vec::new(),
        node_type: Ident::new_no_span("todo-field"),
        name: Ident::new_no_span(field.name.clone().leak()),
        repeat: None,
        type_specifier: None,
        short_properties: Vec::new(),
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

fn convert_block(block: &Block) -> Result<Node<'static>, DynError> {
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
                convert_object(o).with_message(|| format!("converting object {}", object_name(o)))
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
