import * as device_driver_wasm from '../../pkg';
import * as monaco from 'monaco-editor/esm/vs/editor/editor.api';
import * as KDLMonarch from './kdl.monarch'
import * as AU from 'ansi_up';

await device_driver_wasm;
await monaco;

monaco.languages.register({ id: 'kdl' })
monaco.languages.setMonarchTokensProvider('kdl', KDLMonarch.language)
monaco.languages.setLanguageConfiguration('kdl', KDLMonarch.config)

const DEFAULT_CODE = `device Foo {
    register-address-type: u8,

    /// Doc comments get reflected in the output code!
    register Bar {
        address: 0,
        
        fields: fieldset BarFields {
            size-bits: 8,

            field xena 2:0 <2 by 2> -> u8 as enum Xena {
                A: _,
                B: _,
                D: catch-all 5,
            }
        }
    }
}
`
const DEFAULT_OPTIONS = `-C defmt-feature=defmt`;

const diagnostics = document.getElementById('diagnostics');

var start_code = localStorage.getItem("code-session");
if (start_code == null) {
    start_code = DEFAULT_CODE;
}
var start_target = localStorage.getItem("target");
var start_options = localStorage.getItem("compile-options");
if (start_options == null) {
    start_options = DEFAULT_OPTIONS;
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

/**
 * @type HTMLSelectElement
 */
var target_picker_select = document.getElementById('target-picker-select');
if (start_target != null) {
    target_picker_select.value = start_target;
}
target_picker_select.addEventListener('change', (_) => {
    run_compile();
});

/**
 * @type HTMLInputElement
 */
var compiler_options_input = document.getElementById('compiler-options-input');
if (start_options != null) {
    compiler_options_input.value = start_options;
}
'keyup change'.split(' ').forEach(event => {
    compiler_options_input.addEventListener(event, (_) => {
        run_compile();
    });
});

function run_compile() {
    var text = code_editor.getModel().getValue();

    let target_arg = device_driver_wasm.TargetArg[target_picker_select.value];
    if (target_arg == undefined) {
        console.error("Got an undefined target_arg: " + target_picker_select.value);
        target_picker_select.selectedIndex = 0;
        target_arg = device_driver_wasm.TargetArg[target_picker_select.value];
    }

    var output = device_driver_wasm
        .compile(
            text,
            diagnostics_chars_per_line(),
            target_arg,
            compiler_options_input.value
        );

    output_editor.getModel().setValue(output.code);

    var ansi_up = new AU.AnsiUp();
    diagnostics.innerHTML = replace_paths_with_links(ansi_up.ansi_to_html(output.diagnostics));

    localStorage.setItem("code-session", text);
    localStorage.setItem("target", target_picker_select.value);
    localStorage.setItem("compile-options", compiler_options_input.value);
}


code_editor.getModel().onDidChangeContent((_) => {
    run_compile()
});
var reset_timeout = null;
const ro = new ResizeObserver(entries => {
    if (reset_timeout != null) {
        clearTimeout(reset_timeout);
    }
    reset_timeout = setTimeout(() => { run_compile() }, 500);

    update_grid({ movementX: 0, movementY: 0, force: true });
});
ro.observe(diagnostics);
ro.observe(document.body);
run_compile();

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
    return diagnostics.replace(/\[.+.kdl:\d+:\d+]/gm, (path_block) => { // For miette reports
        var splits = path_block.replace("]", "").split(":");
        return `<a href="javascript:Website.then((w) => w.scroll_to(${Number.parseInt(splits[1])}, ${Number.parseInt(splits[2])}))">${path_block}</a>`;
    }).replace(/\w+.kdl:\d+:\d+/gm, (path_block) => { // For annotate-snippets reports
        var splits = path_block.split(":");
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

export function reset() {
    code_editor.setValue(DEFAULT_CODE);
    target_picker_select.selectedIndex = 0;
    compiler_options_input.value = DEFAULT_OPTIONS;
}

var horizontal_dragging = false;
var horizontal_offset = 0.5;
var horizontal_separator = document.getElementById("horizontal-separator");
horizontal_separator.onpointerdown = (event) => {
    horizontal_dragging = true;
    document.body.style.cursor = "ew-resize";
};
var vertical_dragging = false;
var vertical_offset = document.body.scrollHeight / 3;
var vertical_separator = document.getElementById("vertical-separator");
vertical_separator.onpointerdown = (event) => {
    vertical_dragging = true;
    document.body.style.cursor = "ns-resize";
};

document.addEventListener("mouseup", (event) => {
    horizontal_dragging = false;
    vertical_dragging = false;
    document.body.style.cursor = "initial";
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