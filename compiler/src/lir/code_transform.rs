use askama::Template;
use itertools::Itertools;

use super::*;
use crate::mir::Access;

#[derive(Template)]
#[template(path = "rust/device.rs.j2", escape = "none", whitespace = "minimize")]
pub struct DeviceTemplateRust<'a> {
    device: &'a Device,
}

impl<'a> DeviceTemplateRust<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self { device }
    }
}

fn description_to_docstring(description: &str) -> String {
    description
        .lines()
        .map(|line| format!("///{}{line}", if line.starts_with(' ') { "" } else { " " }))
        .join("\n")
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

    format!("{}: {{{}}}, ", field.name, defmt_type_hint)
}

fn get_command_fieldset_name(fieldset: &Option<String>) -> String {
    match fieldset {
        Some(fs) => fs.clone(),
        None => "()".into(),
    }
}

fn get_enum_base_type<'d>(device: &'d Device, enum_name: &str) -> &'d str {
    &device
        .enums
        .iter()
        .find(|e| e.name == enum_name)
        .expect("This enum reference is checked in a mir pass")
        .base_type
}
