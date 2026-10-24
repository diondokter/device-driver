use std::ops::{Not, Range};

use device_driver_common::specifiers::RepeatSource;
use device_driver_mir::model::{Manifest, Object, Register};
use itertools::Itertools;
use tower_lsp_server::ls_types::{Hover, HoverContents, MarkedString, Position};

use crate::{document::Document, util};

pub(crate) fn get_hover(document: &Document, position: Position) -> Option<Hover> {
    try_get_identifier_hover(document, position)
        .or_else(|| try_get_address_hover(document, position))
        .or_else(|| try_get_field_address_hover(document, position))
}

fn try_get_identifier_hover(document: &Document, position: Position) -> Option<Hover> {
    let (_, identifier) = util::find_identifier_at_position(document, position)?;

    // Find the object we hover over
    let target = document.mir_manifest().search_object(&identifier)?;

    let mut contents = Vec::new();

    contents.push(MarkedString::from_markdown(format!(
        "## {} ({})\n",
        target.name().original(),
        target.name().words_display()
    )));

    let mut node_header_span = target.span();
    if let Some(body_span) = target.body_span() {
        // Don't show the body of the object
        node_header_span = node_header_span.until_start_of(body_span);
    }
    if let Some(conversion) = target.type_conversion()
        && let Some(target_type) = document.mir_manifest().search_type(&conversion.type_ref)
        && let Some(body_span) = target_type.body_span()
    {
        // We have a conversion type with a body. Let's not show that body either
        node_header_span = node_header_span.until_start_of(body_span);
    }

    let node_header_text = &document.source()[Range::from(node_header_span)];

    contents.push(MarkedString::from_language_code(
        "ddsl".into(),
        node_header_text.into(),
    ));

    let mut field_table = None;

    let mut properties = Vec::<(&str, String)>::new();
    match target {
        Object::Device(_) => {}
        Object::Block(block) => {
            let address_text = &document.source()[Range::from(block.address_offset.span)];
            properties.push(("address offset", format!("`{address_text}`")));
        }
        Object::Register(register) => {
            let address_text = &document.source()[Range::from(register.address.span)];
            properties.push((
                "address",
                format!(
                    "`{address_text}`{}",
                    if register.allow_address_overlap {
                        " (overlap allowed)"
                    } else {
                        ""
                    }
                ),
            ));
            if let Some(reset) = register.reset_value.as_ref() {
                let reset_text = &document.source()[Range::from(reset.span)];
                properties.push(("reset", format!("`{reset_text}`")));
            }
            if let Some(access) = register.access {
                properties.push(("access", format!("`{access}`")));
            }
            if let Some(fs) = document
                .mir_manifest()
                .search_fieldset(&register.field_set_ref)
            {
                properties.push((
                    "fields",
                    format!("`{}` (size: {})", fs.name.original(), fs.size_bytes),
                ));
            }

            field_table = Some(MarkedString::from_markdown(get_reset_fieldset_table(
                document.mir_manifest(),
                register,
            )));
        }
        Object::Command(command) => {
            let address_text = &document.source()[Range::from(command.address.span)];
            properties.push((
                "address",
                format!(
                    "`{address_text}`{}",
                    if command.allow_address_overlap {
                        " (overlap allowed)"
                    } else {
                        ""
                    }
                ),
            ));
            if let Some(fields_in) = command.field_set_ref_in.as_ref()
                && let Some(fs_in) = document.mir_manifest().search_fieldset(fields_in)
            {
                properties.push((
                    "fields in",
                    format!("`{}` (size: {})", fs_in.name.original(), fs_in.size_bytes),
                ));
            }
            if let Some(fields_out) = command.field_set_ref_out.as_ref()
                && let Some(fs_out) = document.mir_manifest().search_fieldset(fields_out)
            {
                properties.push((
                    "fields out",
                    format!("`{}` (size: {})", fs_out.name.original(), fs_out.size_bytes),
                ));
            }
        }
        Object::Buffer(buffer) => {
            let address_text = &document.source()[Range::from(buffer.address.span)];
            properties.push(("address", format!("`{address_text}`")));
            if let Some(access) = buffer.access {
                properties.push(("access", format!("`{access}`")));
            }
        }
        Object::FieldSet(field_set) => {
            properties.push(("size bytes", format!("`{}`", field_set.size_bytes)));
            properties.push(("byte order", format!("`{}`", field_set.byte_order.unwrap())));
        }
        Object::Enum(e) => {
            properties.push(("base type", format!("`{}`", e.base_type)));
            properties.push(("size bits", format!("`{}`", e.size_bits.unwrap())));
            properties.push((
                "infallible",
                format!(
                    "`{}`",
                    e.generation_style.as_ref().unwrap().is_fallible().not()
                ),
            ));
        }
        Object::Extern(e) => {
            properties.push(("base type", format!("`{}`", e.base_type)));
            if let Some(size_bits) = e.size_bits {
                properties.push(("size bits", format!("`{}`", size_bits)));
            }
            properties.push(("infallible", format!("`{}`", e.supports_infallible)));
        }
        Object::Field(field) => {
            properties.push(("access", format!("`{}`", field.access.unwrap())));
            properties.push(("base type", format!("`{}`", field.base_type)));
        }
        Object::EnumVariant(_) => {}
    }
    let properties_list = properties
        .into_iter()
        .map(|(name, val)| format!("{name}: {val}"))
        .join(", ");
    contents.push(MarkedString::from_markdown(properties_list));

    if let Some(field_table) = field_table {
        contents.push(field_table);
    }

    if !target.description().is_empty() {
        contents.push(MarkedString::from_markdown(format!(
            "\n---\n{}",
            target.description()
        )));
    }

    Some(Hover {
        contents: HoverContents::Array(contents),
        range: Some(document.translate_span(target.span())),
    })
}

fn try_get_address_hover(document: &Document, position: Position) -> Option<Hover> {
    let offset = document.translate_position(position);

    let target = document.mir_manifest().objects().find(|object| {
        object
            .address()
            .is_some_and(|address| address.span.is_selected_at(offset))
    })?;

    let repeat = target.repeat()?;
    let address = target.address().unwrap();

    let map: Vec<(String, String, i128)> = match &repeat.source.value {
        RepeatSource::Count(non_zero) => (0..non_zero.get())
            .map(|i| {
                (
                    i.to_string(),
                    format!("{i:X}"),
                    i as i128 * repeat.stride.value,
                )
            })
            .collect(),
        RepeatSource::Enum(identifier_ref) => document
            .mir_manifest()
            .search_object(identifier_ref)
            .unwrap()
            .as_enum()
            .unwrap()
            .iter_variants_with_discriminant(&document.mir_manifest().enum_variants)
            .map(|(val, variant)| {
                (
                    format!(
                        "`{}`",
                        document
                            .mir_manifest()
                            .enum_variants
                            .get(variant)
                            .unwrap()
                            .name
                            .original()
                    ),
                    "".into(),
                    val * repeat.stride.value,
                )
            })
            .collect(),
    };

    Some(Hover {
        contents: HoverContents::Scalar(MarkedString::from_markdown(format!(
            "## Repeat addresses\n
| Index | (hex) | Address | (hex) |
| ----: | ----: | ------: | ----: |
{}",
            map.into_iter()
                .map(|(name, name2, val)| format!(
                    "|{name}|{name2}|{addr}|{addr:X}|",
                    addr = address.value + val
                ))
                .join("\n")
        ))),
        range: Some(document.translate_span(target.span())),
    })
}

fn try_get_field_address_hover(document: &Document, position: Position) -> Option<Hover> {
    let offset = document.translate_position(position);

    let target = document
        .mir_manifest()
        .fields
        .iter()
        .find(|object| object.field_address.span.is_selected_at(offset))?;

    let repeat = target.repeat.as_ref()?;
    let address = target.field_address.value;

    let map: Vec<(String, String, i128)> = match &repeat.source.value {
        RepeatSource::Count(non_zero) => (0..non_zero.get())
            .map(|i| {
                (
                    i.to_string(),
                    format!("{i:X}"),
                    i as i128 * repeat.stride.value,
                )
            })
            .collect(),
        RepeatSource::Enum(identifier_ref) => document
            .mir_manifest()
            .search_object(identifier_ref)
            .unwrap()
            .as_enum()
            .unwrap()
            .iter_variants_with_discriminant(&document.mir_manifest().enum_variants)
            .map(|(val, variant)| {
                (
                    format!(
                        "`{}`",
                        document
                            .mir_manifest()
                            .enum_variants
                            .get(variant)
                            .unwrap()
                            .name
                            .original()
                    ),
                    "".into(),
                    val * repeat.stride.value,
                )
            })
            .collect(),
    };

    Some(Hover {
        contents: HoverContents::Scalar(MarkedString::from_markdown(format!(
            "## Repeat addresses\n
| Index | (hex) | Address | (hex) |
| ----: | ----: | ------: | ----: |
{}",
            map.into_iter()
                .map(|(name, name2, val)| format!(
                    "|{name}|{name2}|{addr_end}:{addr_start}|{addr_end:X}:{addr_start:X}|",
                    addr_start = address.start as i128 + val,
                    addr_end = address.end as i128 + val,
                ))
                .join("\n")
        ))),
        range: Some(document.translate_span(target.span)),
    })
}

fn get_reset_fieldset_table(mir_manifest: &Manifest, register: &Register) -> String {
    let fieldset = mir_manifest
        .search_fieldset(&register.field_set_ref)
        .unwrap();
    let reset_value = register
        .reset_value
        .as_ref()
        .map(|rv| rv.as_array().unwrap().clone())
        .unwrap_or_else(|| vec![0; fieldset.size_bytes.value as usize]);

    let decoded =
        match device_driver_mir::runtime::decode_fieldset(mir_manifest, fieldset, &reset_value) {
            Ok(decoded) => decoded,
            Err(e) => return e.to_string(),
        };

    // Build a markdown table
    let mut table = String::new();

    table += "| field | value | conversion |\n";
    table += "| :--- | ---: | ---: |\n";

    for field in decoded.fields {
        table += &format!(
            "| {}{} | {} | `{}` |\n",
            field.name,
            field
                .index_name
                .map(|name| format!("[{name}]"))
                .unwrap_or_default(),
            field.base_value,
            field.converted_value.as_deref().unwrap_or("_")
        );
    }

    table
}
