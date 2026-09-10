#![allow(deprecated)]

use std::str::FromStr;

use device_driver_common::{interner::Istr, specifiers::NodeType};
use device_driver_mir::model::Manifest;
use device_driver_parser::Node;
use tower_lsp_server::ls_types::{DocumentSymbol, SymbolKind};

use crate::IntoRange;

pub fn get_node_symbol(node: &Node, source: &str, mir: &Manifest) -> DocumentSymbol {
    let kind = node_type_symbol_kind(node.node_type.val);

    let properties = node.properties.iter().map(|prop| DocumentSymbol {
        name: prop.name.val.to_string(),
        detail: None,
        kind: if kind == SymbolKind::ENUM {
            SymbolKind::ENUM_MEMBER
        } else {
            SymbolKind::PROPERTY
        },
        tags: None,
        deprecated: None,
        range: prop.span.into_range(source),
        selection_range: prop.name.span.into_range(source),
        children: prop
            .expression
            .as_sub_node()
            .map(|sub_node| vec![get_node_symbol(sub_node, source, mir)]),
    });

    let return_node = node
        .type_specifier
        .as_ref()
        .map(|ts| {
            ts.conversion
                .as_ref()
                .map(|conversion| conversion.as_subnode())
        })
        .flatten()
        .flatten()
        .into_iter()
        .map(|node| get_node_symbol(node, source, mir));

    let sub_nodes = node
        .sub_nodes
        .iter()
        .map(|node| get_node_symbol(node, source, mir));

    let node_name = if node.name.is_auto() {
        // If the name is auto, we still want to display the real name instead of just `_`
        // So we search for object in MIR and take that name
        // The easiest way is to compare by span. The MIR should have unmodified spans
        mir.iter_objects()
            .find(|object| object.span() == node.span)
            .map(|object| object.name().original().as_str())
            .unwrap_or("_")
            .into()
    } else {
        node.name.val.to_string()
    };

    DocumentSymbol {
        name: node_name,
        detail: Some(node.node_type.val.to_string()),
        kind: node_type_symbol_kind(node.node_type.val),
        tags: None,
        deprecated: None,
        range: node.span.into_range(source),
        selection_range: node.name.span.into_range(source),
        children: Some(properties.chain(return_node).chain(sub_nodes).collect()),
    }
}

fn node_type_symbol_kind(node_type: Istr) -> SymbolKind {
    match NodeType::from_str(node_type.as_str()) {
        Ok(NodeType::Manifest) => SymbolKind::FILE,
        Ok(NodeType::Device) => SymbolKind::CLASS,
        Ok(NodeType::Block) => SymbolKind::METHOD,
        Ok(NodeType::Register) => SymbolKind::METHOD,
        Ok(NodeType::Command) => SymbolKind::METHOD,
        Ok(NodeType::Buffer) => SymbolKind::METHOD,
        Ok(NodeType::FieldSet) => SymbolKind::STRUCT,
        Ok(NodeType::Enum) => SymbolKind::ENUM,
        Ok(NodeType::Extern) => SymbolKind::STRUCT,
        Ok(NodeType::Field) => SymbolKind::FIELD,
        Err(_) => SymbolKind::OBJECT,
    }
}
