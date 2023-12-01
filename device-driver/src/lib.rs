use std::collections::HashMap;

use indexmap::IndexMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Device {
    address_type: IntegerType,
    registers: RegisterCollection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegisterCollection(Vec<Register>);

impl<'de> serde::Deserialize<'de> for RegisterCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let registers = HashMap::<String, Register>::deserialize(deserializer)?;

        Ok(Self(
            registers
                .into_iter()
                .map(|(name, mut register)| {
                    register.name = name;
                    register
                })
                .collect(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Register {
    #[serde(skip)]
    name: String,
    rw_capability: RWCapability,
    address: u128,
    size_bytes: u128,
    fields: FieldCollection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FieldCollection(Vec<Field>);

impl<'de> serde::Deserialize<'de> for FieldCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let registers = HashMap::<String, Field>::deserialize(deserializer)?;

        Ok(Self(
            registers
                .into_iter()
                .map(|(name, mut register)| {
                    register.name = name;
                    register
                })
                .collect(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
    #[serde(skip)]
    name: String,
    #[serde(rename = "type")]
    register_type: IntegerType,
    #[serde(rename = "conversion")]
    conversion_type: Option<TypePathOrEnum>,
    start: u32,
    end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(untagged)]
enum TypePathOrEnum {
    TypePath(TypePath),
    Enum(IndexMap<String, Option<i128>>),
}

impl<'de> serde::Deserialize<'de> for TypePathOrEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;
        use std::fmt;

        struct StringOrStruct;

        impl<'de> serde::de::Visitor<'de> for StringOrStruct {
            type Value = TypePathOrEnum;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<TypePathOrEnum, E>
            where
                E: de::Error,
            {
                Ok(TypePathOrEnum::TypePath(TypePath(value.into())))
            }

            fn visit_map<M>(self, map: M) -> Result<TypePathOrEnum, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                <IndexMap<String, Option<i128>> as serde::Deserialize>::deserialize(
                    de::value::MapAccessDeserializer::new(map),
                )
                .map(|map| TypePathOrEnum::Enum(map))
            }
        }

        deserializer.deserialize_any(StringOrStruct)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
struct TypePath(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
enum RWCapability {
    #[serde(alias = "ro", alias = "RO")]
    ReadOnly,
    #[serde(alias = "wo", alias = "WO")]
    WriteOnly,
    #[serde(alias = "rw", alias = "RW")]
    ReadWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum IntegerType {
    #[serde(alias = "unsigned char", alias = "byte")]
    U8,
    #[serde(alias = "unsigned short")]
    U16,
    #[serde(alias = "unsigned int")]
    U32,
    #[serde(alias = "unsigned long")]
    U64,
    #[serde(alias = "unsigned long long")]
    U128,
    #[serde(alias = "unsigned size")]
    Usize,
    #[serde(alias = "char")]
    I8,
    #[serde(alias = "short")]
    I16,
    #[serde(alias = "int")]
    I32,
    #[serde(alias = "long")]
    I64,
    #[serde(alias = "long long")]
    I128,
    #[serde(alias = "size")]
    Isize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_json() {
        let json_string = include_str!("../json_syntax.json");

        let value = serde_json::from_str::<Device>(json_string).unwrap();
        println!("{value:#?}");
    }

    #[test]
    fn serialize() {
        let path = serde_json::to_string_pretty(&TypePathOrEnum::TypePath(TypePath(String::from(
            "my::type::path",
        ))))
        .unwrap();
        println!("{path}");
        serde_json::from_str::<TypePathOrEnum>(&path).unwrap();

        let e = serde_json::to_string_pretty(&TypePathOrEnum::Enum(
            indexmap::indexmap! { "One".into() => Some(1), "Two".into() => None, "Three".into() => Some(3)}
        )).unwrap();
        println!("{e}");
        serde_json::from_str::<TypePathOrEnum>(&e).unwrap();
    }
}
