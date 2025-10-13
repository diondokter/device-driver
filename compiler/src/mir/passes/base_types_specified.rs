use miette::bail;

use crate::mir::{BaseType, Integer, LendingIterator, Manifest};

/// Turn all unspecified base types into either bools or uints based on the size of the field
pub fn run_pass(manifest: &mut Manifest) -> miette::Result<()> {
    let mut iter = manifest.iter_objects_with_config_mut();
    while let Some((object, _)) = iter.next() {
        if let Some(field_set) = object.as_field_set_mut() {
            for field in field_set.fields.iter_mut() {
                let size_bits = field.field_address.len() as u32;
                loop {
                    field.base_type = match field.base_type {
                        BaseType::Unspecified => match size_bits {
                            0 => unreachable!(),
                            1 => BaseType::Bool,
                            _ => BaseType::Uint,
                        },
                        BaseType::Bool => break,
                        BaseType::Uint => match Integer::find_smallest(0, 0, size_bits) {
                            Some(integer) => BaseType::FixedSize(integer),
                            None => bail!(
                                "Field `{}` on field set `{}` uses {size_bits} bits which is too big for any of the supported integers",
                                field.name,
                                field_set.name
                            ),
                        },
                        BaseType::Int => match Integer::find_smallest(-1, 0, size_bits) {
                            Some(integer) => BaseType::FixedSize(integer),
                            None => bail!(
                                "Field `{}` on field set `{}` uses {size_bits} bits which is too big for any of the supported integers",
                                field.name,
                                field_set.name
                            ),
                        },
                        BaseType::FixedSize(_) => break,
                    }
                }
            }
        }
    }

    Ok(())
}
