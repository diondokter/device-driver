use convert_case::Casing;
use itertools::Itertools;

use crate::{
    lir::{Field, FieldConversionMethod, FieldSet},
    mir::{Access, BitOrder, ByteOrder},
};

pub fn generate_field_set(value: &FieldSet, defmt_feature: Option<&str>) -> String {
    let FieldSet {
        cfg_attr,
        doc_attr,
        name,
        byte_order,
        bit_order,
        size_bits,
        reset_value,
        ref_reset_overrides,
        fields,
    } = value;

    if *size_bits == 0 {
        // No need to generate this. All uses are covered with the unit type
        return String::new();
    }

    let size_bytes = size_bits.div_ceil(8);

    let read_functions = fields
        .iter()
        .map(|field| get_read_function(field, *byte_order, *bit_order))
        .join("\n");
    let write_functions = fields
        .iter()
        .map(|field| get_write_function(field, *byte_order, *bit_order))
        .join("\n");

    let from_impl = {
        format!(
            "
            {cfg_attr}
            impl From<[u8; {size_bytes}]> for {name} {{
                fn from(bits: [u8; {size_bytes}]) -> Self {{
                    Self {{
                        bits,
                }}
            }}
        }}
        "
        )
    };

    let into_impl = {
        format!(
            "
            {cfg_attr}
            impl From<{name}> for [u8; {size_bytes}] {{
                fn from(val: {name}) -> Self {{
                    val.bits
                }}
            }}
        "
        )
    };

    let debug_impl = {
        let debug_field_calls = fields
            .iter()
            .map(|f| {
                let name = &f.name;
                format!(".field(\"{name}\", &self.{name}())")
            })
            .join("\n");

        format!(
            "
            {cfg_attr}
            impl core::fmt::Debug for {name} {{
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {{
                    f.debug_struct(\"{name}\")
                        {debug_field_calls}
                        .finish()
                }}
            }}
        "
        )
    };

    let defmt_impl = match defmt_feature {
        Some(feature_name) => {
            let fields_format_string = fields
                .iter()
                .map(|f| {
                    let defmt_type_hint = match f.conversion_method {
                        FieldConversionMethod::None => {
                            let base_type = &f.base_type;
                            format!("={base_type}")
                        }
                        FieldConversionMethod::Bool => "=bool".into(),
                        _ => String::new(),
                    };

                    format!("{}: {{{}}}", f.name, defmt_type_hint)
                })
                .join(", ");

            let type_format_string = format!("{} {{{{ {} }}}}", name, fields_format_string);

            let field_calls = fields
                .iter()
                .map(|f| {
                    let name = &f.name;
                    format!("self.{name}()")
                })
                .join(",\n");

            let separator = if fields.is_empty() { "" } else { "," };

            format!(
                "
                {cfg_attr}
                #[cfg(feature = \"{feature_name}\")]
                impl defmt::Format for {name} {{
                    fn format(&self, f: defmt::Formatter) {{
                        defmt::write!(
                            f,
                            \"{type_format_string}\" {separator}
                            {field_calls}
                        )
                    }}
                }}
            "
            )
        }
        None => String::new(),
    };

    let ref_value_constructors = ref_reset_overrides
        .iter()
        .map(|(ref_name, reset_value)| {
            let name = format!("new_as_{}", ref_name.to_case(convert_case::Case::Snake));
            let docs: String = format!(
                "Create a new instance, loaded with the reset value of the `{ref_name}` ref"
            );

            let reset_value = reset_value.iter().join(",");

            format!(
                "
                    #[doc = \"{docs}\"]
                    pub const fn {name}() -> Self {{
                        Self {{
                            bits: [{reset_value}],
                        }}
                    }}
                "
            )
        })
        .join("\n");

    let reset_value = reset_value.iter().join(",");

    format!(
        "
        {doc_attr}
        {cfg_attr}
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct {name} {{
            /// The internal bits
            bits: [u8; {size_bytes}],
        }}

        {cfg_attr}
        impl ::device_driver::FieldSet for {name} {{
            const SIZE_BITS: u32 = {size_bits};

            fn new_with_zero() -> Self {{
                Self::new_zero()
            }}

            fn get_inner_buffer(&self) -> &[u8] {{
                &self.bits
            }}
            fn get_inner_buffer_mut(&mut self) -> &mut [u8] {{
                &mut self.bits
            }}
        }}

        {cfg_attr}
        impl {name} {{
            /// Create a new instance, loaded with the reset value (if any)
            pub const fn new() -> Self {{
                Self {{
                    bits: [{reset_value}],
                }}
            }}

            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {{
                Self {{
                    bits: [0; {size_bytes}],
                }}
            }}

            {ref_value_constructors}

            {read_functions}

            {write_functions}
        }}

        {from_impl}
        {into_impl}
        {debug_impl}
        {defmt_impl}

        {cfg_attr}
        impl core::ops::BitAnd for {name} {{
            type Output = Self;

            fn bitand(mut self, rhs: Self) -> Self::Output {{
                self &= rhs;
                self
            }}
        }}

        {cfg_attr}
        impl core::ops::BitAndAssign for {name} {{
            fn bitand_assign(&mut self, rhs: Self) {{
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {{
                    *l &= *r;
                }}
            }}
        }}

        {cfg_attr}
        impl core::ops::BitOr for {name} {{
            type Output = Self;

            fn bitor(mut self, rhs: Self) -> Self::Output {{
                self |= rhs;
                self
            }}
        }}

        {cfg_attr}
        impl core::ops::BitOrAssign for {name} {{
            fn bitor_assign(&mut self, rhs: Self) {{
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {{
                    *l |= *r;
                }}
            }}
        }}

        {cfg_attr}
        impl core::ops::BitXor for {name} {{
            type Output = Self;

            fn bitxor(mut self, rhs: Self) -> Self::Output {{
                self ^= rhs;
                self
            }}
        }}

        {cfg_attr}
        impl core::ops::BitXorAssign for {name} {{
            fn bitxor_assign(&mut self, rhs: Self) {{
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {{
                    *l ^= *r;
                }}
            }}
        }}

        {cfg_attr}
        impl core::ops::Not for {name} {{
            type Output = Self;

            fn not(mut self) -> Self::Output {{
                for val in self.bits.iter_mut() {{
                    *val = !*val;
                }}
                self
            }}
        }}
    "
    )
}

fn get_read_function(field: &Field, byte_order: ByteOrder, bit_order: BitOrder) -> String {
    let Field {
        cfg_attr,
        doc_attr,
        name,
        address,
        base_type,
        conversion_method,
        access,
    } = field;

    if !matches!(access, Access::RW | Access::RO) {
        return String::new();
    }

    let load_function = match (byte_order, bit_order) {
        (ByteOrder::LE, BitOrder::LSB0) => {
            format!("::device_driver::ops::load_lsb0::<{base_type}, ::device_driver::ops::LE>")
        }
        (ByteOrder::LE, BitOrder::MSB0) => {
            format!("::device_driver::ops::load_msb0::<{base_type}, ::device_driver::ops::LE>")
        }
        (ByteOrder::BE, BitOrder::LSB0) => {
            format!("::device_driver::ops::load_lsb0::<{base_type}, ::device_driver::ops::BE>")
        }
        (ByteOrder::BE, BitOrder::MSB0) => {
            format!("::device_driver::ops::load_msb0::<{base_type}, ::device_driver::ops::BE>")
        }
    };

    let super_token = get_super_token(conversion_method);

    let return_type = match conversion_method {
        FieldConversionMethod::None => base_type.clone(),
        FieldConversionMethod::Into(conversion_type)
        | FieldConversionMethod::UnsafeInto(conversion_type) => {
            format!("{super_token} {conversion_type}")
        }
        FieldConversionMethod::TryInto(conversion_type) => {
            format!(
                "Result<{super_token} {conversion_type}, <{super_token} {conversion_type} as TryFrom<{base_type}>>::Error>"
            )
        }
        FieldConversionMethod::Bool => "bool".to_string(),
    };

    let start_bit = &address.start;
    let end_bit = &address.end;

    let conversion = match conversion_method {
        FieldConversionMethod::None => "raw",
        FieldConversionMethod::Into(_) => "raw.into()",
        FieldConversionMethod::UnsafeInto(_) => "unsafe { raw.try_into().unwrap_unchecked() }",
        FieldConversionMethod::TryInto(_) => "raw.try_into()",
        FieldConversionMethod::Bool => "raw > 0",
    };

    let function_description = format!("Read the `{name}` field of the register.");

    format!(
        "
        #[doc = \"{function_description}\"]
        #[doc = \"\"]
        {doc_attr}
        {cfg_attr}
        pub fn {name}(&self) -> {return_type} {{
            let raw = unsafe {{ {load_function}(&self.bits, {start_bit}, {end_bit}) }};
            {conversion}
        }}
    "
    )
}

fn get_write_function(field: &Field, byte_order: ByteOrder, bit_order: BitOrder) -> String {
    let Field {
        cfg_attr,
        doc_attr,
        name,
        address,
        base_type,
        conversion_method,
        access,
    } = field;

    if !matches!(access, Access::RW | Access::WO) {
        return String::new();
    }

    let store_function = match (byte_order, bit_order) {
        (ByteOrder::LE, BitOrder::LSB0) => {
            format!("::device_driver::ops::store_lsb0::<{base_type}, ::device_driver::ops::LE>")
        }
        (ByteOrder::LE, BitOrder::MSB0) => {
            format!("::device_driver::ops::store_msb0::<{base_type}, ::device_driver::ops::LE>")
        }
        (ByteOrder::BE, BitOrder::LSB0) => {
            format!("::device_driver::ops::store_lsb0::<{base_type}, ::device_driver::ops::BE>")
        }
        (ByteOrder::BE, BitOrder::MSB0) => {
            format!("::device_driver::ops::store_msb0::<{base_type}, ::device_driver::ops::BE>")
        }
    };

    let super_token = get_super_token(conversion_method);

    let input_type = match conversion_method {
        FieldConversionMethod::None => base_type,
        FieldConversionMethod::Into(conversion_type)
        | FieldConversionMethod::UnsafeInto(conversion_type)
        | FieldConversionMethod::TryInto(conversion_type) => conversion_type,
        FieldConversionMethod::Bool => "bool",
    };

    let start_bit = &address.start;
    let end_bit = &address.end;

    let conversion = match conversion_method {
        FieldConversionMethod::None => "value",
        FieldConversionMethod::Bool => "value as _",
        _ => "value.into()",
    };

    let function_description = format!("Write the `{name}` field of the register.");
    let function_name = format!("set_{name}");

    format!(
        "
        #[doc = \"{function_description}\"]
        #[doc = \"\"]
        {doc_attr}
        {cfg_attr}
        pub fn {function_name}(&mut self, value: {super_token} {input_type}) {{
            let raw = {conversion};
            unsafe {{ {store_function}(raw, {start_bit}, {end_bit}, &mut self.bits) }};
        }}
    "
    )
}

fn get_super_token(conversion_method: &FieldConversionMethod) -> &'static str {
    match conversion_method.conversion_type() {
        Some(ct) if !ct.trim_start().starts_with("::") && !ct.trim_start().starts_with("crate") => {
            "super::"
        }
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use quote::quote;

    #[test]
    fn field_set_correct() {
        let output = generate_field_set(
            &FieldSet {
                cfg_attr: quote! { #[cfg(windows)] },
                doc_attr: quote! { #[doc = "Hiya!"] },
                name: "MyRegister".to_string(),
                byte_order: ByteOrder::BE,
                bit_order: BitOrder::LSB0,
                size_bits: 20,
                reset_value: vec![1, 2, 3],
                ref_reset_overrides: vec![("MyRef".into(), vec![0, 1, 2])],
                fields: vec![
                    Field {
                        cfg_attr: quote! { #[cfg(linux)] },
                        doc_attr: quote! { #[doc = "Hiya again!"] },
                        name: "my_field".to_string(),
                        address: 0..4,
                        base_type: "u8".to_string(),
                        conversion_method: FieldConversionMethod::UnsafeInto(
                            "FieldEnum".to_string(),
                        ),
                        access: Access::RW,
                    },
                    Field {
                        cfg_attr: quote! {},
                        doc_attr: quote! {},
                        name: "my_field2".to_string(),
                        address: 4..16,
                        base_type: "i16".to_string(),
                        conversion_method: FieldConversionMethod::None,
                        access: Access::WO,
                    },
                ],
            },
            Some("defmt-03"),
        );

        pretty_assertions::assert_eq!(
            prettyplease::unparse(&syn::parse_str(&output).unwrap()),
            indoc! {"
            ///Hiya!
            #[cfg(windows)]
            #[derive(Copy, Clone, Eq, PartialEq)]
            pub struct MyRegister {
                /// The internal bits
                bits: [u8; 3],
            }
            #[cfg(windows)]
            impl ::device_driver::FieldSet for MyRegister {
                const SIZE_BITS: u32 = 20;
                fn new_with_zero() -> Self {
                    Self::new_zero()
                }
                fn get_inner_buffer(&self) -> &[u8] {
                    &self.bits
                }
                fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
                    &mut self.bits
                }
            }
            #[cfg(windows)]
            impl MyRegister {
                /// Create a new instance, loaded with the reset value (if any)
                pub const fn new() -> Self {
                    Self { bits: [1, 2, 3] }
                }
                /// Create a new instance, loaded with all zeroes
                pub const fn new_zero() -> Self {
                    Self { bits: [0; 3] }
                }
                ///Create a new instance, loaded with the reset value of the `MyRef` ref
                pub const fn new_as_my_ref() -> Self {
                    Self { bits: [0, 1, 2] }
                }
                ///Read the `my_field` field of the register.
                ///
                ///Hiya again!
                #[cfg(linux)]
                pub fn my_field(&self) -> super::FieldEnum {
                    let raw = unsafe {
                        ::device_driver::ops::load_lsb0::<
                            u8,
                            ::device_driver::ops::BE,
                        >(&self.bits, 0, 4)
                    };
                    unsafe { raw.try_into().unwrap_unchecked() }
                }
                ///Write the `my_field` field of the register.
                ///
                ///Hiya again!
                #[cfg(linux)]
                pub fn set_my_field(&mut self, value: super::FieldEnum) {
                    let raw = value.into();
                    unsafe {
                        ::device_driver::ops::store_lsb0::<
                            u8,
                            ::device_driver::ops::BE,
                        >(raw, 0, 4, &mut self.bits)
                    };
                }
                ///Write the `my_field2` field of the register.
                ///
                pub fn set_my_field2(&mut self, value: i16) {
                    let raw = value;
                    unsafe {
                        ::device_driver::ops::store_lsb0::<
                            i16,
                            ::device_driver::ops::BE,
                        >(raw, 4, 16, &mut self.bits)
                    };
                }
            }
            #[cfg(windows)]
            impl From<[u8; 3]> for MyRegister {
                fn from(bits: [u8; 3]) -> Self {
                    Self { bits }
                }
            }
            #[cfg(windows)]
            impl From<MyRegister> for [u8; 3] {
                fn from(val: MyRegister) -> Self {
                    val.bits
                }
            }
            #[cfg(windows)]
            impl core::fmt::Debug for MyRegister {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                    f.debug_struct(\"MyRegister\")
                        .field(\"my_field\", &self.my_field())
                        .field(\"my_field2\", &self.my_field2())
                        .finish()
                }
            }
            #[cfg(windows)]
            #[cfg(feature = \"defmt-03\")]
            impl defmt::Format for MyRegister {
                fn format(&self, f: defmt::Formatter) {
                    defmt::write!(
                        f, \"MyRegister {{ my_field: {}, my_field2: {=i16} }}\", self.my_field(), self
                        .my_field2()
                    )
                }
            }
            #[cfg(windows)]
            impl core::ops::BitAnd for MyRegister {
                type Output = Self;
                fn bitand(mut self, rhs: Self) -> Self::Output {
                    self &= rhs;
                    self
                }
            }
            #[cfg(windows)]
            impl core::ops::BitAndAssign for MyRegister {
                fn bitand_assign(&mut self, rhs: Self) {
                    for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                        *l &= *r;
                    }
                }
            }
            #[cfg(windows)]
            impl core::ops::BitOr for MyRegister {
                type Output = Self;
                fn bitor(mut self, rhs: Self) -> Self::Output {
                    self |= rhs;
                    self
                }
            }
            #[cfg(windows)]
            impl core::ops::BitOrAssign for MyRegister {
                fn bitor_assign(&mut self, rhs: Self) {
                    for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                        *l |= *r;
                    }
                }
            }
            #[cfg(windows)]
            impl core::ops::BitXor for MyRegister {
                type Output = Self;
                fn bitxor(mut self, rhs: Self) -> Self::Output {
                    self ^= rhs;
                    self
                }
            }
            #[cfg(windows)]
            impl core::ops::BitXorAssign for MyRegister {
                fn bitxor_assign(&mut self, rhs: Self) {
                    for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                        *l ^= *r;
                    }
                }
            }
            #[cfg(windows)]
            impl core::ops::Not for MyRegister {
                type Output = Self;
                fn not(mut self) -> Self::Output {
                    for val in self.bits.iter_mut() {
                        *val = !*val;
                    }
                    self
                }
            }
            "}
        )
    }
}
