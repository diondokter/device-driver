use super::Device;

mod addresses_non_overlapping;

pub fn run_passes(device: &mut Device) -> anyhow::Result<()> {
    addresses_non_overlapping::run_pass(device)?;

    Ok(())
}
