use std::fmt::Write;

use clap::Parser;
use device_driver_diagnostics::{Diagnostics, DynError, ResultExt};

pub use device_driver_codegen::{
    DocsCodegenOptions, File, RustCodegenOptions, Target as CodegenTarget,
};
pub use device_driver_diagnostics::Metadata;
pub use device_driver_mir::MirOptions;

use crate::timings::Timings;

mod timings;

#[derive(Parser, Debug, Clone)]
#[command(no_binary_name = true, bin_name = "")]
pub struct CompileOptions {
    #[command(flatten)]
    pub general_options: GeneralOptions,
    #[command(flatten)]
    pub mir_options: MirOptions,
    #[command(subcommand)]
    pub target: CodegenTarget,
}

#[derive(Parser, Debug, Clone, Default)]
#[command(no_binary_name = true)]
pub struct GeneralOptions {
    /// Improves reproducibility across versions
    #[arg(long = "unstable-ui-test-mode", global = true)]
    pub ui_test_mode: bool,
    /// When enabled, a diagnostic is printed with information about the compiler performance.
    /// Exact format of the diagnostic is unstable.
    #[arg(long, global = true, require_equals = true, default_value = "off")]
    pub timings: TimingsMode,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, Default)]
pub enum TimingsMode {
    #[default]
    Off,
    Show,
    Verbose,
}

pub fn compile(
    source: &str,
    options: CompileOptions,
) -> Result<(Vec<File>, Diagnostics), DynError> {
    let mut timings = Timings::new(options.general_options.timings);
    let mut diagnostics = Diagnostics::new();

    let tokens = {
        let _t = timings.start_lexer();
        device_driver_lexer::lex(source)
    };
    let ast = {
        let _t = timings.start_parser();
        device_driver_parser::parse(&tokens, &mut diagnostics)
    };
    let (mir, mir_timings) = {
        let _t = timings.start_mir();
        device_driver_mir::lower_ast(ast, &options.mir_options, &mut diagnostics)
            .with_message(|| "could not lower AST to MIR")?
    };
    timings.set_mir_timings(mir_timings);
    let lir = {
        let _t = timings.start_lir();
        device_driver_lir::lower_mir(mir).with_message(|| "could not lower MIR to LIR")?
    };
    let mut code_files = {
        let _t = timings.start_codegen();
        device_driver_codegen::codegen(&options.target, &lir, source)
    }
    .with_message(|| "could not generate code")?;

    if !matches!(options.general_options.timings, TimingsMode::Off) {
        diagnostics.add(timings);
    }

    for code_file in code_files.iter_mut() {
        if diagnostics.has_error() {
            let _ = writeln!(
                code_file.contents,
                "{}",
                options.target.create_error_message()
            );
        }

        let preamble = options.target.to_comments(&format!(
            "This code was generated using device-driver `{}` ({}),
a tool distributed under {} by {}
This version was built for {} using {}

For more information about device-driver, visit the website: {}",
            if options.general_options.ui_test_mode {
                "xx.xx.xx"
            } else {
                env!("CARGO_PKG_VERSION")
            },
            if options.general_options.ui_test_mode {
                "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
            } else {
                env!("BUILDRS_GIT_SHA")
            },
            env!("CARGO_PKG_LICENSE"),
            env!("CARGO_PKG_AUTHORS"),
            if options.general_options.ui_test_mode {
                "xxxx-xxxx-xxxx"
            } else {
                env!("BUILDRS_TARGET")
            },
            if options.general_options.ui_test_mode {
                "rustc 1.xx.x (xxxxxxxxx xxxx-xx-xx)"
            } else {
                env!("BUILDRS_RUSTC")
            },
            env!("CARGO_PKG_HOMEPAGE"),
        ));

        code_file.contents = preamble + "\n\n" + &code_file.contents;
    }

    Ok((code_files, diagnostics))
}

#[cfg(feature = "gen-docs")]
pub fn gen_docs(output_path: &std::path::Path) -> Result<(), DynError> {
    std::fs::create_dir_all(output_path).with_message(|| {
        format!(
            "creating folder for gen-docs output at {}",
            output_path.display()
        )
    })?;

    let parser_folder = output_path.join("parser");
    if !parser_folder.exists() {
        std::fs::create_dir(&parser_folder).with_message(|| {
            format!(
                "creating folder for gen-docs parser output at {}",
                parser_folder.display()
            )
        })?;
    }
    device_driver_parser::gen_docs::gen_docs(&parser_folder)
        .with_message(|| "gen-docs for parser")?;

    let mir_shapes_folder = output_path.join("mir-shapes");
    if !mir_shapes_folder.exists() {
        std::fs::create_dir(&mir_shapes_folder).with_message(|| {
            format!(
                "creating folder for gen-docs mir-shapes output at {}",
                mir_shapes_folder.display()
            )
        })?;
    }
    device_driver_mir::gen_docs(&mir_shapes_folder).with_message(|| "gen-docs for mir shapes")?;

    Ok(())
}
