use device_driver_lir::model::Driver;

mod rust;

pub enum Target {
    Rust { defmt_feature: Option<String> },
}

pub fn codegen(target: Target, lir_driver: Driver) -> String {
    match target {
        Target::Rust { defmt_feature } => {
            rust::DeviceTemplateRust::new(&lir_driver, defmt_feature).to_string()
        }
    }
}
