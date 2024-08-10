use proc_macro2::Span;

use crate::{dsl_hir, mir};

pub fn transform(device: dsl_hir::Device) -> Result<mir::Device, syn::Error> {
    let global_config = device.global_config_list.try_into()?;
    let mut objects = Vec::new();
    transform_object_list(device.object_list, &global_config, &mut objects)?;

    Ok(mir::Device {
        global_config,
        objects,
    })
}

impl From<dsl_hir::Access> for mir::Access {
    fn from(value: dsl_hir::Access) -> Self {
        match value {
            dsl_hir::Access::RW => mir::Access::RW,
            dsl_hir::Access::RC => mir::Access::RC,
            dsl_hir::Access::RO => mir::Access::RO,
            dsl_hir::Access::WO => mir::Access::WO,
            dsl_hir::Access::CO => mir::Access::CO,
        }
    }
}

impl From<dsl_hir::ByteOrder> for mir::ByteOrder {
    fn from(value: dsl_hir::ByteOrder) -> Self {
        match value {
            dsl_hir::ByteOrder::LE => mir::ByteOrder::LE,
            dsl_hir::ByteOrder::BE => mir::ByteOrder::BE,
        }
    }
}

impl From<dsl_hir::BitOrder> for mir::BitOrder {
    fn from(value: dsl_hir::BitOrder) -> Self {
        match value {
            dsl_hir::BitOrder::LSB0 => mir::BitOrder::LSB0,
            dsl_hir::BitOrder::MSB0 => mir::BitOrder::MSB0,
        }
    }
}

impl TryFrom<syn::Ident> for mir::Integer {
    type Error = syn::Error;

    fn try_from(value: syn::Ident) -> Result<Self, Self::Error> {
        match value.to_string().as_str() {
            "u8" => Ok(mir::Integer::U8),
            "u16" => Ok(mir::Integer::U16),
            "u32" => Ok(mir::Integer::U32),
            "u64" => Ok(mir::Integer::U64),
            "u128" => Ok(mir::Integer::U128),
            "i8" => Ok(mir::Integer::I8),
            "i16" => Ok(mir::Integer::I16),
            "i32" => Ok(mir::Integer::I32),
            "i64" => Ok(mir::Integer::I64),
            "i128" => Ok(mir::Integer::I128),
            _ => Err(syn::Error::new(value.span(), "Must be an integer type")),
        }
    }
}

impl From<dsl_hir::NameCase> for mir::NameCase {
    fn from(value: dsl_hir::NameCase) -> Self {
        match value {
            dsl_hir::NameCase::Varying => mir::NameCase::Varying,
            dsl_hir::NameCase::Pascal => mir::NameCase::Pascal,
            dsl_hir::NameCase::Snake => mir::NameCase::Snake,
            dsl_hir::NameCase::ScreamingSnake => mir::NameCase::ScreamingSnake,
            dsl_hir::NameCase::Camel => mir::NameCase::Camel,
            dsl_hir::NameCase::Kebab => mir::NameCase::Kebab,
            dsl_hir::NameCase::Cobol => mir::NameCase::Cobol,
        }
    }
}

impl TryFrom<dsl_hir::GlobalConfigList> for mir::GlobalConfig {
    type Error = syn::Error;

    fn try_from(value: dsl_hir::GlobalConfigList) -> Result<Self, Self::Error> {
        let mut global_config = mir::GlobalConfig::default();

        for config in value.configs.iter() {
            let same_config_count = value
                .configs
                .iter()
                .filter(|check_config| {
                    std::mem::discriminant(*check_config) == std::mem::discriminant(config)
                })
                .count();

            if same_config_count > 1 {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Duplicate global config found: `{config:?}`"),
                ));
            }

            match config.clone() {
                dsl_hir::GlobalConfig::DefaultRegisterAccess(value) => {
                    global_config.default_register_access = value.into()
                }
                dsl_hir::GlobalConfig::DefaultFieldAccess(value) => {
                    global_config.default_field_access = value.into()
                }
                dsl_hir::GlobalConfig::DefaultBufferAccess(value) => {
                    global_config.default_buffer_access = value.into()
                }
                dsl_hir::GlobalConfig::DefaultByteOrder(value) => {
                    global_config.default_byte_order = value.into()
                }
                dsl_hir::GlobalConfig::DefaultBitOrder(value) => {
                    global_config.default_bit_order = value.into()
                }
                dsl_hir::GlobalConfig::RegisterAddressType(value) => {
                    global_config.register_address_type = Some(value.try_into()?)
                }
                dsl_hir::GlobalConfig::CommandAddressType(value) => {
                    global_config.command_address_type = Some(value.try_into()?)
                }
                dsl_hir::GlobalConfig::BufferAddressType(value) => {
                    global_config.buffer_address_type = Some(value.try_into()?)
                }
                dsl_hir::GlobalConfig::NameCase(value) => global_config.name_case = value.into(),
            }
        }

        Ok(global_config)
    }
}

fn get_description(attrs: &dsl_hir::AttributeList) -> String {
    attrs
        .attributes
        .iter()
        .filter_map(|attr| match attr {
            dsl_hir::Attribute::Doc(val, _) => Some(val.as_str()),
            dsl_hir::Attribute::Cfg(_, _) => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_cfg_attr(attrs: &dsl_hir::AttributeList) -> Result<Option<String>, syn::Error> {
    let mut cfg_attrs = attrs
        .attributes
        .iter()
        .filter_map(|attr| match attr {
            dsl_hir::Attribute::Cfg(val, span) => Some((val, span)),
            dsl_hir::Attribute::Doc(_, _) => None,
        })
        .collect::<Vec<_>>();

    match cfg_attrs.len() {
        0 => Ok(None),
        1 => Ok(Some(cfg_attrs.remove(0).0.clone())),
        n => Err(syn::Error::new(
            cfg_attrs.remove(1).1.clone(),
            &format!("Only one cfg attribute is allowed, but {n} are found"),
        )),
    }
}

fn transform_object_list(
    list: dsl_hir::ObjectList,
    global_config: &mir::GlobalConfig,
    objects: &mut Vec<mir::Object>,
) -> Result<(), syn::Error> {
    for object in list.objects.into_iter() {
        let object = match object {
            dsl_hir::Object::Block(_) => todo!(),
            dsl_hir::Object::Register(_) => todo!(),
            dsl_hir::Object::Command(_) => todo!(),
            dsl_hir::Object::Buffer(buffer) => mir::Object::Buffer(mir::Buffer {
                cfg_attrs: get_cfg_attr(&buffer.attribute_list)?,
                description: get_description(&buffer.attribute_list),
                name: buffer.identifier.to_string(),
                access: buffer
                    .access
                    .map(Into::into)
                    .unwrap_or(global_config.default_buffer_access),
                address: buffer
                    .address
                    .ok_or_else(|| {
                        syn::Error::new(
                            Span::call_site(),
                            &format!("Buffer `{}` has no address", buffer.identifier.to_string()),
                        )
                    })?
                    .base10_parse()?,
            }),
            dsl_hir::Object::Ref(_) => todo!(),
        };

        objects.push(object);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_double_global_settings() {
        let device = syn::parse_str::<dsl_hir::Device>(
            "config { type DefaultRegisterAccess = RW; type DefaultRegisterAccess = RW; }",
        )
        .unwrap();

        assert_eq!(
            transform(device).unwrap_err().to_string(),
            "Duplicate global config found: `DefaultRegisterAccess(RW)`"
        );
    }

    #[test]
    fn global_settings_correct() {
        let device = syn::parse_str::<dsl_hir::Device>(
            "config {
                type DefaultRegisterAccess = RO;
                type DefaultFieldAccess = RC;
                type DefaultBufferAccess = WO;
                type DefaultByteOrder = LE;
                type DefaultBitOrder = MSB0;
                type RegisterAddressType = i8;
                type CommandAddressType = u128;
                type BufferAddressType = u32;
                type NameCase = Pascal;
            }",
        )
        .unwrap();

        let device = transform(device).unwrap();

        assert_eq!(
            device.global_config,
            mir::GlobalConfig {
                default_register_access: mir::Access::RO,
                default_field_access: mir::Access::RC,
                default_buffer_access: mir::Access::WO,
                default_byte_order: mir::ByteOrder::LE,
                default_bit_order: mir::BitOrder::MSB0,
                register_address_type: Some(mir::Integer::I8),
                command_address_type: Some(mir::Integer::U128),
                buffer_address_type: Some(mir::Integer::U32),
                name_case: mir::NameCase::Pascal,
            }
        );
    }

    #[test]
    fn buffer() {
        assert_eq!(
            transform(
                syn::parse_str::<dsl_hir::Device>(
                    "
                    /// Hello world!
                    #[cfg(feature = \"foo\")]
                    /// This should be in order!
                    buffer Foo: RO = 5
                    ",
                )
                .unwrap()
            )
            .unwrap()
            .objects,
            &[mir::Object::Buffer(mir::Buffer {
                cfg_attrs: Some("feature = \"foo\"".into()),
                description: " Hello world!\n This should be in order!".into(),
                name: "Foo".into(),
                access: mir::Access::RO,
                address: 5,
            })]
        );

        assert_eq!(
            transform(
                syn::parse_str::<dsl_hir::Device>(
                    "
                    buffer Foo
                    ",
                )
                .unwrap()
            )
            .unwrap_err()
            .to_string(),
            "Buffer `Foo` has no address"
        );
    }

    #[test]
    fn max_one_cfg_attr() {
        assert_eq!(
            transform(
                syn::parse_str::<dsl_hir::Device>(
                    "
                    buffer Foo = 5
                    "
                )
                .unwrap()
            )
            .unwrap()
            .objects,
            &[mir::Object::Buffer(mir::Buffer {
                cfg_attrs: None,
                description: "".into(),
                name: "Foo".into(),
                access: mir::Access::default(),
                address: 5,
            })]
        );
        assert_eq!(
            transform(
                syn::parse_str::<dsl_hir::Device>(
                    "
                    #[cfg(foo)]
                    buffer Foo = 5
                    "
                )
                .unwrap()
            )
            .unwrap()
            .objects,
            &[mir::Object::Buffer(mir::Buffer {
                cfg_attrs: Some("foo".into()),
                description: "".into(),
                name: "Foo".into(),
                access: mir::Access::default(),
                address: 5,
            })]
        );
        assert_eq!(
            transform(
                syn::parse_str::<dsl_hir::Device>(
                    "
                    #[cfg(foo)]
                    #[cfg(too_many)]
                    buffer Foo = 5
                    "
                )
                .unwrap()
            )
            .unwrap_err()
            .to_string(),
            "Only one cfg attribute is allowed, but 2 are found"
        );
    }
}
