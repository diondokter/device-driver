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
    value: start_code,
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

    update_grid({ movementX: 0, movementY: 0, force: true });
});
ro.observe(diagnostics);
ro.observe(document.body);
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
        return `<a href="javascript:Website.then((w) => w.scroll_to(${Number.parseInt(splits[1])}, ${Number.parseInt(splits[2])}))">${path_block}</a>`;
    });
}

/** 
 * @param {Number} line
 * @param {Number} column
 * */
export function scroll_to(line, column) {
    code_editor.setPosition({ lineNumber: line, column: column });
    code_editor.revealPositionInCenterIfOutsideViewport({ lineNumber: line, column: column });
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

export function reset() {
    code_editor.setValue(DEFAULT_CODE);
}

var horizontal_dragging = false;
var horizontal_offset = 0.5;
var horizontal_separator = document.getElementById("horizontal-separator");
horizontal_separator.onpointerdown = (event) => {
    horizontal_dragging = true;
};
var vertical_dragging = false;
var vertical_offset = 200;
var vertical_separator = document.getElementById("vertical-separator");
vertical_separator.onpointerdown = (event) => {
    vertical_dragging = true;
};

document.addEventListener("mouseup", (event) => {
    horizontal_dragging = false;
    vertical_dragging = false;
});

var editor_container = document.getElementById("editor-container");

function update_grid(event) {
    var force = event.force !== undefined && event.force;

    if (horizontal_dragging || force) {
        var container_width = editor_container.scrollWidth;
        horizontal_offset += event.movementX / container_width;
        horizontal_offset = clamp(horizontal_offset, 0.05, 0.95);
        editor_container.style.gridTemplateColumns = `calc(${horizontal_offset * 100}% - 2.5px) 5px auto`;
    }
    if (vertical_dragging || force) {
        vertical_offset -= event.movementY;
        vertical_offset = clamp(vertical_offset, 50, document.body.scrollHeight - 100);
        document.body.style.gridTemplateRows = `auto minmax(0, 100%) ${vertical_offset}px`
    }
}

document.addEventListener("mousemove", update_grid);

document.onselectstart = (event) => {
    if (horizontal_dragging || vertical_dragging) {
        event.preventDefault();
    }
};

/**
 * Returns a number whose value is limited to the given range.
 *
 * @param {Number} num The input
 * @param {Number} min The lower boundary of the output range
 * @param {Number} max The upper boundary of the output range
 * @returns A number in the range [min, max]
 * @type Number
 */
function clamp(num, min, max) {
    return Math.min(Math.max(num, min), max);
};