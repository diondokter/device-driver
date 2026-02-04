use device_driver_lir::model::Driver;

mod rust;

pub enum Target {
    Rust,
}

pub fn codegen(target: Target, lir_driver: Driver) -> String {
    match target {
        Target::Rust => rust::DeviceTemplateRust::new(&lir_driver).to_string(),
    }
}
