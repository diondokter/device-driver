use std::ops::Range;

use askama::Template;
use clap::Parser;
use convert_case::Case;
use device_driver_common::{
    identifier::{Identifier, IdentifierType, Type},
    specifiers::{Access, AddressMode},
};
use device_driver_lir::model::{
    BlockMethod, BlockMethodType, Driver, Field, FieldConversionMethod, Repeat,
};

#[derive(Parser, Debug, Clone, Default)]
#[command(no_binary_name = true)]
pub struct RustCodegenOptions {
    /// When specified, defmt implementations will be generated using this cfg feature flag
    #[arg(
        long = "rust-defmt-feature",
        value_name = "FEATURE",
        require_equals = true
    )]
    pub defmt_feature: Option<String>,
}

#[derive(Template)]
#[template(path = "rust/driver.rs.j2", escape = "none", whitespace = "minimize")]
pub struct DriverTemplateRust<'a> {
    driver: &'a Driver,
    source: &'a str,
    codegen_options: &'a RustCodegenOptions,
}

impl<'a> DriverTemplateRust<'a> {
    pub fn new(
        device: &'a Driver,
        source: &'a str,
        codegen_options: &'a RustCodegenOptions,
    ) -> Self {
        Self {
            driver: device,
            source,
            codegen_options,
        }
    }

    fn defmt_feature(&self) -> Option<&str> {
        self.codegen_options.defmt_feature.as_deref()
    }

    fn get_reset_value_text(&self, method: &BlockMethod) -> Option<&'a str> {
        let reset_value = match &method.method_type {
            BlockMethodType::Register { reset_value, .. } => reset_value.as_ref(),
            _ => None,
        }?;

        self.source.get(Range::from(reset_value.span))
    }
}

fn description_to_docstring(description: &str) -> String {
    use std::fmt::Write;

    let mut docstring = String::new();

    for line in description.lines() {
        writeln!(
            &mut docstring,
            "///{}{line}",
            if line.starts_with(' ') { "" } else { " " }
        )
        .unwrap();
    }

    docstring
}

fn get_defmt_fmt_string(field: &Field) -> String {
    let defmt_type_hint = match field.conversion_method {
        FieldConversionMethod::None => {
            let base_type = &field.base_type;
            format!("={base_type}")
        }
        FieldConversionMethod::Bool => "=bool".into(),
        _ => String::new(),
    };

    format!(
        "{}: {{{}}}, ",
        field.name.to_case(Case::Snake),
        defmt_type_hint
    )
}

fn get_command_fieldset_name(fieldset: &Option<Identifier<Type>>) -> String {
    match fieldset {
        Some(fs) => fs.to_case(Case::Pascal),
        None => "()".into(),
    }
}

fn get_enum_base_type<'d>(driver: &'d Driver, enum_name: &Identifier<Type>) -> &'d str {
    &driver
        .enums
        .iter()
        .find(|e| e.name == *enum_name)
        .expect("This enum reference is checked in a mir pass")
        .base_type
}

fn get_address_mode_const_value(value: &Option<AddressMode>) -> &'static str {
    match value {
        Some(AddressMode::Mapped) => "::device_driver::MappedAddressMode",
        Some(AddressMode::Indexed) => "::device_driver::IndexedAddressMode",
        None => "()",
    }
}

fn maybe_doc_alias<T: IdentifierType>(identifier: &Identifier<T>, case: Case) -> String {
    if identifier.to_case(case) == identifier.original() {
        return String::new();
    }

    format!("#[doc(alias = \"{}\")]", identifier.original())
}
