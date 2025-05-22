use askama::Template;

use super::*;

#[derive(Template)]
#[template(path = "rust/device.rs.j2")]
pub struct DeviceTemplateRust<'a> {
    device: &'a Device,
}

impl<'a> DeviceTemplateRust<'a> {
    pub fn new(device: &'a Device) -> Self {
        Self { device }
    }
}