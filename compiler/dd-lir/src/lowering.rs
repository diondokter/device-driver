use std::ops::Add;

use convert_case::Case;
use device_driver_common::{
    identifier::Identifier,
    specifiers::{BaseType, Integer, Repeat, RepeatSource},
};

use crate::model as lir;
use device_driver_mir::{
    find_min_max_addresses,
    model::{self as mir, Object},
    search_object,
};

pub fn transform_devices(manifest: &mir::Manifest) -> Vec<lir::Device> {
    manifest
        .iter_devices_with_config()
        .map(|(device, config)| {
            // Create a root block and pass the device objects to it
            let blocks = collect_into_blocks(
                BorrowedBlock {
                    description: &format!(
                        "{}Root block of the {} driver",
                        if device.description.is_empty() {
                            String::new()
                        } else {
                            format!("{}\n\n", device.description)
                        },
                        device.name.to_case(Case::Pascal),
                    ),
                    name: &device.name,
                    address_offset: &0,
                    repeat: &None,
                    objects: &device.objects,
                },
                true,
                &device.device_config,
                manifest,
            );

            lir::Device {
                internal_address_type: find_best_internal_address_type(manifest, device),
                blocks,
                defmt_feature: config.defmt_feature.clone(),
            }
        })
        .collect()
}

fn collect_into_blocks(
    block: BorrowedBlock,
    is_root: bool,
    global_config: &mir::DeviceConfig,
    manifest: &mir::Manifest,
) -> Vec<lir::Block> {
    let mut blocks = Vec::new();

    let BorrowedBlock {
        description,
        name,
        address_offset: _,
        repeat: _,
        objects,
    } = block;

    let mut methods = Vec::new();

    for object in objects {
        let Some(method) = get_method(object, &mut blocks, global_config, manifest) else {
            continue;
        };

        methods.push(method);
    }

    let new_block = lir::Block {
        description: description.clone(),
        root: is_root,
        name: name.clone(),
        methods,
    };

    blocks.insert(0, new_block);

    blocks
}

fn get_method(
    object: &mir::Object,
    blocks: &mut Vec<lir::Block>,
    global_config: &mir::DeviceConfig,
    manifest: &mir::Manifest,
) -> Option<lir::BlockMethod> {
    match object {
        mir::Object::Device(_) => None,
        mir::Object::Block(
            b @ mir::Block {
                description,
                name,
                address_offset,
                repeat,
                ..
            },
        ) => {
            blocks.extend(collect_into_blocks(
                b.into(),
                false,
                global_config,
                manifest,
            ));

            Some(lir::BlockMethod {
                description: description.clone(),
                name: name.value.clone(),
                address: address_offset.value,
                repeat: repeat_to_method_kind(repeat, manifest),
                method_type: lir::BlockMethodType::Block {
                    name: name.value.clone(),
                },
            })
        }
        mir::Object::Register(mir::Register {
            description,
            name,
            address,
            access,
            repeat,
            field_set_ref,
            reset_value,
            ..
        }) => {
            let field_set =
                search_object(manifest, field_set_ref).expect("Existence checked in MIR pass");

            Some(lir::BlockMethod {
                description: description.clone(),
                name: name.value.clone(),
                address: address.value,
                repeat: repeat_to_method_kind(repeat, manifest),
                method_type: lir::BlockMethodType::Register {
                    field_set_name: field_set.name().clone(),
                    access: *access,
                    address_type: global_config
                        .register_address_type
                        .expect("The presence of the address type is already checked in a mir pass")
                        .value,
                    reset_value: reset_value.clone().map(|rv| {
                        rv.as_array()
                        .expect(
                            "Reset value should already be converted to array here in a mir pass",
                        )
                        .clone()
                    }),
                },
            })
        }
        mir::Object::Command(mir::Command {
            description,
            name,
            address,
            repeat,
            field_set_ref_in,
            field_set_ref_out,
            ..
        }) => {
            let field_set_in = field_set_ref_in.as_ref().map(|id_ref| {
                search_object(manifest, id_ref).expect("Existence checked in MIR pass")
            });
            let field_set_out = field_set_ref_out.as_ref().map(|id_ref| {
                search_object(manifest, id_ref).expect("Existence checked in MIR pass")
            });

            Some(lir::BlockMethod {
                description: description.clone(),
                name: name.value.clone(),
                address: address.value,
                repeat: repeat_to_method_kind(repeat, manifest),
                method_type: lir::BlockMethodType::Command {
                    field_set_name_in: field_set_in.map(|fs_in| fs_in.name().clone()),
                    field_set_name_out: field_set_out.map(|fs_out| fs_out.name().clone()),
                    address_type: global_config
                        .command_address_type
                        .expect("The presence of the address type is already checked in a mir pass")
                        .value,
                },
            })
        }
        mir::Object::Buffer(mir::Buffer {
            description,
            name,
            access,
            address,
        }) => Some(lir::BlockMethod {
            description: description.clone(),
            name: name.value.clone(),
            address: address.value,
            repeat: lir::Repeat::None, // Buffers can't be repeated (for now?)
            method_type: lir::BlockMethodType::Buffer {
                access: *access,
                address_type: global_config
                    .buffer_address_type
                    .expect("The presence of the address type is already checked in a mir pass")
                    .value,
            },
        }),
        mir::Object::FieldSet(_) => None,
        mir::Object::Enum(_) => None,
        mir::Object::Extern(_) => None,
    }
}

pub fn transform_field_sets(manifest: &mir::Manifest) -> Vec<lir::FieldSet> {
    manifest
        .iter_objects_with_config()
        .filter_map(|(o, config)| {
            if let Object::FieldSet(fs) = o {
                Some((fs, config))
            } else {
                None
            }
        })
        .map(|(fs, config)| transform_field_set(manifest, fs, &config))
        .collect()
}

fn transform_field_set(
    manifest: &mir::Manifest,
    field_set: &mir::FieldSet,
    config: &mir::DeviceConfig,
) -> lir::FieldSet {
    let fields = field_set
        .fields
        .iter()
        .map(|field| {
            let mir::Field {
                description,
                name,
                access,
                base_type,
                field_conversion,
                field_address,
                repeat,
            } = field;

            let (base_type, conversion_method) = match (base_type.value, field_conversion) {
                (BaseType::Unspecified | BaseType::Uint | BaseType::Int, _) => {
                    unreachable!("Nothing is left unspecified or unsized in the mir passes")
                }
                (BaseType::Bool, None) if field_address.len() == 1 => {
                    ("u8".to_string(), lir::FieldConversionMethod::Bool)
                }
                (BaseType::Bool, _) => unreachable!(
                    "Checked in a MIR pass. Bools can only be 1 bit and have no conversion"
                ),
                (BaseType::FixedSize(integer), None) => {
                    (integer.to_string(), lir::FieldConversionMethod::None)
                }
                (BaseType::FixedSize(integer), Some(fc)) => (integer.to_string(), {
                    let field_bits = field.field_address.len() as u32;

                    let fc_identifier = search_object(manifest, &fc.type_name)
                        .expect("Object existence checked in MIR pass")
                        .name()
                        .clone();

                    // Always use try if that's specified
                    if fc.fallible {
                        lir::FieldConversionMethod::TryInto(fc_identifier)
                    }
                    // Are we pointing at a potentially infallible enum and do we fulfil the requirements?
                    else if let Some(mir::Enum {
                        generation_style: Some(mir::EnumGenerationStyle::InfallibleWithinRange),
                        size_bits,
                        ..
                    }) = manifest
                        .iter_enums()
                        .find(|e| e.name.take_ref() == fc.type_name.value)
                        && field_bits <= size_bits.expect("Size_bits set in an earlier mir pass")
                    {
                        // This field is equal or smaller in bits than the infallible enum. So we can do the unsafe into
                        lir::FieldConversionMethod::UnsafeInto(fc_identifier)
                    } else {
                        // Fallback is to use the into trait.
                        // This is correct because in the field_conversion_valid mir pass we've already exited if we need a try and didn't specify it.
                        // The only other option is the unsafe into and we've just checked that.
                        lir::FieldConversionMethod::Into(fc_identifier)
                    }
                }),
            };

            lir::Field {
                description: description.clone(),
                name: name.value.clone(),
                address: field_address.value.clone(),
                base_type,
                conversion_method,
                access: *access,
                repeat: repeat
                    .as_ref()
                    .map_or(lir::Repeat::None, |repeat| match &repeat.source {
                        RepeatSource::Count(c) => lir::Repeat::Count {
                            count: *c,
                            stride: repeat.stride,
                        },
                        RepeatSource::Enum(enum_name) => {
                            let target_enum = search_object(manifest, enum_name)
                                .expect("Existence checked in MIR pass")
                                .as_enum()
                                .expect("checked in MIR pass");
                            lir::Repeat::Enum {
                                enum_name: target_enum.name.value.clone(),
                                enum_variants: target_enum
                                    .variants
                                    .iter()
                                    .map(|variant| variant.name.value.clone())
                                    .collect(),
                                stride: repeat.stride,
                            }
                        }
                    }),
            }
        })
        .collect();

    lir::FieldSet {
        description: field_set.description.clone(),
        name: field_set.name.value.clone(),
        byte_order: field_set
            .byte_order
            .expect("Byte order should never be none at this point after the MIR passes"),
        bit_order: field_set
            .bit_order
            .expect("Bitorder should never be none at this point after the MIR passes"),
        size_bits: field_set.size_bits.value,
        fields,
        defmt_feature: config.defmt_feature.clone(),
    }
}

pub fn transform_enums(manifest: &mir::Manifest) -> Vec<lir::Enum> {
    manifest.iter_enums_with_config().map(|(e, config)| {
        let mir::Enum {
            description,
            name,
            variants: _,
            base_type,
            size_bits: _,
            generation_style: _,
        } = e;

        let base_type = match base_type.value {
            BaseType::FixedSize(integer) => integer.to_string(),
            _ => {
                panic!("Enum base type should be set to fixed size integer in a mir pass at this point")
            }
        };

        let variants = e
            .iter_variants_with_discriminant()
            .map(|(discriminant, v)| {
                let mir::EnumVariant {
                    description,
                    name,
                    value,
                } = v;

                lir::EnumVariant {
                    description: description.clone(),
                    name: name.value.clone(),
                    discriminant,
                    default: matches!(value, mir::EnumValue::Default),
                    catch_all: matches!(value, mir::EnumValue::CatchAll),
                }
            })
            .collect();

        lir::Enum {
            description: description.clone(),
            name: name.value.clone(),
            base_type,
            variants,
            defmt_feature: config.defmt_feature.clone(),
        }
    }).collect()
}

fn repeat_to_method_kind(repeat: &Option<Repeat>, manifest: &mir::Manifest) -> lir::Repeat {
    match repeat {
        Some(Repeat {
            source: RepeatSource::Count(count),
            stride,
        }) => lir::Repeat::Count {
            count: *count,
            stride: *stride,
        },
        Some(Repeat {
            source: RepeatSource::Enum(enum_name),
            stride,
        }) => {
            let target_enum = search_object(manifest, enum_name)
                .expect("Existence checked in MIR pass")
                .as_enum()
                .expect("checked in MIR pass");
            lir::Repeat::Enum {
                enum_name: target_enum.name.value.clone(),
                enum_variants: target_enum
                    .variants
                    .iter()
                    .map(|variant| variant.name.value.clone())
                    .collect(),
                stride: *stride,
            }
        }
        None => lir::Repeat::None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorrowedBlock<'o> {
    pub description: &'o String,
    pub name: &'o Identifier,
    pub address_offset: &'o i128,
    pub repeat: &'o Option<Repeat>,
    pub objects: &'o [mir::Object],
}

impl<'o> From<&'o mir::Block> for BorrowedBlock<'o> {
    fn from(value: &'o mir::Block) -> Self {
        let mir::Block {
            description,
            name,
            address_offset,
            repeat,
            objects,
        } = value;

        Self {
            description,
            name,
            address_offset,
            repeat,
            objects,
        }
    }
}

fn find_best_internal_address_type(manifest: &mir::Manifest, device: &mir::Device) -> Integer {
    let (min_address_found, max_address_found) = find_min_max_addresses(manifest, device, |_| true)
        .map(|((min, _), (max, _))| (min, max))
        .unwrap_or_default();

    let needs_signed = min_address_found < 0;
    let needs_bits = (min_address_found
        .unsigned_abs()
        .max(max_address_found.unsigned_abs())
        .add(1)
        .next_power_of_two()
        .ilog2()
        + u32::from(needs_signed))
    .next_power_of_two()
    .max(8);

    if needs_signed {
        match needs_bits {
            8 => Integer::I8,
            16 => Integer::I16,
            32 => Integer::I32,
            64 => Integer::I64,
            _ => unreachable!(),
        }
    } else {
        match needs_bits {
            8 => Integer::U8,
            16 => Integer::U16,
            32 => Integer::U32,
            64 => Integer::U64,
            _ => unreachable!(),
        }
    }
}
