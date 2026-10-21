use tower_lsp_server::ls_types::{DocumentHighlight, DocumentHighlightKind, Position};

use crate::{document::Document, util};

pub(crate) fn find_highlights(
    document: &Document,
    position: Position,
) -> Option<Vec<DocumentHighlight>> {
    // Find the ref for the position of the cursor
    let (_, ref_val) = util::find_identifier_at_position(document, position)?;
    // Find every range that has the same ref
    let references = util::find_all_references(document, &ref_val, true);

    // In the future, this could add more highlights, like:
    // - References in doc comments

    Some(
        references
            .into_iter()
            .map(|span| DocumentHighlight {
                range: document.translate_span(span),
                kind: Some(DocumentHighlightKind::TEXT),
            })
            .collect(),
    )
}
