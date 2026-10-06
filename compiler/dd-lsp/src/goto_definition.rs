use tower_lsp_server::ls_types::{LocationLink, Position};

use crate::{document::Document, util::find_identifier_at_position};

pub fn goto_definition(document: &Document, position: Position) -> Option<LocationLink> {
    let (range, identifier_ref) = find_identifier_at_position(document, position)?;
    let target = document.mir_manifest().search_object(&identifier_ref)?;

    Some(LocationLink {
        origin_selection_range: Some(range),
        target_uri: document.uri().clone(),
        target_range: document.translate_span(target.span()),
        target_selection_range: document.translate_span(target.name_span()),
    })
}
