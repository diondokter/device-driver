use askama::Template;
use itertools::Itertools;

use super::*;
use crate::mir::Access;

#[derive(Template)]
#[template(path = "rust/device.rs.j2", escape = "none")]
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

fn get_super_prefix(conversion_method: &FieldConversionMethod) -> &'static str {
    match conversion_method.conversion_type() {
        Some(ct) if !ct.trim_start().starts_with("::") && !ct.trim_start().starts_with("crate") => {
            "super::"
        }
        _ => "",
    }
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
        Some(fs) => format!("field_sets::{fs}"),
        None => "()".into(),
    }
}
