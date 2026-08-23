use std::range::Range;

use askama::Template;
use clap::Parser;
use device_driver_common::{span::Spanned, specifiers::Access};
use device_driver_diagnostics::DynError;
use device_driver_lir::model::{BlockMethod, BlockMethodType, Driver, Field, FieldSet, Repeat};

use crate::File;

#[derive(Parser, Debug, Clone, Default)]
#[command(no_binary_name = true, bin_name = "")]
pub struct DocsCodegenOptions {
    /// How many bits wide should the tables be
    #[arg(
        long = "docs-max-table-bit-width",
        value_name = "NUMBER",
        require_equals = true,
        default_value = "16"
    )]
    pub max_table_bit_width: u32,
}

pub fn codegen(
    codegen_options: &DocsCodegenOptions,
    lir_driver: &Driver,
    source: &str,
) -> Result<Vec<File>, DynError> {
    let mut output_files = Vec::new();

    // Write out all operations
    for method in lir_driver
        .devices
        .iter()
        .flat_map(|device| device.blocks.iter())
        .flat_map(|block| block.methods.iter())
    {
        match &method.method_type {
            BlockMethodType::Block { .. } => {}
            BlockMethodType::Register {
                field_set_name,
                access,
                reset_value,
            } => {
                output_files.push(File {
                    name: format!("register.{}.html", method.name.original()),
                    contents: RegisterTemplateDocs {
                        method,
                        fieldset_template: FieldsetTemplateDocs {
                            fieldset: lir_driver
                                .field_sets
                                .iter()
                                .find(|fs| fs.name == *field_set_name)
                                .ok_or_else(|| {
                                    DynError::new(format!(
                                        "could not find fieldset {}",
                                        field_set_name.original()
                                    ))
                                })?,
                            codegen_options,
                        },
                        access,
                        reset_value,
                    }
                    .to_string(),
                });
            }
            BlockMethodType::Command {
                field_set_name_in,
                field_set_name_out,
            } => todo!(),
            BlockMethodType::Buffer { access } => todo!(),
        }
    }

    Ok(output_files)
}

#[derive(Template)]
#[template(
    path = "docs/register_page.html.j2",
    escape = "none",
    whitespace = "minimize"
)]
pub struct RegisterTemplateDocs<'a> {
    method: &'a BlockMethod,
    fieldset_template: FieldsetTemplateDocs<'a>,
    access: &'a Access,
    reset_value: &'a Option<Spanned<Vec<u8>>>,
}

#[derive(Template)]
#[template(
    path = "docs/fieldset.html.j2",
    escape = "none",
    whitespace = "minimize"
)]
pub struct FieldsetTemplateDocs<'a> {
    fieldset: &'a FieldSet,
    codegen_options: &'a DocsCodegenOptions,
}

impl<'a> FieldsetTemplateDocs<'a> {
    fn overview_tables(&self) -> impl Iterator<Item = Range<u32>> {
        (0..self.fieldset.size_bytes * 8)
            .step_by(self.codegen_options.max_table_bit_width as usize)
            .map(|bit_start| {
                (bit_start
                    ..(bit_start + self.codegen_options.max_table_bit_width)
                        .min(self.fieldset.size_bytes * 8))
                    .into()
            })
            .rev()
    }

    fn fields_in_range(
        &self,
        mut bit_range: Range<u32>,
    ) -> impl Iterator<Item = (Option<&Field>, u32)> {
        std::iter::from_fn(move || {
            // TODO: What about overlapping fields?

            if bit_range.start >= bit_range.end || bit_range.end == 0 {
                return None;
            }

            let last_bit = bit_range.end - 1;
            let Some(field) = self
                .fieldset
                .fields
                .iter()
                .find(|field| (field.address.start..=field.address.end).contains(&last_bit))
            else {
                let first_next_bit = self
                    .fieldset
                    .fields
                    .iter()
                    .map(|field| field.address.end + 1)
                    .filter(|end| *end < bit_range.end)
                    .max()
                    .unwrap_or_default();
                let change = bit_range.end - first_next_bit.max(bit_range.start);
                bit_range.end -= change;

                return Some((None, change));
            };

            let first_next_bit = field.address.start;
            let change = bit_range.end - first_next_bit.max(bit_range.start);
            bit_range.end -= change;

            Some((Some(field), change))
        })
    }
}
