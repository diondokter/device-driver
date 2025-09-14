import * as wasm from './pkg';
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
  a

  register Bar {
    address 0
    fields size-bits=8 {}
  }
}
`

const diagnostics_window = document.getElementById('diagnostics-window');
const diagnostics = document.getElementById('diagnostics');

/** 
 * @param {String} text
 * @param {monaco.editor.IStandaloneCodeEditor} output_editor
 * */
function run_compile(text, output_editor) {
    console.debug("Running compile");
    var output = wasm.compile(text, diagnostics_chars_per_line());

    output_editor.getModel().setValue(output.code);
    diagnostics.innerText = output.diagnostics;
}

var code_editor = monaco.editor.create(document.getElementById('code-editor'), {
    value: DEFAULT_CODE,
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
