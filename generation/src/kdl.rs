use std::{collections::HashMap, path::Path, str::FromStr};

use kdl::{KdlDocument, KdlNode, KdlValue};
use miette::SourceSpan;

use crate::{
    mir::{Access, Device, GlobalConfig, Object, Register, ResetValue},
    reporting::{Diagnostics, NamedSourceCode, errors},
};

pub fn transform(source: &str, file_path: &Path, diagnostics: &mut Diagnostics) -> Vec<Device> {
    let source_code = NamedSourceCode::new(file_path.display().to_string(), source.into());

    let document = match kdl::KdlDocument::parse(source) {
        Ok(document) => document,
        Err(e) => {
            for diagnostic in e.diagnostics {
                diagnostics.add(diagnostic);
            }
            return Vec::new();
        }
    };

    document
        .nodes()
        .iter()
        .filter_map(|node| transform_device(node, source_code.clone(), diagnostics))
        .collect()
}

fn transform_device(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<Device> {
    if node.name().value() != "device" {
        diagnostics.add(errors::UnexpectedNode {
            source_code,
            node_name: node.name().span(),
            expected_names: vec!["device"],
        });
        return None;
    }

    let device_name = match node.entries().first() {
        Some(entry) => {
            if let KdlValue::String(device_name) = entry.value()
                && entry.name().is_none()
            {
                device_name.clone()
            } else {
                diagnostics.add(errors::MissingObjectName {
                    source_code,
                    object_keyword: node.name().span(),
                    found_instead: Some(entry.span()),
                    object_type: node.name().value().into(),
                });
                return None;
            }
        }
        None => {
            diagnostics.add(errors::MissingObjectName {
                source_code,
                object_keyword: node.name().span(),
                found_instead: None,
                object_type: node.name().value().into(),
            });
            return None;
        }
    };

    if node.entries().len() > 1 {
        diagnostics.add(errors::UnexpectedEntries {
            source_code,
            superfluous_entries: node.entries().iter().skip(1).map(|e| e.span()).collect(),
            unexpected_name_entries: Vec::new(),
            not_anonymous_entries: Vec::new(),
        });
        return None;
    }

    let mut device = Device {
        name: Some(device_name),
        global_config: GlobalConfig::default(),
        objects: Vec::new(),
    };

    if let Some(device_document) = node.children()
        && !device_document.nodes().is_empty()
    {
        transform_device_internals(&mut device, device_document, source_code, diagnostics);
    } else {
        diagnostics.add(errors::EmptyNode {
            source_code,
            node: node.span(),
        });
    }

    Some(device)
}

fn transform_device_internals(
    device: &mut Device,
    device_document: &KdlDocument,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) {
    let mut seen_global_configs = HashMap::<GlobalConfigType, SourceSpan>::new();

    for node in device_document.nodes() {
        if let Ok(global_config_type) = node.name().value().parse::<GlobalConfigType>() {
            match seen_global_configs.insert(global_config_type, node.span()) {
                None => transform_global_config_node(
                    device,
                    node,
                    global_config_type,
                    source_code.clone(),
                    diagnostics,
                ),
                Some(original_node) => {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: node.span(),
                        original: original_node,
                    });
                }
            }
        } else if let Ok(object_type) = node.name().value().parse::<ObjectType>() {
            let object = match object_type {
                ObjectType::Block => todo!(),
                ObjectType::Register => {
                    transform_register(node, source_code.clone(), diagnostics).map(Object::Register)
                }
                ObjectType::Command => todo!(),
                ObjectType::Buffer => todo!(),
                ObjectType::Ref => todo!(),
            };

            if let Some(object) = object {
                device.objects.push(object);
            }
        } else {
            diagnostics.add(errors::UnexpectedNode {
                source_code: source_code.clone(),
                node_name: node.name().span(),
                expected_names: GLOBAL_CONFIG_TYPES
                    .iter()
                    .map(|v| v.0)
                    .chain(OBJECT_TYPES.iter().map(|v| v.0))
                    .collect(),
            });
        }
    }
}

fn transform_register(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<Register> {
    let name = parse_single_string_entry(node, source_code.clone(), diagnostics, None).0;

    let mut access = None;
    let mut allow_address_overlap = None;
    let mut address = None;
    let mut reset_value = None;
    let mut repeat = None;
    let mut field_set = None;

    for child in node.iter_children() {
        match child.name().value().parse() {
            Ok(RegisterField::Access) => {
                access =
                    parse_single_string_value::<Access>(child, source_code.clone(), diagnostics);
            }
            Ok(RegisterField::AllowAddressOverlap) => {
                ensure_zero_entries(child, source_code.clone(), diagnostics);
                allow_address_overlap = Some(true);
            }
            Ok(RegisterField::Address) => {
                address = parse_single_integer_entry(node, source_code.clone(), diagnostics).0;
            }
            Ok(RegisterField::ResetValue) => {
                reset_value = parse_reset_value_entries(node, source_code.clone(), diagnostics)
            }
            Ok(RegisterField::Repeat) => {
                todo!()
            }
            Ok(RegisterField::FieldSet) => {
                todo!()
            }
            Err(()) => {
                diagnostics.add(errors::UnexpectedNode {
                    source_code: source_code.clone(),
                    node_name: node.name().span(),
                    expected_names: REGISTER_FIELDS.iter().map(|v| v.0).collect(),
                });
            }
        }
    }

    let mut error = false;
    if name.is_none() {
        error = true;
        // Just continue. Error is already emitted
    }

    if address.is_none() {
        error = true;
        todo!("Emit error");
    }

    if field_set.is_none() {
        error = true;
        todo!("Emit error");
    }

    if error {
        None
    } else {
        let mut register = Register::default();

        register.name = name.unwrap();
        if let Some(access) = access {
            register.access = access;
        }
        if let Some(allow_address_overlap) = allow_address_overlap {
            register.allow_address_overlap = allow_address_overlap;
        }
        // TODO: Change internal address types to i128
        register.address = address.unwrap();
        register.reset_value = reset_value;
        register.repeat = repeat;
        register.field_set = field_set.unwrap();

        Some(register)
    }
}

fn transform_global_config_node(
    device: &mut Device,
    node: &KdlNode,
    global_config_type: GlobalConfigType,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) {
    match global_config_type {
        GlobalConfigType::DefaultRegisterAccess => {
            if let Some(value) = parse_single_string_value(node, source_code.clone(), diagnostics) {
                device.global_config.default_register_access = value;
            }
        }
        GlobalConfigType::DefaultFieldAccess => {
            if let Some(value) = parse_single_string_value(node, source_code.clone(), diagnostics) {
                device.global_config.default_field_access = value;
            }
        }
        GlobalConfigType::DefaultBufferAccess => {
            if let Some(value) = parse_single_string_value(node, source_code.clone(), diagnostics) {
                device.global_config.default_buffer_access = value;
            }
        }
        GlobalConfigType::DefaultByteOrder => {
            if let Some(value) = parse_single_string_value(node, source_code.clone(), diagnostics) {
                device.global_config.default_byte_order = Some(value);
            }
        }
        GlobalConfigType::DefaultBitOrder => {
            if let Some(value) = parse_single_string_value(node, source_code.clone(), diagnostics) {
                device.global_config.default_bit_order = value;
            }
        }
        GlobalConfigType::RegisterAddressType => {
            if let Some(value) = parse_single_string_value(node, source_code.clone(), diagnostics) {
                device.global_config.register_address_type = Some(value);
            }
        }
        GlobalConfigType::CommandAddressType => {
            if let Some(value) = parse_single_string_value(node, source_code.clone(), diagnostics) {
                device.global_config.command_address_type = Some(value);
            }
        }
        GlobalConfigType::BufferAddressType => {
            if let Some(value) = parse_single_string_value(node, source_code.clone(), diagnostics) {
                device.global_config.buffer_address_type = Some(value);
            }
        }
        GlobalConfigType::NameWordBoundaries => {
            if let Some(value) =
                parse_single_string_entry(node, source_code.clone(), diagnostics, None).0
            {
                device.global_config.name_word_boundaries =
                    convert_case::Boundary::list_from(&value);
            }
        }
        GlobalConfigType::DefmtFeature => {
            if let Some(value) =
                parse_single_string_entry(node, source_code.clone(), diagnostics, None).0
            {
                device.global_config.defmt_feature = Some(value);
            }
        }
    }
}

fn ensure_zero_entries(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) {
    if !node.entries().is_empty() {
        diagnostics.add(errors::UnexpectedEntries {
            source_code: source_code.clone(),
            superfluous_entries: node.entries().iter().map(|entry| entry.span()).collect(),
            unexpected_name_entries: Vec::new(),
            not_anonymous_entries: Vec::new(),
        });
    }
}

fn parse_reset_value_entries(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<ResetValue> {
    let mut error = false;
    let mut array = Vec::new();

    for entry in node.entries() {
        match entry.value() {
            KdlValue::Integer(val) => array.push((*val, entry.span())),
            _ => {
                error = true;
                diagnostics.add(errors::UnexpectedType {
                    source_code: source_code.clone(),
                    value_name: entry.span(),
                    expected_type: "integer",
                });
            }
        }
    }

    if error {
        return None;
    }

    if array.len() == 1 {
        let (integer, span) = array[0];

        if integer.is_negative() {
            diagnostics.add(errors::ValueOutOfRange {
                source_code: source_code.clone(),
                value: span,
                range: "0..".into(),
                context: Some("Negative reset values are not allowed".into()),
            });
            None
        } else {
            Some(ResetValue::Integer(integer as u128))
        }
    } else {
        let mut error = false;
        for (byte, span) in array.iter() {
            if !(0..=255).contains(byte) {
                error = true;
                diagnostics.add(errors::ValueOutOfRange {
                    source_code: source_code.clone(),
                    value: *span,
                    range: "0..256".into(),
                    context: Some(
                        "When specifying the reset values as an array, all numbers must be bytes"
                            .into(),
                    ),
                });
            }
        }

        if error {
            None
        } else {
            Some(ResetValue::Array(
                array.iter().map(|(byte, _)| *byte as u8).collect(),
            ))
        }
    }
}

fn parse_single_integer_entry(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> (Option<i128>, Option<SourceSpan>) {
    let unexpected_entries = errors::UnexpectedEntries {
        source_code: source_code.clone(),
        superfluous_entries: node
            .entries()
            .iter()
            .skip(1)
            .map(|entry| entry.span())
            .collect(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: node
            .entries()
            .first()
            .iter()
            .filter_map(|entry| entry.name().map(|_| entry.span()))
            .collect(),
    };
    if !unexpected_entries.is_empty() {
        diagnostics.add(unexpected_entries);
    }

    match node.entries().first() {
        Some(entry) if entry.name().is_none() => match entry.value() {
            KdlValue::Integer(val) => (Some(*val), Some(entry.span())),
            _ => {
                diagnostics.add(errors::UnexpectedType {
                    source_code,
                    value_name: entry.span(),
                    expected_type: "integer",
                });
                (None, Some(entry.span()))
            }
        },
        _ => {
            diagnostics.add(errors::MissingEntry {
                source_code,
                node_name: node.name().span(),
                expected_entries: vec!["integer"],
            });
            (None, None)
        }
    }
}

fn parse_single_string_entry(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
    expected_entries: Option<&[&'static str]>,
) -> (Option<String>, Option<SourceSpan>) {
    let unexpected_entries = errors::UnexpectedEntries {
        source_code: source_code.clone(),
        superfluous_entries: node
            .entries()
            .iter()
            .skip(1)
            .map(|entry| entry.span())
            .collect(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: node
            .entries()
            .first()
            .iter()
            .filter_map(|entry| entry.name().map(|_| entry.span()))
            .collect(),
    };
    if !unexpected_entries.is_empty() {
        diagnostics.add(unexpected_entries);
    }

    match node.entries().first() {
        Some(entry) if entry.name().is_none() => match entry.value() {
            KdlValue::String(val) => (Some(val.clone()), Some(entry.span())),
            _ => {
                diagnostics.add(errors::UnexpectedType {
                    source_code,
                    value_name: entry.span(),
                    expected_type: "string",
                });
                (None, Some(entry.span()))
            }
        },
        _ => {
            diagnostics.add(errors::MissingEntry {
                source_code,
                node_name: node.name().span(),
                expected_entries: expected_entries
                    .map(|ee| ee.to_vec())
                    .unwrap_or_else(|| vec!["string"]),
            });
            (None, None)
        }
    }
}

fn parse_single_string_value<T: strum::VariantNames + FromStr>(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<T> {
    match parse_single_string_entry(node, source_code.clone(), diagnostics, Some(T::VARIANTS)) {
        (Some(val), _) if T::from_str(&val).is_ok() => T::from_str(&val).ok(),
        (Some(_), Some(entry)) => {
            diagnostics.add(errors::UnexpectedValue {
                source_code,
                value_name: entry,
                expected_values: T::VARIANTS.to_vec(),
            });
            None
        }
        _ => None,
    }
}

#[rustfmt::skip]
const REGISTER_FIELDS: &[(&str, RegisterField)] = &[
    ("access", RegisterField::Access),
    ("allow-address-overlap", RegisterField::AllowAddressOverlap),
    ("address", RegisterField::Address),
    ("reset-value", RegisterField::ResetValue),
    ("repeat", RegisterField::Repeat),
    ("fields", RegisterField::FieldSet),
];
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum RegisterField {
    Access,
    AllowAddressOverlap,
    Address,
    ResetValue,
    Repeat,
    FieldSet,
}

impl FromStr for RegisterField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (name, val) in REGISTER_FIELDS {
            if *name == s {
                return Ok(*val);
            }
        }

        Err(())
    }
}

#[rustfmt::skip]
const GLOBAL_CONFIG_TYPES: &[(&str, GlobalConfigType)] = &[
    ("default-register-access", GlobalConfigType::DefaultRegisterAccess),
    ("default-field-access", GlobalConfigType::DefaultFieldAccess),
    ("default-buffer-access", GlobalConfigType::DefaultBufferAccess),
    ("default-byte-order", GlobalConfigType::DefaultByteOrder),
    ("default-bit-order", GlobalConfigType::DefaultBitOrder),
    ("register-address-type", GlobalConfigType::RegisterAddressType),
    ("command-address-type", GlobalConfigType::CommandAddressType),
    ("buffer-address-type", GlobalConfigType::BufferAddressType),
    ("name-word-boundaries", GlobalConfigType::NameWordBoundaries),
    ("defmt-feature", GlobalConfigType::DefmtFeature),
];
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum GlobalConfigType {
    DefaultRegisterAccess,
    DefaultFieldAccess,
    DefaultBufferAccess,
    DefaultByteOrder,
    DefaultBitOrder,
    RegisterAddressType,
    CommandAddressType,
    BufferAddressType,
    NameWordBoundaries,
    DefmtFeature,
}

impl FromStr for GlobalConfigType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (name, val) in GLOBAL_CONFIG_TYPES {
            if *name == s {
                return Ok(*val);
            }
        }

        Err(())
    }
}

#[rustfmt::skip]
const OBJECT_TYPES: &[(&str, ObjectType)] = &[
    ("block", ObjectType::Block),
    ("register", ObjectType::Register),
    ("command", ObjectType::Command),
    ("buffer", ObjectType::Buffer),
    ("ref", ObjectType::Ref),
];
#[derive(Clone, Copy)]
enum ObjectType {
    Block,
    Register,
    Command,
    Buffer,
    Ref,
}

impl FromStr for ObjectType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (name, val) in OBJECT_TYPES {
            if *name == s {
                return Ok(*val);
            }
        }

        Err(())
    }
}
