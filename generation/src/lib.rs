#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use proc_macro2::Span;

mod dsl_hir;
mod lir;
mod manifest;
mod mir;

/// Transform the tokens of the DSL lang to the generated device driver (or a compile error).
///
/// The `driver_name` arg is used to name the root block of the driver.
/// It should be given in `PascalCase` form.
pub fn transform_dsl(
    input: proc_macro2::TokenStream,
    driver_name: &str,
) -> proc_macro2::TokenStream {
    let mir = match _private_transform_dsl_mir(input) {
        Ok(mir) => mir,
        Err(e) => return e.into_compile_error(),
    };

    transform_mir(mir, driver_name)
}

#[doc(hidden)]
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
pub fn transform_json(source: &str, driver_name: &str) -> proc_macro2::TokenStream {
    let mir = match _private_transform_json_mir(source) {
        Ok(mir) => mir,
        Err(e) => return syn::Error::new(Span::call_site(), e).into_compile_error(),
    };

    transform_mir(mir, driver_name)
}

#[doc(hidden)]
pub fn _private_transform_json_mir(source: &str) -> anyhow::Result<mir::Device> {
    let value = dd_manifest_tree::parse_manifest::<dd_manifest_tree::JsonValue>(source)?;
    let mir = manifest::transform(value)?;

    Ok(mir)
}

/// Transform the yaml string to the generated device driver (or a compile error).
///
/// The `driver_name` arg is used to name the root block of the driver.
/// It should be given in `PascalCase` form.
pub fn transform_yaml(source: &str, driver_name: &str) -> proc_macro2::TokenStream {
    let mir = match _private_transform_yaml_mir(source) {
        Ok(mir) => mir,
        Err(e) => return syn::Error::new(Span::call_site(), e).into_compile_error(),
    };

    transform_mir(mir, driver_name)
}

#[doc(hidden)]
pub fn _private_transform_yaml_mir(source: &str) -> anyhow::Result<mir::Device> {
    let value = dd_manifest_tree::parse_manifest::<dd_manifest_tree::YamlValue>(source)?;
    let mir = manifest::transform(value)?;

    Ok(mir)
}

/// Transform the toml string to the generated device driver (or a compile error).
///
/// The `driver_name` arg is used to name the root block of the driver.
/// It should be given in `PascalCase` form.
pub fn transform_toml(source: &str, driver_name: &str) -> proc_macro2::TokenStream {
    let mir = match _private_transform_toml_mir(source) {
        Ok(mir) => mir,
        Err(e) => return syn::Error::new(Span::call_site(), e).into_compile_error(),
    };

    transform_mir(mir, driver_name)
}

#[doc(hidden)]
pub fn _private_transform_toml_mir(source: &str) -> anyhow::Result<mir::Device> {
    let value = dd_manifest_tree::parse_manifest::<dd_manifest_tree::TomlValue>(source)?;
    let mir = manifest::transform(value)?;

    Ok(mir)
}

fn transform_mir(mut mir: mir::Device, driver_name: &str) -> proc_macro2::TokenStream {
    // Run the MIR passes
    match mir::passes::run_passes(&mut mir) {
        Ok(()) => {}
        Err(e) => return syn::Error::new(proc_macro2::Span::call_site(), e).into_compile_error(),
    }

    // Transform into LIR
    let mut lir = match mir::lir_transform::transform(mir, driver_name) {
        Ok(lir) => lir,
        Err(e) => return syn::Error::new(proc_macro2::Span::call_site(), e).into_compile_error(),
    };

    // Run the LIR passes
    match lir::passes::run_passes(&mut lir) {
        Ok(()) => {}
        Err(e) => return syn::Error::new(proc_macro2::Span::call_site(), e).into_compile_error(),
    };

    // Transform into Rust source token output
    lir::token_transform::transform(lir)
}
