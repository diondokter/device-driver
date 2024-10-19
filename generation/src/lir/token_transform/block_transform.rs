use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, ToTokens};

use crate::lir::{Block, BlockMethod, BlockMethodKind, BlockMethodType};

pub fn generate_block(value: &Block, internal_address_type: &Ident) -> TokenStream {
    let Block {
        cfg_attr,
        doc_attr,
        root,
        name,
        methods,
    } = value;

    let (generics, interface_decleration, address_param, address_specifier, interface_borrow) =
        if *root {
            (
                quote! { I },
                quote! { I },
                None,
                quote! { 0 },
                quote! { &mut self.interface },
            )
        } else {
            (
                quote! { 'i, I },
                quote! { &'i mut I },
                Some(quote! { base_address: #internal_address_type }),
                quote! { base_address },
                quote! { self.interface },
            )
        };

    let methods = methods
        .iter()
        .map(|m| generate_method(m, internal_address_type));

    let (new_hidden_if_not_root, new_access) = if *root {
        (quote! {}, quote! { pub })
    } else {
        (quote! { #[doc(hidden)] }, quote! {})
    };

    quote! {
        #doc_attr
        #cfg_attr
        #[derive(Debug)]
        pub struct #name<#generics> {
            pub(crate) interface: #interface_decleration,
            #[doc(hidden)]
            base_address: #internal_address_type,
        }

        #cfg_attr
        impl<#generics> #name<#generics> {
            /// Create a new instance of the block based on device interface
            #new_hidden_if_not_root
            #new_access const fn new(interface: #interface_decleration, #address_param) -> Self {
                Self {
                    interface,
                    base_address: #address_specifier,
                }
            }

            pub(crate) fn interface(&mut self) -> &mut I {
                #interface_borrow
            }

            #(#methods)*
        }
    }
}

fn generate_method(method: &BlockMethod, internal_address_type: &Ident) -> TokenStream {
    let BlockMethod {
        cfg_attr,
        doc_attr,
        name,
        address,
        allow_address_overlap: _,
        kind,
        method_type,
    } = method;

    let (return_type, address_conversion, default_arg) = match method_type {
        BlockMethodType::Block { name } => (quote! { #name::<'_, I> }, quote! {}, quote! {}),
        BlockMethodType::Register {
            field_set_name,
            access,
            address_type,
            reset_value_function: default_value_function_name,
        } => (
            quote! { ::device_driver::RegisterOperation::<'_, I, #address_type, #field_set_name, ::device_driver::#access>  },
            quote! { as #address_type },
            quote! { , #field_set_name::#default_value_function_name },
        ),
        BlockMethodType::Command {
            field_set_name_in,
            field_set_name_out,
            address_type,
        } => {
            let field_set_name_in = match field_set_name_in {
                Some(val) => val.to_token_stream(),
                None => quote! { () },
            };
            let field_set_name_out = match field_set_name_out {
                Some(val) => val.to_token_stream(),
                None => quote! { () },
            };
            (
                quote! { ::device_driver::CommandOperation::<'_, I, #address_type, #field_set_name_in, #field_set_name_out>  },
                quote! { as #address_type },
                quote! {},
            )
        }
        BlockMethodType::Buffer {
            access,
            address_type,
        } => (
            quote! { ::device_driver::BufferOperation::<'_, I, #address_type, ::device_driver::#access>  },
            quote! { as #address_type },
            quote! {},
        ),
    };

    let (index_param, address_calc, index_doc) = match kind {
        BlockMethodKind::Normal => (None, quote! { self.base_address + #address }, None),
        BlockMethodKind::Repeated { count, stride } => {
            let doc = format!("Valid index range: 0..{count}");

            let stride = stride.to_string().parse::<i64>().unwrap();

            let operator = if stride.is_negative() {
                quote! { - }
            } else {
                quote! { + }
            };

            let stride = Literal::u64_unsuffixed(stride.unsigned_abs());

            (
                Some(quote! { index: usize, }),
                quote! { {
                    assert!(index < #count);
                    self.base_address + #address #operator index as #internal_address_type * #stride
                } },
                Some(quote! {
                    #[doc = ""]
                    #[doc = #doc]
                }),
            )
        }
    };

    quote! {
        #doc_attr
        #index_doc
        #cfg_attr
        pub fn #name(&mut self, #index_param) -> #return_type {
            let address = #address_calc;
            #return_type::new(self.interface(), address #address_conversion #default_arg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use proc_macro2::Literal;
    use quote::format_ident;

    #[test]
    fn root_block_correct() {
        let output = generate_block(
            &Block {
                cfg_attr: quote! { #[cfg(unix)] },
                doc_attr: quote! { #[doc = "Hello!"] },
                root: true,
                name: format_ident!("RootBlock"),
                methods: vec![BlockMethod {
                    cfg_attr: quote! { #[cfg(unix)] },
                    doc_attr: quote! { #[doc = "42 is the answer"] },
                    name: format_ident!("my_register1"),
                    address: Literal::i64_unsuffixed(5),
                    allow_address_overlap: false,
                    kind: BlockMethodKind::Normal,
                    method_type: BlockMethodType::Register {
                        field_set_name: format_ident!("MyRegister"),
                        access: crate::mir::Access::RW,
                        address_type: format_ident!("u8"),
                        reset_value_function: format_ident!("new"),
                    },
                }],
            },
            &format_ident!("u8"),
        );

        pretty_assertions::assert_eq!(
            prettyplease::unparse(&syn::parse2(output).unwrap()),
            indoc! {"
                ///Hello!
                #[cfg(unix)]
                #[derive(Debug)]
                pub struct RootBlock<I> {
                    pub(crate) interface: I,
                    #[doc(hidden)]
                    base_address: u8,
                }
                #[cfg(unix)]
                impl<I> RootBlock<I> {
                    /// Create a new instance of the block based on device interface
                    pub const fn new(interface: I) -> Self {
                        Self { interface, base_address: 0 }
                    }
                    pub(crate) const fn interface(&mut self) -> &mut I {
                        &mut self.interface
                    }
                    ///42 is the answer
                    #[cfg(unix)]
                    pub fn my_register1(
                        &mut self,
                    ) -> ::device_driver::RegisterOperation<'_, I, u8, MyRegister, ::device_driver::RW> {
                        let address = self.base_address + 5;
                        ::device_driver::RegisterOperation::<
                            '_,
                            I,
                            u8,
                            MyRegister,
                            ::device_driver::RW,
                        >::new(self.interface(), address as u8, MyRegister::new)
                    }
                }
            "}
        )
    }

    #[test]
    fn non_root_block_correct() {
        let output = generate_block(
            &Block {
                cfg_attr: quote! { #[cfg(unix)] },
                doc_attr: quote! { #[doc = "Hello!"] },
                root: false,
                name: format_ident!("AnyBlock"),
                methods: vec![BlockMethod {
                    cfg_attr: quote! { #[cfg(unix)] },
                    doc_attr: quote! { #[doc = "42 is the answer"] },
                    name: format_ident!("my_buffer"),
                    address: Literal::i64_unsuffixed(5),
                    allow_address_overlap: false,
                    kind: BlockMethodKind::Repeated {
                        count: Literal::i64_unsuffixed(4),
                        stride: Literal::i64_unsuffixed(1),
                    },
                    method_type: BlockMethodType::Buffer {
                        access: crate::mir::Access::RO,
                        address_type: format_ident!("i16"),
                    },
                }],
            },
            &format_ident!("u8"),
        );

        pretty_assertions::assert_eq!(
            prettyplease::unparse(&syn::parse2(output).unwrap()),
            indoc! {"
                ///Hello!
                #[cfg(unix)]
                #[derive(Debug)]
                pub struct AnyBlock<'i, I> {
                    pub(crate) interface: &'i mut I,
                    #[doc(hidden)]
                    base_address: u8,
                }
                #[cfg(unix)]
                impl<'i, I> AnyBlock<'i, I> {
                    /// Create a new instance of the block based on device interface
                    #[doc(hidden)]
                    const fn new(interface: &'i mut I, base_address: u8) -> Self {
                        Self {
                            interface,
                            base_address: base_address,
                        }
                    }
                    pub(crate) const fn interface(&mut self) -> &mut I {
                        self.interface
                    }
                    ///42 is the answer
                    ///
                    ///Valid index range: 0..4
                    #[cfg(unix)]
                    pub fn my_buffer(
                        &mut self,
                        index: usize,
                    ) -> ::device_driver::BufferOperation<'_, I, i16, ::device_driver::RO> {
                        let address = {
                            assert!(index < 4);
                            self.base_address + 5 + index as u8 * 1
                        };
                        ::device_driver::BufferOperation::<
                            '_,
                            I,
                            i16,
                            ::device_driver::RO,
                        >::new(self.interface(), address as i16)
                    }
                }
            "}
        )
    }
}
