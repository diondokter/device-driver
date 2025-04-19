use anyhow::ensure;

use crate::mir::{Conversion, Device, EnumGenerationStyle, EnumValue};

use super::recurse_objects_mut;

pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    recurse_objects_mut(&mut device.objects, &mut |object| {
        let Some(repeat) = object.repeat_mut() else {
            return Ok(());
        };

        match &mut repeat.count {
            crate::mir::RepeatCount::Value(_) => {}
            crate::mir::RepeatCount::Conversion(Conversion::Enum {
                enum_value,
                use_try,
            }) => {
                ensure!(
                    !*use_try,
                    "Try conversions are not supported for repeat counts: found in object \"{}\"",
                    object.name(),
                );

                enum_value.generation_style = Some(EnumGenerationStyle::Index);

                for variant in &mut enum_value.variants {
                    ensure!(
                        matches!(
                            variant.value,
                            EnumValue::Unspecified | EnumValue::Specified(_)
                        ),
                        "Repeat count conversions don't support 'default' and 'catch-all' variants: found in object \"{}\"",
                        object.name()
                    );
                }
            }
            crate::mir::RepeatCount::Conversion(Conversion::Direct { use_try, .. }) => {
                ensure!(
                    !*use_try,
                    "Try conversions are not supported for repeat counts: found in object \"{}\"",
                    object.name(),
                );
            }
        }

        Ok(())
    })
}
