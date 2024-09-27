use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};

use crate::{
    lir::{Field, FieldConversionMethod, FieldSet},
    mir::{Access, BitOrder, ByteOrder},
};

pub fn generate_field_set(value: &FieldSet) -> TokenStream {
    let FieldSet {
        cfg_attr,
        doc_attr,
        name,
        byte_order,
        bit_order,
        size_bits,
        reset_value,
        fields,
    } = value;

    if *size_bits == 0 {
        // No need to generate this. All uses are covered with the unit type
        return TokenStream::new();
    }

    let size_bytes = Literal::u32_unsuffixed(size_bits.div_ceil(8));
    let bit_order = match bit_order {
        BitOrder::LSB0 => format_ident!("Lsb0"),
        BitOrder::MSB0 => format_ident!("Msb0"),
    };

    let read_functions = fields.iter().map(get_read_function);
    let write_functions = fields.iter().map(get_write_function);

    let from_impl = {
        let be_reverse = match byte_order {
            ByteOrder::LE => quote! {},
            ByteOrder::BE => quote! {
                val[..].reverse();
            },
        };

        quote! {
            #cfg_attr
            impl From<[u8; #size_bytes]> for #name {
                fn from(mut val: [u8; #size_bytes]) -> Self {
                    #be_reverse
                    Self {
                        bits: ::device_driver::bitvec::array::BitArray::new(val),
                    }
                }
            }
        }
    };

    let into_impl = {
        let be_reverse = match byte_order {
            ByteOrder::LE => quote! {},
            ByteOrder::BE => quote! {
                val[..].reverse();
            },
        };

        quote! {
            #cfg_attr
            impl From<#name> for [u8; #size_bytes] {
                fn from(val: #name) -> Self {
                    let mut val = val.bits.into_inner();
                    #be_reverse
                    val
                }
            }
        }
    };

    let debug_impl = {
        let debug_field_calls = fields.iter().map(|f| {
            let name = &f.name;
            let name_string = name.to_string();
            quote! {.field(#name_string, &self.#name()) }
        });

        let name_string = name.to_string();
        quote! {
            #cfg_attr
            impl core::fmt::Debug for #name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                    f.debug_struct(#name_string)
                        #(#debug_field_calls)*
                        .finish()
                }
            }
        }
    };

    // TODO:
    // - Add defmt impl

    quote! {
        #doc_attr
        #cfg_attr
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct #name {
            /// The internal bits. Always LE format
            bits: ::device_driver::bitvec::array::BitArray<[u8; #size_bytes], ::device_driver::bitvec::order::#bit_order>,
        }

        #cfg_attr
        impl ::device_driver::FieldSet for #name {
            type BUFFER = [u8; #size_bytes];

            fn new_with_default() -> Self {
                Self::new()
            }

            fn new_with_zero() -> Self {
                Self::new_zero()
            }
        }

        #cfg_attr
        impl #name {
            /// Create a new instance, loaded with the default value (if any)
            pub const fn new() -> Self {
                use ::device_driver::bitvec::array::BitArray;
                Self {
                    bits: BitArray { data: [#(#reset_value),*], ..BitArray::ZERO },
                }
            }

            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {
                use ::device_driver::bitvec::array::BitArray;
                Self {
                    bits: BitArray::ZERO,
                }
            }

            #(#read_functions)*

            #(#write_functions)*
        }

        #from_impl
        #into_impl
        #debug_impl

        #cfg_attr
        impl core::ops::BitAnd for #name {
            type Output = Self;

            fn bitand(self, rhs: Self) -> Self::Output {
                Self {
                    bits: self.bits & rhs.bits
                }
            }
        }

        #cfg_attr
        impl core::ops::BitAndAssign for #name {
            fn bitand_assign(&mut self, rhs: Self) {
                self.bits &= rhs.bits;
            }
        }

        #cfg_attr
        impl core::ops::BitOr for #name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self {
                    bits: self.bits | rhs.bits
                }
            }
        }

        #cfg_attr
        impl core::ops::BitOrAssign for #name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.bits |= rhs.bits;
            }
        }

        #cfg_attr
        impl core::ops::BitXor for #name {
            type Output = Self;

            fn bitxor(self, rhs: Self) -> Self::Output {
                Self {
                    bits: self.bits ^ rhs.bits
                }
            }
        }

        #cfg_attr
        impl core::ops::BitXorAssign for #name {
            fn bitxor_assign(&mut self, rhs: Self) {
                self.bits ^= rhs.bits;
            }
        }

        #cfg_attr
        impl core::ops::Not for #name {
            type Output = Self;

            fn not(self) -> Self::Output {
                Self {
                    bits: !self.bits
                }
            }
        }
    }
}

fn get_read_function(field: &Field) -> TokenStream {
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
        return TokenStream::new();
    }

    let return_type = match conversion_method {
        FieldConversionMethod::None => base_type.to_token_stream(),
        FieldConversionMethod::Into(conversion_type)
        | FieldConversionMethod::UnsafeInto(conversion_type) => conversion_type.to_token_stream(),
        FieldConversionMethod::TryInto(conversion_type) => {
            quote! { Result<#conversion_type, <#conversion_type as TryFrom<#base_type>>::Error> }
        }
        FieldConversionMethod::Bool => format_ident!("bool").into_token_stream(),
    };

    let start_bit = &address.start;
    let end_bit = &address.end;

    let conversion = match conversion_method {
        FieldConversionMethod::None => quote! { raw },
        FieldConversionMethod::Into(_) => quote! { raw.into() },
        FieldConversionMethod::UnsafeInto(_) => {
            quote! { unsafe { raw.try_into().unwrap_unchecked() } }
        }
        FieldConversionMethod::TryInto(_) => quote! { raw.try_into() },
        FieldConversionMethod::Bool => quote! { raw > 0 },
    };

    let function_description = format!("Read the `{name}` field of the register.");

    quote! {
        #[doc = #function_description]
        #[doc = ""]
        #doc_attr
        #cfg_attr
        pub fn #name(&self) -> #return_type {
            use ::device_driver::bitvec::field::BitField;
            let raw = self.bits[#start_bit..#end_bit].load_le::<#base_type>();
            #conversion
        }
    }
}

fn get_write_function(field: &Field) -> TokenStream {
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
        return TokenStream::new();
    }

    let input_type = match conversion_method {
        FieldConversionMethod::None => &base_type.to_token_stream(),
        FieldConversionMethod::Into(conversion_type)
        | FieldConversionMethod::UnsafeInto(conversion_type)
        | FieldConversionMethod::TryInto(conversion_type) => conversion_type,
        FieldConversionMethod::Bool => &quote! { bool },
    };

    let start_bit = &address.start;
    let end_bit = &address.end;

    let conversion = match conversion_method {
        FieldConversionMethod::None => quote! { value },
        FieldConversionMethod::Bool => quote! { value as _ },
        _ => quote! { value.into() },
    };

    let function_description = format!("Write the `{name}` field of the register.");
    let function_name = format_ident!("set_{name}");

    quote! {
        #[doc = #function_description]
        #[doc = ""]
        #doc_attr
        #cfg_attr
        pub fn #function_name(&mut self, value: #input_type) {
            use ::device_driver::bitvec::field::BitField;
            let raw = #conversion;
            self.bits[#start_bit..#end_bit].store_le::<#base_type>(raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use proc_macro2::Literal;

    #[test]
    fn field_set_correct() {
        let output = generate_field_set(&FieldSet {
            cfg_attr: quote! { #[cfg(windows)] },
            doc_attr: quote! { #[doc = "Hiya!"] },
            name: format_ident!("MyRegister"),
            byte_order: ByteOrder::BE,
            bit_order: BitOrder::LSB0,
            size_bits: 20,
            reset_value: vec![1, 2, 3],
            fields: vec![
                Field {
                    cfg_attr: quote! { #[cfg(linux)] },
                    doc_attr: quote! { #[doc = "Hiya again!"] },
                    name: format_ident!("my_field"),
                    address: Literal::u64_unsuffixed(0)..Literal::u64_unsuffixed(4),
                    base_type: format_ident!("u8"),
                    conversion_method: FieldConversionMethod::UnsafeInto(quote! { FieldEnum }),
                    access: Access::RW,
                },
                Field {
                    cfg_attr: quote! {},
                    doc_attr: quote! {},
                    name: format_ident!("my_field2"),
                    address: Literal::u64_unsuffixed(4)..Literal::u64_unsuffixed(16),
                    base_type: format_ident!("i16"),
                    conversion_method: FieldConversionMethod::None,
                    access: Access::WO,
                },
            ],
        });

        pretty_assertions::assert_eq!(
            prettyplease::unparse(&syn::parse2(output).unwrap()),
            indoc! {"
            ///Hiya!
            #[cfg(windows)]
            #[derive(Copy, Clone, Eq, PartialEq)]
            pub struct MyRegister {
                /// The internal bits. Always LE format
                bits: ::device_driver::bitvec::array::BitArray<
                    [u8; 3],
                    ::device_driver::bitvec::order::Lsb0,
                >,
            }
            #[cfg(windows)]
            impl ::device_driver::FieldSet for MyRegister {
                type BUFFER = [u8; 3];
                fn new_with_default() -> Self {
                    Self::new()
                }
                fn new_with_zero() -> Self {
                    Self::new_zero()
                }
            }
            #[cfg(windows)]
            impl MyRegister {
                /// Create a new instance, loaded with the default value (if any)
                pub const fn new() -> Self {
                    use ::device_driver::bitvec::array::BitArray;
                    Self {
                        bits: BitArray {
                            data: [1u8, 2u8, 3u8],
                            ..BitArray::ZERO
                        },
                    }
                }
                /// Create a new instance, loaded with all zeroes
                pub const fn new_zero() -> Self {
                    use ::device_driver::bitvec::array::BitArray;
                    Self { bits: BitArray::ZERO }
                }
                ///Read the `my_field` field of the register.
                ///
                ///Hiya again!
                #[cfg(linux)]
                pub fn my_field(&self) -> FieldEnum {
                    use ::device_driver::bitvec::field::BitField;
                    let raw = self.bits[0..4].load_le::<u8>();
                    unsafe { raw.try_into().unwrap_unchecked() }
                }
                ///Write the `my_field` field of the register.
                ///
                ///Hiya again!
                #[cfg(linux)]
                pub fn set_my_field(&mut self, value: FieldEnum) {
                    use ::device_driver::bitvec::field::BitField;
                    let raw = value.into();
                    self.bits[0..4].store_le::<u8>(raw);
                }
                ///Write the `my_field2` field of the register.
                ///
                pub fn set_my_field2(&mut self, value: i16) {
                    use ::device_driver::bitvec::field::BitField;
                    let raw = value;
                    self.bits[4..16].store_le::<i16>(raw);
                }
            }
            #[cfg(windows)]
            impl From<[u8; 3]> for MyRegister {
                fn from(mut val: [u8; 3]) -> Self {
                    val[..].reverse();
                    Self {
                        bits: ::device_driver::bitvec::array::BitArray::new(val),
                    }
                }
            }
            #[cfg(windows)]
            impl From<MyRegister> for [u8; 3] {
                fn from(val: MyRegister) -> Self {
                    let mut val = val.bits.into_inner();
                    val[..].reverse();
                    val
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
            impl core::ops::BitAnd for MyRegister {
                type Output = Self;
                fn bitand(self, rhs: Self) -> Self::Output {
                    Self { bits: self.bits & rhs.bits }
                }
            }
            #[cfg(windows)]
            impl core::ops::BitAndAssign for MyRegister {
                fn bitand_assign(&mut self, rhs: Self) {
                    self.bits &= rhs.bits;
                }
            }
            #[cfg(windows)]
            impl core::ops::BitOr for MyRegister {
                type Output = Self;
                fn bitor(self, rhs: Self) -> Self::Output {
                    Self { bits: self.bits | rhs.bits }
                }
            }
            #[cfg(windows)]
            impl core::ops::BitOrAssign for MyRegister {
                fn bitor_assign(&mut self, rhs: Self) {
                    self.bits |= rhs.bits;
                }
            }
            #[cfg(windows)]
            impl core::ops::BitXor for MyRegister {
                type Output = Self;
                fn bitxor(self, rhs: Self) -> Self::Output {
                    Self { bits: self.bits ^ rhs.bits }
                }
            }
            #[cfg(windows)]
            impl core::ops::BitXorAssign for MyRegister {
                fn bitxor_assign(&mut self, rhs: Self) {
                    self.bits ^= rhs.bits;
                }
            }
            #[cfg(windows)]
            impl core::ops::Not for MyRegister {
                type Output = Self;
                fn not(self) -> Self::Output {
                    Self { bits: !self.bits }
                }
            }
            "}
        )
    }
}
