use core::ops::Not;
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
    ansi::*,
    bitdump::{BitdumpFormatter, Field},
    decoder::Operation,
};

pub fn render_event_into(
    w: &mut impl std::io::Write,
    style: DisplayStyle,
    manifests: &HashMap<u32, Manifest>,
    event: crate::decoder::Event,
) -> Result<(), DynError> {
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

    CorrelatedEvent {
        timestamp: event.timestamp,
        manifest_name: manifest.name.original(),
        device_name: device_name.unwrap_or(manifest.name.original()),
        operations: event
            .operations
            .into_iter()
            .map(|op| correlate_operation(manifest, op))
            .collect::<Result<Vec<_>, DynError>>()?,
    }
    .fmt_into(w, style)
    .with_message(|| "could not render the event")
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

impl CorrelatedEvent {
    fn fmt_into(&self, w: &mut impl std::io::Write, style: DisplayStyle) -> std::io::Result<()> {
        if let Some(timestamp) = self.timestamp {
            write!(w, "{DIM}[{:8.3}]{RESET} ", timestamp as f32 / 1_000_000.0)?;
        }

        write!(w, "TRACE ")?;

        if self.manifest_name == self.device_name {
            write!(w, "{} ", self.device_name)?;
        } else {
            write!(
                w,
                "{DIM}({}){RESET} {YELLOW}{}{RESET} ",
                self.manifest_name, self.device_name
            )?;
        }
        for operation in &self.operations {
            operation.fmt_into(w, style)?;
            writeln!(w)?;
        }

        Ok(())
    }
}

pub struct CorrelatedOperation {
    address: i128,
    actions: Vec<CorrelatedAction>,
}

impl CorrelatedOperation {
    fn fmt_into(&self, w: &mut impl std::io::Write, style: DisplayStyle) -> std::io::Result<()> {
        if self.address.is_negative() {
            write!(w, "{DIM}@ {}{RESET}", self.address)?;
        } else {
            write!(w, "{DIM}@ {:#04X}{RESET}", self.address)?;
        }

        if self.actions.len() > 1 {
            writeln!(w)?;
        } else {
            write!(w, ", ")?;
        }

        for action in &self.actions {
            action.fmt_into(w, style)?;
        }

        Ok(())
    }
}

struct CorrelatedAction {
    action: String,
    read_first: bool,
    is_modify: bool,
    operation_name: Istr,
    index: Option<String>,
    read_fs: Option<DecodedFieldset>,
    write_fs: Option<DecodedFieldset>,
    read_data: Vec<u8>,
    write_data: Vec<u8>,
}

impl CorrelatedAction {
    fn fmt_into(&self, w: &mut impl std::io::Write, style: DisplayStyle) -> std::io::Result<()> {
        write!(w, "{} {YELLOW}{}{RESET}", self.action, self.operation_name)?;
        if let Some(index) = self.index.as_ref() {
            writeln!(w, "[{index}]")?;
        } else {
            writeln!(w)?;
        }

        fn fmt_bytes(
            w: &mut impl std::io::Write,
            style: DisplayStyle,
            fs: Option<&DecodedFieldset>,
            fs_diff: Option<&DecodedFieldset>,
            bytes: &[u8],
            bytes_diff: Option<&[u8]>,
            operation_name: Istr,
        ) -> std::io::Result<()> {
            if let Some(fs) = fs
                && style != DisplayStyle::Raw
            {
                let diff_fs = if let Some(fs_diff) = &fs_diff {
                    fs_diff.fields.iter().map(Some).collect()
                } else {
                    vec![None; fs.fields.len()]
                };

                write!(
                    w,
                    "{}",
                    BitdumpFormatter::new(
                        (fs.name != operation_name).then(|| fs.name.to_string()),
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
                        style == DisplayStyle::Fancy
                    )
                )
            } else if !bytes.is_empty() {
                // Just bytes without associated fieldset (likely buffer)
                write!(w, "{DIM}>{RESET} ")?;
                for (index, byte) in bytes.iter().enumerate() {
                    if index != 0 {
                        write!(w, ", ")?;
                    }
                    write!(w, "{GREEN}{byte:02X}{RESET}")?;
                }
                writeln!(w)
            } else {
                Ok(())
            }
        }

        if self.is_modify {
            fmt_bytes(
                w,
                style,
                self.write_fs.as_ref(),
                self.read_fs.as_ref(),
                &self.write_data,
                Some(&self.read_data),
                self.operation_name,
            )?;
        } else {
            if self.read_first {
                fmt_bytes(
                    w,
                    style,
                    self.read_fs.as_ref(),
                    None,
                    &self.read_data,
                    None,
                    self.operation_name,
                )?;
                fmt_bytes(
                    w,
                    style,
                    self.write_fs.as_ref(),
                    None,
                    &self.write_data,
                    None,
                    self.operation_name,
                )?;
            } else {
                fmt_bytes(
                    w,
                    style,
                    self.write_fs.as_ref(),
                    None,
                    &self.write_data,
                    None,
                    self.operation_name,
                )?;
                fmt_bytes(
                    w,
                    style,
                    self.read_fs.as_ref(),
                    None,
                    &self.read_data,
                    None,
                    self.operation_name,
                )?;
            }
        }

        Ok(())
    }
}

fn operation_action(
    used_access: Option<Access>,
    operation_type: ObjectType,
) -> Result<(String, bool, bool), DynError> {
    match (used_access, operation_type) {
        (None, ObjectType::Command) => Ok((format!("{RED}dispatch{RESET} command"), false, false)),
        (Some(Access::RO), ObjectType::Command) => Ok((
            format!("{RED}dispatch {UNDERLINE}out{RESET} command"),
            false,
            false,
        )),
        (Some(Access::WO), ObjectType::Command) => Ok((
            format!("{RED}dispatch {UNDERLINE}in{RESET} command"),
            false,
            false,
        )),
        (Some(Access::RW), ObjectType::Command) => Ok((
            format!("{RED}dispatch {UNDERLINE}inout{RESET} command"),
            false,
            false,
        )),
        (Some(Access::RO), ObjectType::Register) => {
            Ok((format!("{RED}read{RESET} register"), false, true))
        }
        (Some(Access::WO), ObjectType::Register) => {
            Ok((format!("{RED}write{RESET} register"), false, true))
        }
        (Some(Access::RW), ObjectType::Register) => {
            Ok((format!("{RED}modify{RESET} register"), true, true))
        }
        (Some(Access::RO), ObjectType::Buffer) => {
            Ok((format!("{RED}read{RESET} buffer"), false, true))
        }
        (Some(Access::WO), ObjectType::Buffer) => {
            Ok((format!("{RED}write{RESET} buffer"), false, true))
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayStyle {
    Fancy,
    Compact,
    Raw,
}
