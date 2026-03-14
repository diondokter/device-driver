use std::collections::HashMap;

use device_driver_lir::model::Driver;

mod rust;

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Rust,
}

impl Target {
    pub fn create_error_message(&self) -> &'static str {
        match self {
            Target::Rust => {
                "compile_error!(\"The device driver input has errors that need to be solved!\");"
            }
        }
    }

    pub fn get_compile_options(&self) -> CompileOptions {
        match self {
            Target::Rust => CompileOptions {
                possible_options: ["defmt-feature"].into(),
                selected: Default::default(),
            },
        }
    }
}

pub struct CompileOptions {
    possible_options: Vec<&'static str>,
    selected: HashMap<&'static str, String>,
}

impl CompileOptions {
    pub fn possible_options(&self) -> &[&'static str] {
        &self.possible_options
    }

    pub fn get<'s>(&'s self, key: &'s str) -> Option<&'s str> {
        assert!(self.possible_options.contains(&key));
        self.selected.get(&key).map(|s| s.as_str())
    }

    #[must_use = "Check bool to see if operation succeeded"]
    pub fn add(&mut self, key: &str, value: String) -> bool {
        let Some(found_key) = self
            .possible_options
            .iter()
            .copied()
            .find(|val| *val == key)
        else {
            return false;
        };

        self.selected.insert(found_key, value).is_none()
    }
}

pub fn codegen(target: Target, lir_driver: &Driver, compile_options: &CompileOptions) -> String {
    match target {
        Target::Rust => rust::DeviceTemplateRust::new(lir_driver, compile_options).to_string(),
    }
}
