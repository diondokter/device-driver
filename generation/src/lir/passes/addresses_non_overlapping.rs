use anyhow::bail;
use convert_case::{Case, Casing};

use crate::lir::{Block, BlockMethodKind, BlockMethodType, Device};

pub fn run_pass(device: &mut Device) -> anyhow::Result<()> {
    let root_block = device
        .blocks
        .iter()
        .find(|b| b.root)
        .expect("There's always a root block");

    let claimed_addresses = get_block_claimed_addresses(device, root_block, 0, &[])?;

    for (i, claimed_address) in claimed_addresses.iter().enumerate() {
        for other_claimed_address in claimed_addresses.get(i + 1..).unwrap_or_default() {
            let types_same = claimed_address.address_type == other_claimed_address.address_type;
            let both_allow_overlap =
                claimed_address.allow_overlap && other_claimed_address.allow_overlap;
            let address_same = claimed_address.address == other_claimed_address.address;

            if address_same && types_same && !both_allow_overlap {
                let mut name0 = claimed_address.name.clone();
                if let Some(repeat_index) = claimed_address.repeat_index {
                    name0 += &format!(" (index: {repeat_index})");
                }
                let mut name1 = other_claimed_address.name.clone();
                if let Some(repeat_index) = other_claimed_address.repeat_index {
                    name1 += &format!(" (index: {repeat_index})");
                }
                bail!(
                    "Objects \"{name0}\" and \"{name1}\" use the same address ({}). If this is intended, then allow address overlap on both objects.",
                    claimed_address.address
                );
            }
        }
    }

    Ok(())
}

fn get_block_claimed_addresses(
    device: &Device,
    block: &Block,
    current_address_offset: i64,
    name_stack: &[String],
) -> anyhow::Result<Vec<ClaimedAddress>> {
    let mut claimed_adresses = Vec::new();

    for method in &block.methods {
        let current_address_offset =
            current_address_offset + method.address.to_string().parse::<i64>().unwrap();

        let (repeat_count, repeat_stride, repeat): (i64, i64, bool) = match &method.kind {
            BlockMethodKind::Normal => (1, 0, false),
            BlockMethodKind::RepeatedNumber { count, stride } => (
                count.to_string().parse().unwrap(),
                stride.to_string().parse().unwrap(),
                true,
            ),
            // We don't know the possibilities of this type, so we can't really check
            BlockMethodKind::RepeatedFromType { .. } => continue,
        };

        let claimed_address_type = match &method.method_type {
            BlockMethodType::Block { name } => {
                let sub_block = device
                    .blocks
                    .iter()
                    .find(|b| b.name == *name)
                    .expect("There's always a root block");

                for i in 0..repeat_count {
                    let next_name_stack = name_stack
                        .iter()
                        .cloned()
                        .chain([format!(
                            "{} (index: {i})",
                            name.to_string().to_case(Case::Pascal)
                        )])
                        .collect::<Vec<_>>();

                    claimed_adresses.extend(get_block_claimed_addresses(
                        device,
                        sub_block,
                        current_address_offset + i * repeat_stride,
                        &next_name_stack,
                    )?);
                }
                continue;
            }
            BlockMethodType::Register { .. } => ClaimedAddressType::Register,
            BlockMethodType::Command { .. } => ClaimedAddressType::Command,
            BlockMethodType::Buffer { .. } => ClaimedAddressType::Buffer,
        };

        for i in 0..repeat_count {
            use itertools::Itertools;

            claimed_adresses.push(ClaimedAddress {
                name: name_stack
                    .iter()
                    .cloned()
                    .chain([method.name.to_string().to_case(Case::Pascal)])
                    .join("::"),
                repeat_index: repeat.then_some(i),
                address: current_address_offset + i * repeat_stride,
                allow_overlap: method.allow_address_overlap,
                address_type: claimed_address_type,
            });
        }
    }

    Ok(claimed_adresses)
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ClaimedAddressType {
    Register,
    Command,
    Buffer,
}

struct ClaimedAddress {
    name: String,
    repeat_index: Option<i64>,
    address: i64,
    allow_overlap: bool,
    address_type: ClaimedAddressType,
}

#[cfg(test)]
mod tests {
    use crate::lir::BlockMethod;

    use super::*;
    use indoc::indoc;
    use proc_macro2::Literal;
    use quote::format_ident;

    #[test]
    fn deep_overlap_detected() {
        let mut device = Device {
            internal_address_type: format_ident!("u8"),
            register_address_type: format_ident!("u8"),
            blocks: vec![
                Block {
                    cfg_attr: Default::default(),
                    doc_attr: Default::default(),
                    root: true,
                    name: format_ident!("Root"),
                    methods: vec![
                        BlockMethod {
                            cfg_attr: Default::default(),
                            doc_attr: Default::default(),
                            name: format_ident!("second_block"),
                            address: Literal::i64_unsuffixed(10),
                            allow_address_overlap: false,
                            kind: BlockMethodKind::RepeatedNumber {
                                count: Literal::i64_unsuffixed(10),
                                stride: Literal::i64_unsuffixed(10),
                            },
                            method_type: BlockMethodType::Block {
                                name: format_ident!("SecondBlock"),
                            },
                        },
                        BlockMethod {
                            cfg_attr: Default::default(),
                            doc_attr: Default::default(),
                            name: format_ident!("register0"),
                            address: Literal::i64_unsuffixed(75),
                            allow_address_overlap: false,
                            kind: BlockMethodKind::RepeatedNumber {
                                count: Literal::i64_unsuffixed(2),
                                stride: Literal::i64_unsuffixed(5),
                            },
                            method_type: BlockMethodType::Register {
                                field_set_name: format_ident!("bla"),
                                access: crate::mir::Access::RW,
                                address_type: format_ident!("bla"),
                                reset_value_function: format_ident!("new"),
                            },
                        },
                    ],
                },
                Block {
                    cfg_attr: Default::default(),
                    doc_attr: Default::default(),
                    root: true,
                    name: format_ident!("SecondBlock"),
                    methods: vec![BlockMethod {
                        cfg_attr: Default::default(),
                        doc_attr: Default::default(),
                        name: format_ident!("register1"),
                        address: Literal::i64_unsuffixed(0),
                        allow_address_overlap: false,
                        kind: BlockMethodKind::Normal,
                        method_type: BlockMethodType::Register {
                            field_set_name: format_ident!("bla"),
                            access: crate::mir::Access::RW,
                            address_type: format_ident!("bla"),
                            reset_value_function: format_ident!("new"),
                        },
                    }],
                },
            ],
            field_sets: Vec::new(),
            enums: Vec::new(),
            defmt_feature: None,
        };

        pretty_assertions::assert_eq!(
            run_pass(&mut device).unwrap_err().to_string(),
            indoc!(
                "Objects \"SecondBlock (index: 7)::Register1\" and \"Register0 (index: 1)\" use the same address (80). If this is intended, then allow address overlap on both objects."
            )
        );
    }
}
