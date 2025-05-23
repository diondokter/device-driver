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
        .map(|line| format!("#[doc = \"{line}\"]"))
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

fn get_defmt_fmt_string(field_set: &FieldSet) -> String {
    let fields_format_string = field_set
        .fields
        .iter()
        .map(|f| {
            let defmt_type_hint = match f.conversion_method {
                FieldConversionMethod::None => {
                    let base_type = &f.base_type;
                    format!("={base_type}")
                }
                FieldConversionMethod::Bool => "=bool".into(),
                _ => String::new(),
            };

            format!("{}: {{{}}}", f.name, defmt_type_hint)
        })
        .join(", ");

    format!("{} {{{{ {} }}}}", field_set.name, fields_format_string)
}

fn get_command_fieldset_name(fieldset: &Option<String>) -> String {
    match fieldset {
        Some(fs) => format!("field_sets::{fs}"),
        None => "()".into(),
    }
}
