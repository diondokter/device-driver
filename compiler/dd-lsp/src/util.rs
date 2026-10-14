use device_driver_common::{
    identifier::{IdentifierRef, RuntimeNamespace},
    span::Span,
    specifiers::RepeatSource,
};
use device_driver_mir::model::{FieldsetRef, Object, TypeRef};
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
        if !object.span().is_selected_at(offset) {
            continue;
        }

        for (place, ref_val) in object_ref_places(object, true) {
            if place.is_selected_at(offset) {
                return Some((document.translate_span(place), ref_val));
            }
        }
    }
    None
}

pub fn find_all_references(
    document: &Document,
    identifier_ref: &IdentifierRef<RuntimeNamespace>,
    include_declaration: bool,
) -> Vec<Span> {
    let mut references = Vec::new();

    for object in document.mir_manifest().objects() {
        for (place, ref_val) in object_ref_places(object, include_declaration) {
            if ref_val.is_same_ref_as(identifier_ref) {
                references.push(place);
            }
        }
    }

    references
}

fn object_ref_places(
    object: Object<'_>,
    include_object_name: bool,
) -> Vec<(Span, IdentifierRef<RuntimeNamespace>)> {
    let mut ref_places = Vec::with_capacity(4 + include_object_name as usize);

    if include_object_name {
        ref_places.push((object.name_span(), object.name().take_ref()));
    }

    if let Some(repeat) = object.repeat()
        && let RepeatSource::Enum(identifier_ref) = &repeat.source.value
    {
        ref_places.push((
            repeat.source.span,
            identifier_ref.clone().to_runtime_namespace(),
        ));
    }

    if let Some(conversion) = &object.type_conversion()
        && let TypeRef::Identifier(identifier) = &conversion.type_ref.value
    {
        ref_places.push((
            conversion.type_ref.span,
            identifier.clone().to_runtime_namespace(),
        ));
    }

    for fs_ref in object.fieldset_refs() {
        if let FieldsetRef::Identifier(identifier) = &fs_ref.value {
            ref_places.push((fs_ref.span, identifier.clone().to_runtime_namespace()));
        }
    }

    ref_places
}
