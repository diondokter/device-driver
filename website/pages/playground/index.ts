import * as device_driver_wasm from '../../pkg';
import * as monaco from 'monaco-editor';
import * as DDSLMonarch from './ddsl.monarch'
import * as AU from 'ansi_up';

enum Theme {
    Dark,
    Light,
}

const DEFAULT_CODE = `device Foo {
    register-address-type: u8,

    /// Doc comments get reflected in the output code!
    register Bar {
        address: 0,
        
        fields: fieldset BarFields {
            size-bytes: 1,

            field xena 1:0 -> u8 as enum Xena {
                A: _,
                B: _,
                C: default 3,
            },
            field quux 7:2 -> u8,
        }
    }
}
`
const DEFAULT_OPTIONS = `-C defmt-feature=defmt`;

function setup(): PageContext {
    let draggingSetup = setupDragging();

    const diagnostics = document.getElementById('diagnostics') ?? throwExpression("No diagnostics");
    const targetPickerSelect = (document.getElementById('target-picker-select') ?? throwExpression("No target-picker-select")) as HTMLSelectElement;
    const compilerOptionsInput = (document.getElementById('compiler-options-input') ?? throwExpression("No compiler-options-input")) as HTMLTextAreaElement;

    let startCode = localStorage.getItem("code-session");
    if (startCode == null) {
        startCode = DEFAULT_CODE;
    }
    let startTarget = localStorage.getItem("target");
    if (startTarget != null) {
        targetPickerSelect.value = startTarget;
    }

    let startOptions = localStorage.getItem("compile-options");
    if (startOptions == null) {
        startOptions = DEFAULT_OPTIONS;
    }
    compilerOptionsInput.value = startOptions;

    const darkThemeMq = window.matchMedia("(prefers-color-scheme: dark)");
    darkThemeMq.addEventListener('change', onThemeChange);
    let theme = darkThemeMq.matches ? Theme.Dark : Theme.Light;

    let editors = setup_monaco(startCode, theme);

    let recompile = () => {
        let source = (editors.codeEditor.getModel() ?? throwExpression("No code-editor model")).getValue();
        let target = device_driver_wasm.TargetArg[targetPickerSelect.value as keyof typeof device_driver_wasm.TargetArg];
        if (target == undefined) {
            console.error("Got an undefined target_arg: " + targetPickerSelect.value);
            targetPickerSelect.selectedIndex = 0;
            target = device_driver_wasm.TargetArg[targetPickerSelect.value as keyof typeof device_driver_wasm.TargetArg];
        }

        let charsPerLine = elementCharWidth(diagnostics);

        let output = compile(source, target, compilerOptionsInput.value, charsPerLine);
        (editors.outputEditor.getModel() ?? throwExpression("No output-editor model")).setValue(output.generated);
        diagnostics.innerHTML = output.diagnostics;

        localStorage.setItem("code-session", source);
        localStorage.setItem("target", targetPickerSelect.value);
        localStorage.setItem("compile-options", compilerOptionsInput.value);

    };

    // Set the recompile events

    targetPickerSelect.addEventListener('change', recompile);

    'keyup change'.split(' ').forEach(event => {
        compilerOptionsInput.addEventListener(event, recompile);
    });

    (editors.codeEditor.getModel() ?? throwExpression("No code-editor model")).onDidChangeContent(recompile);
    let reset_timeout: any = null;
    const ro = new ResizeObserver(_ => {
        if (reset_timeout != null) {
            clearTimeout(reset_timeout);
        }
        reset_timeout = setTimeout(recompile, 500);

        draggingSetup.updateGrid({ movementX: 0, movementY: 0, force: true });
    });
    ro.observe(diagnostics);
    ro.observe(document.body);

    // Trigger the first recompile

    recompile();

    return {
        editors,
        compilerOptionsInput,
        targetPickerSelect,
        currentTheme: theme,
    };
}

function setup_monaco(start_code: string, theme: Theme): Editors {
    monaco.languages.register({ id: 'ddsl' })
    monaco.languages.onLanguage('ddsl', () => {
        monaco.languages.setMonarchTokensProvider('ddsl', DDSLMonarch.language);
        monaco.languages.setLanguageConfiguration('ddsl', DDSLMonarch.config);
    });

    let codeEditor = monaco.editor.create(document.getElementById('code-editor') ?? throwExpression("No code-editor"), {
        value: start_code,
        language: 'ddsl',
        theme: monacoThemeString(theme),
        automaticLayout: true,
    });

    let outputEditor = monaco.editor.create(document.getElementById('output-editor') ?? throwExpression("No output-editor"), {
        value: "",
        language: 'rust',
        theme: monacoThemeString(theme),
        readOnly: true,
        automaticLayout: true,
    });

    return {
        codeEditor,
        outputEditor
    };
}

type Editors = {
    codeEditor: monaco.editor.IStandaloneCodeEditor,
    outputEditor: monaco.editor.IStandaloneCodeEditor,
};

function compile(source: string, target: device_driver_wasm.TargetArg, options: string, diagnosticsCharsPerLine: number): CompileOutput {
    let output = device_driver_wasm
        .compile(
            source,
            diagnosticsCharsPerLine,
            target,
            options
        );

    let ansi_up = new AU.AnsiUp();
    let diagnostics = replace_paths_with_links(ansi_up.ansi_to_html(output.diagnostics));

    return {
        generated: output.code,
        diagnostics,
    };
}

type CompileOutput = {
    generated: string,
    diagnostics: string,
};


function elementCharWidth(element: Element): number {
    const style = window.getComputedStyle(element);
    const font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
    const testChar = 'M';
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d') ?? throwExpression("No 2d context");
    ctx.font = font;
    const charWidth = ctx.measureText(testChar).width;
    const elementWidth = element.clientWidth;
    const charsPerLine = Math.floor(elementWidth / charWidth);

    return charsPerLine;
}

/** 
 * @param {String} diagnostics
 * @returns {String}
 * */
function replace_paths_with_links(diagnostics: string): string {
    return diagnostics.replace(/\[.+.ddsl:\d+:\d+]/gm, (path_block) => { // For miette reports
        let splits = path_block.replace("]", "").split(":");
        return `<a href="javascript:Website.then((w) => w.scroll_to(${Number.parseInt(splits[1])}, ${Number.parseInt(splits[2])}))">${path_block}</a>`;
    }).replace(/\w+.ddsl:\d+:\d+/gm, (path_block) => { // For annotate-snippets reports
        let splits = path_block.split(":");
        return `<a href="javascript:Website.then((w) => w.scroll_to(${Number.parseInt(splits[1])}, ${Number.parseInt(splits[2])}))">${path_block}</a>`;
    });
}

function setupDragging(): DraggingSetup {
    const horizontalSeparator = document.getElementById("horizontal-separator") ?? throwExpression("No horizontal-separator");
    const verticalSeparator = document.getElementById("vertical-separator") ?? throwExpression("No vertical-separator");
    const editorContainer = document.getElementById("editor-container") ?? throwExpression("No editor-container");

    let isDraggingHorizontal = false;
    let offsetHorizontal = 0.5;
    horizontalSeparator.onpointerdown = (_) => {
        isDraggingHorizontal = true;
        document.body.style.cursor = "ew-resize";
    };
    let isDraggingVertical = false;
    let offsetVertical = document.body.scrollHeight / 3;
    verticalSeparator.onpointerdown = (_) => {
        isDraggingVertical = true;
        document.body.style.cursor = "ns-resize";
    };

    document.addEventListener("mouseup", (_) => {
        isDraggingHorizontal = false;
        isDraggingVertical = false;
        document.body.style.cursor = "initial";
    });

    let updateGrid = (event: ForceableMouseEvent) => {
        let force = event.force !== undefined && event.force;

        if (isDraggingHorizontal || force) {
            let containerWidth = editorContainer.scrollWidth;
            offsetHorizontal += event.movementX / containerWidth;
            offsetHorizontal = clamp(offsetHorizontal, 0.05, 0.95);
            editorContainer.style.gridTemplateColumns = `calc(${offsetHorizontal * 100}% - 2.5px) 5px auto`;
        }
        if (isDraggingVertical || force) {
            offsetVertical -= event.movementY;
            offsetVertical = clamp(offsetVertical, 50, document.body.scrollHeight - 100);
            document.body.style.gridTemplateRows = `auto minmax(0, 100%) ${offsetVertical}px`
        }
    }

    document.addEventListener("mousemove", (e) => updateGrid({ movementX: e.movementX, movementY: e.movementY, force: false }));

    document.onselectstart = (event) => {
        if (isDraggingHorizontal || isDraggingVertical) {
            event.preventDefault();
        }
    };

    return {
        updateGrid
    };
}

interface ForceableMouseEvent { movementX: number, movementY: number, force: boolean };
type DraggingSetup = {
    updateGrid: (e: ForceableMouseEvent) => void,
}

/**
 * Returns a number whose value is limited to the given range.
 *
 * @param {Number} num The input
 * @param {Number} min The lower boundary of the output range
 * @param {Number} max The upper boundary of the output range
 * @returns A number in the range [min, max]
 * @type Number
 */
function clamp(num: number, min: number, max: number) {
    return Math.min(Math.max(num, min), max);
};

let page_ctx = setup();

interface PageContext {
    editors: Editors,
    targetPickerSelect: HTMLSelectElement,
    compilerOptionsInput: HTMLTextAreaElement,
    currentTheme: Theme,
}

function onThemeChange(e: MediaQueryListEvent) {
    page_ctx.currentTheme = e.matches ? Theme.Dark : Theme.Light;
    monaco.editor.setTheme(monacoThemeString(page_ctx.currentTheme));
}

function monacoThemeString(theme: Theme): string {
    switch (theme) {
        case Theme.Dark: return 'vs-dark';
        case Theme.Light: return 'vs-light';
        default: throw "unknown theme";
    }
}

export function scroll_to(line: number, column: number) {
    page_ctx.editors.codeEditor.setPosition({ lineNumber: line, column: column });
    page_ctx.editors.codeEditor.revealPositionInCenterIfOutsideViewport({ lineNumber: line, column: column });
    page_ctx.editors.codeEditor.focus();
}

export function reset() {
    page_ctx.editors.codeEditor.setValue(DEFAULT_CODE);
    page_ctx.targetPickerSelect.selectedIndex = 0;
    page_ctx.compilerOptionsInput.value = DEFAULT_OPTIONS;
}

function throwExpression(errorMessage: string): never {
    throw new Error(errorMessage);
}