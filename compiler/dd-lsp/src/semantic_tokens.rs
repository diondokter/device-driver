use device_driver_common::span::Span;
use device_driver_lexer::semantic_token_object::SemanticTokenObject;
use device_driver_parser::NodeId;
use tower_lsp_server::ls_types::{Position, Range, SemanticToken, SemanticTokens};

use crate::document::Document;

pub fn calculate_semantic_tokens(
    root_node: NodeId,
    document: &Document,
    range: Option<Range>,
) -> SemanticTokens {
    let ast = document.ast();

    let span_limit = range
        .map(|range| document.translate_range(range))
        .unwrap_or(Span {
            start: 0,
            end: u32::MAX,
        });

    let root_node = ast.node(root_node);

    let semantic_tokens = root_node
        .to_respanned_tokens(document.source(), root_node.span, ast)
        .into_iter()
        .skip_while(|token| !span_limit.overlaps(token.span))
        .take_while(|token| span_limit.overlaps(token.span))
        .filter(|token| token.semantic_type.is_some())
        .scan(
            Position {
                line: 0,
                character: 0,
            },
            |previous_token_start, token| {
                let current_token_range = document.translate_span(token.span);

                let delta_line = current_token_range.start.line - previous_token_start.line;

                let semantic_token = SemanticToken {
                    delta_line,
                    delta_start: if delta_line == 0 {
                        // Same line, so delta since last
                        current_token_range.start.character - previous_token_start.character
                    } else {
                        // New line, so delta from start of line
                        current_token_range.start.character
                    },
                    length: current_token_range.end.character - current_token_range.start.character,
                    token_type: token.semantic_type.unwrap() as u32,
                    token_modifiers_bitset: token.modifiers,
                };

                *previous_token_start = current_token_range.start;

                Some(semantic_token)
            },
        )
        .collect();

    SemanticTokens {
        result_id: None,
        data: semantic_tokens,
    }
}
