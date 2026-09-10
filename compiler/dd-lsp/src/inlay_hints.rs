use device_driver_common::span::Span;
use device_driver_mir::model::Manifest;
use device_driver_parser::Node;
use tower_lsp_server::ls_types::{InlayHint, InlayHintKind, InlayHintLabel, TextEdit};

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
        tooltip: None,
        padding_left: Some(true),
        padding_right: None,
        data: None,
    })
}
