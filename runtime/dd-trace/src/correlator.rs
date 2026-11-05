use core::fmt::Display;
use std::collections::HashMap;

use device_driver_common::{
    interner::Istr,
    specifiers::{Access, Integer},
};
use device_driver_diagnostics::DynError;
use device_driver_mir::model::{Manifest, ObjectId, ObjectType};

use crate::decoder::Operation;

pub fn correlate(
    manifests: HashMap<u8, &Manifest>,
    event: crate::decoder::Event,
) -> Result<CorrelatedEvent, DynError> {
    let manifest = *manifests.get(&event.instance).ok_or_else(|| {
        DynError::new(format!(
            "got event with instance {}, but no manifest is registered with that value",
            event.instance
        ))
    })?;

    Ok(CorrelatedEvent {
        timestamp: event.timestamp,
        manifest_name: manifest.name.original(),
        operations: event
            .operations
            .into_iter()
            .map(|op| correlate_operation(manifest, op))
            .collect::<Result<Vec<_>, DynError>>()?,
    })
}

fn correlate_operation(
    manifest: &Manifest,
    operation: Operation,
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

    if operation.address.len() != address_type.size_bits() as usize * 8 {
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

    Ok(CorrelatedOperation {
        operation_name: operation_name(used_access, first_object_id.object_type())?,
        address,
        read_data: operation.read_data,
        write_data: operation.write_data,
    })
}

pub struct CorrelatedEvent {
    timestamp: Option<u64>,
    manifest_name: Istr,
    operations: Vec<CorrelatedOperation>,
}

impl Display for CorrelatedEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(timestamp) = self.timestamp {
            write!(f, "[{:8.3}] ", timestamp as f32 / 1_000_000.0)?;
        }

        writeln!(f, "TRACE ({})", self.manifest_name)?;
        for operation in &self.operations {
            writeln!(f, "{operation}")?;
        }

        Ok(())
    }
}

pub struct CorrelatedOperation {
    operation_name: &'static str,
    address: i128,
    read_data: Vec<u8>,
    write_data: Vec<u8>,
}

impl Display for CorrelatedOperation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "@{:#X} {}", self.address, self.operation_name)
    }
}

fn operation_name(
    used_access: Option<Access>,
    operation_type: ObjectType,
) -> Result<&'static str, DynError> {
    match (used_access, operation_type) {
        (_, ObjectType::Command) => Ok("dispatch"),
        (Some(Access::RO), ObjectType::Register | ObjectType::Buffer) => Ok("read"),
        (Some(Access::WO), ObjectType::Register | ObjectType::Buffer) => Ok("write"),
        (Some(Access::RW), ObjectType::Register) => Ok("modify"),
        _ => Err(DynError::new(
            "unexpected combination of access pattern and operation type. Are we looking at the correct manifest?",
        )),
    }
}
