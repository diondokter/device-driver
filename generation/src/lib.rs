#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use itertools::Itertools;

#[cfg(feature = "dsl")]
mod dsl_hir;
mod lir;
#[cfg(feature = "manifest")]
mod manifest;
pub mod mir;

/// Transform the tokens of the DSL lang to the generated device driver (or a compile error).
///
/// The `driver_name` arg is used to name the root block of the driver.
/// It should be given in `PascalCase` form.
#[cfg(feature = "dsl")]
pub fn transform_dsl(input: proc_macro2::TokenStream, driver_name: &str) -> String {
    let mir = match _private_transform_dsl_mir(input) {
        Ok(mir) => mir,
        Err(e) => return e.into_compile_error().to_string(),
    };

    transform_mir(mir, driver_name)
}

#[doc(hidden)]
#[cfg(feature = "dsl")]
pub fn _private_transform_dsl_mir(
    input: proc_macro2::TokenStream,
) -> Result<mir::Device, syn::Error> {
    // Construct the HIR
    let hir = syn::parse2::<dsl_hir::Device>(input)?;

    // Transform into MIR
    let mir = dsl_hir::mir_transform::transform(hir)?;

    Ok(mir)
}

/// Transform the json string to the generated device driver (or a compile error).
///
/// The `driver_name` arg is used to name the root block of the driver.
/// It should be given in `PascalCase` form.
#[cfg(feature = "json")]
pub fn transform_json(source: &str, driver_name: &str) -> String {
    let mir = match _private_transform_json_mir(source) {
        Ok(mir) => mir,
        Err(e) => return anyhow_error_to_compile_error(e),
    };

    transform_mir(mir, driver_name)
}

#[doc(hidden)]
#[cfg(feature = "json")]
pub fn _private_transform_json_mir(source: &str) -> anyhow::Result<mir::Device> {
    let value = dd_manifest_tree::parse_manifest::<dd_manifest_tree::JsonValue>(source)?;
    let mir = manifest::transform(value)?;

    Ok(mir)
}

/// Transform the yaml string to the generated device driver (or a compile error).
///
/// The `driver_name` arg is used to name the root block of the driver.
/// It should be given in `PascalCase` form.
#[cfg(feature = "yaml")]
pub fn transform_yaml(source: &str, driver_name: &str) -> String {
    let mir = match _private_transform_yaml_mir(source) {
        Ok(mir) => mir,
        Err(e) => return anyhow_error_to_compile_error(e),
    };

    transform_mir(mir, driver_name)
}

#[doc(hidden)]
#[cfg(feature = "yaml")]
pub fn _private_transform_yaml_mir(source: &str) -> anyhow::Result<mir::Device> {
    let value = dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(source)?;
    let mir = manifest::transform(value)?;

    Ok(mir)
}

/// Transform the toml string to the generated device driver (or a compile error).
///
/// The `driver_name` arg is used to name the root block of the driver.
/// It should be given in `PascalCase` form.
#[cfg(feature = "toml")]
pub fn transform_toml(source: &str, driver_name: &str) -> String {
    let mir = match _private_transform_toml_mir(source) {
        Ok(mir) => mir,
        Err(e) => return anyhow_error_to_compile_error(e),
    };

    transform_mir(mir, driver_name)
}

#[doc(hidden)]
#[cfg(feature = "toml")]
pub fn _private_transform_toml_mir(source: &str) -> anyhow::Result<mir::Device> {
    let value = dd_manifest_tree::parse_manifest::<dd_manifest_tree::TomlValue>(source)?;
    let mir = manifest::transform(value)?;

    Ok(mir)
}

fn transform_mir(mut mir: mir::Device, driver_name: &str) -> String {
    // Run the MIR passes
    match mir::passes::run_passes(&mut mir) {
        Ok(()) => {}
        Err(e) => return anyhow_error_to_compile_error(e),
    }

    // Transform into LIR
    let mut lir = match mir::lir_transform::transform(mir, driver_name) {
        Ok(lir) => lir,
        Err(e) => return anyhow_error_to_compile_error(e),
    };

    // Run the LIR passes
    match lir::passes::run_passes(&mut lir) {
        Ok(()) => {}
        Err(e) => return anyhow_error_to_compile_error(e),
    };

    // Transform into Rust source token output
    let output = lir::code_transform::DeviceTemplateRust::new(&lir).to_string();

    match format_code(&output) {
        Ok(formatted_output) => formatted_output,
        Err(e) => format!(
            "{}\n\n{output}",
            e.to_string().lines().map(|e| format!("// {e}")).join("\n")
        ),
    }
}

fn anyhow_error_to_compile_error(error: anyhow::Error) -> String {
    syn::Error::new(proc_macro2::Span::call_site(), format!("{error:#}"))
        .into_compile_error()
        .to_string()
}

fn format_code(input: &str) -> Result<String, anyhow::Error> {
    Ok(prettyplease::unparse(&syn::parse_file(input)?))
}
