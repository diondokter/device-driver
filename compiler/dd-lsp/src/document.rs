use device_driver_common::span::Span;
use device_driver_diagnostics::{Diagnostics, DynError, ResultExt};
use device_driver_mir::model::Manifest;
use device_driver_parser::Ast;
use tower_lsp_server::ls_types::{Position, Range};

pub struct Document {
    version: i32,
    source_cache: SourceCache,
    ast: Ast,
    mir: Manifest,
}

impl Document {
    pub fn compile(source: String, version: i32) -> Result<(Self, Diagnostics), DynError> {
        let mut diagnostics = Diagnostics::new();

        let tokens = device_driver_lexer::lex(&source);
        let ast = device_driver_parser::parse(&tokens, &mut diagnostics);
        let (mir, _) = device_driver_mir::lower_ast(&ast, &Default::default(), &mut diagnostics)
            .with_message(|| "lower ast into MIR")?;

        Ok((
            Document {
                version,
                source_cache: SourceCache::new(source),
                ast,
                mir,
            },
            diagnostics,
        ))
    }

    pub fn translate_span(&self, span: Span) -> Range {
        self.source_cache.translate_span_to_range(span)
    }

    pub fn translate_range(&self, range: Range) -> Span {
        self.source_cache.translate_range_to_span(range)
    }

    pub fn source(&self) -> &str {
        &self.source_cache.source
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn mir(&self) -> &Manifest {
        &self.mir
    }

    pub fn ast(&self) -> &Ast {
        &self.ast
    }
}

pub struct SourceCache {
    source: String,
    /// A cache that maps the line number (index) to the byte offset
    line_cache: Vec<u32>,
}

impl SourceCache {
    fn new(source: String) -> Self {
        let mut line_cache = Vec::with_capacity(128); // Start with a decent cap
        line_cache.push(0); // First line starts at offset 0
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_cache.push(
                    (offset + 1)
                        .try_into()
                        .expect("source is smaller than 4GiB"),
                );
            }
        }

        Self { source, line_cache }
    }

    pub fn translate_offset_to_position(&self, offset: u32) -> Position {
        match self.line_cache.binary_search(&offset) {
            Ok(line) => Position {
                line: line as u32,
                character: 0,
            },
            Err(next_line) => {
                let line = next_line - 1;
                let line_offset = self.line_cache[line];

                let line_slice =
                    &self.source[line_offset as usize..self.line_cache[next_line] as usize];
                let line_char_offset = offset - line_offset;

                if line_slice.is_ascii() {
                    // It's just ascii, so shortcut by just using the byte offset
                    Position {
                        line: line as u32,
                        character: line_char_offset,
                    }
                } else {
                    Position {
                        line: line as u32,
                        character: line_slice[..line_char_offset as usize]
                            .chars()
                            .map(|c| c.len_utf16() as u32)
                            .sum(),
                    }
                }
            }
        }
    }

    pub fn translate_span_to_range(&self, span: Span) -> Range {
        if span.start == span.end {
            let val = self.translate_offset_to_position(span.start);
            Range {
                start: val,
                end: val,
            }
        } else {
            Range {
                start: self.translate_offset_to_position(span.start),
                end: self.translate_offset_to_position(span.end),
            }
        }
    }

    pub fn translate_position_to_offset(&self, mut position: Position) -> u32 {
        let line_offset = self
            .line_cache
            .get(position.line as usize)
            .copied()
            .unwrap_or_else(|| self.line_cache.last().copied().unwrap_or_default());
        let next_line_offset = self
            .line_cache
            .get(position.line as usize + 1)
            .copied()
            .unwrap_or_else(|| self.line_cache.last().copied().unwrap_or_default());

        let line_slice = &self.source[line_offset as usize..next_line_offset as usize];
        for (offset, c) in line_slice.char_indices() {
            position.character = position.character.saturating_sub(c.len_utf16() as u32);
            if position.character == 0 {
                return line_offset + offset as u32;
            }
        }

        // Didn't find it? Fall back
        next_line_offset
    }

    pub fn translate_range_to_span(&self, range: Range) -> Span {
        if range.start == range.end {
            let val = self.translate_position_to_offset(range.start);
            Span {
                start: val,
                end: val,
            }
        } else {
            Span {
                start: self.translate_position_to_offset(range.start),
                end: self.translate_position_to_offset(range.end),
            }
        }
    }
}
