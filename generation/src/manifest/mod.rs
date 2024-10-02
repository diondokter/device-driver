use anyhow::{anyhow, bail, Context};
use convert_case::Boundary;
use dd_manifest_tree::{Map, Value};

use crate::mir;

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
    println!("{value:?}");

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
                todo!()
            }
            "repeat" => {
                todo!()
            }
            "allow_bit_overlap" => {
                todo!()
            }
            "allow_address_overlap" => {
                todo!()
            }
            "fields" => {
                todo!()
            }
            val => {
                bail!("Unexpected key found: '{val}'")
            }
        }
    }

    todo!()
}

fn transform_command(name: &str, map: &impl Map) -> anyhow::Result<mir::Command> {
    todo!()
}

fn transform_buffer(name: &str, map: &impl Map) -> anyhow::Result<mir::Buffer> {
    todo!()
}

fn transform_ref(name: &str, map: &impl Map) -> anyhow::Result<mir::RefObject> {
    todo!()
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

#[cfg(test)]
mod tests {
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
}
