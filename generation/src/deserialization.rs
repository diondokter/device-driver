use std::collections::HashMap;

use indexmap::IndexMap;

use crate::{Field, Register, TypePath, TypePathOrEnum};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterCollection(Vec<Register>);

impl std::ops::Deref for RegisterCollection {
    type Target = Vec<Register>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for RegisterCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let registers = HashMap::<String, Register>::deserialize(deserializer)?;

        let mut registers: Vec<_> = registers
            .into_iter()
            .map(|(name, mut register)| {
                register.name = name;
                register
            })
            .collect();

        registers.sort();

        Ok(Self(registers))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldCollection(Vec<Field>);

impl std::ops::Deref for FieldCollection {
    type Target = Vec<Field>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for FieldCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let fields = HashMap::<String, Field>::deserialize(deserializer)?;

        let mut fields: Vec<_> = fields
            .into_iter()
            .map(|(name, mut field)| {
                field.name = name;
                field
            })
            .collect();

        fields.sort();

        Ok(Self(fields))
    }
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
                .map(TypePathOrEnum::Enum)
            }
        }

        deserializer.deserialize_any(StringOrStruct)
    }
}