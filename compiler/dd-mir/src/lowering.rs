#![allow(unused_assignments)]

use std::{collections::HashMap, str::FromStr};

use convert_case::Boundary;
use device_driver_common::{
    identifier::{Identifier, IdentifierRef},
    span::{SpanExt, Spanned},
    specifiers::{
        Access, BaseType, BitOrder, ByteOrder, Integer, Repeat, RepeatSource, ResetValue,
        TypeConversion, VariantNames,
    },
};
use itertools::Itertools;
use kdl::{KdlDiagnostic, KdlDocument, KdlEntry, KdlIdentifier, KdlNode, KdlValue};
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::model::{
    Block, Buffer, Command, Device, DeviceConfig, Enum, EnumValue, EnumVariant, Extern, Field,
    FieldSet, Manifest, Object, Register, Unique,
};
use device_driver_diagnostics::{
    Diagnostics,
    errors::{self, InvalidIdentifier, UnexpectedEntries},
};

pub fn transform(
    file_contents: &str,
    source_span: Option<SourceSpan>,
    diagnostics: &mut Diagnostics,
) -> Manifest {
    let file_subslice = if let Some(span) = source_span {
        file_contents
            .get(span.offset()..span.offset() + span.len())
            .unwrap()
    } else {
        file_contents
    };

    let mut document = match kdl::KdlDocument::parse(file_subslice) {
        Ok(document) => document,
        Err(e) => {
            for diagnostic in e.diagnostics {
                diagnostics.add_miette(ConvertedKdlDiagnostic::from_original_and_span(
                    diagnostic,
                    source_span,
                ));
            }
            return Manifest {
                root_objects: Vec::new(),
                config: DeviceConfig::default(),
            };
        }
    };

    if let Some(source_span) = source_span {
        change_document_span(&mut document, &source_span);
    }

    transform_manifest(&document, diagnostics)
}

fn transform_manifest(manifest_document: &KdlDocument, diagnostics: &mut Diagnostics) -> Manifest {
    let mut manifest = Manifest {
        root_objects: Vec::new(),
        config: DeviceConfig::default(), // TODO: Parse this
    };

    for node in manifest_document.nodes() {
        if let Ok(root_object_type) = node.name().value().parse::<RootObjectType>() {
            match root_object_type {
                RootObjectType::Device => {
                    let Some(device) = transform_device(node, diagnostics) else {
                        continue;
                    };
                    manifest.root_objects.push(Object::Device(device));
                }
                RootObjectType::FieldSet => {
                    let (fs, enums) = transform_field_set(node, diagnostics, None);
                    if let Some(fs) = fs {
                        manifest.root_objects.push(Object::FieldSet(fs));
                    }
                    manifest
                        .root_objects
                        .extend(enums.into_iter().map(Object::Enum));
                }
                RootObjectType::Enum => {
                    if let Some(enum_value) = transform_enum(node, diagnostics) {
                        manifest.root_objects.push(Object::Enum(enum_value));
                    }
                }
                RootObjectType::Extern => {
                    if let Some(extern_value) = transform_extern(node, diagnostics) {
                        manifest.root_objects.push(Object::Extern(extern_value));
                    }
                }
            }
        } else {
            diagnostics.add_miette(errors::UnexpectedNode {
                node_name: node.name().span().into(),
                expected_names: ROOT_OBJECT_TYPES.iter().map(|v| v.0).collect(),
            });
        }
    }

    manifest
}

fn transform_device(node: &KdlNode, diagnostics: &mut Diagnostics) -> Option<Device> {
    let (device_name, device_name_span) = parse_single_string_entry(node, diagnostics, None, true);
    let (device_name, device_name_span) = (device_name?, device_name_span?);

    let device_name = match Identifier::try_parse(&device_name) {
        Ok(id) => id,
        Err(e) => {
            diagnostics.add(InvalidIdentifier::new(e, device_name_span.into()));
            return None;
        }
    };

    let mut device = Device {
        description: parse_description(node),
        name: device_name.with_span(device_name_span),
        device_config: DeviceConfig::default(),
        objects: Vec::new(),
        span: node.span().into(),
    };

    device.device_config.owner = Some(device.id());

    if let Some(device_document) = node.children()
        && !device_document.nodes().is_empty()
    {
        transform_device_internals(&mut device, device_document, diagnostics);
    } else {
        diagnostics.add_miette(errors::EmptyNode {
            node: node.span().into(),
        });
    }

    Some(device)
}

fn transform_device_internals(
    device: &mut Device,
    device_document: &KdlDocument,
    diagnostics: &mut Diagnostics,
) {
    let mut seen_device_configs = HashMap::<DeviceConfigType, SourceSpan>::new();

    for node in device_document.nodes() {
        if let Ok(device_config_type) = node.name().value().parse::<DeviceConfigType>() {
            match seen_device_configs.insert(device_config_type, node.span()) {
                None => transform_device_config_node(device, node, device_config_type, diagnostics),
                Some(original_node) => {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: node.span().into(),
                        original: original_node.into(),
                    });
                }
            }
        } else if let Ok(object_type) = node.name().value().parse::<ObjectType>() {
            for object in transform_object(node, diagnostics, object_type) {
                device.objects.push(object);
            }
        } else {
            diagnostics.add_miette(errors::UnexpectedNode {
                node_name: node.name().span().into(),
                expected_names: DEVICE_CONFIG_TYPES
                    .iter()
                    .map(|v| v.0)
                    .chain(OBJECT_TYPES.iter().map(|v| v.0))
                    .collect(),
            });
        }
    }
}

fn transform_object(
    node: &KdlNode,
    diagnostics: &mut Diagnostics,
    object_type: ObjectType,
) -> Vec<Object> {
    match object_type {
        ObjectType::Block => transform_block(node, diagnostics)
            .map(Object::Block)
            .into_iter()
            .collect(),
        ObjectType::Register => {
            let (register, fieldset, enums) = transform_register(node, diagnostics);
            register
                .map(Object::Register)
                .into_iter()
                .chain(fieldset.map(Object::FieldSet))
                .chain(enums.into_iter().map(Object::Enum))
                .collect()
        }
        ObjectType::Command => {
            let (command, field_sets, enums) = transform_command(node, diagnostics);
            command
                .map(Object::Command)
                .into_iter()
                .chain(field_sets.into_iter().map(Object::FieldSet))
                .chain(enums.into_iter().map(Object::Enum))
                .collect()
        }
        ObjectType::Buffer => transform_buffer(node, diagnostics)
            .map(Object::Buffer)
            .into_iter()
            .collect(),
        ObjectType::FieldSet => {
            let (fs, enums) = transform_field_set(node, diagnostics, None);
            fs.map(Object::FieldSet)
                .into_iter()
                .chain(enums.into_iter().map(Object::Enum))
                .collect()
        }
        ObjectType::Enum => transform_enum(node, diagnostics)
            .map(Object::Enum)
            .into_iter()
            .collect(),
        ObjectType::Extern => transform_extern(node, diagnostics)
            .map(Object::Extern)
            .into_iter()
            .collect(),
    }
}

fn transform_block(node: &KdlNode, diagnostics: &mut Diagnostics) -> Option<Block> {
    let (Some(name), Some(name_span)) = parse_single_string_entry(node, diagnostics, None, true)
    else {
        return None;
    };

    let name = match Identifier::try_parse(&name) {
        Ok(id) => id,
        Err(e) => {
            diagnostics.add(InvalidIdentifier::new(e, name_span.into()));
            return None;
        }
    };

    let mut block_objects = Vec::new();
    let mut offset = None;
    let mut repeat = None;

    for child in node.iter_children() {
        if let Ok(buffer_field) = child.name().value().parse::<BlockField>() {
            match buffer_field {
                BlockField::Offset => {
                    if let Some((_, span)) = offset {
                        diagnostics.add_miette(errors::DuplicateNode {
                            duplicate: child.name().span().into(),
                            original: span,
                        });
                        continue;
                    }

                    offset = parse_single_integer_entry(child, diagnostics)
                        .0
                        .map(|val| (val, child.name().span().into()));
                }
                BlockField::Repeat => {
                    if let Some((_, span)) = repeat {
                        diagnostics.add_miette(errors::DuplicateNode {
                            duplicate: child.name().span().into(),
                            original: span,
                        });
                        continue;
                    }

                    repeat = parse_repeat_entries(child, diagnostics, true)
                        .map(|val| (val, child.name().span().into()));
                }
            }
        } else if let Ok(object_type) = child.name().value().parse::<ObjectType>() {
            for object in transform_object(child, diagnostics, object_type) {
                block_objects.push(object);
            }
        } else {
            diagnostics.add_miette(errors::UnexpectedNode {
                node_name: child.name().span().into(),
                expected_names: BLOCK_FIELDS
                    .iter()
                    .map(|v| v.0)
                    .chain(OBJECT_TYPES.iter().map(|v| v.0))
                    .collect(),
            });
        }
    }

    Some(Block {
        description: parse_description(node),
        name: (name, name_span).into(),
        address_offset: offset.unwrap_or((0, name_span.into())).into(),
        repeat: repeat.map(|(r, _)| r),
        objects: block_objects,
        span: node.span().into(),
    })
}

fn transform_register(
    node: &KdlNode,
    diagnostics: &mut Diagnostics,
) -> (Option<Register>, Option<FieldSet>, Vec<Enum>) {
    let (Some(name), Some(name_span)) = parse_single_string_entry(node, diagnostics, None, true)
    else {
        return (None, None, Vec::new());
    };

    let name = match Identifier::try_parse(&name) {
        Ok(id) => id,
        Err(e) => {
            diagnostics.add(InvalidIdentifier::new(e, name_span.into()));
            return (None, None, Vec::new());
        }
    };

    let mut inline_enums = Vec::new();

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
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                access = parse_single_string_value::<Access>(child, diagnostics)
                    .map(|val| (val, child.name().span().into()));
            }
            Ok(RegisterField::AllowAddressOverlap) => {
                if let Some((_, span)) = allow_address_overlap {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                ensure_zero_entries(child, diagnostics);
                allow_address_overlap = Some(true).map(|val| (val, child.name().span().into()));
            }
            Ok(RegisterField::Address) => {
                if let Some((_, span)) = address {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                address = parse_single_integer_entry(child, diagnostics)
                    .0
                    .map(|val| (val, child.name().span().into()));
            }
            Ok(RegisterField::ResetValue) => {
                if let Some((_, span)) = reset_value {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                reset_value = parse_reset_value_entries(child, diagnostics)
                    .map(|val| (val, child.name().span().into()));
            }
            Ok(RegisterField::Repeat) => {
                if let Some((_, span)) = repeat {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                repeat = parse_repeat_entries(child, diagnostics, true)
                    .map(|val| (val, child.name().span().into()));
            }
            Ok(RegisterField::FieldSet) => {
                if let Some((_, span)) = field_set {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                let (fs, mut enums) = transform_field_set(
                    child,
                    diagnostics,
                    Some(
                        name.clone().concat(
                            Identifier::try_parse("field_set")
                                .unwrap()
                                .apply_boundaries(&[Boundary::Underscore]),
                        ),
                    ),
                );

                field_set = fs.map(|val| (val, child.name().span().into()));
                inline_enums.append(&mut enums);
            }
            Err(()) => {
                diagnostics.add_miette(errors::UnexpectedNode {
                    node_name: child.name().span().into(),
                    expected_names: REGISTER_FIELDS.iter().map(|v| v.0).collect(),
                });
            }
        }
    }

    let mut error = false;

    if address.is_none() {
        error = true;
        diagnostics.add_miette(errors::MissingChildNode {
            node: name_span.into(),
            node_type: Some("register"),
            missing_node_type: "address",
        });
    }

    if field_set.is_none() {
        error = true;
        diagnostics.add_miette(errors::MissingChildNode {
            node: name_span.into(),
            node_type: Some("register"),
            missing_node_type: "fields",
        });
    }

    if error {
        (None, field_set.map(|(fs, _)| fs), inline_enums)
    } else {
        let mut register = Register {
            description: parse_description(node),
            name: (name, name_span).into(),
            address: address.unwrap().into(),
            reset_value: reset_value.map(Into::into),
            repeat: repeat.map(|(r, _)| r),
            field_set_ref: field_set.as_ref().unwrap().0.name.take_ref(),
            access: Default::default(),
            allow_address_overlap: Default::default(),
            span: node.span().into(),
        };

        if let Some((access, _)) = access {
            register.access = access.value;
        }
        if let Some((allow_address_overlap, _)) = allow_address_overlap {
            register.allow_address_overlap = allow_address_overlap;
        }

        (Some(register), Some(field_set.unwrap().0), inline_enums)
    }
}

fn transform_command(
    node: &KdlNode,
    diagnostics: &mut Diagnostics,
) -> (Option<Command>, Vec<FieldSet>, Vec<Enum>) {
    let (Some(name), Some(name_span)) = parse_single_string_entry(node, diagnostics, None, true)
    else {
        return (None, Vec::new(), Vec::new());
    };

    let name = match Identifier::try_parse(&name) {
        Ok(id) => id,
        Err(e) => {
            diagnostics.add(InvalidIdentifier::new(e, name_span.into()));
            return (None, Vec::new(), Vec::new());
        }
    };

    let mut inline_enums = Vec::new();

    let mut allow_address_overlap = None;
    let mut address = None;
    let mut repeat = None;
    let mut field_set_in = None;
    let mut field_set_out = None;

    for child in node.iter_children() {
        match child.name().value().parse() {
            Ok(CommandField::AllowAddressOverlap) => {
                if let Some((_, span)) = allow_address_overlap {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                ensure_zero_entries(child, diagnostics);
                allow_address_overlap = Some(true).map(|val| (val, child.name().span().into()));
            }
            Ok(CommandField::Address) => {
                if let Some((_, span)) = address {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                address = parse_single_integer_entry(child, diagnostics)
                    .0
                    .map(|val| (val, child.name().span().into()));
            }
            Ok(CommandField::Repeat) => {
                if let Some((_, span)) = repeat {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                repeat = parse_repeat_entries(child, diagnostics, true)
                    .map(|val| (val, child.name().span().into()));
            }
            Ok(CommandField::FieldSetIn) => {
                if let Some((_, span)) = field_set_in {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                let (fs, mut enums) = transform_field_set(
                    child,
                    diagnostics,
                    Some(
                        name.clone().concat(
                            Identifier::try_parse("field_set_in")
                                .unwrap()
                                .apply_boundaries(&[Boundary::Underscore]),
                        ),
                    ),
                );

                field_set_in = fs.map(|val| (val, child.name().span().into()));
                inline_enums.append(&mut enums);
            }
            Ok(CommandField::FieldSetOut) => {
                if let Some((_, span)) = field_set_out {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                let (fs, mut enums) = transform_field_set(
                    child,
                    diagnostics,
                    Some(
                        name.clone().concat(
                            Identifier::try_parse("field_set_out")
                                .unwrap()
                                .apply_boundaries(&[Boundary::Underscore]),
                        ),
                    ),
                );

                field_set_out = fs.map(|val| (val, child.name().span().into()));
                inline_enums.append(&mut enums);
            }
            Err(()) => {
                diagnostics.add_miette(errors::UnexpectedNode {
                    node_name: child.name().span().into(),
                    expected_names: COMMAND_FIELDS.iter().map(|v| v.0).collect(),
                });
            }
        }
    }

    let mut error = false;

    if address.is_none() {
        error = true;
        diagnostics.add_miette(errors::MissingChildNode {
            node: name_span.into(),
            node_type: Some("command"),
            missing_node_type: "address",
        });
    }

    if error {
        (
            None,
            [field_set_in, field_set_out]
                .into_iter()
                .filter_map(|fs| Some(fs?.0))
                .collect(),
            inline_enums,
        )
    } else {
        let mut command = Command {
            description: parse_description(node),
            name: (name, name_span).into(),
            address: address.unwrap().into(),
            allow_address_overlap: Default::default(),
            repeat: repeat.map(|(r, _)| r),
            field_set_ref_in: field_set_in.as_ref().map(|(f, _)| f.name.take_ref()),
            field_set_ref_out: field_set_out.as_ref().map(|(f, _)| f.name.take_ref()),
            span: node.span().into(),
        };

        if let Some((allow_address_overlap, _)) = allow_address_overlap {
            command.allow_address_overlap = allow_address_overlap;
        }

        (
            Some(command),
            [field_set_in, field_set_out]
                .into_iter()
                .filter_map(|fs| Some(fs?.0))
                .collect(),
            inline_enums,
        )
    }
}

fn transform_buffer(node: &KdlNode, diagnostics: &mut Diagnostics) -> Option<Buffer> {
    let (Some(name), Some(name_span)) = parse_single_string_entry(node, diagnostics, None, true)
    else {
        return None;
    };

    let name = match Identifier::try_parse(&name) {
        Ok(id) => id,
        Err(e) => {
            diagnostics.add(InvalidIdentifier::new(e, name_span.into()));
            return None;
        }
    };

    let mut access = None;
    let mut address = None;

    for child in node.iter_children() {
        match child.name().value().parse() {
            Ok(BufferField::Access) => {
                if let Some((_, span)) = access {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                access = parse_single_string_value::<Access>(child, diagnostics)
                    .map(|val| (val, child.name().span().into()));
            }
            Ok(BufferField::Address) => {
                if let Some((_, span)) = address {
                    diagnostics.add_miette(errors::DuplicateNode {
                        duplicate: child.name().span().into(),
                        original: span,
                    });
                    continue;
                }

                address = parse_single_integer_entry(child, diagnostics)
                    .0
                    .map(|val| (val, child.name().span().into()));
            }
            Err(()) => {
                diagnostics.add_miette(errors::UnexpectedNode {
                    node_name: child.name().span().into(),
                    expected_names: BUFFER_FIELDS.iter().map(|v| v.0).collect(),
                });
            }
        }
    }

    let mut error = false;

    if address.is_none() {
        error = true;
        diagnostics.add_miette(errors::MissingChildNode {
            node: name_span.into(),
            node_type: Some("register"),
            missing_node_type: "address",
        });
    }

    if error {
        None
    } else {
        let mut buffer = Buffer {
            description: parse_description(node),
            name: (name, name_span).into(),
            access: Default::default(),
            address: address.unwrap().into(),
            span: node.span().into(),
        };

        if let Some((access, _)) = access {
            buffer.access = access.value;
        }

        Some(buffer)
    }
}

fn transform_device_config_node(
    device: &mut Device,
    node: &KdlNode,
    device_config_type: DeviceConfigType,
    diagnostics: &mut Diagnostics,
) {
    match device_config_type {
        DeviceConfigType::RegisterAccess => {
            if let Some(value) = parse_single_string_value(node, diagnostics) {
                device.device_config.register_access = Some(value.value);
            }
        }
        DeviceConfigType::FieldAccess => {
            if let Some(value) = parse_single_string_value(node, diagnostics) {
                device.device_config.field_access = Some(value.value);
            }
        }
        DeviceConfigType::BufferAccess => {
            if let Some(value) = parse_single_string_value(node, diagnostics) {
                device.device_config.buffer_access = Some(value.value);
            }
        }
        DeviceConfigType::ByteOrder => {
            if let Some(value) = parse_single_string_value(node, diagnostics) {
                device.device_config.byte_order = Some(value.value);
            }
        }
        DeviceConfigType::BitOrder => {
            if let Some(value) = parse_single_string_value(node, diagnostics) {
                device.device_config.bit_order = Some(value.value);
            }
        }
        DeviceConfigType::RegisterAddressType => {
            if let Some(value) = parse_single_string_value(node, diagnostics) {
                device.device_config.register_address_type = Some(value);
            }
        }
        DeviceConfigType::CommandAddressType => {
            if let Some(value) = parse_single_string_value(node, diagnostics) {
                device.device_config.command_address_type = Some(value);
            }
        }
        DeviceConfigType::BufferAddressType => {
            if let Some(value) = parse_single_string_value(node, diagnostics) {
                device.device_config.buffer_address_type = Some(value);
            }
        }
        DeviceConfigType::NameWordBoundaries => {
            if let Some(value) = parse_single_string_entry(node, diagnostics, None, false).0 {
                device.device_config.name_word_boundaries =
                    Some(convert_case::Boundary::defaults_from(&value));
            }
        }
        DeviceConfigType::DefmtFeature => {
            if let Some(value) = parse_single_string_entry(node, diagnostics, None, false).0 {
                device.device_config.defmt_feature = Some(value);
            }
        }
    }
}

fn transform_field_set(
    node: &KdlNode,
    diagnostics: &mut Diagnostics,
    default_name: Option<Identifier>,
) -> (Option<FieldSet>, Vec<Enum>) {
    let mut inline_enums = Vec::new();

    let mut field_set = FieldSet {
        description: parse_description(node),
        name: Default::default(),
        size_bits: Default::default(),
        byte_order: Default::default(),
        bit_order: Default::default(),
        allow_bit_overlap: Default::default(),
        fields: Default::default(),
        span: node.span().into(),
    };

    let mut unexpected_entries = errors::UnexpectedEntries {
        superfluous_entries: Vec::new(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: Vec::new(),
        unexpected_anonymous_entries: Vec::new(),
    };

    let mut name: Option<&kdl::KdlEntry> = None;
    let mut size_bits: Option<&kdl::KdlEntry> = None;
    let mut byte_order: Option<&kdl::KdlEntry> = None;
    let mut bit_order: Option<&kdl::KdlEntry> = None;
    let mut allow_bit_overlap: Option<&kdl::KdlEntry> = None;

    for (i, entry) in node.entries().iter().enumerate() {
        match entry.name().map(kdl::KdlIdentifier::value) {
            Some("size-bits") => {
                if let Some(size_bits) = size_bits {
                    diagnostics.add_miette(errors::DuplicateEntry {
                        duplicate: entry.span().into(),
                        original: size_bits.span().into(),
                    });
                } else {
                    size_bits = Some(entry);
                }
            }
            Some("byte-order") => {
                if let Some(byte_order) = byte_order {
                    diagnostics.add_miette(errors::DuplicateEntry {
                        duplicate: entry.span().into(),
                        original: byte_order.span().into(),
                    });
                } else {
                    byte_order = Some(entry);
                }
            }
            Some("bit-order") => {
                if let Some(bit_order) = bit_order {
                    diagnostics.add_miette(errors::DuplicateEntry {
                        duplicate: entry.span().into(),
                        original: bit_order.span().into(),
                    });
                } else {
                    bit_order = Some(entry);
                }
            }
            Some("allow-bit-overlap") => {
                if let Some(allow_bit_overlap) = allow_bit_overlap {
                    diagnostics.add_miette(errors::DuplicateEntry {
                        duplicate: entry.span().into(),
                        original: allow_bit_overlap.span().into(),
                    });
                } else {
                    allow_bit_overlap = Some(entry);
                }
            }
            Some(_) => {
                unexpected_entries
                    .unexpected_name_entries
                    .push(entry.span().into());
            }
            None => {
                if entry.value().as_string() == Some("allow-bit-overlap") {
                    if let Some(allow_bit_overlap) = allow_bit_overlap {
                        diagnostics.add_miette(errors::DuplicateEntry {
                            duplicate: entry.span().into(),
                            original: allow_bit_overlap.span().into(),
                        });
                    } else {
                        allow_bit_overlap = Some(entry);
                    }
                } else if i == 0 {
                    name = Some(entry);
                } else {
                    unexpected_entries
                        .unexpected_anonymous_entries
                        .push(entry.span().into());
                }
            }
        }
    }

    if !unexpected_entries.is_empty() {
        diagnostics.add_miette(unexpected_entries);
    }

    if let Some(name) = name {
        match name.value() {
            KdlValue::String(name_value) => {
                match Identifier::try_parse(name_value) {
                    Ok(id) => {
                        field_set.name = id.with_span(name.span());
                    }
                    Err(e) => {
                        diagnostics.add(InvalidIdentifier::new(e, name.span().into()));
                    }
                };
            }
            _ => {
                diagnostics.add_miette(errors::UnexpectedType {
                    value_name: name.span().into(),
                    expected_type: "string",
                });
            }
        }
    } else if let Some(default_name) = default_name {
        field_set.name = default_name.with_span((node.span().offset(), node.span().offset()));
    } else {
        diagnostics.add_miette(errors::MissingObjectName {
            object_keyword: node.name().span().into(),
            found_instead: None,
            object_type: node.name().value().into(),
        });
    }

    if let Some(size_bits) = size_bits {
        match size_bits.value() {
            KdlValue::Integer(sb) if (0..=i128::from(u32::MAX)).contains(sb) => {
                field_set.size_bits = (*sb as u32).with_span(size_bits.span());
            }
            KdlValue::Integer(_) => {
                diagnostics.add_miette(errors::ValueOutOfRange {
                    value: size_bits.span().into(),
                    context: Some("size-bits is encoded as a u32"),
                    range: "0..2^32",
                });
            }
            _ => {
                diagnostics.add_miette(errors::UnexpectedType {
                    value_name: size_bits.span().into(),
                    expected_type: "integer",
                });
            }
        }
    } else {
        diagnostics.add_miette(errors::MissingEntry {
            node_name: node.name().span().into(),
            expected_entries: vec!["size-bits=<integer>"],
        });
    }

    if let Some(byte_order) = byte_order {
        match byte_order.value() {
            KdlValue::String(s) => match s.parse() {
                Ok(byte_order) => {
                    field_set.byte_order = Some(byte_order);
                }
                Err(_) => {
                    diagnostics.add_miette(errors::UnexpectedValue {
                        value_name: byte_order.span().into(),
                        expected_values: ByteOrder::VARIANTS.to_vec(),
                    });
                }
            },
            _ => {
                diagnostics.add_miette(errors::UnexpectedType {
                    value_name: byte_order.span().into(),
                    expected_type: "string",
                });
            }
        }
    }

    if let Some(bit_order) = bit_order {
        match bit_order.value() {
            KdlValue::String(s) => match s.parse() {
                Ok(bit_order) => {
                    field_set.bit_order = Some(bit_order);
                }
                Err(_) => {
                    diagnostics.add_miette(errors::UnexpectedValue {
                        value_name: bit_order.span().into(),
                        expected_values: BitOrder::VARIANTS.to_vec(),
                    });
                }
            },
            _ => {
                diagnostics.add_miette(errors::UnexpectedType {
                    value_name: bit_order.span().into(),
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
                diagnostics.add_miette(errors::UnexpectedType {
                    value_name: allow_bit_overlap.span().into(),
                    expected_type: "bool",
                });
            }
        }
    }

    for field_node in node.iter_children() {
        let (field, inline_enum) = transform_field(field_node, diagnostics);

        if let Some(field) = field {
            field_set.fields.push(field);
        }
        if let Some(inline_enum) = inline_enum {
            inline_enums.push(inline_enum);
        }
    }

    if field_set.name.is_empty() {
        // No name is a major error state we can't let go
        (None, inline_enums)
    } else {
        (Some(field_set), inline_enums)
    }
}

#[allow(clippy::collapsible_else_if)]
fn transform_field(node: &KdlNode, diagnostics: &mut Diagnostics) -> (Option<Field>, Option<Enum>) {
    let mut inline_enum = None;

    let mut unexpected_entries = UnexpectedEntries {
        superfluous_entries: Vec::new(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: Vec::new(),
        unexpected_anonymous_entries: Vec::new(),
    };

    let mut address = None;
    let mut access = None;

    let repeat = parse_repeat_entries(node, diagnostics, false);

    for entry in node.entries() {
        match entry.name().map(kdl::KdlIdentifier::value) {
            // Ignore the repeat fields. they're parsed separately
            Some("count" | "with" | "stride") => continue,
            Some(_) => {
                unexpected_entries
                    .not_anonymous_entries
                    .push(entry.span().into());
                continue;
            }
            None => {}
        }

        match entry.value() {
            KdlValue::String(s) if s.starts_with('@') => {
                if let Some((_, span)) = address {
                    diagnostics.add_miette(errors::DuplicateEntry {
                        duplicate: entry.span().into(),
                        original: span,
                    });
                    continue;
                }

                let trimmed_string = s.trim_start_matches('@');

                if let Some((end, start)) = trimmed_string.split_once(':') {
                    if let Ok(end) = end.parse::<u32>()
                        && let Ok(start) = start.parse::<u32>()
                    {
                        if end >= start {
                            address = Some((start..end + 1, entry.span().into()));
                        } else {
                            diagnostics.add_miette(errors::AddressWrongOrder {
                                address_entry: entry.span().into(),
                                end,
                                start,
                            });
                        }
                    } else {
                        diagnostics.add_miette(errors::BadValueFormat {
                            span: entry.span().into(),
                            expected_format: "@<u32>:<u32>",
                            example: "@7:0",
                        });
                    }
                } else {
                    if let Ok(addr) = trimmed_string.parse::<u32>() {
                        address = Some((addr..addr + 1, entry.span().into()));
                    } else {
                        diagnostics.add_miette(errors::BadValueFormat {
                            span: entry.span().into(),
                            expected_format: "@<u32>",
                            example: "@10",
                        });
                    }
                }
            }
            KdlValue::String(s) if s.parse::<Access>().is_ok() => {
                if let Some((_, span)) = access {
                    diagnostics.add_miette(errors::DuplicateEntry {
                        duplicate: entry.span().into(),
                        original: span,
                    });
                    continue;
                }

                access = Some((s.parse().unwrap(), entry.span().into()));
            }
            KdlValue::String(_) => {
                diagnostics.add_miette(errors::UnexpectedValue {
                    value_name: entry.span().into(),
                    expected_values: [
                        "@<u32>",
                        "@<u32>:<u32>",
                        "count=<integer>",
                        "with=<string>",
                        "stride=<integer>",
                    ]
                    .iter()
                    .chain(Access::VARIANTS)
                    .copied()
                    .collect(),
                });
            }
            _ => {
                diagnostics.add_miette(errors::UnexpectedType {
                    value_name: entry.span().into(),
                    expected_type: "string",
                });
            }
        }
    }

    if !unexpected_entries.is_empty() {
        diagnostics.add_miette(unexpected_entries);
    }

    let (base_type, field_conversion) = parse_type(node.ty(), diagnostics);

    if let Some(variants) = node.children() {
        if let Some(field_conversion) = field_conversion.as_ref() {
            // This is an enum, change the field conversion with that info
            let variants = transform_enum_variants(variants, diagnostics);

            match Identifier::try_parse(field_conversion.type_name.original()) {
                Ok(enum_name) => {
                    inline_enum = Some(Enum::new(
                        // Take the description of the field
                        parse_description(node),
                        enum_name.with_span(field_conversion.type_name.span),
                        variants,
                        base_type,
                        address.as_ref().map(|(address, _)| address.len() as u32),
                        node.span().into(),
                    ));
                }
                Err(e) => {
                    diagnostics.add(InvalidIdentifier::new(e, field_conversion.type_name.span));
                }
            }
        } else {
            diagnostics.add_miette(errors::InlineEnumDefinitionWithoutName {
                field_name: node.name().span().into(),
                existing_ty: node.ty().map(|ty| ty.span().into()),
            });
        }
    }

    if address.is_none() {
        diagnostics.add_miette(errors::MissingEntry {
            node_name: node.name().span().into(),
            expected_entries: vec!["address (\"@<u32>:<u32>\")"],
        });
        return (None, inline_enum);
    }

    let name = match Identifier::try_parse(node.name().value()) {
        Ok(id) => id,
        Err(e) => {
            diagnostics.add(InvalidIdentifier::new(e, node.name().span().into()));
            return (None, inline_enum);
        }
    };

    (
        Some(Field {
            description: parse_description(node),
            name: (name, node.name().span()).into(),
            access: access.map(|(a, _)| a).unwrap_or_default(),
            base_type,
            field_conversion,
            field_address: address.unwrap().into(),
            repeat,
            span: node.span().into(),
        }),
        inline_enum,
    )
}

fn transform_enum(node: &KdlNode, diagnostics: &mut Diagnostics) -> Option<Enum> {
    let mut unexpected_entries = errors::UnexpectedEntries {
        superfluous_entries: Vec::new(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: Vec::new(),
        unexpected_anonymous_entries: Vec::new(),
    };

    let (base_type, field_conversion) = parse_type(node.ty(), diagnostics);

    if let Some(field_conversion) = field_conversion {
        diagnostics.add_miette(errors::OnlyBaseTypeAllowed {
            existing_ty: node.ty().unwrap().span().into(),
            field_conversion,
        });
    }

    let mut enum_value = Enum::new(
        parse_description(node),
        Identifier::default().with_dummy_span(),
        node.children()
            .map(|children| transform_enum_variants(children, diagnostics))
            .unwrap_or_default(),
        base_type,
        None,
        node.span().into(),
    );

    let mut name: Option<&kdl::KdlEntry> = None;
    let mut size_bits: Option<&kdl::KdlEntry> = None;

    for (i, entry) in node.entries().iter().enumerate() {
        match entry.name().map(kdl::KdlIdentifier::value) {
            Some("size-bits") => {
                if let Some(size_bits) = size_bits {
                    diagnostics.add_miette(errors::DuplicateEntry {
                        duplicate: entry.span().into(),
                        original: size_bits.span().into(),
                    });
                } else {
                    size_bits = Some(entry);
                }
            }
            Some(_) => {
                unexpected_entries
                    .unexpected_name_entries
                    .push(entry.span().into());
            }
            None => {
                if i == 0 {
                    name = Some(entry);
                } else {
                    unexpected_entries
                        .unexpected_anonymous_entries
                        .push(entry.span().into());
                }
            }
        }
    }

    if !unexpected_entries.is_empty() {
        diagnostics.add_miette(unexpected_entries);
    }

    if let Some(name) = name {
        match name.value() {
            KdlValue::String(name_value) => match Identifier::try_parse(name_value) {
                Ok(id) => {
                    enum_value.name = id.with_span(name.span());
                }
                Err(e) => {
                    diagnostics.add(InvalidIdentifier::new(e, name.span().into()));
                }
            },
            _ => {
                diagnostics.add_miette(errors::UnexpectedType {
                    value_name: name.span().into(),
                    expected_type: "string",
                });
            }
        }
    } else {
        diagnostics.add_miette(errors::MissingObjectName {
            object_keyword: node.name().span().into(),
            found_instead: None,
            object_type: node.name().value().into(),
        });
    }

    if let Some(size_bits) = size_bits {
        match size_bits.value() {
            KdlValue::Integer(sb) if (0..=i128::from(u32::MAX)).contains(sb) => {
                enum_value.size_bits = Some(*sb as u32);
            }
            KdlValue::Integer(_) => {
                diagnostics.add_miette(errors::ValueOutOfRange {
                    value: size_bits.span().into(),
                    context: Some("size-bits is encoded as a u32"),
                    range: "0..2^32",
                });
            }
            _ => {
                diagnostics.add_miette(errors::UnexpectedType {
                    value_name: size_bits.span().into(),
                    expected_type: "integer",
                });
            }
        }
    }

    if !enum_value.name.is_empty() {
        Some(enum_value)
    } else {
        None
    }
}

fn transform_enum_variants(nodes: &KdlDocument, diagnostics: &mut Diagnostics) -> Vec<EnumVariant> {
    nodes
        .nodes()
        .iter()
        .filter_map(|node| {
            let variant_name = node.name();

            let variant_value = match node.entries().len() {
                0 => None,
                1 if node.entries()[0].name().is_none() => Some(&node.entries()[0]),
                _ => {
                    diagnostics.add_miette(errors::UnexpectedEntries {
                        superfluous_entries: node
                            .entries()
                            .get(1..)
                            .map(|superfluous_entries| {
                                superfluous_entries
                                    .iter()
                                    .map(|entry| entry.span().into())
                                    .collect()
                            })
                            .unwrap_or_default(),
                        unexpected_name_entries: Vec::new(),
                        not_anonymous_entries: node
                            .entries()
                            .first()
                            .into_iter()
                            .filter_map(|entry| {
                                entry.name().is_some().then_some(entry.span().into())
                            })
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
                        diagnostics.add_miette(errors::UnexpectedValue {
                            value_name: variant_value.span().into(),
                            expected_values: vec!["", "<integer>", "default", "catch-all"],
                        });
                        return None;
                    }
                },
                None => EnumValue::Unspecified,
            };

            let name = match Identifier::try_parse(variant_name.value()) {
                Ok(id) => id,
                Err(e) => {
                    diagnostics.add(InvalidIdentifier::new(e, variant_name.span().into()));
                    return None;
                }
            };

            Some(EnumVariant {
                description: parse_description(node),
                name: name.with_span(variant_name.span()),
                value: variant_value,
                span: node.span().into(),
            })
        })
        .collect()
}

fn transform_extern(node: &KdlNode, diagnostics: &mut Diagnostics) -> Option<Extern> {
    let mut extern_value = Extern {
        description: Default::default(),
        name: Default::default(),
        base_type: Default::default(),
        supports_infallible: Default::default(),
        span: node.span().into(),
    };

    let mut unexpected_entries = errors::UnexpectedEntries {
        superfluous_entries: Vec::new(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: Vec::new(),
        unexpected_anonymous_entries: Vec::new(),
    };

    let (base_type, field_conversion) = parse_type(node.ty(), diagnostics);

    extern_value.base_type = base_type;

    if let Some(field_conversion) = field_conversion {
        diagnostics.add_miette(errors::OnlyBaseTypeAllowed {
            existing_ty: node.ty().unwrap().span().into(),
            field_conversion,
        });
    }

    if let Some(children) = node.children() {
        diagnostics.add_miette(errors::NoChildrenExpected {
            children: children.span().into(),
        });
    }

    let mut name: Option<&kdl::KdlEntry> = None;
    let mut infallible: Option<&kdl::KdlEntry> = None;

    for (i, entry) in node.entries().iter().enumerate() {
        match entry.name().map(kdl::KdlIdentifier::value) {
            Some(_) => {
                unexpected_entries
                    .unexpected_name_entries
                    .push(entry.span().into());
            }
            None => {
                if entry.value().as_string() == Some("infallible") {
                    if let Some(infallible) = infallible {
                        diagnostics.add_miette(errors::DuplicateEntry {
                            duplicate: entry.span().into(),
                            original: infallible.span().into(),
                        });
                    } else {
                        infallible = Some(entry);
                    }
                } else if i == 0 {
                    name = Some(entry);
                } else {
                    unexpected_entries
                        .unexpected_anonymous_entries
                        .push(entry.span().into());
                }
            }
        }
    }

    if !unexpected_entries.is_empty() {
        diagnostics.add_miette(unexpected_entries);
    }

    if let Some(name) = name {
        match name.value() {
            KdlValue::String(name_value) => match Identifier::try_parse(name_value) {
                Ok(id) => {
                    extern_value.name = id.with_span(name.span());
                }
                Err(e) => {
                    diagnostics.add(InvalidIdentifier::new(e, name.span().into()));
                }
            },
            _ => {
                diagnostics.add_miette(errors::UnexpectedType {
                    value_name: name.span().into(),
                    expected_type: "string",
                });
            }
        }
    } else {
        diagnostics.add_miette(errors::MissingObjectName {
            object_keyword: node.name().span().into(),
            found_instead: None,
            object_type: node.name().value().into(),
        });
    }

    extern_value.supports_infallible = infallible.is_some();

    if !extern_value.name.is_empty() {
        Some(extern_value)
    } else {
        None
    }
}

fn parse_type(
    ty: Option<&KdlIdentifier>,
    diagnostics: &mut Diagnostics,
) -> (Spanned<BaseType>, Option<TypeConversion>) {
    let Some(ty) = ty else {
        return (BaseType::Unspecified.with_dummy_span(), None);
    };

    let ty_str = ty.value();
    let mut field_conversion = None;
    let base_type_str;

    if let Some((base_type, conversion)) = ty_str.split_once(':') {
        base_type_str = base_type;

        let use_try = conversion.ends_with('?');
        let conversion = IdentifierRef::new(conversion.trim_end_matches('?').into());

        field_conversion = Some(TypeConversion {
            type_name: conversion.with_span(ty.span()),
            fallible: use_try,
        });
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
            diagnostics.add_miette(errors::UnexpectedValue {
                value_name: ty.span().into(),
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

    (base_type.with_span(ty.span()), field_conversion)
}

fn ensure_zero_entries(node: &KdlNode, diagnostics: &mut Diagnostics) {
    if !node.entries().is_empty() {
        diagnostics.add_miette(errors::UnexpectedEntries {
            superfluous_entries: node
                .entries()
                .iter()
                .map(|entry| entry.span().into())
                .collect(),
            unexpected_name_entries: Vec::new(),
            not_anonymous_entries: Vec::new(),
            unexpected_anonymous_entries: Vec::new(),
        });
    }
}

/// Parse the repeat specifiers.
///
/// If `standalone_repeat` is true, it's expected only repeat entries are to be found and appropriate errors will be emitted.
/// If false, then the only errors emitted is when the repeat is incomplete
fn parse_repeat_entries(
    node: &KdlNode,
    diagnostics: &mut Diagnostics,
    standalone_repeat: bool,
) -> Option<Repeat> {
    let mut unexpected_entries = errors::UnexpectedEntries {
        superfluous_entries: Vec::new(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: Vec::new(),
        unexpected_anonymous_entries: Vec::new(),
    };

    let mut count = None;
    let mut stride = None;
    let mut with = None;

    for entry in node.entries() {
        match (entry.name().map(kdl::KdlIdentifier::value), entry.value()) {
            (Some("count"), KdlValue::Integer(val)) => count = Some((*val, entry.span())),
            (Some("stride"), KdlValue::Integer(val)) => stride = Some((*val, entry.span())),
            (Some("with"), KdlValue::String(val)) => {
                with = Some((IdentifierRef::new(val.clone()), entry.span()))
            }
            (Some("count" | "stride"), _) => diagnostics.add_miette(errors::UnexpectedType {
                value_name: entry.span().into(),
                expected_type: "integer",
            }),
            (Some("with"), _) => diagnostics.add_miette(errors::UnexpectedType {
                value_name: entry.span().into(),
                expected_type: "string",
            }),
            (Some(_), _) => {
                unexpected_entries
                    .unexpected_name_entries
                    .push(entry.span().into());
            }
            (None, _) => {
                unexpected_entries
                    .superfluous_entries
                    .push(entry.span().into());
            }
        }
    }

    if !unexpected_entries.is_empty() && standalone_repeat {
        diagnostics.add_miette(unexpected_entries);
    }

    let mut error = false;

    if let (Some((_, count_span)), Some((_, with_span))) = (&count, &with) {
        error = true;
        diagnostics.add_miette(errors::RepeatOverSpecified {
            count: (*count_span).into(),
            with: (*with_span).into(),
        });
    }

    let mut missing_entry_error = errors::MissingEntry {
        node_name: node.name().span().into(),
        expected_entries: Vec::new(),
    };

    if standalone_repeat || count.is_some() || with.is_some() || stride.is_some() {
        if count.is_none() && with.is_none() {
            error = true;
            missing_entry_error.expected_entries.push("count=<integer>");
            missing_entry_error.expected_entries.push("with=<string>");
        }
        if stride.is_none() {
            error = true;
            missing_entry_error
                .expected_entries
                .push("stride=<integer>");
        }
    }

    if !missing_entry_error.expected_entries.is_empty() {
        diagnostics.add_miette(missing_entry_error);
    }

    if let Some((count, span)) = count
        && !(0..=i128::from(u64::MAX)).contains(&count)
    {
        error = true;
        diagnostics.add_miette(errors::ValueOutOfRange {
            value: span.into(),
            context: Some("The count is encoded as a u64"),
            range: "0..2^64",
        });
    }
    if let Some((stride, span)) = stride
        && stride == 0
    {
        error = true;
        diagnostics.add_miette(errors::ValueOutOfRange {
            value: span.into(),
            context: Some("The stride must not be 0"),
            range: "any non-0 number",
        });
    }

    if error {
        None
    } else {
        match (count, with, stride) {
            (None, Some((with, with_span)), Some((stride, _))) => Some(Repeat {
                source: RepeatSource::Enum(with.with_span(with_span)),
                stride,
            }),
            (Some((count, _)), None, Some((stride, _))) => Some(Repeat {
                source: RepeatSource::Count(count as u64),
                stride,
            }),
            (None, None, None) => None,
            _ => unreachable!(),
        }
    }
}

fn parse_reset_value_entries(node: &KdlNode, diagnostics: &mut Diagnostics) -> Option<ResetValue> {
    let mut error = false;
    let mut array = Vec::new();

    for entry in node.entries() {
        if let KdlValue::Integer(val) = entry.value() {
            array.push((*val, entry.span()));
        } else {
            error = true;
            diagnostics.add_miette(errors::UnexpectedType {
                value_name: entry.span().into(),
                expected_type: "integer",
            });
        }
    }

    if error {
        return None;
    }

    if array.len() == 1 {
        let (integer, span) = array[0];

        if integer.is_negative() {
            diagnostics.add_miette(errors::ValueOutOfRange {
                value: span.into(),
                range: "0..",
                context: Some("Negative reset values are not allowed"),
            });
            None
        } else {
            Some(ResetValue::Integer(integer as u128))
        }
    } else {
        let mut error = false;
        for (byte, span) in &array {
            if !(0..=255).contains(byte) {
                error = true;
                diagnostics.add_miette(errors::ValueOutOfRange {
                    value: (*span).into(),
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
    diagnostics: &mut Diagnostics,
) -> (Option<i128>, Option<SourceSpan>) {
    let unexpected_entries = errors::UnexpectedEntries {
        superfluous_entries: node
            .entries()
            .iter()
            .skip(1)
            .map(|entry| entry.span().into())
            .collect(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: node
            .entries()
            .first()
            .iter()
            .filter_map(|entry| entry.name().map(|_| entry.span().into()))
            .collect(),
        unexpected_anonymous_entries: Vec::new(),
    };
    if !unexpected_entries.is_empty() {
        diagnostics.add_miette(unexpected_entries);
    }

    match node.entries().first() {
        Some(entry) if entry.name().is_none() => {
            if let KdlValue::Integer(val) = entry.value() {
                (Some(*val), Some(entry.span()))
            } else {
                diagnostics.add_miette(errors::UnexpectedType {
                    value_name: entry.span().into(),
                    expected_type: "integer",
                });
                (None, Some(entry.span()))
            }
        }
        _ => {
            diagnostics.add_miette(errors::MissingEntry {
                node_name: node.name().span().into(),
                expected_entries: vec!["integer"],
            });
            (None, None)
        }
    }
}

fn parse_single_string_entry(
    node: &KdlNode,
    diagnostics: &mut Diagnostics,
    expected_entries: Option<&[&'static str]>,
    is_name: bool,
) -> (Option<String>, Option<SourceSpan>) {
    let unexpected_entries = errors::UnexpectedEntries {
        superfluous_entries: node
            .entries()
            .iter()
            .skip(1)
            .map(|entry| entry.span().into())
            .collect(),
        unexpected_name_entries: Vec::new(),
        not_anonymous_entries: node
            .entries()
            .first()
            .iter()
            .filter_map(|entry| entry.name().map(|_| entry.span().into()))
            .collect(),
        unexpected_anonymous_entries: Vec::new(),
    };
    if !unexpected_entries.is_empty() {
        diagnostics.add_miette(unexpected_entries);
    }

    match node.entries().first() {
        Some(entry) if entry.name().is_none() => {
            if let KdlValue::String(val) = entry.value() {
                (Some(val.clone()), Some(entry.span()))
            } else {
                if is_name {
                    diagnostics.add_miette(errors::MissingObjectName {
                        object_keyword: node.name().span().into(),
                        object_type: node.name().value().into(),
                        found_instead: Some(entry.span().into()),
                    });
                } else {
                    diagnostics.add_miette(errors::UnexpectedType {
                        value_name: entry.span().into(),
                        expected_type: "string",
                    });
                }
                (None, Some(entry.span()))
            }
        }
        _ => {
            if is_name {
                diagnostics.add_miette(errors::MissingObjectName {
                    object_keyword: node.name().span().into(),
                    object_type: node.name().value().into(),
                    found_instead: None,
                });
            } else {
                diagnostics.add_miette(errors::MissingEntry {
                    node_name: node.name().span().into(),
                    expected_entries: expected_entries
                        .map_or_else(|| vec!["string"], <[&str]>::to_vec),
                });
            }
            (None, None)
        }
    }
}

fn parse_single_string_value<T: VariantNames + FromStr>(
    node: &KdlNode,
    diagnostics: &mut Diagnostics,
) -> Option<Spanned<T>> {
    match parse_single_string_entry(node, diagnostics, Some(T::VARIANTS), false) {
        (Some(val), Some(entry)) if T::from_str(&val).is_ok() => {
            T::from_str(&val).ok().map(|val| val.with_span(entry))
        }
        (Some(_), Some(entry)) => {
            diagnostics.add_miette(errors::UnexpectedValue {
                value_name: entry.into(),
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
            .filter(|line| line.trim_start().starts_with("///"))
            .map(|line| line.trim().trim_start_matches("///"))
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
const BLOCK_FIELDS: &[(&str, BlockField)] = &[
    ("offset", BlockField::Offset),
    ("repeat", BlockField::Repeat),
];
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum BlockField {
    Offset,
    Repeat,
}

impl FromStr for BlockField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (name, val) in BLOCK_FIELDS {
            if *name == s {
                return Ok(*val);
            }
        }

        Err(())
    }
}

#[rustfmt::skip]
const DEVICE_CONFIG_TYPES: &[(&str, DeviceConfigType)] = &[
    ("register-access", DeviceConfigType::RegisterAccess),
    ("field-access", DeviceConfigType::FieldAccess),
    ("buffer-access", DeviceConfigType::BufferAccess),
    ("byte-order", DeviceConfigType::ByteOrder),
    ("bit-order", DeviceConfigType::BitOrder),
    ("register-address-type", DeviceConfigType::RegisterAddressType),
    ("command-address-type", DeviceConfigType::CommandAddressType),
    ("buffer-address-type", DeviceConfigType::BufferAddressType),
    ("name-word-boundaries", DeviceConfigType::NameWordBoundaries),
    ("defmt-feature", DeviceConfigType::DefmtFeature),
];
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum DeviceConfigType {
    RegisterAccess,
    FieldAccess,
    BufferAccess,
    ByteOrder,
    BitOrder,
    RegisterAddressType,
    CommandAddressType,
    BufferAddressType,
    NameWordBoundaries,
    DefmtFeature,
}

impl FromStr for DeviceConfigType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (name, val) in DEVICE_CONFIG_TYPES {
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
    ("fieldset", ObjectType::FieldSet),
    ("enum", ObjectType::Enum),
    ("extern", ObjectType::Extern),
];
#[derive(Clone, Copy)]
enum ObjectType {
    Block,
    Register,
    Command,
    Buffer,
    FieldSet,
    Enum,
    Extern,
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

#[rustfmt::skip]
const ROOT_OBJECT_TYPES: &[(&str, RootObjectType)] = &[
    ("device", RootObjectType::Device),
    ("fieldset", RootObjectType::FieldSet),
    ("enum", RootObjectType::Enum),
    ("extern", RootObjectType::Extern),
];
#[derive(Clone, Copy)]
enum RootObjectType {
    Device,
    FieldSet,
    Enum,
    Extern,
}

impl FromStr for RootObjectType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (name, val) in ROOT_OBJECT_TYPES {
            if *name == s {
                return Ok(*val);
            }
        }

        Err(())
    }
}

fn change_document_span(document: &mut KdlDocument, source_span: &SourceSpan) {
    document.set_span((
        document.span().offset() + source_span.offset(),
        document.span().len(),
    ));

    for node in document.nodes_mut() {
        change_node_span(node, source_span);
    }
}

fn change_node_span(node: &mut KdlNode, source_span: &SourceSpan) {
    node.set_span((
        node.span().offset() + source_span.offset(),
        node.span().len(),
    ));

    if let Some(ty) = node.ty_mut() {
        change_identifier_span(ty, source_span);
    }

    change_identifier_span(node.name_mut(), source_span);

    for entry in node.entries_mut() {
        change_entry_span(entry, source_span);
    }

    if let Some(children) = node.children_mut() {
        change_document_span(children, source_span);
    }
}

fn change_identifier_span(id: &mut KdlIdentifier, source_span: &SourceSpan) {
    id.set_span((id.span().offset() + source_span.offset(), id.span().len()));
}

fn change_entry_span(entry: &mut KdlEntry, source_span: &SourceSpan) {
    entry.set_span((
        entry.span().offset() + source_span.offset(),
        entry.span().len(),
    ));

    if let Some(ty) = entry.ty_mut() {
        change_identifier_span(ty, source_span);
    }

    if let Some(name) = entry.name_mut() {
        change_identifier_span(name, source_span);
    }
}

/// The same as the [`kdl::KdlDiagnostic`], but with a named source input and updated spans
#[derive(Debug, Diagnostic, Clone, Eq, PartialEq, Error)]
#[error("{}", message.clone().unwrap_or_else(|| "Unexpected error".into()))]
pub struct ConvertedKdlDiagnostic {
    /// Offset in chars of the error.
    #[label("{}", label.clone().unwrap_or_else(|| "here".into()))]
    span: SourceSpan,

    /// Message for the error itself.
    message: Option<String>,

    /// Label text for this span. Defaults to `"here"`.
    label: Option<String>,

    /// Suggestion for fixing the parser error.
    #[help]
    help: Option<String>,

    /// Severity level for the Diagnostic.
    #[diagnostic(severity)]
    severity: miette::Severity,
}

impl ConvertedKdlDiagnostic {
    #[must_use]
    pub fn from_original_and_span(original: KdlDiagnostic, input_span: Option<SourceSpan>) -> Self {
        let KdlDiagnostic {
            input: _,
            span,
            message,
            label,
            help,
            severity,
        } = original;

        Self {
            span: if let Some(input_span) = input_span {
                (input_span.offset() + span.offset(), span.len()).into()
            } else {
                span
            },
            message,
            label,
            help,
            severity,
        }
    }
}
