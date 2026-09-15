use std::str::FromStr;

use device_driver_common::{
    span::Span,
    specifiers::{BaseType, NodeType},
};
use device_driver_mir::model::Manifest;
use device_driver_parser::{Expression, Node};
use tower_lsp_server::ls_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintTooltip, TextEdit,
};

use crate::ToRange;

pub(crate) fn get_hints(
    root_node: &Node,
    visible_span: Span,
    source: &str,
    mir: &Manifest,
) -> Vec<InlayHint> {
    if !visible_span.overlaps(root_node.span) {
        return Vec::new();
    }

    let subnodes = root_node
        .sub_nodes
        .iter()
        .chain(
            root_node
                .type_specifier
                .as_ref()
                .and_then(|ts| ts.conversion.as_ref())
                .and_then(|conversion| conversion.as_subnode()),
        )
        .chain(
            root_node
                .properties
                .iter()
                .filter_map(|prop| prop.expression.as_sub_node()),
        );

    let subnode_hints = subnodes
        .filter(|subnode| visible_span.overlaps(subnode.span))
        .flat_map(|subnode| get_hints(subnode, visible_span, source, mir));

    subnode_hints
        .chain(auto_name_hint(root_node, source, mir))
        .chain(auto_base_type_hint(root_node, source, mir))
        .chain(
            enum_variant_hints(root_node, source, mir)
                .into_iter()
                .flatten(),
        )
        .collect()
}

fn auto_name_hint(node: &Node, source: &str, mir: &Manifest) -> Option<InlayHint> {
    if !node.name.is_auto() {
        return None;
    }

    let true_node_name = mir
        .iter_objects()
        .find(|object| object.span() == node.span)
        .map(|object| object.name().original().as_str())?;

    let node_name_range = node.name.span.to_range(source);

    Some(InlayHint {
        position: node_name_range.end,
        label: InlayHintLabel::String(true_node_name.into()),
        kind: Some(InlayHintKind::TYPE),
        text_edits: Some(
            [TextEdit {
                range: node_name_range,
                new_text: true_node_name.into(),
            }]
            .into(),
        ),
        tooltip: Some(InlayHintTooltip::String("Inferred object name".into())),
        padding_left: Some(true),
        padding_right: None,
        data: None,
    })
}

fn auto_base_type_hint(node: &Node, source: &str, mir: &Manifest) -> Option<InlayHint> {
    // Optimization, skip nodes without base types that are not fieldsets
    if !matches!(
        NodeType::from_str(node.node_type.val.as_str()),
        Ok(NodeType::Enum | NodeType::Extern | NodeType::Field)
    ) {
        return None;
    }

    // Do two searches at once:
    // - Search for the object (or field) that represents the current node
    // - Get its base type if that's supported
    let (mir_base_type, short_properties_span) = mir.iter_objects().find_map(|object| {
        if object.span() == node.span {
            Some(
                object
                    .base_type()
                    .map(|bt| (bt, object.short_properties_span())),
            )
        } else if let Some(fs) = object.as_field_set() {
            fs.fields.iter().find_map(|field| {
                if field.span == node.span {
                    Some(Some((&field.base_type, field.short_properties_span)))
                } else {
                    None
                }
            })
        } else {
            None
        }
    })??;

    match node.type_specifier.as_ref() {
        Some(ts) => match ts.base_type.value {
            BaseType::Unspecified => {
                let base_type_range = ts.base_type.span.to_range(source);

                Some(InlayHint {
                    position: base_type_range.end,
                    label: InlayHintLabel::String(mir_base_type.value.to_string()),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: Some(
                        [TextEdit {
                            range: base_type_range,
                            new_text: mir_base_type.value.to_string(),
                        }]
                        .into(),
                    ),
                    tooltip: Some(InlayHintTooltip::String("Inferred base type".into())),
                    padding_left: Some(true),
                    padding_right: None,
                    data: None,
                })
            }
            BaseType::Int | BaseType::Uint => {
                let base_type_range = ts.base_type.span.to_range(source);

                Some(InlayHint {
                    position: base_type_range.end,
                    label: InlayHintLabel::String(
                        mir_base_type
                            .value
                            .as_fixed_size()
                            .unwrap()
                            .size_bits()
                            .to_string(),
                    ),
                    kind: Some(InlayHintKind::TYPE),
                    text_edits: Some(
                        [TextEdit {
                            range: base_type_range,
                            new_text: mir_base_type.value.to_string(),
                        }]
                        .into(),
                    ),
                    tooltip: Some(InlayHintTooltip::String("Number of inferred bits".into())),
                    padding_left: None,
                    padding_right: None,
                    data: None,
                })
            }
            BaseType::FixedSize(_) | BaseType::Bool => None,
        },
        None => {
            let type_conversion_range = short_properties_span.collapse_to_end().to_range(source);

            Some(InlayHint {
                position: type_conversion_range.start,
                label: InlayHintLabel::String(format!("-> {mir_base_type}")),
                kind: Some(InlayHintKind::TYPE),
                text_edits: Some(
                    [TextEdit {
                        range: type_conversion_range,
                        new_text: format!(" -> {mir_base_type}"),
                    }]
                    .into(),
                ),
                tooltip: Some(InlayHintTooltip::String("Inferred type specifier".into())),
                padding_left: Some(true),
                padding_right: None,
                data: None,
            })
        }
    }
}

fn enum_variant_hints(node: &Node, source: &str, mir: &Manifest) -> Option<Vec<InlayHint>> {
    if !matches!(
        NodeType::from_str(node.node_type.val.as_str()),
        Ok(NodeType::Enum)
    ) {
        return None;
    }

    let enum_value = mir
        .iter_enums()
        .find(|enum_value| enum_value.span == node.span)?;

    let mut hints = Vec::new();

    // Go over each variant, which are properties in the AST
    for property in node.properties.iter() {
        let Some((value, _)) = enum_value
            .iter_variants_with_discriminant()
            .find(|(_, variant)| variant.name.original() == property.name.val)
        else {
            // Variant not found. Probably removed in a MIR pass
            continue;
        };

        let replacement_expression = match &property.expression.value {
            Expression::DefaultNumber(None) => Expression::DefaultNumber(Some(value)),
            Expression::CatchAllNumber(None) => Expression::CatchAllNumber(Some(value)),
            Expression::Auto => Expression::Number(value),
            _ => continue,
        };

        let expression_range = property.expression.span.to_range(source);

        hints.push(InlayHint {
            position: expression_range.end,
            label: InlayHintLabel::String(value.to_string()),
            kind: Some(InlayHintKind::TYPE),
            text_edits: Some(
                [TextEdit {
                    range: expression_range,
                    new_text: replacement_expression.get_human_string().to_string(),
                }]
                .into(),
            ),
            tooltip: Some(InlayHintTooltip::String("Inferred value".into())),
            padding_left: Some(true),
            padding_right: None,
            data: None,
        });
    }

    Some(hints)
}
