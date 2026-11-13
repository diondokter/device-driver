use device_driver_diagnostics::{DynError, ResultExt};

mod lowering;
pub mod model;

pub fn lower_mir(
    source: &str,
    manifest: device_driver_mir::model::Manifest,
) -> Result<model::Driver, DynError> {
    let enums = lowering::transform_enums(&manifest);
    let field_sets = lowering::transform_field_sets(&manifest)
        .with_message(|| "could not transform fieldsets")?;
    let devices =
        lowering::transform_devices(&manifest).with_message(|| "could not transform devices")?;

    Ok(model::Driver {
        source_crc: crc32(source.as_bytes()),
        devices,
        field_sets,
        enums,
    })
}

fn crc32(data: &[u8]) -> u32 {
    const POLY: u32 = 0x82f6_3b78; // Castagnoli

    let mut crc = 0xFFFF_FFFF;

    for byte in data {
        crc ^= u32::from(*byte);

        for _ in 0..8 {
            let lowest_bit_set = crc & 1 > 0;
            crc >>= 1;
            if lowest_bit_set {
                crc ^= POLY;
            }
        }
    }

    !crc
}
