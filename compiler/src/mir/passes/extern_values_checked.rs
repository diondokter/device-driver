use miette::{bail, ensure};

use crate::mir::{BaseType, Device, Integer, Object};

use super::recurse_objects_mut;

/// Checks if externs are fully specified
pub fn run_pass(device: &mut Device) -> miette::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        let object_name = object.name().to_string();

        if let Object::Extern(extern_value) = object {
            let (base_type_integer, size_bits) = match (
                extern_value.base_type,
                extern_value.size_bits,
            ) {
                (BaseType::Unspecified, None) => {
                    bail!(
                        "Extern `{object_name}` has an unspecified base type, but doesn't specify its bit size. This is not allowed"
                    )
                }
                (BaseType::Bool, _) => {
                    bail!("Extern `{object_name}` uses a bool as base type, which is not allowed")
                }
                (BaseType::Uint | BaseType::Int, None) => {
                    bail!(
                        "Extern `{object_name}` uses a dynamic width integer as base type, but doesn't specify its bit size. This is not allowed"
                    )
                }
                (BaseType::Uint | BaseType::Unspecified, Some(size_bits)) => {
                    (Integer::find_smallest(0, 0, size_bits), size_bits)
                }
                (BaseType::Int, Some(size_bits)) => {
                    (Integer::find_smallest(-1, 0, size_bits), size_bits)
                }
                (BaseType::FixedSize(integer), size_bits) => {
                    ensure!(
                        integer.size_bits() >= size_bits.unwrap_or_default(),
                        "Extern `{object_name}` has specified a bit size that is larger than its base type. This is not allowed"
                    );
                    (Some(integer), size_bits.unwrap_or(integer.size_bits()))
                }
            };

            let Some(base_type_integer) = base_type_integer else {
                bail!(
                    "No valid base type could be selected for extern `{object_name}`. The specified bit size cannot fit within any of the supported integer types"
                )
            };

            extern_value.base_type = BaseType::FixedSize(base_type_integer);
            extern_value.size_bits = Some(size_bits);
        }

        Ok(())
    })
}
