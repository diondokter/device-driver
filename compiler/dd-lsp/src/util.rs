use device_driver_common::{
    identifier::{IdentifierRef, RuntimeNamespace},
    specifiers::RepeatSource,
};
use device_driver_mir::model::{FieldsetRef, TypeRef};
use tower_lsp_server::ls_types::{Position, Range};

use crate::document::Document;

pub fn find_identifier_at_position(
    document: &Document,
    position: Position,
) -> Option<(Range, IdentifierRef<RuntimeNamespace>)> {
    let offset = document.translate_position(position);

    if document.mir_manifest().name.span.is_selected_at(offset) {
        return Some((
            document.translate_span(document.mir_manifest().name.span),
            document
                .mir_manifest()
                .name
                .value
                .as_runtime_namespace()
                .take_ref(),
        ));
    }

    for object in document.mir_manifest().objects() {
        if object.name_span().is_selected_at(offset) {
            return Some((
                document.translate_span(object.name_span()),
                object.name().take_ref(),
            ));
        }

        if let Some(repeat) = object.repeat()
            && repeat.source.span.is_selected_at(offset)
            && let RepeatSource::Enum(identifier_ref) = &repeat.source.value
        {
            return Some((
                document.translate_span(repeat.source.span),
                identifier_ref.clone().to_runtime_namespace(),
            ));
        }

        if let Some(conversion) = &object.type_conversion()
            && let TypeRef::Identifier(identifier) = &conversion.type_ref.value
            && conversion.type_ref.span.is_selected_at(offset)
        {
            return Some((
                document.translate_span(conversion.type_ref.span),
                identifier.clone().to_runtime_namespace(),
            ));
        }

        for fs_ref in object.fieldset_refs() {
            if let FieldsetRef::Identifier(identifier) = &fs_ref.value
                && fs_ref.span.is_selected_at(offset)
            {
                return Some((
                    document.translate_span(fs_ref.span),
                    identifier.clone().to_runtime_namespace(),
                ));
            }
        }
    }
    None
}
