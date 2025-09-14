import * as wasm from './pkg';

await wasm;

const source = document.getElementById('source');
const code_output = document.getElementById('code_output');
const diagnostics_output = document.getElementById('diagnostics_output');

source.addEventListener('change', run_compile);
source.addEventListener('input', run_compile);

run_compile();

function run_compile() {
    var output = wasm.compile(source.value);

    code_output.innerText = output.code;
    diagnostics_output.innerText = output.diagnostics;
}
