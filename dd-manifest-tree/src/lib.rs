mod format;
pub use format::*;

use std::{
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValueError {
    expected: &'static str,
    actual: &'static str,
}

impl Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Value had an unexpected type. `{}` was expected, but the actual value was `{}`",
            self.expected, self.actual
        )
    }
}

impl Error for ValueError {}

pub trait Value: Debug + Clone + Sized {
    type MapType: Map<Value = Self>;

    fn type_name(&self) -> &'static str;
    fn as_null(&self) -> Result<(), ValueError>;
    fn as_bool(&self) -> Result<bool, ValueError>;
    fn as_uint(&self) -> Result<u64, ValueError>;
    fn as_int(&self) -> Result<i64, ValueError>;
    fn as_float(&self) -> Result<f64, ValueError>;
    fn as_string(&self) -> Result<&str, ValueError>;
    fn as_array(&self) -> Result<&[Self], ValueError>;
    fn as_map(&self) -> Result<&Self::MapType, ValueError>;
}

impl Value for serde_json::Value {
    type MapType = serde_json::Map<String, serde_json::Value>;

    fn type_name(&self) -> &'static str {
        match self {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "bool",
            serde_json::Value::Number(n) if n.is_u64() => "uint",
            serde_json::Value::Number(n) if n.is_i64() => "int",
            serde_json::Value::Number(n) if n.is_f64() => "float",
            serde_json::Value::Number(_) => unreachable!(),
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "map",
        }
    }

    fn as_null(&self) -> Result<(), ValueError> {
        self.as_null().ok_or_else(|| ValueError {
            expected: "null",
            actual: self.type_name(),
        })
    }

    fn as_bool(&self) -> Result<bool, ValueError> {
        self.as_bool().ok_or_else(|| ValueError {
            expected: "bool",
            actual: self.type_name(),
        })
    }

    fn as_uint(&self) -> Result<u64, ValueError> {
        self.as_u64().ok_or_else(|| ValueError {
            expected: "uint",
            actual: self.type_name(),
        })
    }

    fn as_int(&self) -> Result<i64, ValueError> {
        self.as_i64().ok_or_else(|| ValueError {
            expected: "int",
            actual: self.type_name(),
        })
    }

    fn as_float(&self) -> Result<f64, ValueError> {
        self.as_f64().ok_or_else(|| ValueError {
            expected: "float",
            actual: self.type_name(),
        })
    }

    fn as_string(&self) -> Result<&str, ValueError> {
        self.as_str().ok_or_else(|| ValueError {
            expected: "string",
            actual: self.type_name(),
        })
    }

    fn as_array(&self) -> Result<&[Self], ValueError> {
        self.as_array()
            .map(|v| v.as_slice())
            .ok_or_else(|| ValueError {
                expected: "array",
                actual: self.type_name(),
            })
    }

    fn as_map(&self) -> Result<&Self::MapType, ValueError> {
        self.as_object().ok_or_else(|| ValueError {
            expected: "map",
            actual: self.type_name(),
        })
    }
}

impl Value for yaml_rust2::Yaml {
    type MapType = yaml_rust2::yaml::Hash;

    fn type_name(&self) -> &'static str {
        match self {
            yaml_rust2::Yaml::Real(_) => "float",
            yaml_rust2::Yaml::Integer(0..) => "(u)int",
            yaml_rust2::Yaml::Integer(..0) => "int",
            yaml_rust2::Yaml::String(_) => "string",
            yaml_rust2::Yaml::Boolean(_) => "bool",
            yaml_rust2::Yaml::Array(_) => "array",
            yaml_rust2::Yaml::Hash(_) => "map",
            yaml_rust2::Yaml::Alias(_) => "alias",
            yaml_rust2::Yaml::Null => "null",
            yaml_rust2::Yaml::BadValue => "bad value",
        }
    }

    fn as_null(&self) -> Result<(), ValueError> {
        self.is_null().then_some(()).ok_or_else(|| ValueError {
            expected: "null",
            actual: self.type_name(),
        })
    }

    fn as_bool(&self) -> Result<bool, ValueError> {
        self.as_bool().ok_or_else(|| ValueError {
            expected: "bool",
            actual: self.type_name(),
        })
    }

    fn as_uint(&self) -> Result<u64, ValueError> {
        self.as_i64()
            .and_then(|val| (val >= 0).then_some(val as u64))
            .ok_or_else(|| ValueError {
                expected: "uint",
                actual: self.type_name(),
            })
    }

    fn as_int(&self) -> Result<i64, ValueError> {
        self.as_i64().ok_or_else(|| ValueError {
            expected: "int",
            actual: self.type_name(),
        })
    }

    fn as_float(&self) -> Result<f64, ValueError> {
        self.as_f64().ok_or_else(|| ValueError {
            expected: "float",
            actual: self.type_name(),
        })
    }

    fn as_string(&self) -> Result<&str, ValueError> {
        self.as_str().ok_or_else(|| ValueError {
            expected: "string",
            actual: self.type_name(),
        })
    }

    fn as_array(&self) -> Result<&[Self], ValueError> {
        self.as_vec()
            .map(|v| v.as_slice())
            .ok_or_else(|| ValueError {
                expected: "array",
                actual: self.type_name(),
            })
    }

    fn as_map(&self) -> Result<&Self::MapType, ValueError> {
        self.as_hash().ok_or_else(|| ValueError {
            expected: "map",
            actual: self.type_name(),
        })
    }
}

impl Value for toml::Value {
    type MapType = toml::value::Table;

    fn type_name(&self) -> &'static str {
        match self {
            toml::Value::String(_) => "string",
            toml::Value::Integer(0..) => "(u)int",
            toml::Value::Integer(..0) => "int",
            toml::Value::Float(_) => "float",
            toml::Value::Boolean(_) => "bool",
            toml::Value::Datetime(_) => "datetime",
            toml::Value::Array(_) => "array",
            toml::Value::Table(_) => "map",
        }
    }

    fn as_null(&self) -> Result<(), ValueError> {
        Err(ValueError {
            expected: "null",
            actual: self.type_name(),
        })
    }

    fn as_bool(&self) -> Result<bool, ValueError> {
        self.as_bool().ok_or_else(|| ValueError {
            expected: "bool",
            actual: self.type_name(),
        })
    }

    fn as_uint(&self) -> Result<u64, ValueError> {
        self.as_integer()
            .and_then(|val| (val >= 0).then_some(val as u64))
            .ok_or_else(|| ValueError {
                expected: "uint",
                actual: self.type_name(),
            })
    }

    fn as_int(&self) -> Result<i64, ValueError> {
        self.as_integer().ok_or_else(|| ValueError {
            expected: "int",
            actual: self.type_name(),
        })
    }

    fn as_float(&self) -> Result<f64, ValueError> {
        self.as_float().ok_or_else(|| ValueError {
            expected: "float",
            actual: self.type_name(),
        })
    }

    fn as_string(&self) -> Result<&str, ValueError> {
        self.as_str().ok_or_else(|| ValueError {
            expected: "string",
            actual: self.type_name(),
        })
    }

    fn as_array(&self) -> Result<&[Self], ValueError> {
        self.as_array()
            .map(|v| v.as_slice())
            .ok_or_else(|| ValueError {
                expected: "array",
                actual: self.type_name(),
            })
    }

    fn as_map(&self) -> Result<&Self::MapType, ValueError> {
        self.as_table().ok_or_else(|| ValueError {
            expected: "map",
            actual: self.type_name(),
        })
    }
}

pub trait Map {
    type Value: Value;

    fn iter(&self) -> impl Iterator<Item = (&str, &Self::Value)>;
    fn get(&self, key: &str) -> Option<&Self::Value>;
    fn contains_key(&self, key: &str) -> bool;
}

impl Map for serde_json::Map<String, serde_json::Value> {
    type Value = serde_json::Value;

    fn iter(&self) -> impl Iterator<Item = (&str, &Self::Value)> {
        self.iter().map(|(k, v)| (k.as_str(), v))
    }

    fn get(&self, key: &str) -> Option<&Self::Value> {
        self.get(key)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.contains_key(key)
    }
}

impl Map for yaml_rust2::yaml::Hash {
    type Value = yaml_rust2::Yaml;

    fn iter(&self) -> impl Iterator<Item = (&str, &Self::Value)> {
        self.iter().map(|(k, v)| (k.as_str().unwrap(), v))
    }

    fn get(&self, key: &str) -> Option<&Self::Value> {
        self.get(&yaml_rust2::Yaml::String(key.into()))
    }

    fn contains_key(&self, key: &str) -> bool {
        self.contains_key(&yaml_rust2::Yaml::String(key.into()))
    }
}

impl Map for toml::Table {
    type Value = toml::Value;

    fn iter(&self) -> impl Iterator<Item = (&str, &Self::Value)> {
        self.iter().map(|(k, v)| (k.as_str(), v))
    }

    fn get(&self, key: &str) -> Option<&Self::Value> {
        self.get(key)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.contains_key(key)
    }
}
