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
    todo!()
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
