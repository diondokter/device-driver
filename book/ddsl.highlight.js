// Slightly modified from the home page, to work better with the book.
// In comparison, this uses hljs 10, is written in JS instead of TS and uses `attr` for the node types instead of `type`

function ddslLanguage(hljs) {
    const nodeTypes = "manifest device register command buffer block extern enum field fieldset";
    const KEYWORDS = "default catch-all allow as stride";
    const typeKeywords = "bool int uint u8 u16 u32 u64 i8 i16 i32 i64 LE BE _ RO RW WO mapped indexed";

    return {
        name: 'ddsl',
        keywords: {
            keyword: KEYWORDS,
            attr: nodeTypes,
            built_in: typeKeywords,
        },
        contains: [
            hljs.C_LINE_COMMENT_MODE,
            hljs.inherit(hljs.QUOTE_STRING_MODE),
            {
                className: 'number',
                variants: [
                    { begin: '\\b0b([01_]+)' },
                    { begin: '\\b0o([0-7_]+)' },
                    { begin: '\\b0x([A-Fa-f0-9_]+)' },
                    {
                        begin: '\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)'
                    }
                ],
                relevance: 0
            },
            {
                className: "punctuation",
                begin: /(->)|\*|,/
            },
            {
                className: 'title',
                begin: /[\w][\w-]*:/
            },
            {
                className: 'operator',
                begin: /[{}\[\]]/,
            },

        ]
    };
}

hljs.registerLanguage('ddsl', ddslLanguage);
hljs.initHighlightingOnLoad();