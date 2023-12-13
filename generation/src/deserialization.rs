use std::collections::HashMap;

use indexmap::IndexMap;
use proc_macro2::Span;

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

#[derive(Debug, Clone)]
pub struct ResetValue {
    data: Vec<u8>,
    /// When true, the length of the vec is the known length.
    /// If false, the length is not know and it may be extended with prepended zeroes.
    fixed: bool,
}

impl ResetValue {
    pub fn new(mut data: Vec<u8>, fixed: bool) -> Self {
        if !fixed {
            let non_zero_index = data
                .iter()
                .enumerate()
                .find(|(_, b)| **b != 0)
                .map(|(idx, _)| idx)
                .unwrap_or(data.len());

            data = data.split_at(non_zero_index).1.into();
        }

        Self { data, fixed }
    }

    pub fn get_data(&self, num_bytes: usize, register_name: &str) -> Result<Vec<u8>, syn::Error> {
        if self.fixed {
            if self.data.len() == num_bytes {
                Ok(self.data.clone())
            } else {
                Err(syn::Error::new(Span::call_site(), format!("Reset value of register `{register_name}` has the wrong length ({} bytes): Must be {num_bytes} bytes", self.data.len())))
            }
        } else {
            if num_bytes < self.data.len() {
                Err(syn::Error::new(Span::call_site(), format!("Reset value of register `{register_name}` has the wrong length ({} bytes): Must be {num_bytes} bytes", self.data.len())))
            } else {
                let extra_length_required = num_bytes - self.data.len();
                let mut data = self.data.clone();

                for _ in 0..extra_length_required {
                    data.insert(0, 0);
                }

                Ok(data)
            }
        }
    }
}

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

            fn visit_u128<E>(self, value: u128) -> Result<ResetValue, E>
            where
                E: de::Error,
            {
                Ok(ResetValue::new(value.to_be_bytes().into(), false))
            }

            fn visit_u64<E>(self, value: u64) -> Result<ResetValue, E>
            where
                E: de::Error,
            {
                Ok(ResetValue::new(value.to_be_bytes().into(), false))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut bytes = Vec::new();

                while let Some(elem) = seq.next_element()? {
                    bytes.push(elem);
                }

                Ok(ResetValue::new(bytes, true))
            }
        }

        deserializer.deserialize_any(IntegerOrBytes)
    }
}

impl PartialEq for ResetValue {
    fn eq(&self, other: &Self) -> bool {
        if self.fixed == other.fixed {
            self.data == other.data
        } else {
            let shortest = self.data.len().min(other.data.len());

            let (ssd, sed) = self.data.split_at(self.data.len() - shortest);
            let (osd, oed) = other.data.split_at(other.data.len() - shortest);

            sed == oed && ssd.iter().all(|b| *b == 0) && osd.iter().all(|b| *b == 0)
        }
    }
}

impl Eq for ResetValue {}
