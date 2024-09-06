use proc_macro2::TokenStream;
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

    let size_bytes = size_bits.div_ceil(8);
    let bit_order = match bit_order {
        BitOrder::LSB0 => format_ident!("Lsb0"),
        BitOrder::MSB0 => format_ident!("Msb0"),
    };

    let read_functions = fields.iter().map(get_read_function);
    let write_functions = fields.iter().map(get_write_function);
    let clear_functions = fields.iter().map(get_clear_function);

    let from_impl = {
        let be_reverse = match byte_order {
            ByteOrder::LE => quote! {},
            ByteOrder::BE => quote! {
                val[..].reverse();
            },
        };

        quote! {
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
            impl From<#name> for [u8; #size_bytes] {
                fn from(val: #name) -> Self {
                    let mut val = val.bits.into_inner();
                    #be_reverse
                    val
                }
            }
        }
    };

    quote! {
        #doc_attr
        #cfg_attr
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct #name {
            /// The internal bits. Always LE format
            bits: ::device_driver::bitvec::array::BitArray<[u8; #size_bytes], ::device_driver::bitvec::order::#bit_order>,
        }

        impl #name {
            /// Create a new instance, loaded with the default value (if any)
            pub const fn new() -> Self {
                Self {
                    bits: ::device_driver::bitvec::array::BitArray::new([#(#reset_value),*]),
                }
            }

            /// Create a new instance, loaded all 0's
            pub const fn new_zero() -> Self {
                Self {
                    bits: ::device_driver::bitvec::array::BitArray::new([0; #size_bytes]),
                }
            }

            #(#read_functions)*

            #(#write_functions)*

            #(#clear_functions)*
        }

        #from_impl
        #into_impl
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

    if !matches!(access, Access::RW | Access::RC | Access::RO) {
        return TokenStream::new();
    }

    let return_type = match conversion_method {
        FieldConversionMethod::None => base_type.to_token_stream(),
        FieldConversionMethod::Into(conversion_type)
        | FieldConversionMethod::UnsafeInto(conversion_type) => conversion_type.to_token_stream(),
        FieldConversionMethod::TryInto(conversion_type) => {
            quote! { Result<#conversion_type, #base_type> }
        }
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
    };

    let function_description = format!("Read the {name} field of the register.");

    quote! {
        #[doc = #function_description]
        #[doc = ""]
        #doc_attr
        #cfg_attr
        fn #name(&self) -> #return_type {
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
        FieldConversionMethod::None => base_type,
        FieldConversionMethod::Into(conversion_type)
        | FieldConversionMethod::UnsafeInto(conversion_type)
        | FieldConversionMethod::TryInto(conversion_type) => conversion_type,
    };

    let start_bit = &address.start;
    let end_bit = &address.end;

    let conversion = match conversion_method {
        FieldConversionMethod::None => quote! { value },
        _ => quote! { value.into() },
    };

    let function_description = format!("Write the {name} field of the register.");
    let function_name = format_ident!("set_{name}");

    quote! {
        #[doc = #function_description]
        #[doc = ""]
        #doc_attr
        #cfg_attr
        fn #function_name(&mut self, value: #input_type) {
            let raw = #conversion;
            self.bits[#start_bit..#end_bit].store_le::<#base_type>(raw);
        }
    }
}

fn get_clear_function(field: &Field) -> TokenStream {
    let Field {
        cfg_attr,
        doc_attr,
        name,
        address,
        base_type,
        conversion_method: _,
        access,
    } = field;

    if !matches!(access, Access::RC | Access::CO) {
        return TokenStream::new();
    }

    let start_bit = &address.start;
    let end_bit = &address.end;

    let function_description =
        format!("Clear (invert from default) the {name} field of the register.");
    let function_name = format_ident!("clear_{name}");

    quote! {
        #[doc = #function_description]
        #[doc = ""]
        #doc_attr
        #cfg_attr
        fn #function_name(&mut self) {
            let default = Self::new().bits[#start_bit..#end_bit].load_le::<#base_type>();
            self.bits[#start_bit..#end_bit].store_le::<#base_type>(default ^ (0.wrapping_sub(1)));
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
                    conversion_method: FieldConversionMethod::UnsafeInto(format_ident!(
                        "FieldEnum"
                    )),
                    access: Access::RW,
                },
                Field {
                    cfg_attr: quote! {},
                    doc_attr: quote! {},
                    name: format_ident!("my_field2"),
                    address: Literal::u64_unsuffixed(4)..Literal::u64_unsuffixed(16),
                    base_type: format_ident!("i16"),
                    conversion_method: FieldConversionMethod::None,
                    access: Access::RC,
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
                    [u8; 3usize],
                    ::device_driver::bitvec::order::Lsb0,
                >,
            }
            impl MyRegister {
                /// Create a new instance, loaded with the default value (if any)
                pub const fn new() -> Self {
                    Self {
                        bits: ::device_driver::bitvec::array::BitArray::new([1u8, 2u8, 3u8]),
                    }
                }
                /// Create a new instance, loaded all 0's
                pub const fn new_zero() -> Self {
                    Self {
                        bits: ::device_driver::bitvec::array::BitArray::new([0; 3usize]),
                    }
                }
                ///Read the my_field field of the register.
                ///
                ///Hiya again!
                #[cfg(linux)]
                fn my_field(&self) -> FieldEnum {
                    let raw = self.bits[0..4].load_le::<u8>();
                    unsafe { raw.try_into().unwrap_unchecked() }
                }
                ///Read the my_field2 field of the register.
                ///
                fn my_field2(&self) -> i16 {
                    let raw = self.bits[4..16].load_le::<i16>();
                    raw
                }
                ///Write the my_field field of the register.
                ///
                ///Hiya again!
                #[cfg(linux)]
                fn set_my_field(&mut self, value: FieldEnum) {
                    let raw = value.into();
                    self.bits[0..4].store_le::<u8>(raw);
                }
                ///Clear (invert from default) the my_field2 field of the register.
                ///
                fn clear_my_field2(&mut self) {
                    let default = Self::new().bits[4..16].load_le::<i16>();
                    self.bits[4..16].store_le::<i16>(default ^ (0.wrapping_sub(1)));
                }
            }
            impl From<[u8; 3usize]> for MyRegister {
                fn from(mut val: [u8; 3usize]) -> Self {
                    val[..].reverse();
                    Self {
                        bits: ::device_driver::bitvec::array::BitArray::new(val),
                    }
                }
            }
            impl From<MyRegister> for [u8; 3usize] {
                fn from(val: MyRegister) -> Self {
                    let mut val = val.bits.into_inner();
                    val[..].reverse();
                    val
                }
            }
            "}
        )
    }
}
