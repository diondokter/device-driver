use std::{collections::HashMap, path::Path, str::FromStr};

use kdl::{KdlDocument, KdlNode, KdlValue};
use miette::SourceSpan;

use crate::{
    mir::{Device, GlobalConfig},
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
            if let Some(value) = parse_single_string(node, source_code.clone(), diagnostics, None).0
            {
                device.global_config.name_word_boundaries =
                    convert_case::Boundary::list_from(&value);
            }
        }
        GlobalConfigType::DefmtFeature => {
            if let Some(value) = parse_single_string(node, source_code.clone(), diagnostics, None).0
            {
                device.global_config.defmt_feature = Some(value);
            }
        }
    }
}

fn parse_single_string(
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
    match parse_single_string(node, source_code.clone(), diagnostics, Some(T::VARIANTS)) {
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
