use std::ops::{Add, Not};

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::{lir, mir};

use super::passes::{recurse_objects, recurse_objects_with_depth};

pub fn transform(device: mir::Device, driver_name: &str) -> anyhow::Result<lir::Device> {
    let mir_enums = collect_enums(&device)?;
    let lir_enums = mir_enums
        .iter()
        .map(|(e, base_type, size_bits)| transform_enum(e, *base_type, *size_bits))
        .collect::<Result<_, anyhow::Error>>()?;

    let field_sets = transform_field_sets(&device, mir_enums.iter().map(|(e, _, _)| e))?;

    // Create a root block and pass the device objects to it
    let blocks = collect_into_blocks(
        BorrowedBlock {
            cfg_attr: &None,
            description: &format!("Root block of the {driver_name} driver"),
            name: &driver_name.into(),
            address_offset: &0,
            repeat: &None,
            objects: &device.objects,
        },
        true,
        &device.global_config,
        &device.objects,
    )?;

    Ok(lir::Device {
        internal_address_type: find_best_internal_address(&device),
        blocks,
        field_sets,
        enums: lir_enums,
    })
}

fn collect_into_blocks(
    block: BorrowedBlock,
    is_root: bool,
    global_config: &mir::GlobalConfig,
    device_objects: &[mir::Object],
) -> anyhow::Result<Vec<lir::Block>> {
    let mut blocks = Vec::new();

    let BorrowedBlock {
        cfg_attr,
        description,
        name,
        address_offset: _,
        repeat: _,
        objects,
    } = block;

    let cfg_attr = cfg_attr_string_to_tokens(cfg_attr.as_deref())?;

    let mut methods = Vec::new();

    for object in objects {
        let method = get_method(object, &mut blocks, global_config, device_objects)?;

        methods.push(method);
    }

    let new_block = lir::Block {
        cfg_attr,
        doc_attr: quote! { #[doc = #description] },
        root: is_root,
        name: format_ident!("{name}"),
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
) -> Result<lir::BlockMethod, anyhow::Error> {
    use convert_case::Casing;

    Ok(match object {
        mir::Object::Block(
            b @ mir::Block {
                cfg_attr,
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

            lir::BlockMethod {
                cfg_attr: cfg_attr_string_to_tokens(cfg_attr.as_deref())?,
                doc_attr: quote! { #[doc = #description] },
                name: format_ident!("{}", name.to_case(convert_case::Case::Snake)),
                address: Literal::i64_unsuffixed(*address_offset),
                allow_address_overlap: false,
                kind: repeat_to_method_kind(repeat),
                method_type: lir::BlockMethodType::Block {
                    name: format_ident!("{name}"),
                },
            }
        }
        mir::Object::Register(mir::Register {
            cfg_attr,
            description,
            name,
            allow_address_overlap,
            address,
            access,
            repeat,
            ..
        }) => lir::BlockMethod {
            cfg_attr: cfg_attr_string_to_tokens(cfg_attr.as_deref())?,
            doc_attr: quote! { #[doc = #description] },
            name: format_ident!("{}", name.to_case(convert_case::Case::Snake)),
            address: Literal::i64_unsuffixed(*address),
            allow_address_overlap: *allow_address_overlap,
            kind: repeat_to_method_kind(repeat),
            method_type: lir::BlockMethodType::Register {
                field_set_name: format_ident!("{name}"),
                access: *access,
                address_type: global_config
                    .register_address_type
                    .expect("The presence of the address type is already checked in a mir pass")
                    .into(),
            },
        },
        mir::Object::Command(mir::Command {
            cfg_attr,
            description,
            name,
            allow_address_overlap,
            address,
            repeat,
            in_fields,
            out_fields,
            ..
        }) => lir::BlockMethod {
            cfg_attr: cfg_attr_string_to_tokens(cfg_attr.as_deref())?,
            doc_attr: quote! { #[doc = #description] },
            name: format_ident!("{}", name.to_case(convert_case::Case::Snake)),
            address: Literal::i64_unsuffixed(*address),
            allow_address_overlap: *allow_address_overlap,
            kind: repeat_to_method_kind(repeat),
            method_type: lir::BlockMethodType::Command {
                field_set_name_in: in_fields
                    .is_empty()
                    .not()
                    .then(|| format_ident!("{name}In")),
                field_set_name_out: out_fields
                    .is_empty()
                    .not()
                    .then(|| format_ident!("{name}Out")),
                address_type: global_config
                    .command_address_type
                    .expect("The presence of the address type is already checked in a mir pass")
                    .into(),
            },
        },
        mir::Object::Buffer(mir::Buffer {
            cfg_attr,
            description,
            name,
            access,
            address,
        }) => lir::BlockMethod {
            cfg_attr: cfg_attr_string_to_tokens(cfg_attr.as_deref())?,
            doc_attr: quote! { #[doc = #description] },
            name: format_ident!("{}", name.to_case(convert_case::Case::Snake)),
            address: Literal::i64_unsuffixed(*address),
            allow_address_overlap: false,
            kind: lir::BlockMethodKind::Normal, // Buffers can't be repeated (for now?)
            method_type: lir::BlockMethodType::Buffer {
                access: *access,
                address_type: global_config
                    .buffer_address_type
                    .expect("The presence of the address type is already checked in a mir pass")
                    .into(),
            },
        },
        mir::Object::Ref(mir::RefObject {
            cfg_attr,
            description,
            name,
            object_override,
        }) => {
            let mut reffed_object = search_object(object_override.name(), device_objects)
                .expect("All refs are validated in a mir pass")
                .clone();

            match object_override {
                mir::ObjectOverride::Block(override_values) => {
                    let reffed_object = reffed_object
                        .as_block_mut()
                        .expect("All refs are validated in a mir pass");
                    reffed_object.cfg_attr = cfg_attr.clone();
                    reffed_object.description = description.clone();

                    if let Some(address_offset) = override_values.address_offset {
                        reffed_object.address_offset = address_offset;
                    }
                    if let Some(repeat) = override_values.repeat {
                        reffed_object.repeat = Some(repeat);
                    }
                }
                mir::ObjectOverride::Register(override_values) => {
                    let reffed_object = reffed_object
                        .as_register_mut()
                        .expect("All refs are validated in a mir pass");
                    reffed_object.cfg_attr = cfg_attr.clone();
                    reffed_object.description = description.clone();

                    if let Some(access) = override_values.access {
                        reffed_object.access = access;
                    }
                    if let Some(address) = override_values.address {
                        reffed_object.address = address;
                    }
                    if let Some(reset_value) = override_values.reset_value.clone() {
                        reffed_object.reset_value = Some(reset_value);
                    }
                    if let Some(repeat) = override_values.repeat {
                        reffed_object.repeat = Some(repeat);
                    }
                }
                mir::ObjectOverride::Command(override_values) => {
                    let reffed_object = reffed_object
                        .as_command_mut()
                        .expect("All refs are validated in a mir pass");
                    reffed_object.cfg_attr = cfg_attr.clone();
                    reffed_object.description = description.clone();

                    if let Some(address) = override_values.address {
                        reffed_object.address = address;
                    }
                    if let Some(repeat) = override_values.repeat {
                        reffed_object.repeat = Some(repeat);
                    }
                }
            }

            let mut method = get_method(&reffed_object, blocks, global_config, device_objects)?;

            // We kept the old name in the reffed object so it generates with the correct field sets.
            // But we do want to have the name of ref to be the method name.
            method.name = format_ident!("{}", name.to_case(convert_case::Case::Snake));

            method
        }
    })
}

fn search_object<'d>(name: &str, objects: &'d [mir::Object]) -> Option<&'d mir::Object> {
    for object in objects {
        if object.name() == name {
            return Some(object);
        }

        if let Some(block_objects) = object.get_block_object_list() {
            match search_object(name, block_objects) {
                None => {}
                found => return found,
            }
        }
    }

    None
}

fn transform_field_sets<'a>(
    device: &mir::Device,
    mir_enums: impl Iterator<Item = &'a mir::Enum> + Clone,
) -> anyhow::Result<Vec<lir::FieldSet>> {
    let mut field_sets = Vec::new();

    recurse_objects(&device.objects, &mut |object| {
        match object {
            mir::Object::Register(r) => {
                field_sets.push(transform_field_set(
                    &r.fields,
                    format_ident!("{}", r.name),
                    r.cfg_attr.as_deref(),
                    &r.description,
                    r.byte_order.unwrap(),
                    r.bit_order,
                    r.size_bits,
                    r.reset_value
                        .as_ref()
                        .map(|rv| rv.as_array().unwrap().clone()),
                    mir_enums.clone(),
                )?);
            }
            mir::Object::Command(c) => {
                field_sets.push(transform_field_set(
                    &c.in_fields,
                    format_ident!("{}In", c.name),
                    c.cfg_attr.as_deref(),
                    &c.description,
                    c.byte_order.unwrap(),
                    c.bit_order,
                    c.size_bits_in,
                    None,
                    mir_enums.clone(),
                )?);
                field_sets.push(transform_field_set(
                    &c.out_fields,
                    format_ident!("{}Out", c.name),
                    c.cfg_attr.as_deref(),
                    &c.description,
                    c.byte_order.unwrap(),
                    c.bit_order,
                    c.size_bits_out,
                    None,
                    mir_enums.clone(),
                )?);
            }
            _ => {}
        }

        Ok(())
    })?;

    Ok(field_sets)
}

#[allow(clippy::too_many_arguments)] // Though it is correct... it's too many args
fn transform_field_set<'a>(
    field_set: &[mir::Field],
    field_set_name: proc_macro2::Ident,
    cfg_attr: Option<&str>,
    description: &str,
    byte_order: mir::ByteOrder,
    bit_order: mir::BitOrder,
    size_bits: u32,
    reset_value: Option<Vec<u8>>,
    enum_list: impl Iterator<Item = &'a mir::Enum> + Clone,
) -> anyhow::Result<lir::FieldSet> {
    let cfg_attr = cfg_attr_string_to_tokens(cfg_attr)?;

    let fields = field_set
        .iter()
        .map(|field| {
            let mir::Field {
                cfg_attr,
                description,
                name,
                access,
                base_type,
                field_conversion,
                field_address,
            } = field;

            let cfg_attr = cfg_attr_string_to_tokens(cfg_attr.as_deref())?;

            let address = Literal::u32_unsuffixed(field_address.start)
                ..Literal::u32_unsuffixed(field_address.end);

            let (base_type, conversion_method) = match (
                base_type,
                field.field_address.clone().count(),
                field_conversion,
            ) {
                (mir::BaseType::Bool, 1, None) => {
                    (format_ident!("u8"), lir::FieldConversionMethod::Bool)
                }
                (mir::BaseType::Bool, _, _) => unreachable!(
                    "Checked in a MIR pass. Bools can only be 1 bit and have no conversion"
                ),
                (mir::BaseType::Uint | mir::BaseType::Int, val, None) => (
                    format_ident!(
                        "{}{}",
                        match base_type {
                            mir::BaseType::Bool => unreachable!(),
                            mir::BaseType::Uint => 'u',
                            mir::BaseType::Int => 'i',
                        },
                        val.max(8).next_power_of_two()
                    ),
                    lir::FieldConversionMethod::None,
                ),
                (mir::BaseType::Uint | mir::BaseType::Int, val, Some(fc)) => (
                    format_ident!(
                        "{}{}",
                        match base_type {
                            mir::BaseType::Bool => unreachable!(),
                            mir::BaseType::Uint => 'u',
                            mir::BaseType::Int => 'i',
                        },
                        val.max(8).next_power_of_two()
                    ),
                    match enum_list.clone().find(|e| e.name == fc.type_name()) {
                        // Always use try if that's specified
                        _ if fc.use_try() => {
                            lir::FieldConversionMethod::TryInto(format_ident!("{}", fc.type_name()))
                        }
                        // There is an enum we generate so we can look at its metadata
                        Some(mir::Enum {
                            generation_style:
                                Some(mir::EnumGenerationStyle::Infallible { bit_size }),
                            ..
                        }) if field.field_address.clone().count() <= *bit_size as usize => {
                            // This field is equal or smaller in bits than the infallible enum. So we can do the unsafe into
                            lir::FieldConversionMethod::UnsafeInto(format_ident!(
                                "{}",
                                fc.type_name()
                            ))
                        }
                        // Fallback is to require the into trait
                        _ => lir::FieldConversionMethod::Into(format_ident!("{}", fc.type_name())),
                    },
                ),
            };

            Ok(lir::Field {
                cfg_attr,
                doc_attr: quote! { #[doc = #description] },
                name: format_ident!("{name}"),
                address,
                base_type,
                conversion_method,
                access: *access,
            })
        })
        .collect::<Result<_, anyhow::Error>>()?;

    Ok(lir::FieldSet {
        cfg_attr,
        doc_attr: quote! { #[doc = #description] },
        name: field_set_name,
        byte_order,
        bit_order,
        size_bits,
        reset_value: reset_value.unwrap_or_else(|| vec![0; size_bits.div_ceil(8) as usize]),
        fields,
    })
}

fn collect_enums(device: &mir::Device) -> anyhow::Result<Vec<(mir::Enum, mir::BaseType, usize)>> {
    let mut enums = Vec::new();

    recurse_objects(&device.objects, &mut |object| {
        for field in object.field_sets().flatten() {
            if let Some(mir::FieldConversion::Enum { enum_value, .. }) = &field.field_conversion {
                enums.push((
                    enum_value.clone(),
                    field.base_type,
                    field.field_address.clone().count(),
                ))
            }
        }

        Ok(())
    })?;

    Ok(enums)
}

fn transform_enum(
    e: &mir::Enum,
    base_type: mir::BaseType,
    size_bits: usize,
) -> anyhow::Result<lir::Enum> {
    let mir::Enum {
        cfg_attr,
        description,
        name,
        variants,
        generation_style: _,
    } = e;

    let cfg_attr = cfg_attr_string_to_tokens(cfg_attr.as_deref())?;

    let base_type = match (base_type, size_bits) {
        (mir::BaseType::Bool, _) => format_ident!("u8"),
        (mir::BaseType::Uint, val) => format_ident!("u{}", val.max(8).next_power_of_two()),
        (mir::BaseType::Int, val) => format_ident!("i{}", val.max(8).next_power_of_two()),
    };

    let mut next_variant_number = None;
    let variants = variants
        .iter()
        .map(|v| {
            let mir::EnumVariant {
                cfg_attr,
                description,
                name,
                value,
            } = v;

            let cfg_attr = cfg_attr_string_to_tokens(cfg_attr.as_deref())?;

            let number = match value {
                mir::EnumValue::Unspecified
                | mir::EnumValue::Default
                | mir::EnumValue::CatchAll => {
                    let val = next_variant_number.unwrap_or_default();
                    next_variant_number = Some(val + 1);
                    val
                }
                mir::EnumValue::Specified(num) => {
                    next_variant_number = Some(*num + 1);
                    *num
                }
            };

            Ok(lir::EnumVariant {
                cfg_attr,
                doc_attr: quote! { #[doc = #description] },
                name: format_ident!("{name}"),
                number: Literal::i128_unsuffixed(number),
                default: matches!(value, mir::EnumValue::Default),
                catch_all: matches!(value, mir::EnumValue::CatchAll),
            })
        })
        .collect::<Result<_, anyhow::Error>>()?;

    Ok(lir::Enum {
        cfg_attr,
        doc_attr: quote! { #[doc = #description] },
        name: format_ident!("{name}"),
        base_type,
        variants,
    })
}

fn cfg_attr_string_to_tokens(cfg_attr: Option<&str>) -> anyhow::Result<TokenStream> {
    match cfg_attr {
        Some(val) => {
            let cfg_content = syn::parse_str::<proc_macro2::TokenStream>(val)?;
            Ok(quote! { #[cfg(#cfg_content)] })
        }
        None => Ok(quote! {}),
    }
}

fn repeat_to_method_kind(repeat: &Option<mir::Repeat>) -> lir::BlockMethodKind {
    match repeat {
        Some(mir::Repeat { count, stride }) => lir::BlockMethodKind::Repeated {
            count: Literal::u64_unsuffixed(*count),
            stride: Literal::i64_unsuffixed(*stride),
        },
        None => lir::BlockMethodKind::Normal,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorrowedBlock<'o> {
    pub cfg_attr: &'o Option<String>,
    pub description: &'o String,
    pub name: &'o String,
    pub address_offset: &'o i64,
    pub repeat: &'o Option<mir::Repeat>,
    pub objects: &'o [mir::Object],
}

impl<'o> From<&'o mir::Block> for BorrowedBlock<'o> {
    fn from(value: &'o mir::Block) -> Self {
        let mir::Block {
            cfg_attr,
            description,
            name,
            address_offset,
            repeat,
            objects,
        } = value;

        Self {
            cfg_attr,
            description,
            name,
            address_offset,
            repeat,
            objects,
        }
    }
}

fn find_best_internal_address(device: &mir::Device) -> proc_macro2::Ident {
    let mut min_address_found = 0;
    let mut max_address_found = 0;

    let mut last_depth = 0;
    let mut address_offsets = vec![0];

    recurse_objects_with_depth(&device.objects, &mut |object, depth| {
        while depth < last_depth {
            address_offsets.pop();
            last_depth -= 1;
        }

        if let Some(address) = object.address() {
            let repeat = object.repeat().unwrap_or(mir::Repeat {
                count: 1,
                stride: 0,
            });

            let total_address_offsets = address_offsets.iter().sum::<i64>();

            let count_0_address = total_address_offsets + address;
            let count_max_address =
                count_0_address + (repeat.count.saturating_sub(1) as i64 * repeat.stride);

            min_address_found = min_address_found
                .min(count_0_address)
                .min(count_max_address);
            max_address_found = max_address_found
                .max(count_0_address)
                .max(count_max_address);
        }

        if let mir::Object::Block(b) = object {
            // Push an offset because the next objects are gonna be deeper
            address_offsets.push(b.address_offset);
            last_depth += 1;
        }

        Ok(())
    })
    .unwrap();

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
        format_ident!("i{needs_bits}")
    } else {
        format_ident!("u{needs_bits}")
    }
}
