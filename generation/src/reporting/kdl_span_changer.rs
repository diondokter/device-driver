use kdl::{KdlDocument, KdlEntry, KdlIdentifier, KdlNode};
use miette::SourceSpan;

pub fn change_document_span(document: &mut KdlDocument, source_span: &SourceSpan) {
    document.set_span((
        document.span().offset() + source_span.offset(),
        document.span().len(),
    ));

    for node in document.nodes_mut() {
        change_node_span(node, source_span);
    }
}

fn change_node_span(node: &mut KdlNode, source_span: &SourceSpan) {
    node.set_span((
        node.span().offset() + source_span.offset(),
        node.span().len(),
    ));

    if let Some(ty) = node.ty_mut() {
        change_identifier_span(ty, source_span);
    }

    change_identifier_span(node.name_mut(), source_span);

    for entry in node.entries_mut() {
        change_entry_span(entry, source_span);
    }

    if let Some(children) = node.children_mut() {
        change_document_span(children, source_span);
    }
}

fn change_identifier_span(id: &mut KdlIdentifier, source_span: &SourceSpan) {
    id.set_span((id.span().offset() + source_span.offset(), id.span().len()));
}

fn change_entry_span(entry: &mut KdlEntry, source_span: &SourceSpan) {
    entry.set_span((
        entry.span().offset() + source_span.offset(),
        entry.span().len(),
    ));

    if let Some(ty) = entry.ty_mut() {
        change_identifier_span(ty, source_span);
    }

    if let Some(name) = entry.name_mut() {
        change_identifier_span(name, source_span);
    }
}
