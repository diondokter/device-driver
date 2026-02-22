#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use device_driver_diagnostics::Diagnostics;
#[cfg(not(feature = "prettyplease"))]
use device_driver_diagnostics::DynError;
use itertools::Itertools;

pub fn compile(source: &str) -> (String, Diagnostics) {
    let mut diagnostics = Diagnostics::new();

    let tokens = device_driver_lexer::lex(source);
    let ast = device_driver_parser::parse(&tokens, &mut diagnostics);
    let mir = device_driver_mir::lower_ast(ast, &mut diagnostics);
    let lir = device_driver_lir::lower_mir(mir);
    let mut code = device_driver_codegen::codegen(device_driver_codegen::Target::Rust, lir);

    if diagnostics.has_error() {
        code +=
            "\ncompile_error!(\"The device driver input has errors that need to be solved!\");\n";
    }

    let formatted_code = match format_code(&code) {
        Ok(formatted_code) => formatted_code,
        Err(e) => format!(
            "{}\n\n{code}",
            e.to_string().lines().map(|e| format!("// {e}")).join("\n")
        ),
    };

    (formatted_code, diagnostics)
}

#[cfg(not(feature = "prettyplease"))]
fn format_code(input: &str) -> Result<String, DynError> {
    use std::io::{Read, Write};
    use std::process::Stdio;

    use device_driver_diagnostics::ResultExt;

    let mut cmd = std::process::Command::new("rustfmt");

    cmd.args(["--edition", "2024"])
        .args(["--config", "newline_style=native"])
        .args(["--color", "never"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().into_dyn_result()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    // Write to stdin in a new thread, so that we can read from stdout on this
    // thread. This keeps the child from blocking on writing to its stdout which
    // might block us from writing to its stdin.
    let output = std::thread::scope(|s| {
        s.spawn(|| {
            child_stdin.write_all(input.as_bytes()).unwrap();
            child_stdin.flush().unwrap();
            drop(child_stdin);
        });
        let handle: std::thread::ScopedJoinHandle<'_, Result<Vec<u8>, DynError>> = s.spawn(|| {
            let mut output = Vec::new();
            child_stdout.read_to_end(&mut output).into_dyn_result()?;
            Ok(output)
        });

        handle.join()
    });

    let status = child.wait().into_dyn_result()?;
    if !status.success() {
        return Err(DynError::new(format!(
            "rustfmt exited unsuccessfully ({status}):\n{}",
            child
                .stderr
                .map(|mut stderr| {
                    let mut err = String::new();
                    stderr.read_to_string(&mut err).unwrap();
                    err
                })
                .unwrap_or_default()
        )));
    }

    let output = match output {
        Ok(output) => output,
        Err(e) => std::panic::resume_unwind(e),
    };

    String::from_utf8(output?).into_dyn_result()
}

#[cfg(feature = "prettyplease")]
fn format_code(input: &str) -> Result<String, syn::Error> {
    Ok(prettyplease::unparse(&syn::parse_file(input)?))
}
