use proc_macro2::TokenStream;
use quote::quote;

use crate::lir::{Block, BlockMethod, BlockMethodKind, BlockMethodType};

pub fn generate_block(value: &Block) -> TokenStream {
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
                Some(quote! { base_address: i64 }),
                quote! { base_address },
                quote! { self.interface },
            )
        };

    let methods = methods.iter().map(generate_method);

    quote! {
        #doc_attr
        #cfg_attr
        #[derive(Debug)]
        pub struct #name<#generics> {
            interface: #interface_decleration,
            base_address: i64,
        }

        #cfg_attr
        impl<#generics> #name<#generics> {
            /// Create a new instance of the block based on device interface
            pub fn new(interface: #interface_decleration, #address_param) -> Self {
                Self {
                    interface,
                    base_address: #address_specifier,
                }
            }

            fn interface(&mut self) -> &mut I {
                #interface_borrow
            }

            #(#methods)*
        }
    }
}

fn generate_method(method: &BlockMethod) -> TokenStream {
    let BlockMethod {
        cfg_attr,
        doc_attr,
        name,
        address,
        allow_address_overlap: _,
        kind,
        method_type,
    } = method;

    let (return_type, where_bounds, address_conversion) = match method_type {
        BlockMethodType::Block { name } => (quote! { #name<'_, I> }, quote! {}, quote! {}),
        BlockMethodType::Register {
            field_set_name,
            access,
            address_type,
        } => (
            quote! { ::device_driver::RegisterOperation::<'_, I, #field_set_name, #access>  },
            quote! { where I: ::device_driver::RegisterInterface<AddressType = #address_type> },
            quote! { as #address_type },
        ),
        BlockMethodType::SimpleCommand { address_type } => (
            quote! { ::device_driver::CommandOperation::<'_, I, (), ()>  },
            quote! { where I: ::device_driver::CommandInterface<AddressType = #address_type> },
            quote! { as #address_type },
        ),
        BlockMethodType::Command {
            field_set_name_in,
            field_set_name_out,
            address_type,
        } => (
            quote! { ::device_driver::CommandOperation::<'_, I, #field_set_name_in, #field_set_name_out>  },
            quote! { where I: ::device_driver::CommandInterface<AddressType = #address_type> },
            quote! { as #address_type },
        ),
        BlockMethodType::Buffer {
            access,
            address_type,
        } => (
            quote! { ::device_driver::BufferOperation::<'_, I, #access>  },
            quote! { where I: ::device_driver::BufferInterface<AddressType = #address_type> },
            quote! { as #address_type },
        ),
    };

    let (index_param, address_calc, index_doc) = match kind {
        BlockMethodKind::Normal => (None, quote! { self.base_address + #address }, None),
        BlockMethodKind::Repeated { count, stride } => {
            let doc = format!("Valid index range: 0..{count}");
            (
                Some(quote! { index: i64, }),
                quote! { {
                    assert!(index < #count);
                    self.base_address + #address + index * #stride
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
        pub fn #name(#index_param) -> #return_type
        #where_bounds
        {
            let address = #address_calc;
            #return_type::new(self.interface(), address #address_conversion)
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
        let output = generate_block(&Block {
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
                },
            }],
        });

        pretty_assertions::assert_eq!(
            prettyplease::unparse(&syn::parse2(output).unwrap()),
            indoc! {"
                ///Hello!
                #[cfg(unix)]
                #[derive(Debug)]
                pub struct RootBlock<I> {
                    interface: I,
                    base_address: i64,
                }
                #[cfg(unix)]
                impl<I> RootBlock<I> {
                    /// Create a new instance of the block based on device interface
                    pub fn new(interface: I) -> Self {
                        Self { interface, base_address: 0 }
                    }
                    fn interface(&mut self) -> &mut I {
                        &mut self.interface
                    }
                    ///42 is the answer
                    #[cfg(unix)]
                    pub fn my_register1() -> ::device_driver::RegisterOperation<'_, I, MyRegister, RW>
                    where
                        I: ::device_driver::RegisterInterface<AddressType = u8>,
                    {
                        let address = self.base_address + 5;
                        ::device_driver::RegisterOperation::<
                            '_,
                            I,
                            MyRegister,
                            RW,
                        >::new(self.interface(), address as u8)
                    }
                }
            "}
        )
    }

    #[test]
    fn non_root_block_correct() {
        let output = generate_block(&Block {
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
        });

        pretty_assertions::assert_eq!(
            prettyplease::unparse(&syn::parse2(output).unwrap()),
            indoc! {"
                ///Hello!
                #[cfg(unix)]
                #[derive(Debug)]
                pub struct AnyBlock<'i, I> {
                    interface: &'i mut I,
                    base_address: i64,
                }
                #[cfg(unix)]
                impl<'i, I> AnyBlock<'i, I> {
                    /// Create a new instance of the block based on device interface
                    pub fn new(interface: &'i mut I, base_address: i64) -> Self {
                        Self {
                            interface,
                            base_address: base_address,
                        }
                    }
                    fn interface(&mut self) -> &mut I {
                        self.interface
                    }
                    ///42 is the answer
                    ///
                    ///Valid index range: 0..4
                    #[cfg(unix)]
                    pub fn my_buffer(index: i64) -> ::device_driver::BufferOperation<'_, I, RO>
                    where
                        I: ::device_driver::BufferInterface<AddressType = i16>,
                    {
                        let address = {
                            assert!(index < 4);
                            self.base_address + 5 + index * 1
                        };
                        ::device_driver::BufferOperation::<
                            '_,
                            I,
                            RO,
                        >::new(self.interface(), address as i16)
                    }
                }
            "}
        )
    }
}
