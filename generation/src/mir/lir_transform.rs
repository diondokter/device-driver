use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::{lir, mir};

use super::passes::recurse_objects;

pub fn transform(device: mir::Device) -> anyhow::Result<lir::Device> {
    let mir_enums = collect_enums(&device)?;
    let lir_enums = mir_enums
        .iter()
        .map(|(e, base_type, size_bits)| transform_enum(e, *base_type, *size_bits))
        .collect::<Result<_, anyhow::Error>>()?;

    Ok(lir::Device {
        blocks: todo!(),
        field_sets: todo!(),
        enums: lir_enums,
    })
}

fn transform_field_sets(device: &mir::Device) -> anyhow::Result<Vec<lir::FieldSet>> {
    let mut field_sets = Vec::new();

    recurse_objects(&device.objects, &mut |object| {
        match object {
            mir::Object::Register(r) => {
                field_sets.push(transform_field_set(&r.fields, format_ident!("{}", r.name))?);
            }
            mir::Object::Command(c) => {
                field_sets.push(transform_field_set(
                    &c.in_fields,
                    format_ident!("{}In", c.name),
                )?);
                field_sets.push(transform_field_set(
                    &c.out_fields,
                    format_ident!("{}Out", c.name),
                )?);
            }
            _ => {}
        }

        Ok(())
    })?;

    Ok(field_sets)
}

fn transform_field_set(
    field_set: &[mir::Field],
    field_set_name: proc_macro2::Ident,
    cfg_attr: Option<&str>,
    description: &str,
    byte_order: mir::ByteOrder,
    bit_order: mir::BitOrder,
    size_bits: usize,
    reset_value: Option<Vec<u8>>,
) -> anyhow::Result<lir::FieldSet> {
    let cfg_attr = if let Some(cfg_attr) = cfg_attr {
        let val = syn::parse_str::<TokenStream>(cfg_attr)?;
        quote! { #[cfg(#val)] }
    } else {
        TokenStream::new()
    };

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

            let cfg_attr = if let Some(cfg_attr) = cfg_attr {
                let val = syn::parse_str::<TokenStream>(cfg_attr)?;
                quote! { #[cfg(#val)] }
            } else {
                TokenStream::new()
            };

            let address = Literal::u64_unsuffixed(field_address.start)..Literal::u64_unsuffixed(field_address.end);

            // TODO:
            // let (base_type, conversion_method) = match (base_type, size_bits) {
            //     (mir::BaseType::Bool, _) => format_ident!("u8"),
            //     (mir::BaseType::Uint, val) => format_ident!("u{}", val.max(8).next_power_of_two()),
            //     (mir::BaseType::Int, val) => format_ident!("i{}", val.max(8).next_power_of_two()),
            // };        

            Ok(lir::Field {
                cfg_attr,
                doc_attr: quote! { #[doc = #description] },
                name: format_ident!("{name}"),
                address,
                base_type: todo!(),
                conversion_method: todo!(),
                access: todo!(),
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
        reset_value: reset_value.unwrap_or_else(|| vec![0; size_bits.div_ceil(8)]),
        fields,
    })
}

fn collect_enums(device: &mir::Device) -> anyhow::Result<Vec<(mir::Enum, mir::BaseType, usize)>> {
    let mut enums = Vec::new();

    recurse_objects(&device.objects, &mut |object| {
        for field in object.field_sets().flatten() {
            match &field.field_conversion {
                Some(mir::FieldConversion::Enum(e)) => enums.push((
                    e.clone(),
                    field.base_type,
                    field.field_address.clone().count(),
                )),
                _ => {}
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

    let cfg_attr = match cfg_attr {
        Some(val) => {
            let cfg_content = syn::parse_str::<proc_macro2::TokenStream>(val)?;
            quote! { #[cfg(#cfg_content)] }
        }
        None => quote! {},
    };

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

            let cfg_attr = match cfg_attr {
                Some(val) => {
                    let cfg_content = syn::parse_str::<proc_macro2::TokenStream>(val)?;
                    quote! { #[cfg(#cfg_content)] }
                }
                None => quote! {},
            };

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
                number: Literal::u64_unsuffixed(number),
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
