use std::collections::HashMap;

use indexmap::IndexMap;
use proc_macro2::Span;

use crate::{
    Buffer, Command, EnumVariant, EnumVariantValue, Field, Register, RegisterKind, TypePath,
    TypePathOrEnum,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterCollection(Vec<Register>);

impl RegisterCollection {
    pub fn resolve(&self, name: &str) -> Option<&Register> {
        match self.0.iter().find(|x| x.name == name) {
            None => None,
            Some(Register {
                kind: RegisterKind::Ref { copy_of, .. },
                ..
            }) => self.resolve(copy_of),
            Some(r) => Some(r),
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCollection(Vec<Command>);

impl From<Vec<Command>> for CommandCollection {
    fn from(value: Vec<Command>) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for CommandCollection {
    type Target = Vec<Command>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for CommandCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let registers = HashMap::<String, Command>::deserialize(deserializer)?;

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
pub struct BufferCollection(Vec<Buffer>);

impl From<Vec<Buffer>> for BufferCollection {
    fn from(value: Vec<Buffer>) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for BufferCollection {
    type Target = Vec<Buffer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for BufferCollection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let registers = HashMap::<String, Buffer>::deserialize(deserializer)?;

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
                <IndexMap<String, EnumVariant> as serde::Deserialize>::deserialize(
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
        } else if num_bytes < self.data.len() {
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

impl<'de> serde::Deserialize<'de> for EnumVariant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;
        use std::fmt;

        struct IntegerOrValue;

        impl<'de> serde::de::Visitor<'de> for IntegerOrValue {
            type Value = EnumVariant;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("null, integer or struct with value and description")
            }

            fn visit_i128<E>(self, value: i128) -> Result<EnumVariant, E>
            where
                E: de::Error,
            {
                Ok(EnumVariant {
                    description: None,
                    value: EnumVariantValue::Specified(value),
                })
            }

            fn visit_i64<E>(self, value: i64) -> Result<EnumVariant, E>
            where
                E: de::Error,
            {
                Ok(EnumVariant {
                    description: None,
                    value: EnumVariantValue::Specified(value as _),
                })
            }

            fn visit_u64<E>(self, value: u64) -> Result<EnumVariant, E>
            where
                E: de::Error,
            {
                Ok(EnumVariant {
                    description: None,
                    value: EnumVariantValue::Specified(value as _),
                })
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(EnumVariant {
                    description: None,
                    value: EnumVariantValue::None,
                })
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(EnumVariant {
                    description: None,
                    value: EnumVariantValue::None,
                })
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = match v {
                    "default" => Ok(EnumVariantValue::Default),
                    "catch-all" => Ok(EnumVariantValue::CatchAll),
                    _ => Err(serde::de::Error::unknown_variant(
                        v,
                        &["default", "catch-all"],
                    )),
                }?;

                Ok(EnumVariant {
                    description: None,
                    value,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut s = EnumVariant {
                    description: None,
                    value: EnumVariantValue::None,
                };

                while let Some(key) = map.next_key::<String>()? {
                    match key.to_lowercase().as_str() {
                        "description" => s.description = Some(map.next_value()?),
                        "value" => s.value = map.next_value()?,
                        _ => {
                            return Err(serde::de::Error::unknown_field(
                                &key,
                                &["description", "value"],
                            ));
                        }
                    }
                }

                Ok(s)
            }
        }

        deserializer.deserialize_any(IntegerOrValue)
    }
}

impl<'de> serde::Deserialize<'de> for EnumVariantValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de;
        use std::fmt;

        struct EnumVariantValueVisitor;

        impl<'de> serde::de::Visitor<'de> for EnumVariantValueVisitor {
            type Value = EnumVariantValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("null, integer, \"default\" or \"catch-all\"")
            }

            fn visit_i128<E>(self, value: i128) -> Result<EnumVariantValue, E>
            where
                E: de::Error,
            {
                Ok(EnumVariantValue::Specified(value))
            }

            fn visit_u128<E>(self, value: u128) -> Result<EnumVariantValue, E>
            where
                E: de::Error,
            {
                Ok(EnumVariantValue::Specified(value as _))
            }

            fn visit_i64<E>(self, value: i64) -> Result<EnumVariantValue, E>
            where
                E: de::Error,
            {
                Ok(EnumVariantValue::Specified(value as _))
            }

            fn visit_u64<E>(self, value: u64) -> Result<EnumVariantValue, E>
            where
                E: de::Error,
            {
                Ok(EnumVariantValue::Specified(value as _))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(EnumVariantValue::None)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(EnumVariantValue::None)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "default" => Ok(EnumVariantValue::Default),
                    "catch-all" => Ok(EnumVariantValue::CatchAll),
                    _ => Err(serde::de::Error::unknown_variant(
                        v,
                        &["default", "catch-all"],
                    )),
                }
            }
        }

        deserializer.deserialize_any(EnumVariantValueVisitor)
    }
}
