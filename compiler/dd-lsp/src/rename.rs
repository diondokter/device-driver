use device_driver_common::{
    identifier::{IdentifierRef, RuntimeNamespace},
    specifiers::RepeatSource,
};
use device_driver_mir::model::{FieldsetRef, TypeRef};
use tower_lsp_server::ls_types::{Position, Range, TextEdit, WorkspaceEdit};

use crate::{document::Document, util::find_identifier_at_position};

pub fn find_rename_target(
    document: &Document,
    position: Position,
) -> Result<(Range, IdentifierRef<RuntimeNamespace>), String> {
    find_identifier_at_position(document, position)
        .ok_or_else(|| "cursor position is not a valid rename target".into())
}

pub fn rename(
    document: &Document,
    position: Position,
    new_name: String,
) -> Result<WorkspaceEdit, String> {
    if new_name.contains(|c: char| c.is_whitespace()) {
        return Err("new name cannot contain whitespace".to_string());
    }

    let (_, target) = find_rename_target(document, position)?;

    let mut edits = Vec::new();

    if target.is_ref_to(&document.mir_manifest().name) {
        edits.push(TextEdit {
            range: document.translate_span(document.mir_manifest().name.span),
            new_text: new_name.clone(),
        });
    }

    for object in document.mir_manifest().objects() {
        if target.is_ref_to(object.name()) {
            edits.push(TextEdit {
                range: document.translate_span(object.name_span()),
                new_text: new_name.clone(),
            });
        }

        if let Some(repeat) = object.repeat()
            && let RepeatSource::Enum(identifier_ref) = &repeat.source.value
            && target.is_same_ref_as(identifier_ref)
        {
            edits.push(TextEdit {
                range: document.translate_span(repeat.source.span),
                new_text: new_name.clone(),
            });
        }

        if let Some(conversion) = &object.type_conversion()
            && let TypeRef::Identifier(identifier) = &conversion.type_ref.value
            && target.is_same_ref_as(identifier)
        {
            edits.push(TextEdit {
                range: document.translate_span(conversion.type_ref.span),
                new_text: new_name.clone(),
            });
        }

        for fs_ref in object.fieldset_refs() {
            if let FieldsetRef::Identifier(identifier) = &fs_ref.value
                && target.is_same_ref_as(identifier)
            {
                edits.push(TextEdit {
                    range: document.translate_span(fs_ref.span),
                    new_text: new_name.clone(),
                });
            }
        }
    }

    Ok(WorkspaceEdit {
        changes: Some([(document.uri().clone(), edits)].into_iter().collect()),
        document_changes: None,
        change_annotations: None,
    })
}
