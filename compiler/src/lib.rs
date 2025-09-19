#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use std::path::Path;

use convert_case::Casing;
use itertools::Itertools;

use crate::reporting::Diagnostics;

pub use miette;

mod kdl;
mod lir;
pub mod mir;

pub mod reporting;

pub fn transform_kdl(
    file_contents: &str,
    source_span: Option<miette::SourceSpan>,
    file_path: &Path,
) -> (String, Diagnostics) {
    let mut reports = Diagnostics::new();

    let mir_devices = crate::kdl::transform(file_contents, source_span, file_path, &mut reports);

    let device_count = mir_devices.len();

    let mut output = String::new();
    for mir_device in mir_devices {
        if device_count > 1 {
            output += &format!(
                "pub mod {} {{\n",
                mir_device
                    .name
                    .clone()
                    .unwrap()
                    .to_case(convert_case::Case::Snake)
            );
        }

        match transform_mir(mir_device) {
            Ok(device_output) if device_count > 1 => {
                output += &device_output.lines().map(|l| format!("    {l}")).join("\n")
            }
            Ok(device_output) => output += &device_output,
            Err(e) => reports.add_msg(e.to_string()),
        }

        if device_count > 1 {
            output += "\n}\n\n";
        }
    }

    if reports.has_error() {
        output +=
            "compile_error!(\"The device driver input has errors that need to be solved!\");\n"
    }

    (output, reports)
}

fn transform_mir(mut mir: mir::Device) -> Result<String, anyhow::Error> {
    // Run the MIR passes
    mir::passes::run_passes(&mut mir)?;

    // Transform into LIR
    let mut lir = mir::lir_transform::transform(mir)?;

    // Run the LIR passes
    lir::passes::run_passes(&mut lir)?;

    // Transform into Rust source token output
    let output = lir::code_transform::DeviceTemplateRust::new(&lir).to_string();

    let formatted_output = match format_code(&output) {
        Ok(formatted_output) => formatted_output,
        Err(e) => format!(
            "{}\n\n{output}",
            e.to_string().lines().map(|e| format!("// {e}")).join("\n")
        ),
    };

    Ok(formatted_output)
}

#[cfg(not(feature = "prettyplease"))]
fn format_code(input: &str) -> Result<String, anyhow::Error> {
    use anyhow::ensure;
    use std::io::{Read, Write};
    use std::process::Stdio;

    let mut cmd = std::process::Command::new("rustfmt");

    cmd.args(["--edition", "2024"])
        .args(["--config", "newline_style=native"])
        .args(["--color", "never"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
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
        let handle: std::thread::ScopedJoinHandle<'_, Result<Vec<u8>, anyhow::Error>> =
            s.spawn(|| {
                let mut output = Vec::new();
                child_stdout.read_to_end(&mut output)?;
                Ok(output)
            });

        handle.join()
    });

    let status = child.wait()?;
    ensure!(
        status.success(),
        "rustfmt exited unsuccessfully ({status}):\n{}",
        child
            .stderr
            .map(|mut stderr| {
                let mut err = String::new();
                stderr.read_to_string(&mut err).unwrap();
                err
            })
            .unwrap_or_default()
    );

    let output = match output {
        Ok(output) => output,
        Err(e) => std::panic::resume_unwind(e),
    };

    Ok(String::from_utf8(output?)?)
}

#[cfg(feature = "prettyplease")]
fn format_code(input: &str) -> Result<String, anyhow::Error> {
    Ok(prettyplease::unparse(&syn::parse_file(input)?))
}
