use std::{borrow::Cow, collections::HashMap, range::Range};

use askama::Template;
use clap::Parser;
use device_driver_common::{
    span::Spanned,
    specifiers::{Access, ByteOrder},
};
use device_driver_diagnostics::DynError;
use device_driver_lir::model::{
    BlockMethod, BlockMethodType, Driver, Field, FieldConversionMethod, FieldSet, Repeat,
};
use itertools::Itertools;

use crate::File;

#[derive(Parser, Debug, Clone, Default)]
#[command(no_binary_name = true, bin_name = "")]
pub struct DocsCodegenOptions {
    /// How many bits wide the tables can be at most.
    /// If 0, then the value is guessed.
    #[arg(
        long = "docs-max-table-bit-width",
        value_name = "NUMBER",
        require_equals = true,
        default_value = "0"
    )]
    pub max_table_bit_width: u32,
}

pub fn codegen(
    mut codegen_options: DocsCodegenOptions,
    lir_driver: &Driver,
    source: &str,
) -> Result<Vec<File>, DynError> {
    if codegen_options.max_table_bit_width == 0 {
        let mut counts = HashMap::<u32, u32>::new();
        *counts.entry(8).or_default() += 1;

        for size in lir_driver.field_sets.iter().map(|fs| fs.size_bytes) {
            if size.is_multiple_of(4) {
                *counts.entry(32).or_default() += 1;
            }
            if size.is_multiple_of(3) {
                *counts.entry(24).or_default() += 1;
            }
            if size.is_multiple_of(2) {
                *counts.entry(16).or_default() += 1;
            }
            if size == 1 {
                *counts.entry(8).or_default() += 1;
            }
        }
        codegen_options.max_table_bit_width = counts
            .into_iter()
            // Grab the most commonly used, but with a little favor for the bigger sizes
            .max_by_key(|(val, count)| *count + (val / 8))
            .unwrap_or_default()
            .0;
    }

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
                            reset_value: Some(reset_value),
                            codegen_options: &codegen_options,
                            source,
                        },
                        access,
                    }
                    .to_string(),
                });
            }
            BlockMethodType::Command {
                field_set_name_in: _,
                field_set_name_out: _,
            } => todo!(),
            BlockMethodType::Buffer { access: _ } => todo!(),
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
}

#[derive(Template)]
#[template(
    path = "docs/fieldset.html.j2",
    escape = "none",
    whitespace = "minimize"
)]
pub struct FieldsetTemplateDocs<'a> {
    fieldset: &'a FieldSet,
    reset_value: Option<&'a Option<Spanned<Vec<u8>>>>,
    codegen_options: &'a DocsCodegenOptions,
    source: &'a str,
}

impl<'a> FieldsetTemplateDocs<'a> {
    fn get_reset_value_text(&self) -> Option<&str> {
        match self.reset_value {
            Some(Some(reset_value)) => {
                Some(&self.source[reset_value.span.start..reset_value.span.end])
            }
            Some(None) => Some("0"),
            None => None,
        }
    }

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

    fn reset_values_in_range(
        &self,
        mut bit_range: Range<u32>,
    ) -> impl Iterator<Item = (Option<u64>, u32)> {
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

                let reset_value_range = bit_range.end..bit_range.end + change;

                return Some((
                    self.reset_value.map(|reset_value| {
                        let reset_value = reset_value
                            .as_ref()
                            .map(|reset_value| reset_value.value.clone())
                            .unwrap_or_else(|| vec![0; self.fieldset.size_bytes as usize]);
                        load_bits(
                            &reset_value,
                            self.fieldset.byte_order,
                            reset_value_range.into(),
                        )
                    }),
                    change,
                ));
            };

            let first_next_bit = field.address.start;
            let change = bit_range.end - first_next_bit.max(bit_range.start);
            bit_range.end -= change;
            let reset_value_range = bit_range.end..bit_range.end + change;

            Some((
                self.reset_value.map(|reset_value| {
                    let reset_value = reset_value
                        .as_ref()
                        .map(|reset_value| reset_value.value.clone())
                        .unwrap_or_else(|| vec![0; self.fieldset.size_bytes as usize]);
                    load_bits(
                        &reset_value,
                        self.fieldset.byte_order,
                        reset_value_range.into(),
                    )
                }),
                change,
            ))
        })
    }

    fn field_conversion_display(&self, field: &Field) -> Cow<'static, str> {
        match &field.conversion_method {
            FieldConversionMethod::None => "".into(),
            FieldConversionMethod::Into(identifier) => {
                format!("{}<br/>from ", identifier.original()).into()
            }
            FieldConversionMethod::UnsafeInto(identifier) => {
                format!("{}<br/>from ", identifier.original()).into()
            }
            FieldConversionMethod::TryInto(identifier) => {
                format!("{}<br/>try from ", identifier.original()).into()
            }
            FieldConversionMethod::Bool => "".into(),
        }
    }

    fn field_reset_value(&self, field: &Field) -> Cow<'static, str> {
        let Some(Some(reset_value)) = self.reset_value else {
            return "0h".into();
        };

        let bits = load_bits(
            &reset_value.value,
            self.fieldset.byte_order,
            (field.address.start..field.address.end + 1).into(),
        );

        // TODO: Show converted value

        format!("{bits:X}h").into()
    }
}

fn load_bits(reset_value: &[u8], byte_order: ByteOrder, mut bit_range: Range<u32>) -> u64 {
    let mut val = 0;

    while bit_range.end > bit_range.start {
        bit_range.end -= 1;
        let bit = bit_range.end;

        let byte_index = match byte_order {
            ByteOrder::LE => bit / 8,
            ByteOrder::BE => reset_value.len() as u32 - 1 - bit / 8,
        };
        let bit_index = bit % 8;

        let bit_value = reset_value[byte_index as usize] & (1 << bit_index);

        if bit_value != 0 {
            val |= 1 << (bit - bit_range.start);
        }
    }
    val
}

fn description_to_html(description: &str) -> String {
    description.lines().join("<br/>")
}
