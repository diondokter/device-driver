use super::Driver;

mod addresses_non_overlapping;

pub fn run_passes(driver: &mut Driver) -> miette::Result<()> {
    addresses_non_overlapping::run_pass(driver)?;

    Ok(())
}
