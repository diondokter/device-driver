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

    let all_mode = quote! { ::device_driver::All };

    let read_trait = quote! { ::device_driver::Read };
    let write_trait = quote! { ::device_driver::Write };
    let clear_trait = quote! { ::device_driver::Clear };

    let read_functions = fields.iter().map(get_read_function);

    let from_impl = {
        let be_reverse = match byte_order {
            ByteOrder::LE => quote! {},
            ByteOrder::BE => quote! {
                val[..].reverse();
            },
        };

        quote! {
            impl From<[u8; #size_bytes]> for #name<#all_mode> {
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
            impl From<#name<#all_mode>> for [u8; #size_bytes] {
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
        pub struct #name<Mode> {
            /// The internal bits. Always LE format
            bits: ::device_driver::bitvec::array::BitArray<[u8; #size_bytes], ::device_driver::bitvec::order::#bit_order>,
            _phantom: core::marker::PhantomData<Mode>,
        }

        impl #name<#all_mode> {
            /// Create a new instance, loaded with the default value (if any)
            pub const fn new() -> Self {
                Self {
                    bits: ::device_driver::bitvec::array::BitArray::new([#(#reset_value),*]),
                    _phantom: core::marker::PhantomData,
                }
            }

            /// Create a new instance, loaded all 0's
            pub const fn new_zero() -> Self {
                Self {
                    bits: ::device_driver::bitvec::array::BitArray::new([0; #size_bytes]),
                    _phantom: core::marker::PhantomData,
                }
            }
        }

        impl<Mode> #name<Mode> {
            pub fn into_mode<M>(self) -> #name<M> {
                Self {
                    bits: self.bits,
                    _phantom: core::marker::PhantomData,
                }
            }
        }

        impl<R: #read_trait> #name<R> {
            #(#read_functions)*
        }

        impl<W: #write_trait> #name<W> {
            #(#write_functions)*
        }

        impl<C: #clear_trait> #name<C> {
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
            fields: vec![Field {
                cfg_attr: quote! { #[cfg(linux)] },
                doc_attr: quote! { #[doc = "Hiya again!"] },
                name: format_ident!("my_field"),
                address: Literal::u64_unsuffixed(0)..Literal::u64_unsuffixed(4),
                base_type: format_ident!("u8"),
                conversion_method: FieldConversionMethod::UnsafeInto(format_ident!("FieldEnum")),
                access: Access::RW,
            }],
        });

        pretty_assertions::assert_eq!(
            prettyplease::unparse(&syn::parse2(output).unwrap()),
            indoc! {"
            #[cfg(windows)]
            ///Hiya!
            #[derive(Copy, Clone, Eq, PartialEq)]
            pub struct MyRegister {
                /// The internal bits. Always LE format
                bits: ::device_driver::bitvec::array::BitArray<
                    [u8; 3usize],
                    ::device_driver::bitvec::order::Lsb0,
                >,
            }
            impl From<[u8; 3usize]> for MyRegister<::device_driver::All> {
                fn from(mut val: [u8; 3usize]) -> Self {
                    val[..].reverse();
                    Self {
                        bits: ::device_driver::bitvec::array::BitArray::new(val),
                    }
                }
            }
            impl From<MyRegister<::device_driver::All>> for [u8; 3usize] {
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
