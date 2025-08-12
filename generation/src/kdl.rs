use std::{collections::HashMap, path::Path, str::FromStr};

use itertools::Itertools;
use kdl::{KdlDocument, KdlIdentifier, KdlNode, KdlValue};
use miette::SourceSpan;
use strum::VariantNames;

use crate::{
    mir::{
        Access, BaseType, BitOrder, Buffer, ByteOrder, Cfg, Command, Device, Enum, EnumValue,
        EnumVariant, Field, FieldConversion, FieldSet, GlobalConfig, Integer, Object, Register,
        Repeat, ResetValue,
    },
    reporting::{
        Diagnostics, NamedSourceCode,
        errors::{self, UnexpectedEntries},
    },
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

    let device_name =
        parse_single_string_entry(node, source_code.clone(), diagnostics, None, true).0?;

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
                ObjectType::Command => {
                    transform_command(node, source_code.clone(), diagnostics).map(Object::Command)
                }
                ObjectType::Buffer => {
                    transform_buffer(node, source_code.clone(), diagnostics).map(Object::Buffer)
                }
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
    let (name, name_span) =
        parse_single_string_entry(node, source_code.clone(), diagnostics, None, true);

    if name.is_none() && node.children().is_none() {
        // We only have a register keyword. No need for further diagnostics
        return None;
    }

    let mut access = None;
    let mut allow_address_overlap = None;
    let mut address = None;
    let mut reset_value = None;
    let mut repeat = None;
    let mut field_set = None;

    for child in node.iter_children() {
        match child.name().value().parse() {
            Ok(RegisterField::Access) => {
                if let Some((_, span)) = access {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                access =
                    parse_single_string_value::<Access>(child, source_code.clone(), diagnostics)
                        .map(|val| (val, child.name().span()));
            }
            Ok(RegisterField::AllowAddressOverlap) => {
                if let Some((_, span)) = allow_address_overlap {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                ensure_zero_entries(child, source_code.clone(), diagnostics);
                allow_address_overlap = Some(true).map(|val| (val, child.name().span()));
            }
            Ok(RegisterField::Address) => {
                if let Some((_, span)) = address {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                address = parse_single_integer_entry(child, source_code.clone(), diagnostics)
                    .0
                    .map(|val| (val, child.name().span()));
            }
            Ok(RegisterField::ResetValue) => {
                if let Some((_, span)) = reset_value {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                reset_value = parse_reset_value_entries(child, source_code.clone(), diagnostics)
                    .map(|val| (val, child.name().span()))
            }
            Ok(RegisterField::Repeat) => {
                if let Some((_, span)) = repeat {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                repeat = parse_repeat_entries(child, source_code.clone(), diagnostics)
                    .map(|val| (val, child.name().span()));
            }
            Ok(RegisterField::FieldSet) => {
                if let Some((_, span)) = field_set {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                field_set = transform_field_set(child, source_code.clone(), diagnostics)
                    .map(|val| (val, child.name().span()));
            }
            Err(()) => {
                diagnostics.add(errors::UnexpectedNode {
                    source_code: source_code.clone(),
                    node_name: child.name().span(),
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
        diagnostics.add(errors::MissingChildNode {
            source_code: source_code.clone(),
            node: name_span.unwrap_or(node.name().span()),
            node_type: Some("register"),
            missing_node_type: "address",
        });
    }

    if field_set.is_none() {
        error = true;
        diagnostics.add(errors::MissingChildNode {
            source_code: source_code.clone(),
            node: name_span.unwrap_or(node.name().span()),
            node_type: Some("register"),
            missing_node_type: "fields",
        });
    }

    if error {
        None
    } else {
        let mut register = Register {
            cfg_attr: Cfg::default(),
            description: parse_description(node),
            name: name.unwrap(),
            address: address.unwrap().0,
            reset_value: reset_value.map(|(rv, _)| rv),
            repeat: repeat.map(|(r, _)| r),
            field_set: field_set.unwrap().0,
            ..Default::default()
        };

        if let Some((access, _)) = access {
            register.access = access;
        }
        if let Some((allow_address_overlap, _)) = allow_address_overlap {
            register.allow_address_overlap = allow_address_overlap;
        }

        Some(register)
    }
}

fn transform_command(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<Command> {
    let (name, name_span) =
        parse_single_string_entry(node, source_code.clone(), diagnostics, None, true);

    if name.is_none() && node.children().is_none() {
        // We only have a command keyword. No need for further diagnostics
        return None;
    }

    let mut allow_address_overlap = None;
    let mut address = None;
    let mut repeat = None;
    let mut field_set_in = None;
    let mut field_set_out = None;

    for child in node.iter_children() {
        match child.name().value().parse() {
            Ok(CommandField::AllowAddressOverlap) => {
                if let Some((_, span)) = allow_address_overlap {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                ensure_zero_entries(child, source_code.clone(), diagnostics);
                allow_address_overlap = Some(true).map(|val| (val, child.name().span()));
            }
            Ok(CommandField::Address) => {
                if let Some((_, span)) = address {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                address = parse_single_integer_entry(child, source_code.clone(), diagnostics)
                    .0
                    .map(|val| (val, child.name().span()));
            }
            Ok(CommandField::Repeat) => {
                if let Some((_, span)) = repeat {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                repeat = parse_repeat_entries(child, source_code.clone(), diagnostics)
                    .map(|val| (val, child.name().span()));
            }
            Ok(CommandField::FieldSetIn) => {
                if let Some((_, span)) = field_set_in {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                field_set_in = transform_field_set(child, source_code.clone(), diagnostics)
                    .map(|val| (val, child.name().span()));
            }
            Ok(CommandField::FieldSetOut) => {
                if let Some((_, span)) = field_set_out {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                field_set_out = transform_field_set(child, source_code.clone(), diagnostics)
                    .map(|val| (val, child.name().span()));
            }
            Err(()) => {
                diagnostics.add(errors::UnexpectedNode {
                    source_code: source_code.clone(),
                    node_name: child.name().span(),
                    expected_names: COMMAND_FIELDS.iter().map(|v| v.0).collect(),
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
        diagnostics.add(errors::MissingChildNode {
            source_code: source_code.clone(),
            node: name_span.unwrap_or(node.name().span()),
            node_type: Some("command"),
            missing_node_type: "address",
        });
    }

    if error {
        None
    } else {
        let mut command = Command {
            cfg_attr: Cfg::default(),
            description: parse_description(node),
            name: name.unwrap(),
            address: address.unwrap().0,
            repeat: repeat.map(|(r, _)| r),
            field_set_in: field_set_in.map(|(f, _)| f),
            field_set_out: field_set_out.map(|(f, _)| f),
            ..Default::default()
        };

        if let Some((allow_address_overlap, _)) = allow_address_overlap {
            command.allow_address_overlap = allow_address_overlap;
        }

        Some(command)
    }
}

fn transform_buffer(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<Buffer> {
    let (name, name_span) =
        parse_single_string_entry(node, source_code.clone(), diagnostics, None, true);

    if name.is_none() && node.children().is_none() {
        // We only have a buffer keyword. No need for further diagnostics
        return None;
    }

    let mut access = None;
    let mut address = None;

    for child in node.iter_children() {
        match child.name().value().parse() {
            Ok(BufferField::Access) => {
                if let Some((_, span)) = access {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                access =
                    parse_single_string_value::<Access>(child, source_code.clone(), diagnostics)
                        .map(|val| (val, child.name().span()));
            }
            Ok(BufferField::Address) => {
                if let Some((_, span)) = address {
                    diagnostics.add(errors::DuplicateNode {
                        source_code: source_code.clone(),
                        duplicate: child.name().span(),
                        original: span,
                    });
                    continue;
                }

                address = parse_single_integer_entry(child, source_code.clone(), diagnostics)
                    .0
                    .map(|val| (val, child.name().span()));
            }
            Err(()) => {
                diagnostics.add(errors::UnexpectedNode {
                    source_code: source_code.clone(),
                    node_name: child.name().span(),
                    expected_names: BUFFER_FIELDS.iter().map(|v| v.0).collect(),
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
        diagnostics.add(errors::MissingChildNode {
            source_code: source_code.clone(),
            node: name_span.unwrap_or(node.name().span()),
            node_type: Some("register"),
            missing_node_type: "address",
        });
    }

    if error {
        None
    } else {
        let mut buffer = Buffer {
            cfg_attr: Cfg::default(),
            description: parse_description(node),
            name: name.unwrap(),
            address: address.unwrap().0,
            ..Default::default()
        };

        if let Some((access, _)) = access {
            buffer.access = access;
        }

        Some(buffer)
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
                parse_single_string_entry(node, source_code.clone(), diagnostics, None, false).0
            {
                device.global_config.name_word_boundaries =
                    convert_case::Boundary::list_from(&value);
            }
        }
        GlobalConfigType::DefmtFeature => {
            if let Some(value) =
                parse_single_string_entry(node, source_code.clone(), diagnostics, None, false).0
            {
                device.global_config.defmt_feature = Some(value);
            }
        }
    }
}

fn transform_field_set(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<FieldSet> {
    let mut field_set = FieldSet::default();

    let mut unexpected_entries = errors::UnexpectedEntries {
        source_code: source_code.clone(),
        superfluous_entries: Vec::new(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: Vec::new(),
        unexpected_anonymous_entries: Vec::new(),
    };

    let mut size_bits: Option<&kdl::KdlEntry> = None;
    let mut byte_order: Option<&kdl::KdlEntry> = None;
    let mut bit_order: Option<&kdl::KdlEntry> = None;
    let mut allow_bit_overlap: Option<&kdl::KdlEntry> = None;

    for entry in node.entries() {
        match entry.name().map(|n| n.value()) {
            Some("size-bits") => {
                if let Some(size_bits) = size_bits {
                    diagnostics.add(errors::DuplicateEntry {
                        source_code: source_code.clone(),
                        duplicate: entry.span(),
                        original: size_bits.span(),
                    });
                } else {
                    size_bits = Some(entry);
                }
            }
            Some("byte-order") => {
                if let Some(byte_order) = byte_order {
                    diagnostics.add(errors::DuplicateEntry {
                        source_code: source_code.clone(),
                        duplicate: entry.span(),
                        original: byte_order.span(),
                    });
                } else {
                    byte_order = Some(entry);
                }
            }
            Some("bit-order") => {
                if let Some(bit_order) = bit_order {
                    diagnostics.add(errors::DuplicateEntry {
                        source_code: source_code.clone(),
                        duplicate: entry.span(),
                        original: bit_order.span(),
                    });
                } else {
                    bit_order = Some(entry);
                }
            }
            Some("allow-bit-overlap") => {
                if let Some(allow_bit_overlap) = allow_bit_overlap {
                    diagnostics.add(errors::DuplicateEntry {
                        source_code: source_code.clone(),
                        duplicate: entry.span(),
                        original: allow_bit_overlap.span(),
                    });
                } else {
                    allow_bit_overlap = Some(entry);
                }
            }
            Some(_) => {
                unexpected_entries
                    .unexpected_name_entries
                    .push(entry.span());
            }
            None => {
                if entry.value().as_string() == Some("allow-bit-overlap") {
                    if let Some(allow_bit_overlap) = allow_bit_overlap {
                        diagnostics.add(errors::DuplicateEntry {
                            source_code: source_code.clone(),
                            duplicate: entry.span(),
                            original: allow_bit_overlap.span(),
                        });
                    } else {
                        allow_bit_overlap = Some(entry);
                    }
                } else {
                    unexpected_entries
                        .unexpected_anonymous_entries
                        .push(entry.span());
                }
            }
        }
    }

    if !unexpected_entries.is_empty() {
        diagnostics.add(unexpected_entries);
    }

    if let Some(size_bits) = size_bits {
        match size_bits.value() {
            KdlValue::Integer(sb) if (0..=u32::MAX as i128).contains(sb) => {
                field_set.size_bits = *sb as u32;
            }
            KdlValue::Integer(_) => {
                diagnostics.add(errors::ValueOutOfRange {
                    source_code: source_code.clone(),
                    value: size_bits.span(),
                    context: Some("size-bits is encoded as a u32"),
                    range: "0..2^32",
                });
            }
            _ => {
                diagnostics.add(errors::UnexpectedType {
                    source_code: source_code.clone(),
                    value_name: size_bits.span(),
                    expected_type: "integer",
                });
            }
        }
    } else {
        diagnostics.add(errors::MissingEntry {
            source_code: source_code.clone(),
            node_name: node.name().span(),
            expected_entries: vec!["size-bits"],
        });
    }

    if let Some(byte_order) = byte_order {
        match byte_order.value() {
            KdlValue::String(s) => match s.parse() {
                Ok(byte_order) => {
                    field_set.byte_order = Some(byte_order);
                }
                Err(_) => {
                    diagnostics.add(errors::UnexpectedValue {
                        source_code: source_code.clone(),
                        value_name: byte_order.span(),
                        expected_values: BitOrder::VARIANTS.to_vec(),
                    });
                }
            },
            _ => {
                diagnostics.add(errors::UnexpectedType {
                    source_code: source_code.clone(),
                    value_name: byte_order.span(),
                    expected_type: "string",
                });
            }
        }
    }

    if let Some(bit_order) = bit_order {
        match bit_order.value() {
            KdlValue::String(s) => match s.parse() {
                Ok(bit_order) => {
                    field_set.bit_order = bit_order;
                }
                Err(_) => {
                    diagnostics.add(errors::UnexpectedValue {
                        source_code: source_code.clone(),
                        value_name: bit_order.span(),
                        expected_values: ByteOrder::VARIANTS.to_vec(),
                    });
                }
            },
            _ => {
                diagnostics.add(errors::UnexpectedType {
                    source_code: source_code.clone(),
                    value_name: bit_order.span(),
                    expected_type: "string",
                });
            }
        }
    }

    if let Some(allow_bit_overlap) = allow_bit_overlap {
        match allow_bit_overlap.value() {
            KdlValue::Bool(b) => {
                field_set.allow_bit_overlap = *b;
            }
            KdlValue::String(s) if s == "allow-bit-overlap" => {
                field_set.allow_bit_overlap = true;
            }
            _ => {
                diagnostics.add(errors::UnexpectedType {
                    source_code: source_code.clone(),
                    value_name: allow_bit_overlap.span(),
                    expected_type: "bool",
                });
            }
        }
    }

    for field_node in node.iter_children() {
        if let Some(field) = transform_field(field_node, source_code.clone(), diagnostics) {
            field_set.fields.push(field);
        }
    }

    Some(field_set)
}

#[allow(clippy::collapsible_else_if)]
fn transform_field(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<Field> {
    let mut unexpected_entries = UnexpectedEntries {
        source_code: source_code.clone(),
        superfluous_entries: Vec::new(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: Vec::new(),
        unexpected_anonymous_entries: Vec::new(),
    };

    let mut address = None;
    let mut access = None;

    for entry in node.entries() {
        if entry.name().is_some() {
            unexpected_entries.not_anonymous_entries.push(entry.span());
            continue;
        }

        match entry.value() {
            KdlValue::String(s) if s.starts_with("@") => {
                if let Some((_, span)) = address {
                    diagnostics.add(errors::DuplicateEntry {
                        source_code: source_code.clone(),
                        duplicate: entry.span(),
                        original: span,
                    });
                    continue;
                }

                let trimmed_string = s.trim_start_matches("@");

                if let Some((end, start)) = trimmed_string.split_once(":") {
                    if let Ok(end) = end.parse::<u32>()
                        && let Ok(start) = start.parse::<u32>()
                    {
                        address = Some((start..end + 1, entry.span()));
                    } else {
                        diagnostics.add(errors::BadValueFormat {
                            source_code: source_code.clone(),
                            span: entry.span(),
                            expected_format: "@<u32>:<u32>",
                            example: "@7:0",
                        });
                    }
                } else {
                    if let Ok(addr) = trimmed_string.parse::<u32>() {
                        address = Some((addr..addr + 1, entry.span()));
                    } else {
                        diagnostics.add(errors::BadValueFormat {
                            source_code: source_code.clone(),
                            span: entry.span(),
                            expected_format: "@<u32>",
                            example: "@10",
                        });
                    }
                }
            }
            KdlValue::String(s) if s.parse::<Access>().is_ok() => {
                if let Some((_, span)) = access {
                    diagnostics.add(errors::DuplicateEntry {
                        source_code: source_code.clone(),
                        duplicate: entry.span(),
                        original: span,
                    });
                    continue;
                }

                access = Some((s.parse().unwrap(), entry.span()));
            }
            KdlValue::String(_) => {
                diagnostics.add(errors::UnexpectedValue {
                    source_code: source_code.clone(),
                    value_name: entry.span(),
                    expected_values: ["@<u32>", "@<u32>:<u32>"]
                        .iter()
                        .chain(Access::VARIANTS)
                        .copied()
                        .collect(),
                });
            }
            _ => {
                diagnostics.add(errors::UnexpectedType {
                    source_code: source_code.clone(),
                    value_name: entry.span(),
                    expected_type: "string",
                });
            }
        }
    }

    if !unexpected_entries.is_empty() {
        diagnostics.add(unexpected_entries);
    }

    let (base_type, mut field_conversion) =
        parse_field_type(node.ty(), source_code.clone(), diagnostics);

    if let Some(variants) = node.children() {
        if let Some(field_conversion) = field_conversion.as_mut() {
            // This is an enum, change the field conversion with that info
            let variants = transform_enum_variants(variants, source_code.clone(), diagnostics);
            *field_conversion = FieldConversion::Enum {
                enum_value: Enum::new(
                    // Take the description of the field
                    parse_description(node),
                    field_conversion.type_name().into(),
                    variants,
                ),
                use_try: field_conversion.use_try(),
            };
        } else {
            diagnostics.add(errors::InlineEnumDefinitionWithoutName {
                source_code: source_code.clone(),
                field_name: node.name().span(),
                existing_ty: node.ty().map(|ty| ty.span()),
            });
        }
    }

    if address.is_none() {
        diagnostics.add(errors::MissingEntry {
            source_code: source_code.clone(),
            node_name: node.name().span(),
            expected_entries: vec!["address (\"@<u32>:<u32>\")"],
        });
        return None;
    }

    Some(Field {
        cfg_attr: Cfg::default(),
        description: parse_description(node),
        name: node.name().value().into(),
        access: access.map(|(a, _)| a).unwrap_or_default(),
        base_type,
        field_conversion,
        field_address: address.unwrap().0,
    })
}

fn parse_field_type(
    ty: Option<&KdlIdentifier>,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> (BaseType, Option<FieldConversion>) {
    let Some(ty) = ty else {
        return (BaseType::Unspecified, None);
    };

    let ty_str = ty.value();
    let mut field_conversion = None;
    let base_type_str;

    if let Some((base_type, conversion)) = ty_str.split_once(":") {
        base_type_str = base_type;

        field_conversion = Some(FieldConversion::Direct {
            type_name: conversion.trim_end_matches('?').into(),
            use_try: conversion.ends_with('?'),
        })
    } else {
        base_type_str = ty_str;
    }

    let base_type = match base_type_str {
        "bool" => BaseType::Bool,
        "uint" => BaseType::Uint,
        "int" => BaseType::Int,
        s if s.parse::<Integer>().is_ok() => BaseType::FixedSize(s.parse().unwrap()),
        "" => BaseType::Unspecified,
        _ => {
            diagnostics.add(errors::UnexpectedValue {
                source_code,
                value_name: ty.span(),
                expected_values: ["bool", "uint", "int"]
                    .iter()
                    .chain(Integer::VARIANTS)
                    .chain(["<base>:<target>", "<base>:<target>?"].iter())
                    .copied()
                    .collect(),
            });
            BaseType::Unspecified
        }
    };

    (base_type, field_conversion)
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
            unexpected_anonymous_entries: Vec::new(),
        });
    }
}

fn parse_repeat_entries(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<Repeat> {
    let mut unexpected_entries = errors::UnexpectedEntries {
        source_code: source_code.clone(),
        superfluous_entries: Vec::new(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: Vec::new(),
        unexpected_anonymous_entries: Vec::new(),
    };

    let mut count = None;
    let mut stride = None;

    for entry in node.entries() {
        match (entry.name().map(|id| id.value()), entry.value()) {
            (Some("count"), KdlValue::Integer(val)) => count = Some((*val, entry.span())),
            (Some("stride"), KdlValue::Integer(val)) => stride = Some(*val),
            (Some("count") | Some("stride"), _) => diagnostics.add(errors::UnexpectedType {
                source_code: source_code.clone(),
                value_name: entry.span(),
                expected_type: "integer",
            }),
            (Some(_), _) => {
                unexpected_entries
                    .unexpected_name_entries
                    .push(entry.span());
            }
            (None, _) => {
                unexpected_entries.superfluous_entries.push(entry.span());
            }
        }
    }

    if !unexpected_entries.is_empty() {
        diagnostics.add(unexpected_entries);
    }

    let mut error = false;

    if count.is_none() && stride.is_none() {
        error = true;
        diagnostics.add(errors::MissingEntry {
            source_code: source_code.clone(),
            node_name: node.name().span(),
            expected_entries: vec!["count=<integer>", "stride=<integer>"],
        });
    } else if count.is_none() {
        error = true;
        diagnostics.add(errors::MissingEntry {
            source_code: source_code.clone(),
            node_name: node.name().span(),
            expected_entries: vec!["count=<integer>"],
        });
    } else if stride.is_none() {
        error = true;
        diagnostics.add(errors::MissingEntry {
            source_code: source_code.clone(),
            node_name: node.name().span(),
            expected_entries: vec!["stride=<integer>"],
        });
    }

    if let Some((count, span)) = count
        && !(0..=u64::MAX as i128).contains(&count)
    {
        error = true;
        diagnostics.add(errors::ValueOutOfRange {
            source_code: source_code.clone(),
            value: span,
            context: Some("The count is encoded as a u64"),
            range: "0..2^64",
        });
    }

    if error {
        None
    } else {
        Some(Repeat {
            count: count.unwrap().0 as u64,
            stride: stride.unwrap(),
        })
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
                range: "0..",
                context: Some("Negative reset values are not allowed"),
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
                    range: "0..256",
                    context: Some(
                        "When specifying the reset values as an array, all numbers must be bytes",
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
        unexpected_anonymous_entries: Vec::new(),
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
    is_name: bool,
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
        unexpected_anonymous_entries: Vec::new(),
    };
    if !unexpected_entries.is_empty() {
        diagnostics.add(unexpected_entries);
    }

    match node.entries().first() {
        Some(entry) if entry.name().is_none() => match entry.value() {
            KdlValue::String(val) => (Some(val.clone()), Some(entry.span())),
            _ => {
                if !is_name {
                    diagnostics.add(errors::UnexpectedType {
                        source_code,
                        value_name: entry.span(),
                        expected_type: "string",
                    });
                } else {
                    diagnostics.add(errors::MissingObjectName {
                        source_code,
                        object_keyword: node.name().span(),
                        object_type: node.name().value().into(),
                        found_instead: Some(entry.span()),
                    });
                }
                (None, Some(entry.span()))
            }
        },
        _ => {
            if !is_name {
                diagnostics.add(errors::MissingEntry {
                    source_code,
                    node_name: node.name().span(),
                    expected_entries: expected_entries
                        .map(|ee| ee.to_vec())
                        .unwrap_or_else(|| vec!["string"]),
                });
            } else {
                diagnostics.add(errors::MissingObjectName {
                    source_code,
                    object_keyword: node.name().span(),
                    object_type: node.name().value().into(),
                    found_instead: None,
                });
            }
            (None, None)
        }
    }
}

fn parse_single_string_value<T: strum::VariantNames + FromStr>(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<T> {
    match parse_single_string_entry(
        node,
        source_code.clone(),
        diagnostics,
        Some(T::VARIANTS),
        false,
    ) {
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

fn parse_description(node: &KdlNode) -> String {
    if let Some(format) = node.format() {
        format
            .leading
            .lines()
            .filter(|line| line.starts_with("///"))
            .map(|line| line.trim_start_matches("///").trim())
            .join("\n")
    } else {
        Default::default()
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
const COMMAND_FIELDS: &[(&str, CommandField)] = &[
    ("allow-address-overlap", CommandField::AllowAddressOverlap),
    ("address", CommandField::Address),
    ("repeat", CommandField::Repeat),
    ("in", CommandField::FieldSetIn),
    ("out", CommandField::FieldSetOut),
];
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum CommandField {
    AllowAddressOverlap,
    Address,
    Repeat,
    FieldSetIn,
    FieldSetOut,
}

impl FromStr for CommandField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (name, val) in COMMAND_FIELDS {
            if *name == s {
                return Ok(*val);
            }
        }

        Err(())
    }
}

#[rustfmt::skip]
const BUFFER_FIELDS: &[(&str, BufferField)] = &[
    ("access", BufferField::Access),
    ("address", BufferField::Address),
];
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum BufferField {
    Access,
    Address,
}

impl FromStr for BufferField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (name, val) in BUFFER_FIELDS {
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
];
#[derive(Clone, Copy)]
enum ObjectType {
    Block,
    Register,
    Command,
    Buffer,
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

fn transform_enum_variants(
    nodes: &KdlDocument,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Vec<EnumVariant> {
    nodes
        .nodes()
        .iter()
        .filter_map(|node| {
            let variant_name = node.name();

            let variant_value = match node.entries().len() {
                0 => None,
                1 if node.entries()[0].name().is_none() => Some(&node.entries()[0]),
                _ => {
                    diagnostics.add(errors::UnexpectedEntries {
                        source_code: source_code.clone(),
                        superfluous_entries: node
                            .entries()
                            .get(1..)
                            .map(|superfluous_entries| {
                                superfluous_entries
                                    .iter()
                                    .map(|entry| entry.span())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        unexpected_name_entries: Vec::new(),
                        not_anonymous_entries: node
                            .entries()
                            .first()
                            .into_iter()
                            .filter_map(|entry| entry.name().is_some().then_some(entry.span()))
                            .collect(),
                        unexpected_anonymous_entries: Vec::new(),
                    });
                    Some(&node.entries()[0])
                }
            };

            let variant_value = match variant_value {
                Some(variant_value) => match variant_value.value() {
                    KdlValue::String(val) if val == "default" => EnumValue::Default,
                    KdlValue::String(val) if val == "catch-all" => EnumValue::CatchAll,
                    KdlValue::Integer(val) => EnumValue::Specified(*val),
                    _ => {
                        diagnostics.add(errors::UnexpectedValue {
                            source_code: source_code.clone(),
                            value_name: variant_value.span(),
                            expected_values: vec!["", "<integer>", "default", "catch-all"],
                        });
                        return None;
                    }
                },
                None => EnumValue::Unspecified,
            };

            Some(EnumVariant {
                cfg_attr: Cfg::default(),
                description: parse_description(node),
                name: variant_name.value().to_string(),
                value: variant_value,
            })
        })
        .collect()
}
