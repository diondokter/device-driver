use tower_lsp_server::ls_types::{Location, Position};

use crate::{document::Document, util};

pub fn find_references(
    document: &Document,
    position: Position,
    include_declaration: bool,
) -> Option<Vec<Location>> {
    // Find the ref for the position of the cursor
    let (_, ref_val) = util::find_identifier_at_position(document, position)?;
    // Find every range that has the same ref
    let references = util::find_all_references(document, &ref_val, include_declaration);

    Some(
        references
            .into_iter()
            .map(|span| Location {
                uri: document.uri().clone(),
                range: document.translate_span(span),
            })
            .collect(),
    )
}
