use crate::DeviceDriverV1Format;
use dd_v1_convert_case::Boundary;
use device_driver_common::{
    span::{Span, SpanExt},
    specifiers::{ByteOrder, Integer},
};
use device_driver_diagnostics::{DynError, ResultExt};
use device_driver_generation::mir::{ByteOrder as V1ByteOrder, Integer as V1Integer};
use device_driver_parser::{Expression, Ident, Node, Property};
use itertools::Itertools;

pub fn convert(source: &str, sub_format: DeviceDriverV1Format) -> Result<String, DynError> {
    let device_mir = match sub_format {
        DeviceDriverV1Format::DSL => device_driver_generation::_private_transform_dsl_mir(
            source
                .parse()
                .map_err(|e| DynError::new(e))
                .with_message(|| "parsing source into tokenstream")?,
        )
        .into_dyn_result(),
        DeviceDriverV1Format::YAML => device_driver_generation::_private_transform_yaml_mir(source)
            .map_err(|e| DynError::new(e)),
        DeviceDriverV1Format::JSON => device_driver_generation::_private_transform_json_mir(source)
            .map_err(|e| DynError::new(e)),
        DeviceDriverV1Format::TOML => device_driver_generation::_private_transform_toml_mir(source)
            .map_err(|e| DynError::new(e)),
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
        sub_nodes: Vec::new(),
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

fn convert_object() {
    // Remember to take the default accesses into account! Doesn't exist in DDSL
    todo!()
}
