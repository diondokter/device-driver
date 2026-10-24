//! The runtime assumes all objects passed here have gone through all of the MIR passes

use std::fmt::Display;

use device_driver_common::{
    interner::Istr,
    specifiers::{BaseType, ByteOrder, Integer, RepeatSource},
};

use crate::model::{Enum, Field, FieldSet, Manifest, Object};

#[derive(Debug)]
pub struct DecodedFieldset {
    pub fields: Vec<DecodedField>,
}

#[derive(Debug)]
pub struct DecodedField {
    pub name: Istr,
    pub index_name: Option<String>,
    pub base_value: FieldBaseValue,
    pub converted_value: Option<String>,
}

#[derive(Debug)]
pub enum FieldBaseValue {
    Integer(i128),
    Bool(bool),
}

impl Display for FieldBaseValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldBaseValue::Integer(val) => write!(f, "{val:#X}"),
            FieldBaseValue::Bool(val) => write!(f, "{val}"),
        }
    }
}

impl FieldBaseValue {
    pub fn as_integer(&self) -> Option<i128> {
        if let Self::Integer(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

pub fn decode_fieldset(
    manifest: &Manifest,
    fieldset_model: &FieldSet,
    bytes: &[u8],
) -> Result<DecodedFieldset, DecodeError> {
    if bytes.len() != fieldset_model.size_bytes.value as usize {
        return Err(DecodeError::SizeBytesMismatch);
    }

    let mut fields = Vec::new();

    for field_id in fieldset_model.fields.iter() {
        let field = manifest.fields.get(*field_id).unwrap();

        let offsets = field
            .repeat
            .as_ref()
            .map(|repeat| match &repeat.source.value {
                RepeatSource::Count(non_zero) => (0..non_zero.get())
                    .map(|index| (index as i128 * repeat.stride.value, Some(index.to_string())))
                    .collect(),
                RepeatSource::Enum(identifier_ref) => {
                    let enum_model = manifest
                        .search_object(identifier_ref)
                        .unwrap()
                        .as_enum()
                        .unwrap();
                    enum_model
                        .iter_variants_with_discriminant(&manifest.enum_variants)
                        .map(|(index, variant_id)| {
                            (
                                index * repeat.stride.value,
                                Some(
                                    manifest
                                        .enum_variants
                                        .get(variant_id)
                                        .unwrap()
                                        .name
                                        .original()
                                        .to_string(),
                                ),
                            )
                        })
                        .collect()
                }
            })
            .unwrap_or_else(|| vec![(0, None)]);

        for (offset, index_name) in offsets {
            let start =
                (field.field_address.start as usize).strict_add_signed(offset.try_into().unwrap());
            let end =
                (field.field_address.end as usize).strict_add_signed(offset.try_into().unwrap());

            let value = match field.base_type.value {
                BaseType::Bool => FieldBaseValue::Bool(
                    load_integer(
                        bytes,
                        fieldset_model.byte_order.unwrap(),
                        Integer::U8,
                        start,
                        end,
                    ) != 0,
                ),
                BaseType::FixedSize(integer) => FieldBaseValue::Integer(load_integer(
                    bytes,
                    fieldset_model.byte_order.unwrap(),
                    integer,
                    start,
                    end,
                )),
                BaseType::Unspecified | BaseType::Uint | BaseType::Int => unreachable!(),
            };

            fields.push(decode_field(manifest, field, index_name, value));
        }
    }

    Ok(DecodedFieldset { fields })
}

pub fn decode_field(
    manifest: &Manifest,
    field_model: &Field,
    index_name: Option<String>,
    value: FieldBaseValue,
) -> DecodedField {
    let converted_value = if let Some(conversion) = &field_model.field_conversion {
        let target_type = manifest.search_type(&conversion.type_ref.value).unwrap();
        match target_type {
            Object::Enum(e) => match decode_enum(manifest, e, value.as_integer().unwrap()) {
                Ok(active_variant) => Some(format!("{}::{active_variant}", e.name.original())),
                Err(e) => Some(format!("<{e}>")),
            },
            Object::Extern(_) => Some("<extern>".into()),
            _ => unreachable!(),
        }
    } else {
        None
    };

    DecodedField {
        name: field_model.name.original(),
        index_name,
        base_value: value,
        converted_value,
    }
}

pub fn decode_enum(
    manifest: &Manifest,
    enum_model: &Enum,
    value: i128,
) -> Result<Istr, DecodeError> {
    let mut fallback_variant = None;

    for (variant_value, variant_id) in
        enum_model.iter_variants_with_discriminant(&manifest.enum_variants)
    {
        let variant = manifest.enum_variants.get(variant_id).unwrap();

        if variant_value == value {
            return Ok(variant.name.original());
        }

        if variant.value.is_catch_all() {
            // Catch-all trumps default
            fallback_variant = Some(variant.name.original());
        }
        if variant.value.is_default() && fallback_variant.is_none() {
            // Only take default if not already set
            fallback_variant = Some(variant.name.original());
        }
    }

    if let Some(fallback) = fallback_variant {
        return Ok(fallback);
    }

    Err(DecodeError::InvalidValue)
}

pub enum DecodeError {
    InvalidValue,
    SizeBytesMismatch,
}

impl Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::InvalidValue => write!(f, "invalid value"),
            DecodeError::SizeBytesMismatch => write!(f, "size-bytes mismatch"),
        }
    }
}

pub fn load_integer(
    bytes: &[u8],
    byte_order: ByteOrder,
    base_type: Integer,
    start: usize,
    end: usize,
) -> i128 {
    let mut val = 0;

    for index in start..=end {
        let byte_index = match byte_order {
            ByteOrder::LE => index / 8,
            ByteOrder::BE => bytes.len() - (index / 8) - 1,
        };
        let bit_index = index % 8;

        let byte = bytes[byte_index];
        let bit = byte & (1 << bit_index);

        if bit > 0 {
            let val_bit_index = index - start;
            val |= 1 << val_bit_index;
        }
    }

    if base_type.is_signed() {
        // Sign extend if required
        let sign_bit_index = end - start;
        if val & (1 << sign_bit_index) > 0 {
            val |= !0i128 << sign_bit_index;
        }
    }

    val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_unsigned_integer() {
        assert_eq!(
            load_integer(&0b1101u16.to_le_bytes(), ByteOrder::LE, Integer::U8, 0, 3),
            0b1101
        );
        assert_eq!(
            load_integer(&0b1101u16.to_le_bytes(), ByteOrder::LE, Integer::U8, 1, 3),
            0b110
        );
        assert_eq!(
            load_integer(&0b1101u16.to_le_bytes(), ByteOrder::LE, Integer::U8, 0, 2),
            0b101
        );

        assert_eq!(
            load_integer(&0b1101u16.to_be_bytes(), ByteOrder::BE, Integer::U8, 0, 3),
            0b1101
        );
        assert_eq!(
            load_integer(&0b1101u16.to_be_bytes(), ByteOrder::BE, Integer::U8, 1, 3),
            0b110
        );
        assert_eq!(
            load_integer(&0b1101u16.to_be_bytes(), ByteOrder::BE, Integer::U8, 0, 2),
            0b101
        );
    }

    #[test]
    fn load_signed_integer() {
        assert_eq!(
            load_integer(&(-5i16).to_le_bytes(), ByteOrder::LE, Integer::I8, 0, 3),
            -5
        );
        assert_eq!(
            load_integer(&(-5i16).to_be_bytes(), ByteOrder::BE, Integer::I8, 0, 3),
            -5
        );
    }
}
