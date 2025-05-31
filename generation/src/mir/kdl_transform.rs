//! Transforms the MIR back into KDL that should be identical (in information) to the input
//!

use itertools::Itertools;
use kdl::{KdlDocument, KdlEntry, KdlNode, KdlNodeFormat, KdlValue};

use super::*;

pub fn transform(device: Device) -> String {
    let mut document = KdlDocument::new();

    document.nodes_mut().push(transform_device(&device));

    document.autoformat();
    document.to_string()
}

fn transform_device(device: &Device) -> KdlNode {
    let mut device_node = KdlNode::new("device");
    device_node.push(KdlEntry::new(KdlValue::String(
        device
            .name
            .clone()
            .unwrap_or_else(|| "Device".into())
            .to_string(),
    )));

    device_node
        .ensure_children()
        .nodes_mut()
        .append(&mut transform_global_config(&device.global_config));

    for object in device.objects.iter() {
        device_node
            .ensure_children()
            .nodes_mut()
            .push(transform_object(object, &device.global_config));
    }

    device_node
}

fn transform_global_config(global_config: &GlobalConfig) -> Vec<KdlNode> {
    let GlobalConfig {
        default_register_access,
        default_field_access,
        default_buffer_access,
        default_byte_order,
        default_bit_order,
        register_address_type,
        command_address_type,
        buffer_address_type,
        name_word_boundaries,
        defmt_feature,
    } = global_config.clone();

    let mut nodes = Vec::new();

    if default_register_access != GlobalConfig::default().default_register_access {
        let mut config_node = KdlNode::new("default-register-access");
        config_node.push(default_register_access.to_string());
        nodes.push(config_node);
    }
    if default_field_access != GlobalConfig::default().default_field_access {
        let mut config_node = KdlNode::new("default-field-access");
        config_node.push(default_field_access.to_string());
        nodes.push(config_node);
    }
    if default_buffer_access != GlobalConfig::default().default_buffer_access {
        let mut config_node = KdlNode::new("default-buffer-access");
        config_node.push(default_buffer_access.to_string());
        nodes.push(config_node);
    }
    if let Some(default_byte_order) = default_byte_order {
        let mut config_node = KdlNode::new("default-byte-order");
        config_node.push(default_byte_order.to_string());
        nodes.push(config_node);
    }
    if default_bit_order != GlobalConfig::default().default_bit_order {
        let mut config_node = KdlNode::new("default-bit-order");
        config_node.push(default_bit_order.to_string());
        nodes.push(config_node);
    }
    if let Some(register_address_type) = register_address_type {
        let mut config_node = KdlNode::new("register-address-type");
        config_node.push(register_address_type.to_string());
        nodes.push(config_node);
    }
    if let Some(command_address_type) = command_address_type {
        let mut config_node = KdlNode::new("command-address-type");
        config_node.push(command_address_type.to_string());
        nodes.push(config_node);
    }
    if let Some(buffer_address_type) = buffer_address_type {
        let mut config_node = KdlNode::new("buffer-address-type");
        config_node.push(buffer_address_type.to_string());
        nodes.push(config_node);
    }
    if name_word_boundaries != GlobalConfig::default().name_word_boundaries {
        let mut config_node = KdlNode::new("name-word-boundaries");
        for boundary in name_word_boundaries {
            config_node.push(format!("{boundary:?}"));
        }
        nodes.push(config_node);
    }
    if let Some(defmt_feature) = defmt_feature {
        let mut config_node = KdlNode::new("defmt-feature");
        config_node.push(defmt_feature);
        nodes.push(config_node);
    }

    nodes
}

fn transform_object(object: &Object, global_config: &GlobalConfig) -> KdlNode {
    let (node_type, ref_target) = match object {
        Object::Block(_) => ("block", None),
        Object::Register(_) => ("register", None),
        Object::Command(_) => ("command", None),
        Object::Buffer(_) => ("buffer", None),
        Object::Ref(ref_object) => match &ref_object.object_override {
            ObjectOverride::Block(override_data) => {
                ("ref", Some(("block-target", override_data.name.clone())))
            }
            ObjectOverride::Register(override_data) => {
                ("ref", Some(("register-target", override_data.name.clone())))
            }
            ObjectOverride::Command(override_data) => {
                ("ref", Some(("command-target", override_data.name.clone())))
            }
        },
    };

    let mut node = KdlNode::new(node_type);

    node.push(object.name());
    if let Some(ref_target) = ref_target {
        node.push(ref_target);
    }

    node.set_format(KdlNodeFormat {
        leading: description_to_leading_comment(object.description()),
        ..Default::default()
    });

    match object {
        Object::Block(block) => node.set_children(transform_block(block, global_config)),
        Object::Register(register) => {
            node.set_children(transform_register(register, global_config))
        }
        Object::Command(command) => {}
        Object::Buffer(buffer) => node.set_children(transform_buffer(buffer, global_config)),
        Object::Ref(ref_object) => {}
    };

    node
}

fn transform_block(block: &Block, global_config: &GlobalConfig) -> KdlDocument {
    let Block {
        cfg_attr,
        description: _,
        name: _,
        address_offset,
        repeat,
        objects,
    } = block;

    let mut document = KdlDocument::new();

    if let Some(cfg_node) = transform_cfg_config(cfg_attr) {
        document.nodes_mut().push(cfg_node);
    }

    let mut address_offset_node = KdlNode::new("address-offset");
    address_offset_node.push(*address_offset as i128);
    document.nodes_mut().push(address_offset_node);

    if let Some(repeat) = repeat {
        document.nodes_mut().push(transform_repeat_config(repeat));
    }

    for object in objects {
        document
            .nodes_mut()
            .push(transform_object(object, global_config));
    }

    document
}

fn transform_register(register: &Register, global_config: &GlobalConfig) -> KdlDocument {
    let Register {
        cfg_attr,
        description: _,
        name: _,
        access,
        byte_order,
        bit_order,
        allow_bit_overlap,
        allow_address_overlap,
        address,
        size_bits,
        reset_value,
        repeat,
        fields,
    } = register;

    let mut document = KdlDocument::new();
    let mut fields_node = KdlNode::new("fields");

    if let Some(cfg_node) = transform_cfg_config(cfg_attr) {
        document.nodes_mut().push(cfg_node);
    }

    if *access != global_config.default_register_access {
        let mut access_node = KdlNode::new("access");
        access_node.push(access.to_string());
        document.nodes_mut().push(access_node);
    }

    if let Some(byte_order) = byte_order {
        fields_node.push(("byte-order", byte_order.to_string()));
    }

    if *bit_order != global_config.default_bit_order {
        fields_node.push(("bit-order", bit_order.to_string()));
    }

    if *allow_bit_overlap {
        fields_node.push("allow-bit-overlap");
    }

    if *allow_address_overlap {
        let address_overlap_node = KdlNode::new("allow-address-overlap");
        document.nodes_mut().push(address_overlap_node);
    }

    let mut address_node = KdlNode::new("address");
    address_node.push(*address as i128);
    document.nodes_mut().push(address_node);

    fields_node.push(("size_bits", *size_bits as i128));

    if let Some(reset_value) = reset_value {
        let mut reset_value_node = KdlNode::new("reset-value");
        match reset_value {
            ResetValue::Integer(num) => reset_value_node.push(*num as i128),
            ResetValue::Array(bytes) => {
                for byte in bytes {
                    reset_value_node.push(*byte as i128)
                }
            }
        }
        document.nodes_mut().push(reset_value_node);
    }

    if let Some(repeat) = repeat {
        document.nodes_mut().push(transform_repeat_config(repeat));
    }

    for field in fields {
        fields_node
            .ensure_children()
            .nodes_mut()
            .push(transform_field(field));
    }
    document.nodes_mut().push(fields_node);

    document
}

fn transform_cfg_config(cfg: &Cfg) -> Option<KdlNode> {
    let mut cfg_node = KdlNode::new("cfg");
    cfg_node.push(cfg.inner()?);

    Some(cfg_node)
}

fn transform_repeat_config(repeat: &Repeat) -> KdlNode {
    let mut repeat_node = KdlNode::new("repeat");
    repeat_node.push(("count", repeat.count as i128));
    repeat_node.push(("stride", repeat.stride as i128));

    repeat_node
}

fn transform_field(field: &Field) -> KdlNode {
    let Field {
        cfg_attr,
        description,
        name,
        access,
        base_type,
        field_conversion,
        field_address,
    } = field;

    let mut node = KdlNode::new(name.as_str());

    node.set_format(KdlNodeFormat {
        leading: description_to_leading_comment(description),
        ..Default::default()
    });

    let base_type_text = match base_type {
        BaseType::Unspecified => "",
        BaseType::Bool => "bool",
        BaseType::Uint => "uint",
        BaseType::Int => "int",
        BaseType::FixedSize(integer) => &integer.to_string(),
    };

    match field_conversion {
        Some(fc) => node.set_ty(format!(
            "{}{}{}{}",
            base_type_text,
            if base_type_text.is_empty() { "" } else { ":" },
            fc.type_name(),
            if fc.use_try() { "?" } else { "" }
        )),
        None => node.set_ty(base_type_text),
    }

    node.push(access.to_string());

    if let Some(cfg) = cfg_attr.inner() {
        node.push(("cfg", cfg));
    }

    if field_address.is_empty() || field_address.len() == 1 {
        node.push(format!("@{}", field_address.start))
    } else {
        node.push(format!(
            "@{}:{}",
            field_address.end - 1,
            field_address.start
        ))
    };

    if let Some(FieldConversion::Enum { enum_value, .. }) = field_conversion {
        let children = node.ensure_children();

        for variant in &enum_value.variants {
            let EnumVariant {
                cfg_attr,
                description,
                name,
                value,
            } = variant;

            let mut variant_node = KdlNode::new(name.as_str());
            match value {
                EnumValue::Unspecified => {}
                EnumValue::Specified(num) => variant_node.push(*num),
                EnumValue::Default => variant_node.push("default"),
                EnumValue::CatchAll => variant_node.push("catch-all"),
            }

            if let Some(cfg) = cfg_attr.inner() {
                variant_node.push(("cfg", cfg));
            }

            variant_node.set_format(KdlNodeFormat {
                leading: description_to_leading_comment(&description),
                ..Default::default()
            });

            children.nodes_mut().push(variant_node);
        }
    }

    node
}

fn description_to_leading_comment(description: &str) -> String {
    description
        .lines()
        .map(|line| format!("/// {line}"))
        .join("\n")
}

fn transform_buffer(buffer: &Buffer, global_config: &GlobalConfig) -> KdlDocument {
    let Buffer {
        cfg_attr,
        description: _,
        name: _,
        access,
        address,
    } = buffer;

    let mut document = KdlDocument::new();

    if let Some(cfg_node) = transform_cfg_config(cfg_attr) {
        document.nodes_mut().push(cfg_node);
    }

    if *access != global_config.default_buffer_access {
        let mut node = KdlNode::new("access");
        node.push(access.to_string());
        document.nodes_mut().push(node);
    }

    let mut address_node = KdlNode::new("address");
    address_node.push(*address as i128);
    document.nodes_mut().push(address_node);

    document
}
