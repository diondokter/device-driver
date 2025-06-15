use std::path::Path;

use kdl::{KdlNode, KdlValue};

use crate::{
    mir,
    reporting::{Diagnostics, NamedSourceCode, errors},
};

pub fn transform(
    source: &str,
    file_path: &Path,
    diagnostics: &mut Diagnostics,
) -> Vec<mir::Device> {
    let source_code = NamedSourceCode::new(file_path.display().to_string(), source.into());

    let document = match kdl::KdlDocument::parse(source) {
        Ok(document) => document,
        Err(e) => {
            for diagnostic in e.diagnostics {
                diagnostics.add(diagnostic);
            }
            return Vec::new();
        }
    };

    let devices = document
        .nodes()
        .iter()
        .filter_map(|node| transform_device(node, source_code.clone(), diagnostics))
        .collect();

    devices
}

fn transform_device(
    node: &KdlNode,
    source_code: NamedSourceCode,
    diagnostics: &mut Diagnostics,
) -> Option<mir::Device> {
    if node.name().value() != "device" {
        diagnostics.add(errors::UnknownRootKeyword {
            source_code,
            keyword: node.name().span(),
        });
        return None;
    }

    let device_name = match node.entries().get(0) {
        Some(entry) => {
            if let KdlValue::String(device_name) = entry.value()
                && entry.name().is_none()
            {
                device_name
            } else {
                diagnostics.add(errors::MissingObjectName {
                    source_code,
                    object_keyword: node.name().span(),
                    found_instead: Some(entry.span()),
                    object_type: node.name().value().into(),
                });
                return None;
            }
        }
        None => {
            diagnostics.add(errors::MissingObjectName {
                source_code,
                object_keyword: node.name().span(),
                found_instead: None,
                object_type: node.name().value().into(),
            });
            return None;
        }
    };

    if node.entries().len() > 1 {
        diagnostics.add(errors::UnexpectedEntries {
            source_code,
            entries: node.entries().iter().skip(1).map(|e| e.span()).collect(),
        });
        return None;
    }

    None
}
