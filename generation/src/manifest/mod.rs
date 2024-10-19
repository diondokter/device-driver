use std::convert::identity;

use anyhow::{anyhow, bail, ensure, Context};
use convert_case::Boundary;
use dd_manifest_tree::{Map, Value};

use crate::mir::{self, Cfg};

pub fn transform(value: impl Value) -> anyhow::Result<mir::Device> {
    let device_map = value.as_map()?;

    let global_config = if let Some(global_config) = device_map.get("config") {
        transform_global_config(global_config).context("Parsing error in global config")?
    } else {
        Default::default()
    };

    let objects = device_map
        .iter()
        .filter(|(k, _)| *k != "config")
        .map(transform_object)
        .collect::<Result<_, _>>()?;

    Ok(mir::Device {
        global_config,
        objects,
    })
}

fn transform_global_config(value: &impl Value) -> anyhow::Result<mir::GlobalConfig> {
    let config_map = value.as_map()?;
    let mut global_config = mir::GlobalConfig::default();

    for (key, value) in config_map.iter() {
        match key {
            "default_register_access" => {
                global_config.default_register_access =
                    transform_access(value).with_context(|| format!("Parsing error for {key}"))?
            }
            "default_field_access" => {
                global_config.default_field_access =
                    transform_access(value).with_context(|| format!("Parsing error for {key}"))?
            }
            "default_buffer_access" => {
                global_config.default_buffer_access =
                    transform_access(value).with_context(|| format!("Parsing error for {key}"))?
            }
            "default_byte_order" => {
                global_config.default_byte_order = Some(
                    transform_byte_order(value)
                        .with_context(|| format!("Parsing error for {key}"))?,
                )
            }
            "default_bit_order" => {
                global_config.default_bit_order = transform_bit_order(value)
                    .with_context(|| format!("Parsing error for {key}"))?
            }
            "register_address_type" => {
                global_config.register_address_type = Some(
                    transform_integer_type(value)
                        .with_context(|| format!("Parsing error for {key}"))?,
                )
            }
            "command_address_type" => {
                global_config.command_address_type = Some(
                    transform_integer_type(value)
                        .with_context(|| format!("Parsing error for {key}"))?,
                )
            }
            "buffer_address_type" => {
                global_config.buffer_address_type = Some(
                    transform_integer_type(value)
                        .with_context(|| format!("Parsing error for {key}"))?,
                )
            }
            "name_word_boundaries" => {
                global_config.name_word_boundaries = transform_name_word_boundaries(value)
                    .with_context(|| format!("Parsing error for {key}"))?
            }
            _ => bail!("No config with key `{key}` is recognized"),
        }
    }

    Ok(global_config)
}

fn transform_access(value: &impl Value) -> anyhow::Result<mir::Access> {
    match value.as_string()? {
        "ReadWrite" | "RW" => Ok(mir::Access::RW),
        "ReadOnly" | "RO" => Ok(mir::Access::RO),
        "WriteOnly" | "WO" => Ok(mir::Access::WO),
        val => Err(anyhow::anyhow!("No access value `{val}` exists. Values are limited to \"ReadWrite\", \"RW\", \"ReadOnly\", \"RO\", \"WriteOnly\", \"WO\"")),
    }
}

fn transform_byte_order(value: &impl Value) -> anyhow::Result<mir::ByteOrder> {
    match value.as_string()? {
        "LE" => Ok(mir::ByteOrder::LE),
        "BE" => Ok(mir::ByteOrder::BE),
        val => Err(anyhow::anyhow!(
            "No byte order value `{val}` exists. Values are limited to \"LE\" and \"BE\""
        )),
    }
}

fn transform_bit_order(value: &impl Value) -> anyhow::Result<mir::BitOrder> {
    match value.as_string()? {
        "LSB0" => Ok(mir::BitOrder::LSB0),
        "MSB0" => Ok(mir::BitOrder::MSB0),
        val => Err(anyhow::anyhow!(
            "No bit order value `{val}` exists. Values are limited to \"LSB0\" and \"MSB0\""
        )),
    }
}

fn transform_integer_type(value: &impl Value) -> anyhow::Result<mir::Integer> {
    match value.as_string()? {
        "u8" => Ok(mir::Integer::U8),
        "u16" => Ok(mir::Integer::U16),
        "u32" => Ok(mir::Integer::U32),
        "i8" => Ok(mir::Integer::I8),
        "i16" => Ok(mir::Integer::I16),
        "i32" => Ok(mir::Integer::I32),
        "i64" => Ok(mir::Integer::I64),
        val => Err(anyhow::anyhow!(
            "No integer type value `{val}` exists. Values are limited to \"u8\", \"u16\", \"u32\", \"i8\", \"i16\", \"i32\", \"i64\""
        )),
    }
}

fn transform_name_word_boundaries(value: &impl Value) -> anyhow::Result<Vec<Boundary>> {
    if let Ok(boundaries_string) = value.as_string() {
        Ok(Boundary::list_from(boundaries_string))
    } else if let Ok(boundaries_array) = value.as_array() {
        Ok(boundaries_array
            .iter()
            .map(|boundary| {
                let boundary = boundary.as_string()?;

                for b in Boundary::all() {
                    if format!("{b:?}").eq_ignore_ascii_case(boundary) {
                        return Ok(b);
                    }
                }

                Err(anyhow!(
                    "`{}` is not a valid boundary name. One of the following was expected: {:?}",
                    boundary,
                    Boundary::all()
                ))
            })
            .collect::<Result<Vec<_>, _>>()?)
    } else {
        Err(anyhow::anyhow!("Expected a string or an array"))
    }
}

fn transform_object((key, value): (&str, &impl Value)) -> anyhow::Result<mir::Object> {
    let try_ = || {
        let object_map = value.as_map()?;

        let object_type = object_map
            .get("type")
            .ok_or_else(|| anyhow!("No 'type' field present"))?
            .as_string()?;

        match object_type {
            "block" => Ok(mir::Object::Block(transform_block(key, object_map)?)),
            "register" => Ok(mir::Object::Register(transform_register(key, object_map)?)),
            "command" => Ok(mir::Object::Command(transform_command(key, object_map)?)),
            "buffer" => Ok(mir::Object::Buffer(transform_buffer(key, object_map)?)),
            "ref" => Ok(mir::Object::Ref(transform_ref(key, object_map)?)),
            val => Err(anyhow!("Unexpected object type '{val}'. Select one of \"block\", \"register\", \"command\", \"buffer\" or \"ref\""))
        }
    };

    try_().with_context(|| format!("Parsing object `{key}`"))
}

fn transform_block(name: &str, map: &impl Map) -> anyhow::Result<mir::Block> {
    let mut block = mir::Block {
        name: name.into(),
        ..Default::default()
    };

    for (key, value) in map.iter() {
        let read_objects = || {
            value
                .as_map()
                .map_err(anyhow::Error::from)
                .map(|object_map| {
                    object_map
                        .iter()
                        .map(transform_object)
                        .collect::<Result<Vec<mir::Object>, anyhow::Error>>()
                })
                .and_then(std::convert::identity)
                .context("Parsing error for 'objects'")
        };

        match key {
            "type" => {},
            "cfg" => block.cfg_attr = mir::Cfg::new(Some(value.as_string().context("Parsing error for 'cfg'")?)),
            "description" => block.description = value.as_string().context("Parsing error for 'description'")?.into(),
            "address_offset" => block.address_offset = value.as_int().context("Parsing error for 'address_offset'")?,
            "repeat" => block.repeat = Some(transform_repeat(value).context("Parsing error for 'repeat'")?),
            "objects" => block.objects = read_objects()?,
            val => bail!(
                "Unexpected key found: '{val}'. Choose one of \"type\", \"cfg\", \"description\", \"address_offset\", \"repeat\" or \"objects\""
            ),
        }
    }

    Ok(block)
}

fn transform_register(name: &str, map: &impl Map) -> anyhow::Result<mir::Register> {
    let mut register = mir::Register {
        name: name.into(),
        ..Default::default()
    };

    for required_key in ["address", "size_bits"] {
        if !map.contains_key(required_key) {
            bail!("Register definition must contain the '{required_key}' field");
        }
    }

    for (key, value) in map.iter() {
        match key {
            "type" => {}
            "cfg" => {
                register.cfg_attr =
                    mir::Cfg::new(Some(value.as_string().context("Parsing error for 'cfg'")?))
            }
            "description" => {
                register.description = value
                    .as_string()
                    .context("Parsing error for 'description'")?
                    .into()
            }
            "access" => {
                register.access = transform_access(value).context("Parsing error for 'access'")?;
            }
            "byte_order" => {
                register.byte_order =
                    Some(transform_byte_order(value).context("Parsing error for 'byte_order'")?);
            }
            "bit_order" => {
                register.bit_order =
                    transform_bit_order(value).context("Parsing error for 'bit_order'")?;
            }
            "address" => {
                register.address = value.as_int().context("Parsing error for 'address'")?;
            }
            "size_bits" => {
                register.size_bits = value
                    .as_uint()
                    .context("Parsing error for 'size_bits'")?
                    .try_into()
                    .context("Parsing error for 'size_bits'")?;
            }
            "reset_value" => {
                register.reset_value = Some(if let Ok(rv) = value.as_uint() {
                    mir::ResetValue::Integer(rv as u128)
                } else if let Ok(rv) = value.as_array() {
                    match rv
                        .iter()
                        .map(|inner| {
                            inner
                                .as_uint()
                                .context("Array must contain bytes")
                                .map(|val| u8::try_from(val).context("Array must contain bytes"))
                                .and_then(identity)
                        })
                        .collect::<Result<Vec<_>, _>>()
                    {
                        Ok(val) => mir::ResetValue::Array(val),
                        Err(e) => return Err(e.context("Parsing error for 'reset_value")),
                    }
                } else {
                    return Err(anyhow!("Field must be an integer or an array")
                        .context("Parsing error for 'reset_value"));
                })
            }
            "repeat" => {
                register.repeat =
                    Some(transform_repeat(value).context("Parsing error for 'repeat")?);
            }
            "allow_bit_overlap" => {
                register.allow_bit_overlap = value
                    .as_bool()
                    .context("Parsing error for 'allow_bit_overlap'")?;
            }
            "allow_address_overlap" => {
                register.allow_address_overlap = value
                    .as_bool()
                    .context("Parsing error for 'allow_address_overlap'")?;
            }
            "fields" => {
                register.fields = transform_fields(value).context("Parsing error for 'fields'")?;
            }
            val => {
                bail!("Unexpected key: '{val}'")
            }
        }
    }

    Ok(register)
}

fn transform_command(name: &str, map: &impl Map) -> anyhow::Result<mir::Command> {
    let mut command = mir::Command {
        name: name.into(),
        ..Default::default()
    };

    for required_key in ["address"] {
        if !map.contains_key(required_key) {
            bail!("Command definition must contain the '{required_key}' field");
        }
    }

    for (key, value) in map.iter() {
        match key {
            "type" => {}
            "cfg" => {
                command.cfg_attr =
                    mir::Cfg::new(Some(value.as_string().context("Parsing error for 'cfg'")?))
            }
            "description" => {
                command.description = value
                    .as_string()
                    .context("Parsing error for 'description'")?
                    .into()
            }
            "byte_order" => {
                command.byte_order =
                    Some(transform_byte_order(value).context("Parsing error for 'byte_order'")?);
            }
            "bit_order" => {
                command.bit_order =
                    transform_bit_order(value).context("Parsing error for 'bit_order'")?;
            }
            "address" => {
                command.address = value.as_int().context("Parsing error for 'address'")?;
            }
            "size_bits_in" => {
                command.size_bits_in = value
                    .as_uint()
                    .context("Parsing error for 'size_bits_in'")?
                    .try_into()
                    .context("Parsing error for 'size_bits_in'")?;
            }
            "size_bits_out" => {
                command.size_bits_out = value
                    .as_uint()
                    .context("Parsing error for 'size_bits_out'")?
                    .try_into()
                    .context("Parsing error for 'size_bits_out'")?;
            }
            "repeat" => {
                command.repeat =
                    Some(transform_repeat(value).context("Parsing error for 'repeat")?);
            }
            "allow_bit_overlap" => {
                command.allow_bit_overlap = value
                    .as_bool()
                    .context("Parsing error for 'allow_bit_overlap'")?;
            }
            "allow_address_overlap" => {
                command.allow_address_overlap = value
                    .as_bool()
                    .context("Parsing error for 'allow_address_overlap'")?;
            }
            "fields_in" => {
                command.in_fields =
                    transform_fields(value).context("Parsing error for 'fields_in'")?;
            }
            "fields_out" => {
                command.out_fields =
                    transform_fields(value).context("Parsing error for 'fields_out'")?;
            }
            val => {
                bail!("Unexpected key: '{val}'")
            }
        }
    }

    Ok(command)
}

fn transform_buffer(name: &str, map: &impl Map) -> anyhow::Result<mir::Buffer> {
    let mut buffer = mir::Buffer {
        name: name.into(),
        ..Default::default()
    };

    for required_key in ["address"] {
        if !map.contains_key(required_key) {
            bail!("Buffer definition must contain the '{required_key}' field");
        }
    }

    for (key, value) in map.iter() {
        match key {
            "type" => {}
            "cfg" => {
                buffer.cfg_attr =
                    mir::Cfg::new(Some(value.as_string().context("Parsing error for 'cfg'")?))
            }
            "description" => {
                buffer.description = value
                    .as_string()
                    .context("Parsing error for 'description'")?
                    .into()
            }
            "access" => {
                buffer.access = transform_access(value).context("Parsing error for 'access'")?;
            }
            "address" => {
                buffer.address = value.as_int().context("Parsing error for 'address'")?;
            }
            val => {
                bail!("Unexpected key: '{val}'")
            }
        }
    }

    Ok(buffer)
}

fn transform_ref(name: &str, map: &impl Map) -> anyhow::Result<mir::RefObject> {
    let mut ref_object = mir::RefObject {
        name: name.into(),
        ..Default::default()
    };

    for required_key in ["target", "override"] {
        if !map.contains_key(required_key) {
            bail!("Ref definition must contain the '{required_key}' field");
        }
    }

    let target = map
        .get("target")
        .unwrap()
        .as_string()
        .context("Parsing error for 'target'")?;
    let override_value = map.get("override").unwrap();

    for (key, value) in map.iter() {
        match key {
            "type" => {}
            "cfg" => {
                ref_object.cfg_attr =
                    mir::Cfg::new(Some(value.as_string().context("Parsing error for 'cfg'")?))
            }
            "description" => {
                ref_object.description = value
                    .as_string()
                    .context("Parsing error for 'description'")?
                    .into()
            }
            "target" => {}
            "override" => {}
            val => {
                bail!("Unexpected key: '{val}'")
            }
        }
    }

    ref_object.object_override = transform_object_override(target, override_value)
        .context("Parsing error for 'override'")?;

    Ok(ref_object)
}

fn transform_object_override(
    target: &str,
    override_value: &impl Value,
) -> anyhow::Result<mir::ObjectOverride> {
    let override_map = override_value.as_map()?;

    let object_type = override_map
        .get("type")
        .ok_or_else(|| anyhow!("No 'type' field present"))?
        .as_string()?;

    match object_type {
        "block" => Ok(mir::ObjectOverride::Block(transform_block_override(
            target,
            override_map,
        )?)),
        "register" => Ok(mir::ObjectOverride::Register(transform_register_override(
            target,
            override_map,
        )?)),
        "command" => Ok(mir::ObjectOverride::Command(transform_command_override(
            target,
            override_map,
        )?)),
        "buffer" => Err(anyhow!("Cannot make refs to 'buffer's")),
        "ref" => Err(anyhow!("Cannot make refs to 'ref's")),
        val => Err(anyhow!(
            "Unexpected object type '{val}'. Select one of \"block\", \"register\" or \"command\""
        )),
    }
}

fn transform_block_override(name: &str, map: &impl Map) -> anyhow::Result<mir::BlockOverride> {
    let mut block = mir::BlockOverride {
        name: name.into(),
        ..Default::default()
    };

    for (key, value) in map.iter() {
        match key {
            "type" => {},
            "address_offset" => block.address_offset = Some(value.as_int().context("Parsing error for 'address_offset'")?),
            "repeat" => block.repeat = Some(transform_repeat(value).context("Parsing error for 'repeat'")?),
            val => bail!(
                "Unexpected key found: '{val}'. Choose one of \"type\", \"address_offset\" or \"repeat\""
            ),
        }
    }

    Ok(block)
}

fn transform_register_override(
    name: &str,
    map: &impl Map,
) -> anyhow::Result<mir::RegisterOverride> {
    let mut register = mir::RegisterOverride {
        name: name.into(),
        ..Default::default()
    };

    for (key, value) in map.iter() {
        match key {
            "type" => {}
            "access" => {
                register.access =
                    Some(transform_access(value).context("Parsing error for 'access'")?);
            }
            "address" => {
                register.address = Some(value.as_int().context("Parsing error for 'address'")?);
            }
            "reset_value" => {
                register.reset_value = Some(if let Ok(rv) = value.as_uint() {
                    mir::ResetValue::Integer(rv as u128)
                } else if let Ok(rv) = value.as_array() {
                    match rv
                        .iter()
                        .map(|inner| {
                            inner
                                .as_uint()
                                .context("Array must contain bytes")
                                .map(|val| u8::try_from(val).context("Array must contain bytes"))
                                .and_then(identity)
                        })
                        .collect::<Result<Vec<_>, _>>()
                    {
                        Ok(val) => mir::ResetValue::Array(val),
                        Err(e) => return Err(e.context("Parsing error for 'reset_value")),
                    }
                } else {
                    return Err(anyhow!("Field must be an integer or an array")
                        .context("Parsing error for 'reset_value"));
                })
            }
            "repeat" => {
                register.repeat =
                    Some(transform_repeat(value).context("Parsing error for 'repeat")?);
            }
            "allow_address_overlap" => {
                register.allow_address_overlap = value
                    .as_bool()
                    .context("Parsing error for 'allow_address_overlap'")?;
            }
            val => {
                bail!("Unexpected key: '{val}'")
            }
        }
    }

    Ok(register)
}

fn transform_command_override(name: &str, map: &impl Map) -> anyhow::Result<mir::CommandOverride> {
    let mut command = mir::CommandOverride {
        name: name.into(),
        ..Default::default()
    };

    for (key, value) in map.iter() {
        match key {
            "type" => {}
            "address" => {
                command.address = Some(value.as_int().context("Parsing error for 'address'")?);
            }
            "repeat" => {
                command.repeat =
                    Some(transform_repeat(value).context("Parsing error for 'repeat")?);
            }
            "allow_address_overlap" => {
                command.allow_address_overlap = value
                    .as_bool()
                    .context("Parsing error for 'allow_address_overlap'")?;
            }
            val => {
                bail!("Unexpected key: '{val}'")
            }
        }
    }

    Ok(command)
}

fn transform_repeat(value: &impl Value) -> anyhow::Result<mir::Repeat> {
    let map = value.as_map()?;

    let count = map
        .get("count")
        .ok_or_else(|| anyhow!("Missing field 'count'"))?
        .as_uint()
        .context("Parsing field 'count'")?;
    let stride = map
        .get("stride")
        .ok_or_else(|| anyhow!("Missing field 'stride'"))?
        .as_int()
        .context("Parsing field 'stride'")?;

    // Check for fields we don't know
    if let Some((key, _)) = map
        .iter()
        .find(|(key, _)| *key != "count" && *key != "stride")
    {
        bail!("Unrecognized key: '{key}'. Only 'count' and 'stride' are valid fields")
    }

    Ok(mir::Repeat { count, stride })
}

fn transform_fields(value: &impl Value) -> anyhow::Result<Vec<mir::Field>> {
    value
        .as_map()?
        .iter()
        .map(|kv| transform_field(kv).with_context(|| format!("Parsing field '{}'", kv.0)))
        .collect()
}

fn transform_field((field_name, field_value): (&str, &impl Value)) -> anyhow::Result<mir::Field> {
    let mut field = mir::Field {
        name: field_name.into(),
        ..Default::default()
    };

    let field_map = field_value.as_map()?;

    for required_key in ["base", "start"] {
        if !field_map.contains_key(required_key) {
            bail!("Field definition must contain the '{required_key}' field");
        }
    }

    for (key, value) in field_map.iter() {
        match key {
            "cfg" => {
                field.cfg_attr =
                    Cfg::new(Some(value.as_string().context("Parsing error for 'cfg'")?))
            }
            "description" => {
                field.description = value
                    .as_string()
                    .context("Parsing error for 'description'")?
                    .into()
            }
            "access" => {
                field.access = transform_access(value).context("Parsing error for 'access'")?
            }
            "base" => {
                field.base_type = transform_base_type(value).context("Parsing error for 'base'")?
            }
            "conversion" => {
                field.field_conversion = Some(
                    transform_field_conversion(value, false)
                        .context("Parsing error for 'conversion'")?,
                )
            }
            "try_conversion" => {
                ensure!(
                    !field_map.contains_key("conversion"),
                    "Cannot have both 'conversion' and 'try_conversion' on a field. Pick one."
                );

                field.field_conversion = Some(
                    transform_field_conversion(value, true)
                        .context("Parsing error for 'try_conversion'")?,
                )
            }
            "start" => {
                field.field_address.start = value
                    .as_uint()
                    .context("Parsing error for 'start'")?
                    .try_into()
                    .context("Parsing error for 'start'")?;

                if !field_map.contains_key("end") {
                    field.field_address.end = field.field_address.start;
                }
            }
            "end" => {
                field.field_address.end = value
                    .as_uint()
                    .context("Parsing error for 'end'")?
                    .try_into()
                    .context("Parsing error for 'end'")?
            }
            val => {
                bail!("Unexpected key: '{val}'")
            }
        }
    }

    Ok(field)
}

fn transform_base_type(value: &impl Value) -> anyhow::Result<mir::BaseType> {
    match value.as_string()? {
        "bool" => Ok(mir::BaseType::Bool),
        "int" => Ok(mir::BaseType::Int),
        "uint" => Ok(mir::BaseType::Uint),
        val => Err(anyhow!(
            "Unexpected value: '{val}'. Choose one of 'bool', 'int' or 'uint'"
        )),
    }
}

fn transform_field_conversion(
    value: &impl Value,
    use_try: bool,
) -> anyhow::Result<mir::FieldConversion> {
    if let Ok(type_name) = value.as_string() {
        Ok(mir::FieldConversion::Direct {
            type_name: type_name.into(),
            use_try,
        })
    } else if let Ok(enum_map) = value.as_map() {
        let name = enum_map
            .get("name")
            .ok_or_else(|| anyhow!("Missing 'name' field"))?
            .as_string()?;
        let description = enum_map
            .get("description")
            .map(|description| description.as_string())
            .transpose()?;
        let variants = enum_map
            .iter()
            .filter(|(key, _)| *key != "name" && *key != "description")
            .map(|kv| {
                transform_enum_variant(kv).with_context(|| format!("Parsing variant '{}'", kv.0))
            })
            .collect::<Result<Vec<_>, _>>()
            .context("Parsing error for enum variant")?;

        Ok(mir::FieldConversion::Enum {
            enum_value: mir::Enum::new(
                description.unwrap_or_default().into(),
                name.into(),
                variants,
            ),
            use_try,
        })
    } else {
        Err(anyhow!(
            "Value must a string (to denote an existing type) or a map (to generate a new enum)"
        ))
    }
}

fn transform_enum_variant(
    (variant_name, variant_value): (&str, &impl Value),
) -> anyhow::Result<mir::EnumVariant> {
    if let Ok(map) = variant_value.as_map() {
        let cfg = map
            .get("cfg")
            .map(|cfg| cfg.as_string())
            .transpose()
            .context("Parsing 'cfg'")?;
        let description = map
            .get("description")
            .map(|descr| descr.as_string())
            .transpose()
            .context("Parsing 'description'")?;
        let value = map
            .get("value")
            .map(|value| transform_enum_value(value))
            .transpose()
            .context("Parsing 'description'")?;

        Ok(mir::EnumVariant {
            name: variant_name.into(),
            value: value.unwrap_or_default(),
            description: description.unwrap_or_default().into(),
            cfg_attr: Cfg::new(cfg),
        })
    } else {
        match transform_enum_value(variant_value) {
            Ok(value) => {
                Ok(mir::EnumVariant {
                    name: variant_name.into(),
                    value,
                    ..Default::default()
                })
            }
            Err(e) => {
                Err(anyhow!(
                    "Enum variant '{variant_name}' not recognized. Must be a 'map' for the extended definition. Cannot parse value as value directly: {e:#}"
                ))
            },
        }
    }
}

fn transform_enum_value(value: &impl Value) -> anyhow::Result<mir::EnumValue> {
    if value.as_null().is_ok() {
        Ok(mir::EnumValue::Unspecified)
    } else if let Ok(specified) = value.as_int() {
        Ok(mir::EnumValue::Specified(specified as i128))
    } else if let Ok(specified) = value.as_string() {
        match specified {
            "default" => Ok(mir::EnumValue::Default),
            "catch_all" => Ok(mir::EnumValue::CatchAll),
            val => Err(anyhow!(
                "Unexpected string value: '{val}'. Choose one of 'default' or 'catch_all'"
            )),
        }
    } else {
        Err(anyhow!(
            "Enum variant value not recognized. Must be one of 'null', 'int' or 'string'"
        ))
    }
}
#[cfg(test)]
mod tests {
    use mir::{ByteOrder, Cfg, Enum, EnumVariant, Field, Object, Register, Repeat, ResetValue};

    use super::*;

    #[test]
    fn global_config_parsed() {
        assert_eq!(
            transform_global_config(
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(
                    "
                        default_register_access: RO
                        default_buffer_access: WO
                        buffer_address_type: u8
                    "
                )
                .unwrap()
            )
            .unwrap(),
            mir::GlobalConfig {
                default_register_access: mir::Access::RO,
                default_buffer_access: mir::Access::WO,
                buffer_address_type: Some(mir::Integer::U8),
                ..Default::default()
            }
        );

        assert_eq!(
            transform_global_config(
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::TomlValue>(
                    "
                        default_field_access = \"RO\"
                        default_byte_order = \"BE\"
                        default_bit_order =  \"MSB0\"
                    "
                )
                .unwrap()
            )
            .unwrap(),
            mir::GlobalConfig {
                default_field_access: mir::Access::RO,
                default_byte_order: Some(mir::ByteOrder::BE),
                default_bit_order: mir::BitOrder::MSB0,
                ..Default::default()
            }
        );

        assert_eq!(
            transform_global_config(
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::JsonValue>(
                    "{
                        \"name_word_boundaries\": \"aA\",
                        \"register_address_type\": \"i16\",
                        \"command_address_type\":  \"u32\"
                    }"
                )
                .unwrap()
            )
            .unwrap(),
            mir::GlobalConfig {
                name_word_boundaries: Boundary::list_from("aA"),
                register_address_type: Some(mir::Integer::I16),
                command_address_type: Some(mir::Integer::U32),
                ..Default::default()
            }
        );

        assert_eq!(
            transform_global_config(
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(
                    "
                        test: 1
                    "
                )
                .unwrap()
            )
            .unwrap_err()
            .root_cause()
            .to_string(),
            "No config with key `test` is recognized"
        );

        assert_eq!(
            transform_global_config(
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(
                    "
                        default_register_access: Blah
                    "
                )
                .unwrap()
            )
            .unwrap_err().root_cause().to_string(),
            "No access value `Blah` exists. Values are limited to \"ReadWrite\", \"RW\", \"ReadOnly\", \"RO\", \"WriteOnly\", \"WO\""
        );

        assert_eq!(
            transform_global_config(
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(
                    "
                        default_register_access: 123
                    "
                )
                .unwrap()
            )
            .unwrap_err().root_cause().to_string(),
            "Value had an unexpected type. `string` was expected, but the actual value was `(u)int`"
        );
    }

    #[test]
    fn register_parsed() {
        assert_eq!(
            transform_object((
                "my_register",
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(
                    "
                        type: register
                    "
                )
                .unwrap()
            ))
            .unwrap_err()
            .root_cause()
            .to_string(),
            "Register definition must contain the 'address' field"
        );

        assert_eq!(
            transform_object((
                "my_register",
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(
                    "
                        type: register
                        address: 42
                    "
                )
                .unwrap()
            ))
            .unwrap_err()
            .root_cause()
            .to_string(),
            "Register definition must contain the 'size_bits' field"
        );

        assert_eq!(
            transform_object((
                "my_register",
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(
                    "
                        type: register
                        address: 42
                        size_bits: 8
                    "
                )
                .unwrap()
            ))
            .unwrap(),
            Object::Register(Register {
                name: "my_register".into(),
                address: 42,
                size_bits: 8,
                ..Default::default()
            })
        );

        assert_eq!(
            transform_object((
                "my_register",
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(
                    "
                        type: register
                        address: 42
                        size_bits: 8
                        access: WO
                        allow_address_overlap: true
                        allow_bit_overlap: true
                        bit_order: MSB0
                        byte_order: BE
                        repeat:
                            count: 12
                            stride: -3
                        reset_value: [1, 2, 3]
                        description: hello!
                        cfg: windows
                    "
                )
                .unwrap()
            ))
            .unwrap(),
            Object::Register(Register {
                name: "my_register".into(),
                address: 42,
                size_bits: 8,
                access: mir::Access::WO,
                allow_address_overlap: true,
                allow_bit_overlap: true,
                bit_order: mir::BitOrder::MSB0,
                byte_order: Some(ByteOrder::BE),
                repeat: Some(Repeat {
                    count: 12,
                    stride: -3
                }),
                reset_value: Some(ResetValue::Array(vec![1, 2, 3])),
                description: "hello!".into(),
                cfg_attr: Cfg::new(Some("windows")),
                ..Default::default()
            })
        );

        pretty_assertions::assert_eq!(
            transform_object((
                "my_register",
                &dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(
                    "
                        type: register
                        address: 42
                        size_bits: 9
                        fields:
                            test:
                                cfg: unix
                                description: The test field
                                base: int
                                access: RO
                                start: 0
                                end: 3
                            test2:
                                base: uint
                                start: 3
                                end: 6
                                try_conversion: MyStruct
                            test3:
                                base: int
                                start: 6
                                end: 9
                                conversion:
                                    name: MyEnum
                                    description: This is my enum
                                    var1: 1
                                    varEmpty:
                                    varDefault: default
                                    varDocumented:
                                        cfg: feature = \"foo\"
                                        description: This one is documented
                                        value: catch_all
                    "
                )
                .unwrap()
            ))
            .unwrap(),
            Object::Register(Register {
                name: "my_register".into(),
                address: 42,
                size_bits: 9,
                fields: vec![
                    Field {
                        cfg_attr: Cfg::new(Some("unix")),
                        description: "The test field".into(),
                        name: "test".into(),
                        access: mir::Access::RO,
                        base_type: mir::BaseType::Int,
                        field_conversion: None,
                        field_address: 0..3,
                    },
                    Field {
                        cfg_attr: Default::default(),
                        description: Default::default(),
                        name: "test2".into(),
                        access: Default::default(),
                        base_type: mir::BaseType::Uint,
                        field_conversion: Some(mir::FieldConversion::Direct {
                            type_name: "MyStruct".into(),
                            use_try: true
                        }),
                        field_address: 3..6
                    },
                    Field {
                        cfg_attr: Default::default(),
                        description: Default::default(),
                        name: "test3".into(),
                        access: Default::default(),
                        base_type: mir::BaseType::Int,
                        field_conversion: Some(mir::FieldConversion::Enum {
                            enum_value: Enum::new(
                                "This is my enum".into(),
                                "MyEnum".into(),
                                vec![
                                    EnumVariant {
                                        cfg_attr: Default::default(),
                                        description: Default::default(),
                                        name: "var1".into(),
                                        value: mir::EnumValue::Specified(1),
                                    },
                                    EnumVariant {
                                        cfg_attr: Default::default(),
                                        description: Default::default(),
                                        name: "varEmpty".into(),
                                        value: mir::EnumValue::Unspecified,
                                    },
                                    EnumVariant {
                                        cfg_attr: Default::default(),
                                        description: Default::default(),
                                        name: "varDefault".into(),
                                        value: mir::EnumValue::Default,
                                    },
                                    EnumVariant {
                                        cfg_attr: Cfg::new(Some("feature = \"foo\"")),
                                        description: "This one is documented".into(),
                                        name: "varDocumented".into(),
                                        value: mir::EnumValue::CatchAll,
                                    }
                                ]
                            ),
                            use_try: false
                        }),
                        field_address: 6..9
                    }
                ],
                ..Default::default()
            })
        );
    }
}
