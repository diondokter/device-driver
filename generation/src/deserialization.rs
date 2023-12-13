use std::collections::HashMap;

use indexmap::IndexMap;

use crate::{Field, Register, TypePath, TypePathOrEnum};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterCollection(Vec<Register>);

impl From<Vec<Register>> for RegisterCollection {
    fn from(value: Vec<Register>) -> Self {
        Self(value)
    }
}

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

impl From<Vec<Field>> for FieldCollection {
    fn from(value: Vec<Field>) -> Self {
        Self(value)
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetValue(pub Vec<u8>);

impl<'de> serde::Deserialize<'de> for ResetValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;
        use std::fmt;

        struct IntegerOrBytes;

        impl<'de> serde::de::Visitor<'de> for IntegerOrBytes {
            type Value = ResetValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("none, unsigned integer or BE bytes")
            }

            fn visit_u64<E>(self, value: u64) -> Result<ResetValue, E>
            where
                E: de::Error,
            {
                Ok(ResetValue(value.to_be_bytes().into()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ResetValue(v.into()))
            }
        }

        deserializer.deserialize_any(IntegerOrBytes)
    }
}

impl From<Vec<u8>> for ResetValue {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}
