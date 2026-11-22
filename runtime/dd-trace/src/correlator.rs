use core::{fmt::Display, ops::Not};
use std::collections::HashMap;

use device_driver_common::{
    interner::Istr,
    specifiers::{Access, Integer, RepeatSource},
};
use device_driver_diagnostics::{DynError, ResultExt};
use device_driver_mir::{
    model::{FieldSet, Manifest, Object, ObjectId, ObjectType},
    runtime::{DecodedFieldset, decode_fieldset},
};

use crate::{
    bitdump::{BitdumpFormatter, Field},
    decoder::Operation,
};

pub fn correlate(
    manifests: &HashMap<u32, Manifest>,
    event: crate::decoder::Event,
) -> Result<CorrelatedEvent, DynError> {
    let manifest = manifests.get(&event.source_crc).ok_or_else(|| {
        DynError::new(format!(
            "got event for manifest with source CRC of \"{}\", but no manifest is found",
            event.source_crc
        ))
    })?;

    let device_name = if let Some(op) = event.operations.first() {
        let object = op
            .objects
            .first()
            .ok_or_else(|| DynError::new("an operation has no associated objects"))?;
        manifest
            .object_config(ObjectId::try_from_raw(object.object_id)?)
            .owner
            .and_then(|owner| owner.get(manifest).map(|dev| dev.name.original()))
    } else {
        None
    };

    Ok(CorrelatedEvent {
        timestamp: event.timestamp,
        manifest_name: manifest.name.original(),
        device_name: device_name.unwrap_or(manifest.name.original()),
        operations: event
            .operations
            .into_iter()
            .map(|op| correlate_operation(manifest, op))
            .collect::<Result<Vec<_>, DynError>>()?,
    })
}

fn correlate_operation(
    manifest: &Manifest,
    mut operation: Operation,
) -> Result<CorrelatedOperation, DynError> {
    let first_object_id = ObjectId::try_from_raw(
        operation
            .objects
            .first()
            .ok_or_else(|| DynError::new("an operation has no associated objects"))?
            .object_id,
    )?;
    let first_object_config = manifest.object_config(first_object_id);
    let address_type = match first_object_id.object_type() {
        ObjectType::Register => first_object_config.register_address_type,
        ObjectType::Command => first_object_config.command_address_type,
        ObjectType::Buffer => first_object_config.buffer_address_type,
        _ => {
            return Err(DynError::new(
                "got an operation that was not an operation type",
            ));
        }
    };

    let Some(address_type) = address_type else {
        return Err(DynError::new(
            "operation doesn't have an address type. Are we looking at the correct manifest?",
        ));
    };

    if operation.address.len() * 8 != address_type.size_bits() as usize {
        return Err(DynError::new(
            "operation address has an unexpected size. Are we looking at the correct manifest?",
        ));
    }

    let address = match address_type.value {
        Integer::U8 => u8::from_le_bytes(operation.address.try_into().unwrap()) as i128,
        Integer::U16 => u16::from_le_bytes(operation.address.try_into().unwrap()) as i128,
        Integer::U32 => u32::from_le_bytes(operation.address.try_into().unwrap()) as i128,
        Integer::U64 => u64::from_le_bytes(operation.address.try_into().unwrap()) as i128,
        Integer::I8 => i8::from_le_bytes(operation.address.try_into().unwrap()) as i128,
        Integer::I16 => i16::from_le_bytes(operation.address.try_into().unwrap()) as i128,
        Integer::I32 => i32::from_le_bytes(operation.address.try_into().unwrap()) as i128,
        Integer::I64 => i64::from_le_bytes(operation.address.try_into().unwrap()) as i128,
    };

    let used_access = match (
        operation.read_data.is_empty(),
        operation.write_data.is_empty(),
    ) {
        (true, true) => None,
        (true, false) => Some(Access::WO),
        (false, true) => Some(Access::RO),
        (false, false) => Some(Access::RW),
    };

    let action_infos = operation
        .objects
        .iter()
        .map(|object| {
            let id = ObjectId::try_from_raw(object.object_id)?;

            let mir_object = id.get(manifest).ok_or_else(|| {
                DynError::new(
                    "could not find operation object. Are we looking at the correct manifest?",
                )
            })?;

            let index = match (object.index, mir_object.repeat()) {
                (None, None) => None,
                (None, Some(_)) | (Some(_), None) => return Err(DynError::new("trace and manifest disagree on object index. Are we looking at the correct manifest?")),
                (Some(num), Some(repeat)) => {
                    match &repeat.source.value {
                        RepeatSource::Count(_) => Some(num.to_string()),
                        RepeatSource::Enum(identifier_ref) => {
                            let repeat_enum = manifest.search_object(identifier_ref).ok_or_else(|| {
                                DynError::new(
                                    "could not find repeat enum. Are we looking at the correct manifest?",
                                )
                            })?;
                            let repeat_enum = repeat_enum.as_enum().ok_or_else(|| DynError::new(
                                    "repeat type is not an enum. Are we looking at the correct manifest?",
                                ))?;
                            let decoded = device_driver_mir::runtime::decode_enum(manifest, repeat_enum, num as _)
                                .with_message(|| "could not decode repeat enum value")?;
                            Some(format!("{}::{decoded}", repeat_enum.name.original()))
                        },
                    }
                },
            };

            match mir_object {
                Object::Register(register) => Ok(ActionInfo {
                    operation_name: register.name.original(),
                    index,
                    read_fs: operation
                        .read_data
                        .is_empty()
                        .not()
                        .then(|| manifest.search_fieldset(&register.field_set_ref).unwrap()),
                    write_fs: operation
                        .write_data
                        .is_empty()
                        .not()
                        .then(|| manifest.search_fieldset(&register.field_set_ref).unwrap()),
                    object_type: ObjectType::Register,
                }),
                Object::Command(command) => Ok(ActionInfo {
                    operation_name: command.name.original(),
                    index,
                    read_fs: command
                        .field_set_ref_out
                        .as_ref()
                        .map(|fs_ref| manifest.search_fieldset(&fs_ref.value).unwrap()),
                    write_fs: command
                        .field_set_ref_in
                        .as_ref()
                        .map(|fs_ref| manifest.search_fieldset(&fs_ref.value).unwrap()),
                    object_type: ObjectType::Command,
                }),
                Object::Buffer(buffer) => Ok(ActionInfo {
                    operation_name: buffer.name.original(),
                    index,
                    read_fs: None,
                    write_fs: None,
                    object_type: ObjectType::Buffer,
                }),
                _ => Err(DynError::new(
                    "operation object has unexpected type. Are we looking at the correct manifest?",
                )),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Only registers are allowed multi-action ops
    let operation_object_type = action_infos[0].object_type;
    if action_infos
        .iter()
        .any(|info| info.object_type != operation_object_type)
    {
        return Err(DynError::new("not all actions are of the same type"));
    }

    if operation_object_type != ObjectType::Buffer {
        let total_fs_read_len = action_infos
            .iter()
            .map(|info| info.read_fs.map(|fs| fs.size_bytes.value).unwrap_or(0))
            .sum::<u32>() as usize;
        let total_fs_write_len = action_infos
            .iter()
            .map(|info| info.write_fs.map(|fs| fs.size_bytes.value).unwrap_or(0))
            .sum::<u32>() as usize;

        if total_fs_read_len != operation.read_data.len() {
            return Err(DynError::new(
                "read data has unexpected length. Are we looking at the correct manifest?",
            ));
        }
        if total_fs_write_len != operation.write_data.len() {
            return Err(DynError::new(
                "write data has unexpected length. Are we looking at the correct manifest?",
            ));
        }
    }

    let actions = action_infos
        .into_iter()
        .map(|info| {
            let read_data = if let Some(fs) = info.read_fs {
                let mut temp = operation.read_data.split_off(fs.size_bytes.value as usize);
                std::mem::swap(&mut temp, &mut operation.read_data);
                temp
            } else {
                operation.read_data.clone()
            };

            let write_data = if let Some(fs) = info.write_fs {
                let mut temp = operation.write_data.split_off(fs.size_bytes.value as usize);
                std::mem::swap(&mut temp, &mut operation.write_data);
                temp
            } else {
                operation.write_data.clone()
            };

            let (action, is_modify, read_first) =
                operation_action(used_access, first_object_id.object_type())?;
            Ok(CorrelatedAction {
                action,
                read_first,
                is_modify,
                operation_name: info.operation_name,
                index: info.index,
                read_fs: info
                    .read_fs
                    .map(|fs| decode_fieldset(manifest, fs, &read_data).into_dyn_result())
                    .transpose()?,
                write_fs: info
                    .write_fs
                    .map(|fs| decode_fieldset(manifest, fs, &write_data).into_dyn_result())
                    .transpose()?,
                read_data,
                write_data,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CorrelatedOperation { address, actions })
}

pub struct CorrelatedEvent {
    timestamp: Option<u64>,
    manifest_name: Istr,
    device_name: Istr,
    operations: Vec<CorrelatedOperation>,
}

impl Display for CorrelatedEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(timestamp) = self.timestamp {
            write!(f, "[{:8.3}] ", timestamp as f32 / 1_000_000.0)?;
        }

        write!(f, "TRACE ")?;

        if self.manifest_name == self.device_name {
            write!(f, "{} ", self.device_name)?;
        } else {
            write!(f, "({}) {} ", self.manifest_name, self.device_name)?;
        }
        for operation in &self.operations {
            writeln!(f, "{operation}")?;
        }

        Ok(())
    }
}

pub struct CorrelatedOperation {
    address: i128,
    actions: Vec<CorrelatedAction>,
}

impl Display for CorrelatedOperation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "address: {:#X}", self.address)?; // TODO: This renders negative addresses quite poorly

        if self.actions.len() > 1 {
            writeln!(f)?;
        } else {
            write!(f, ", ")?;
        }

        for action in &self.actions {
            write!(f, "{action}")?;
        }

        Ok(())
    }
}

struct CorrelatedAction {
    action: &'static str,
    read_first: bool,
    is_modify: bool,
    operation_name: Istr,
    index: Option<String>,
    read_fs: Option<DecodedFieldset>,
    write_fs: Option<DecodedFieldset>,
    read_data: Vec<u8>,
    write_data: Vec<u8>,
}

impl Display for CorrelatedAction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} {}", self.action, self.operation_name)?;
        if let Some(index) = self.index.as_ref() {
            writeln!(f, "[{index}]")?;
        } else {
            writeln!(f)?;
        }

        fn fmt_bytes(
            f: &mut core::fmt::Formatter<'_>,
            fs: Option<&DecodedFieldset>,
            fs_diff: Option<&DecodedFieldset>,
            bytes: &[u8],
            bytes_diff: Option<&[u8]>,
        ) -> core::fmt::Result {
            if let Some(fs) = fs {
                let diff_fs = if let Some(fs_diff) = &fs_diff {
                    fs_diff.fields.iter().map(Some).collect()
                } else {
                    vec![None; fs.fields.len()]
                };

                write!(
                    f,
                    "{}",
                    BitdumpFormatter::new(
                        fs.name.to_string(),
                        bytes.to_vec(),
                        bytes_diff.map(|b| b.to_vec()),
                        fs.fields
                            .iter()
                            .zip(diff_fs)
                            .map(|(field, diff_field)| Field {
                                #[allow(clippy::single_range_in_vec_init)]
                                ranges: vec![
                                    field.address.start as usize..field.address.end as usize + 1
                                ],
                                name: field.name.to_string(),
                                base_value: field.base_value.to_string(),
                                converted_value: field.converted_value.clone(),
                                base_value_diff: if let Some(df) = &diff_field
                                    && df.base_value != field.base_value
                                {
                                    Some(df.base_value.to_string())
                                } else {
                                    None
                                },
                                converted_value_diff: if let Some(df) = &diff_field
                                    && df.converted_value != field.converted_value
                                {
                                    Some(df.converted_value.clone())
                                } else {
                                    None
                                },
                                reserved: false,
                            })
                            .collect(),
                        true
                    )
                )
            } else if !bytes.is_empty() {
                // Just bytes without associated fieldset (likely buffer)
                write!(f, "> {:02X?}", bytes)
            } else {
                Ok(())
            }
        }

        if self.is_modify {
            fmt_bytes(
                f,
                self.write_fs.as_ref(),
                self.read_fs.as_ref(),
                &self.write_data,
                Some(&self.read_data),
            )?;
        } else {
            if self.read_first {
                fmt_bytes(f, self.read_fs.as_ref(), None, &self.read_data, None)?;
                fmt_bytes(f, self.write_fs.as_ref(), None, &self.write_data, None)?;
            } else {
                fmt_bytes(f, self.write_fs.as_ref(), None, &self.write_data, None)?;
                fmt_bytes(f, self.read_fs.as_ref(), None, &self.read_data, None)?;
            }
        }

        Ok(())
    }
}

fn operation_action(
    used_access: Option<Access>,
    operation_type: ObjectType,
) -> Result<(&'static str, bool, bool), DynError> {
    match (used_access, operation_type) {
        (None, ObjectType::Command) => Ok(("dispatch command", false, false)),
        (Some(Access::RO), ObjectType::Command) => Ok(("dispatch out command", false, false)),
        (Some(Access::WO), ObjectType::Command) => Ok(("dispatch in command", false, false)),
        (Some(Access::RW), ObjectType::Command) => Ok(("dispatch inout command", false, false)),
        (Some(Access::RO), ObjectType::Register) => Ok(("read register", false, true)),
        (Some(Access::WO), ObjectType::Register) => Ok(("write register", false, true)),
        (Some(Access::RW), ObjectType::Register) => Ok(("modify register", true, true)),
        (Some(Access::RO), ObjectType::Buffer) => Ok(("read buffer", false, true)),
        (Some(Access::WO), ObjectType::Buffer) => Ok(("write buffer", false, true)),
        _ => Err(DynError::new(
            "unexpected combination of access pattern and operation type. Are we looking at the correct manifest?",
        )),
    }
}

struct ActionInfo<'a> {
    operation_name: Istr,
    object_type: ObjectType,
    index: Option<String>,
    read_fs: Option<&'a FieldSet>,
    write_fs: Option<&'a FieldSet>,
}
