use askama::Template;
use convert_case::Case;
use device_driver_common::{identifier::Identifier, specifiers::Access};

use device_driver_lir::model::{BlockMethodType, Driver, Field, FieldConversionMethod, Repeat};

use crate::CompileOptions;

#[derive(Template)]
#[template(path = "rust/device.rs.j2", escape = "none", whitespace = "minimize")]
pub struct DeviceTemplateRust<'a> {
    driver: &'a Driver,
    compile_options: &'a CompileOptions,
}

impl<'a> DeviceTemplateRust<'a> {
    pub fn new(device: &'a Driver, compile_options: &'a CompileOptions) -> Self {
        Self {
            driver: device,
            compile_options,
        }
    }

    fn defmt_feature(&self) -> Option<&str> {
        self.compile_options.get("defmt-feature")
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

fn get_command_fieldset_name(fieldset: &Option<Identifier>) -> String {
    match fieldset {
        Some(fs) => fs.to_case(Case::Pascal),
        None => "()".into(),
    }
}

fn get_enum_base_type<'d>(driver: &'d Driver, enum_name: &Identifier) -> &'d str {
    &driver
        .enums
        .iter()
        .find(|e| e.name == *enum_name)
        .expect("This enum reference is checked in a mir pass")
        .base_type
}
