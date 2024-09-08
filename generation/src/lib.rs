#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

mod dsl_hir;
mod lir;
mod mir;

/// Transform the tokens of the DSL lang to the generated device driver (or a compile error).
///
/// The `driver_name` arg is used to name the root block of the driver.
/// It should be given in `PascalCase` form.
pub fn transform_dsl(
    input: proc_macro2::TokenStream,
    driver_name: &str,
) -> proc_macro2::TokenStream {
    let hir = match syn::parse2::<dsl_hir::Device>(input) {
        Ok(hir) => hir,
        Err(e) => return e.into_compile_error(),
    };

    let mut mir = match dsl_hir::mir_transform::transform(hir) {
        Ok(mir) => mir,
        Err(e) => return e.into_compile_error(),
    };

    match mir::passes::run_passes(&mut mir) {
        Ok(_) => {}
        Err(e) => return syn::Error::new(proc_macro2::Span::call_site(), e).into_compile_error(),
    }

    let lir = match mir::lir_transform::transform(mir, driver_name) {
        Ok(lir) => lir,
        Err(e) => return syn::Error::new(proc_macro2::Span::call_site(), e).into_compile_error(),
    };

    lir::token_transform::transform(lir)
}
