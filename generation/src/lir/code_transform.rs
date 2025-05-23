use askama::Template;

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

fn get_super_prefix(conversion_method: &FieldConversionMethod) -> &'static str {
    match conversion_method.conversion_type() {
        Some(ct) if !ct.trim_start().starts_with("::") && !ct.trim_start().starts_with("crate") => {
            "super::"
        }
        _ => "",
    }
}

fn get_command_fieldset_name(fieldset: &Option<String>) -> String {
    match fieldset {
        Some(fs) => format!("field_sets::{fs}"),
        None => "()".into(),
    }
}
