import * as wasm from '../../compiler-wasm/pkg';
import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
import * as KDLMonarch from './kdl.monarch'

await wasm;
await monaco;

monaco.languages.register({ id: 'kdl' })
monaco.languages.setMonarchTokensProvider('kdl', KDLMonarch.language)
monaco.languages.setLanguageConfiguration('kdl', KDLMonarch.config)

const DEFAULT_CODE =
`device Foo {
    register-address-type u8

    /// Doc comments get reflected in the output code!
    register Bar {
        address 0
        fields size-bits=8 {
            /// Inline enums :)
            (i8:Xena) xena @7:4 {
                A
                B
                C
                /// D is the default!
                D default
            }
            quux @3:1
            /// One bit? Then this is a bool by default
            bilb @0
        }
    }
}
`

const diagnostics = document.getElementById('diagnostics');

/** 
 * @param {String} text
 * @param {monaco.editor.IStandaloneCodeEditor} output_editor
 * */
function run_compile(text, output_editor) {
    console.debug("Running compile");
    var output = wasm.compile(text, diagnostics_chars_per_line());

    output_editor.getModel().setValue(output.code);
    diagnostics.innerHTML = replace_paths_with_links(escapeHtml(output.diagnostics));

    localStorage.setItem("code-session", text);
}

var start_code = localStorage.getItem("code-session");
if (start_code == null) {
    start_code = DEFAULT_CODE;
}

var code_editor = monaco.editor.create(document.getElementById('code-editor'), {
    value:  start_code,
    language: 'kdl',
    theme: 'vs-dark',
    automaticLayout: true,
});

var output_editor = monaco.editor.create(document.getElementById('output-editor'), {
    value: "",
    language: 'rust',
    theme: 'vs-dark',
    readOnly: true,
    automaticLayout: true,
});

code_editor.getModel().onDidChangeContent((event) => {
    run_compile(code_editor.getModel().getValue(), output_editor)
});
var reset_timeout = null;
const ro = new ResizeObserver(entries => {
    if (reset_timeout != null) {
        clearTimeout(reset_timeout);
    }
    reset_timeout = setTimeout(() => { run_compile(code_editor.getModel().getValue(), output_editor) }, 500);
});
ro.observe(diagnostics);
run_compile(DEFAULT_CODE, output_editor);

function diagnostics_chars_per_line() {
    const style = window.getComputedStyle(diagnostics);
    const font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
    const testChar = 'M';
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    ctx.font = font;
    const charWidth = ctx.measureText(testChar).width;
    const elementWidth = diagnostics.clientWidth;
    const charsPerLine = Math.floor(elementWidth / charWidth);

    return charsPerLine;
}

/** 
 * @param {String} diagnostics
 * @returns {String}
 * */
function replace_paths_with_links(diagnostics) {
    return diagnostics.replace(/\[.+.kdl:\d+:\d+]/gm, (path_block) => {
        var splits = path_block.replace("]", "").split(":");
        return `<a href="javascript:Website.then((w) => w.scroll_to(${Number.parseInt(splits[1])}, ${Number.parseInt(splits[2])}))">${path_block}<\a>`;
    });
}

/** 
 * @param {Number} line
 * @param {Number} column
 * */
export function scroll_to(line, column) {
    code_editor.setPosition({ lineNumber: line, column: column });
    code_editor.focus();
}

function escapeHtml(str) {
    if (typeof str !== 'string') {
        return '';
    }

    const escapeCharacter = (match) => {
        switch (match) {
            case '&': return '&amp;';
            case '<': return '&lt;';
            case '>': return '&gt;';
            case '"': return '&quot;';
            case '\'': return '&#039;';
            case '`': return '&#096;';
            default: return match;
        }
    };

    return str.replace(/[&<>"'`]/g, escapeCharacter);
}
