#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use std::{
    fs::File,
    io::{Read, stderr},
    path::PathBuf,
};

use clap::Parser;
use device_driver_core::{
    CodegenTarget, CompileOptions, GeneralOptions, MirOptions, RustCodegenOptions,
};
use device_driver_diagnostics::{DynError, Metadata, ResultExt};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{LitStr, Token};

/// Macro to implement the device driver.
///
/// ## Usage:
///
/// Manifest:
/// ```rust,ignore
/// # use device_driver_macros::create_device;
/// compile!(
///     options: "",
///     manifest: "path/to/manifest/file.ddsl"
/// );
/// ```
///
/// Inline ddsl (only used for testing):
/// ```rust,ignore
/// # use device_driver_macros::create_device;
/// compile!(
///     options: "",
///     unstable_ddsl: "
///         // DDSL input
///     "
/// );
/// ```
#[proc_macro]
pub fn compile(item: TokenStream) -> TokenStream {
    let input = match syn::parse::<Input>(item) {
        Ok(i) => i,
        Err(e) => return e.into_compile_error().into(),
    };

    match try_create_device(input) {
        Ok(tokens) => tokens,
        Err(e) => syn::Error::new(Span::call_site(), format!("{e:#}"))
            .into_compile_error()
            .into(),
    }
}

#[derive(Parser, Debug, Clone)]
#[command(no_binary_name = true)]
struct MacroCompileOptions {
    #[command(flatten)]
    pub general_options: GeneralOptions,
    #[command(flatten)]
    pub mir_options: MirOptions,
    #[command(flatten)]
    pub rust_codegen_options: RustCodegenOptions,
}

impl From<MacroCompileOptions> for CompileOptions {
    fn from(value: MacroCompileOptions) -> Self {
        Self {
            general_options: value.general_options,
            mir_options: value.mir_options,
            target: CodegenTarget::Rust(value.rust_codegen_options),
        }
    }
}

fn try_create_device(input: Input) -> Result<TokenStream, DynError> {
    let (source, source_path) = match input.source {
        Source::Ddsl(source_lit) => (source_lit.value(), source_lit.span().file()),
        Source::Manifest(path) => {
            let mut source_path = PathBuf::from(path.value());
            if source_path.is_relative() {
                let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
                source_path = manifest_dir.join(source_path);
            }

            let mut source = String::new();
            let mut file = File::open(&source_path).map_err(|e| {
                DynError::new(format!(
                    "could not open the manifest file at '{}': {e}",
                    source_path.display()
                ))
            })?;
            file.read_to_string(&mut source)
                .with_message(|| "could not read manifest file")?;
            (source, source_path.display().to_string())
        }
    };

    let compile_options = input
        .compile_options
        .map(|str| str.value())
        .unwrap_or_default()
        .split(' ')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect::<Vec<_>>();
    let compile_options = MacroCompileOptions::try_parse_from(compile_options).into_dyn_result()?;
    let (output, diagnostics) = device_driver_core::compile(&source, compile_options.into())?;

    diagnostics
        .print_to(
            stderr().lock(),
            Metadata {
                source: &source,
                source_path: &source_path,
                term_width: None,
                ansi: true,
                unicode: true,
                anonymized_line_numbers: false,
            },
        )
        .unwrap();
    output
        .parse()
        .map_err(|e: proc_macro::LexError| DynError::new(e.to_string()))
        .with_message(|| "could not parse the output")
}

enum Source {
    Ddsl(LitStr),
    Manifest(LitStr),
}

struct Input {
    source: Source,
    compile_options: Option<LitStr>,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut compile_options = None;
        let mut source: Option<Source> = None;

        let mut first_loop = true;

        loop {
            if first_loop {
                first_loop = false;
            } else {
                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }
            }

            if input.is_empty()
                && let Some(source) = source
            {
                return Ok(Input {
                    source,
                    compile_options,
                });
            }

            let look = input.lookahead1();

            if compile_options.is_none() && look.peek(kw::options) {
                input.parse::<kw::options>()?;
                input.parse::<syn::Token![:]>()?;

                compile_options = Some(input.parse()?);
            } else if source.is_none() && look.peek(kw::unstable_ddsl) {
                input.parse::<kw::unstable_ddsl>()?;
                input.parse::<syn::Token![:]>()?;

                source = Some(Source::Ddsl(input.parse()?));
            } else if source.is_none() && look.peek(kw::manifest) {
                input.parse::<kw::manifest>()?;
                input.parse::<syn::Token![:]>()?;

                source = Some(Source::Manifest(input.parse()?));
            } else {
                return Err(look.error());
            }
        }
    }
}

mod kw {
    syn::custom_keyword!(options);
    syn::custom_keyword!(unstable_ddsl);
    syn::custom_keyword!(manifest);
}
