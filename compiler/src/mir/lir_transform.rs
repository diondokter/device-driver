use std::ops::Add;

use crate::{
    lir,
    mir::{self, Object},
};

use super::{
    Integer,
    passes::{find_min_max_addresses, recurse_objects},
};

pub fn transform(device: mir::Device) -> miette::Result<lir::Device> {
    let driver_name = device.name.clone().unwrap();
    let mir_enums = collect_enums(&device)?;
    let lir_enums = mir_enums
        .iter()
        .map(transform_enum)
        .collect::<Result<_, miette::Report>>()?;

    let field_sets = transform_field_sets(&device, &mir_enums)?;

    // Create a root block and pass the device objects to it
    let blocks = collect_into_blocks(
        BorrowedBlock {
            description: &format!("Root block of the {driver_name} driver"),
            name: &driver_name,
            address_offset: &0,
            repeat: &None,
            objects: &device.objects,
        },
        true,
        &device.global_config,
        &device.objects,
    )?;

    Ok(lir::Device {
        internal_address_type: find_best_internal_address_type(&device),
        register_address_type: device
            .global_config
            .register_address_type
            .unwrap_or(mir::Integer::U8),
        blocks,
        field_sets,
        enums: lir_enums,
        defmt_feature: device.global_config.defmt_feature,
    })
}

fn collect_into_blocks(
    block: BorrowedBlock,
    is_root: bool,
    global_config: &mir::GlobalConfig,
    device_objects: &[mir::Object],
) -> miette::Result<Vec<lir::Block>> {
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
        let Some(method) = get_method(object, &mut blocks, global_config, device_objects)? else {
            continue;
        };

        methods.push(method);
    }

    let new_block = lir::Block {
        description: description.clone(),
        root: is_root,
        name: name.to_string(),
        methods,
    };

    blocks.insert(0, new_block);

    Ok(blocks)
}

fn get_method(
    object: &mir::Object,
    blocks: &mut Vec<lir::Block>,
    global_config: &mir::GlobalConfig,
    device_objects: &[mir::Object],
) -> Result<Option<lir::BlockMethod>, miette::Report> {
    use convert_case::Casing;

    Ok(match object {
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
                device_objects,
            )?);

            Some(lir::BlockMethod {
                description: description.clone(),
                name: name.to_case(convert_case::Case::Snake),
                address: *address_offset,
                allow_address_overlap: false,
                repeat: repeat_to_method_kind(repeat, device_objects),
                method_type: lir::BlockMethodType::Block {
                    name: name.to_string(),
                },
            })
        }
        mir::Object::Register(mir::Register {
            description,
            name,
            allow_address_overlap,
            address,
            access,
            repeat,
            field_set_ref: field_set,
            reset_value,
        }) => Some(lir::BlockMethod {
            description: description.clone(),
            name: name.to_case(convert_case::Case::Snake),
            address: *address,
            allow_address_overlap: *allow_address_overlap,
            repeat: repeat_to_method_kind(repeat, device_objects),
            method_type: lir::BlockMethodType::Register {
                field_set_name: field_set.0.clone(),
                access: *access,
                address_type: global_config
                    .register_address_type
                    .expect("The presence of the address type is already checked in a mir pass"),
                reset_value: reset_value.clone().map(|rv| {
                    rv.as_array()
                        .expect(
                            "Reset value should already be converted to array here in a mir pass",
                        )
                        .clone()
                }),
            },
        }),
        mir::Object::Command(mir::Command {
            description,
            name,
            allow_address_overlap,
            address,
            repeat,
            field_set_ref_in: field_set_in,
            field_set_ref_out: field_set_out,
            ..
        }) => Some(lir::BlockMethod {
            description: description.clone(),
            name: name.to_case(convert_case::Case::Snake),
            address: *address,
            allow_address_overlap: *allow_address_overlap,
            repeat: repeat_to_method_kind(repeat, device_objects),
            method_type: lir::BlockMethodType::Command {
                field_set_name_in: field_set_in.as_ref().map(|fs_in| fs_in.0.clone()),
                field_set_name_out: field_set_out.as_ref().map(|fs_out| fs_out.0.clone()),
                address_type: global_config
                    .command_address_type
                    .expect("The presence of the address type is already checked in a mir pass"),
            },
        }),
        mir::Object::Buffer(mir::Buffer {
            description,
            name,
            access,
            address,
        }) => Some(lir::BlockMethod {
            description: description.clone(),
            name: name.to_case(convert_case::Case::Snake),
            address: *address,
            allow_address_overlap: false,
            repeat: lir::Repeat::None, // Buffers can't be repeated (for now?)
            method_type: lir::BlockMethodType::Buffer {
                access: *access,
                address_type: global_config
                    .buffer_address_type
                    .expect("The presence of the address type is already checked in a mir pass"),
            },
        }),
        mir::Object::FieldSet(_) => None,
        mir::Object::Enum(_) => None,
        mir::Object::Extern(_) => None,
    })
}

fn transform_field_sets(
    device: &mir::Device,
    mir_enums: &[mir::Enum],
) -> miette::Result<Vec<lir::FieldSet>> {
    let mut field_sets = Vec::new();

    recurse_objects(&device.objects, &mut |object| {
        if let mir::Object::FieldSet(fs) = object {
            field_sets.push(transform_field_set(fs, mir_enums)?);
        }

        Ok(())
    })?;

    Ok(field_sets)
}

fn transform_field_set(
    field_set: &mir::FieldSet,
    enum_list: &[mir::Enum],
) -> miette::Result<lir::FieldSet> {
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

            let (base_type, conversion_method) = match (base_type, field_conversion) {
                (mir::BaseType::Unspecified | mir::BaseType::Uint | mir::BaseType::Int, _) => {
                    unreachable!("Nothing is left unspecified or unsized in the mir passes")
                }
                (mir::BaseType::Bool, None) if field_address.len() == 1 => {
                    ("u8".to_string(), lir::FieldConversionMethod::Bool)
                }
                (mir::BaseType::Bool, _) => unreachable!(
                    "Checked in a MIR pass. Bools can only be 1 bit and have no conversion"
                ),
                (mir::BaseType::FixedSize(integer), None) => {
                    (integer.to_string(), lir::FieldConversionMethod::None)
                }
                (mir::BaseType::FixedSize(integer), Some(fc)) => (integer.to_string(), {
                    let field_bits = field.field_address.len() as u32;

                    // Always use try if that's specified
                    if fc.use_try {
                        lir::FieldConversionMethod::TryInto(fc.type_name.clone())
                    }
                    // Are we pointing at a potentially infallible enum and do we fulfil the requirements?
                    else if let Some(mir::Enum {
                        generation_style: Some(mir::EnumGenerationStyle::InfallibleWithinRange),
                        size_bits,
                        ..
                    }) = enum_list.iter().find(|e| e.name == fc.type_name)
                        && field_bits <= size_bits.expect("Size_bits set in an earlier mir pass")
                    {
                        // This field is equal or smaller in bits than the infallible enum. So we can do the unsafe into
                        lir::FieldConversionMethod::UnsafeInto(fc.type_name.clone())
                    } else {
                        // Fallback is to use the into trait.
                        // This is correct because in the field_conversion_valid mir pass we've already exited if we need a try and didn't specify it.
                        // The only other option is the unsafe into and we've just checked that.
                        lir::FieldConversionMethod::Into(fc.type_name.clone())
                    }
                }),
            };

            Ok(lir::Field {
                description: description.clone(),
                name: name.clone(),
                address: field_address.clone(),
                base_type,
                conversion_method,
                access: *access,
                repeat: repeat
                    .as_ref()
                    .map(|repeat| match &repeat.source {
                        mir::RepeatSource::Count(c) => lir::Repeat::Count {
                            count: *c,
                            stride: repeat.stride,
                        },
                        mir::RepeatSource::Enum(enum_name) => lir::Repeat::Enum {
                            enum_name: enum_name.clone(),
                            enum_variants: enum_list
                                .iter()
                                .find(|e| e.name == *enum_name)
                                .expect("Checked in MIR pass")
                                .variants
                                .iter()
                                .map(|variant| variant.name.clone())
                                .collect(),
                            stride: repeat.stride,
                        },
                    })
                    .unwrap_or(lir::Repeat::None),
            })
        })
        .collect::<Result<_, miette::Report>>()?;

    Ok(lir::FieldSet {
        description: field_set.description.clone(),
        name: field_set.name.clone(),
        byte_order: field_set.byte_order.unwrap(),
        bit_order: field_set
            .bit_order
            .expect("Bitorder should never be none at this point after the MIR passes"),
        size_bits: field_set.size_bits,
        fields,
    })
}

fn collect_enums(device: &mir::Device) -> miette::Result<Vec<mir::Enum>> {
    let mut enums = Vec::new();

    recurse_objects(&device.objects, &mut |object| {
        if let Object::Enum(e) = object {
            enums.push(e.clone());
        }

        Ok(())
    })?;

    Ok(enums)
}

fn transform_enum(e: &mir::Enum) -> miette::Result<lir::Enum> {
    let mir::Enum {
        description,
        name,
        variants: _,
        base_type,
        size_bits: _,
        generation_style: _,
    } = e;

    let base_type = match base_type {
        mir::BaseType::FixedSize(integer) => integer.to_string(),
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

            Ok(lir::EnumVariant {
                description: description.clone(),
                name: name.to_string(),
                discriminant,
                default: matches!(value, mir::EnumValue::Default),
                catch_all: matches!(value, mir::EnumValue::CatchAll),
            })
        })
        .collect::<Result<_, miette::Report>>()?;

    Ok(lir::Enum {
        description: description.clone(),
        name: name.to_string(),
        base_type,
        variants,
    })
}

fn repeat_to_method_kind(
    repeat: &Option<mir::Repeat>,
    device_objects: &[mir::Object],
) -> lir::Repeat {
    match repeat {
        Some(mir::Repeat {
            source: mir::RepeatSource::Count(count),
            stride,
        }) => lir::Repeat::Count {
            count: *count,
            stride: *stride,
        },
        Some(mir::Repeat {
            source: mir::RepeatSource::Enum(enum_name),
            stride,
        }) => lir::Repeat::Enum {
            enum_name: enum_name.clone(),
            enum_variants: device_objects
                .iter()
                .find(|object| object.name() == enum_name)
                .expect("Checked in a MIR pass")
                .as_enum()
                .expect("Checked in a MIR pass")
                .variants
                .iter()
                .map(|variant| variant.name.clone())
                .collect(),
            stride: *stride,
        },
        None => lir::Repeat::None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorrowedBlock<'o> {
    pub description: &'o String,
    pub name: &'o String,
    pub address_offset: &'o i128,
    pub repeat: &'o Option<mir::Repeat>,
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

fn find_best_internal_address_type(device: &mir::Device) -> Integer {
    let (min_address_found, max_address_found) = find_min_max_addresses(&device.objects, |_| true);

    let needs_signed = min_address_found < 0;
    let needs_bits = (min_address_found
        .unsigned_abs()
        .max(max_address_found.unsigned_abs())
        .add(1)
        .next_power_of_two()
        .ilog2()
        + needs_signed as u32)
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
            _ => unreachable!(),
        }
    }
}
