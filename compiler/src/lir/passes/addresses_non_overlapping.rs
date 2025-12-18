use convert_case::{Case, Casing};
use miette::bail;

use crate::lir::{Block, BlockMethodType, Device, Driver, Repeat};

pub fn run_pass(driver: &mut Driver) -> miette::Result<()> {
    for device in driver.devices.iter() {
        let root_block = device
            .blocks
            .iter()
            .find(|b| b.root)
            .expect("There's always a root block");

        let claimed_addresses = get_block_claimed_addresses(driver, device, root_block, 0, &[]);

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
    }

    Ok(())
}

fn get_block_claimed_addresses(
    driver: &Driver,
    device: &Device,
    block: &Block,
    current_address_offset: i128,
    name_stack: &[String],
) -> Vec<ClaimedAddress> {
    let mut claimed_addresses = Vec::new();

    for method in &block.methods {
        let current_address_offset = current_address_offset + method.address;

        let (repeat_addresses, repeat): (Vec<i128>, bool) = match &method.repeat {
            Repeat::None => ((0..1).collect(), false),
            Repeat::Count { count, stride } => {
                ((0..*count).map(|i| i as i128 * stride).collect(), true)
            }
            Repeat::Enum {
                enum_name,
                enum_variants: _,
                stride,
            } => (
                driver
                    .enums
                    .iter()
                    .find(|e| e.name == *enum_name)
                    .expect("Enum existence checked in mir pass")
                    .variants
                    .iter()
                    .map(|variant| variant.discriminant * stride)
                    .collect(),
                true,
            ),
        };

        let claimed_address_type = match &method.method_type {
            BlockMethodType::Block { name } => {
                let sub_block = device
                    .blocks
                    .iter()
                    .find(|b| b.name == *name)
                    .expect("There's always a root block");

                for (i, repeat_address) in repeat_addresses.iter().enumerate() {
                    let next_name_stack = name_stack
                        .iter()
                        .cloned()
                        .chain([format!(
                            "{} (index: {i})",
                            name.to_string().to_case(Case::Pascal)
                        )])
                        .collect::<Vec<_>>();

                    claimed_addresses.extend(get_block_claimed_addresses(
                        driver,
                        device,
                        sub_block,
                        current_address_offset + repeat_address,
                        &next_name_stack,
                    ));
                }
                continue;
            }
            BlockMethodType::Register { .. } => ClaimedAddressType::Register,
            BlockMethodType::Command { .. } => ClaimedAddressType::Command,
            BlockMethodType::Buffer { .. } => ClaimedAddressType::Buffer,
        };

        for (i, repeat_address) in repeat_addresses.iter().enumerate() {
            use itertools::Itertools;

            claimed_addresses.push(ClaimedAddress {
                name: name_stack
                    .iter()
                    .cloned()
                    .chain([method.name.to_string().to_case(Case::Pascal)])
                    .join("::"),
                repeat_index: repeat.then_some(i as _),
                address: current_address_offset + repeat_address,
                allow_overlap: method.allow_address_overlap,
                address_type: claimed_address_type,
            });
        }
    }

    claimed_addresses
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ClaimedAddressType {
    Register,
    Command,
    Buffer,
}

struct ClaimedAddress {
    name: String,
    repeat_index: Option<i128>,
    address: i128,
    allow_overlap: bool,
    address_type: ClaimedAddressType,
}

#[cfg(test)]
mod tests {
    use crate::lir::BlockMethod;

    use super::*;
    use indoc::indoc;

    #[test]
    fn deep_overlap_detected() {
        let mut driver = Driver {
            devices: vec![Device {
                internal_address_type: crate::mir::Integer::U8,
                blocks: vec![
                    Block {
                        description: Default::default(),
                        root: true,
                        name: "Root".to_string(),
                        methods: vec![
                            BlockMethod {
                                description: Default::default(),
                                name: "second_block".to_string(),
                                address: 10,
                                allow_address_overlap: false,
                                repeat: Repeat::Count {
                                    count: 10,
                                    stride: 10,
                                },
                                method_type: BlockMethodType::Block {
                                    name: "SecondBlock".to_string(),
                                },
                            },
                            BlockMethod {
                                description: Default::default(),
                                name: "register0".to_string(),
                                address: 75,
                                allow_address_overlap: false,
                                repeat: Repeat::Count {
                                    count: 2,
                                    stride: 5,
                                },
                                method_type: BlockMethodType::Register {
                                    field_set_name: "bla".to_string(),
                                    access: crate::mir::Access::RW,
                                    address_type: crate::mir::Integer::U8,
                                    reset_value: None,
                                },
                            },
                        ],
                    },
                    Block {
                        description: Default::default(),
                        root: true,
                        name: "SecondBlock".to_string(),
                        methods: vec![BlockMethod {
                            description: Default::default(),
                            name: "register1".to_string(),
                            address: 0,
                            allow_address_overlap: false,
                            repeat: Repeat::None,
                            method_type: BlockMethodType::Register {
                                field_set_name: "bla".to_string(),
                                access: crate::mir::Access::RW,
                                address_type: crate::mir::Integer::U8,
                                reset_value: None,
                            },
                        }],
                    },
                ],
                defmt_feature: None,
            }],
            field_sets: Vec::new(),
            enums: Vec::new(),
        };

        pretty_assertions::assert_eq!(
            run_pass(&mut driver).unwrap_err().to_string(),
            indoc!(
                "Objects \"SecondBlock (index: 7)::Register1\" and \"Register0 (index: 1)\" use the same address (80). If this is intended, then allow address overlap on both objects."
            )
        );
    }
}
