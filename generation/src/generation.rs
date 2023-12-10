use crate::{Device, Field, Register};

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

impl Device {
    pub fn generate_device_impl(&self, mut existing_impl: syn::ItemImpl) -> TokenStream {
        assert!(
            existing_impl.trait_.is_none(),
            "Device impl must not be a block that impl's a trait"
        );
        existing_impl.items = self.generate_device_register_functions();

        existing_impl.into_token_stream()
    }

    pub fn generate_device_register_functions(&self) -> Vec<syn::ImplItem> {
        self.registers
            .iter()
            .map(|register| register.generate_register_function())
            .collect()
    }

    pub fn generate_definitions(&self) -> TokenStream {
        let mut stream = TokenStream::new();

        stream.append_all(
            self.registers
                .iter()
                .map(|register| register.generate_definition(self)),
        );

        stream
    }
}

impl Register {
    fn generate_register_function(&self) -> syn::ImplItem {
        let snake_case_name = syn::Ident::new(&self.name.to_case(Case::Snake), Span::call_site());
        let pascal_case_name = syn::Ident::new(&self.name.to_case(Case::Pascal), Span::call_site());

        syn::parse_quote! {
            pub fn #snake_case_name(&mut self) -> device_driver::RegisterOperation<'_, Self, #pascal_case_name, { #pascal_case_name::SIZE_BYTES }> {
                device_driver::RegisterOperation::new(self)
            }
        }
    }

    fn generate_definition(&self, device: &Device) -> TokenStream {
        let Register {
            name,
            size_bits,
            address,
            rw_capability,
            fields,
        } = self;

        let pascal_case_name_string = name.to_case(Case::Pascal);
        let pascal_case_name = syn::Ident::new(&pascal_case_name_string, Span::call_site());
        let snake_case_name = syn::Ident::new(&name.to_case(Case::Snake), Span::call_site());

        let field_functions_write = TokenStream::from_iter(
            fields
                .iter()
                .map(|field| field.generate_field_function_write()),
        );
        let field_functions_read = TokenStream::from_iter(
            fields
                .iter()
                .map(|field| field.generate_field_function_read(false)),
        );
        let field_functions_read_explicit = TokenStream::from_iter(
            fields
                .iter()
                .map(|field| field.generate_field_function_read(true)),
        );

        let mut field_types = TokenStream::new();
        field_types.append_all(fields.iter().filter_map(|field| {
            field
                .conversion_type
                .as_ref()
                .map(|ct| ct.generate_type_definition(field.register_type.into_type(), &field.name))
        }));

        let address_type = device.address_type.into_type();
        let address = proc_macro2::Literal::u64_unsuffixed(*address);
        let rw_capability = rw_capability.into_type();
        let size_bits_lit = proc_macro2::Literal::u64_unsuffixed(*size_bits);
        let size_bytes_lit = proc_macro2::Literal::u64_unsuffixed(size_bits.div_ceil(8));

        let debug_field_calls = TokenStream::from_iter(fields.iter().map(|field| {
            let name_string = &field.name;
            let name = syn::Ident::new(name_string, Span::call_site());
            quote!(.field(#name_string, &self.#name()))
        }));

        quote! {
            pub struct #pascal_case_name {
                bits: device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]>,
            }

            impl device_driver::Register<{ Self::SIZE_BYTES }> for #pascal_case_name {
                const ZERO: Self = Self {
                    bits: device_driver::bitvec::array::BitArray::ZERO,
                };

                type AddressType = #address_type;
                const ADDRESS: Self::AddressType = #address;

                type RWCapability = #rw_capability;
                const SIZE_BITS: usize = #size_bits_lit;

                type WriteFields = #snake_case_name::W;
                type ReadFields = #snake_case_name::R;

                fn bits_mut(&mut self) -> &mut device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]> {
                    &mut self.bits
                }
                fn bits(&self) -> &device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]> {
                    &self.bits
                }
            }

            impl #pascal_case_name {
                pub const SIZE_BYTES: usize = #size_bytes_lit;
            }

            pub mod #snake_case_name {
                use super::*;

                pub struct W {
                    inner: #pascal_case_name,
                }

                impl From<#pascal_case_name> for W {
                    fn from(val: #pascal_case_name) -> Self {
                        Self {
                            inner: val,
                        }
                    }
                }

                impl From<W> for #pascal_case_name {
                    fn from(val: W) -> Self {
                        val.inner
                    }
                }

                impl core::ops::Deref for W {
                    type Target = #pascal_case_name;

                    fn deref(&self) -> &Self::Target {
                        &self.inner
                    }
                }

                impl core::ops::DerefMut for W {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self.inner
                    }
                }

                impl W {
                    pub const SIZE_BYTES: usize = #size_bytes_lit;
                    #field_functions_write
                    #field_functions_read_explicit
                }

                pub struct R {
                    inner: #pascal_case_name,
                }

                impl From<#pascal_case_name> for R {
                    fn from(val: #pascal_case_name) -> Self {
                        Self {
                            inner: val,
                        }
                    }
                }

                impl From<R> for #pascal_case_name {
                    fn from(val: R) -> Self {
                        val.inner
                    }
                }

                impl core::fmt::Debug for R {
                    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                        fmt.debug_struct(#pascal_case_name_string)
                            #debug_field_calls
                            .finish()
                    }
                }

                impl core::ops::Deref for R {
                    type Target = #pascal_case_name;

                    fn deref(&self) -> &Self::Target {
                        &self.inner
                    }
                }

                impl core::ops::DerefMut for R {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self.inner
                    }
                }

                impl R {
                    pub const SIZE_BYTES: usize = #size_bytes_lit;
                    #field_functions_read
                }
            }

            #field_types
        }
        .into_token_stream()
    }
}

impl Field {
    fn generate_field_function_read(&self, explicit: bool) -> TokenStream {
        let Self {
            name,
            conversion_type,
            register_type,
            start,
            end,
        } = self;

        let function_name = if explicit {
            format!("get_{name}")
        } else {
            name.clone()
        };

        let snake_case_name =
            syn::Ident::new(&function_name.to_case(Case::Snake), Span::call_site());
        let register_type = register_type.into_type();
        let conversion_type = conversion_type
            .as_ref()
            .map(|ct| ct.into_type(name))
            .unwrap_or(register_type.clone());
        let start = proc_macro2::Literal::u32_unsuffixed(*start);
        let end = proc_macro2::Literal::u32_unsuffixed(*end);

        quote! {
            pub fn #snake_case_name(&self) -> Result<#conversion_type, <#conversion_type as TryFrom<#register_type>>::Error> {
                device_driver::read_field::<Self, _, #conversion_type, #register_type, #start, #end, { Self::SIZE_BYTES }>(self)
            }
        }.to_token_stream()
    }

    fn generate_field_function_write(&self) -> TokenStream {
        let Self {
            name,
            conversion_type,
            register_type,
            start,
            end,
        } = self;

        let snake_case_name = syn::Ident::new(&name.to_case(Case::Snake), Span::call_site());
        let register_type = register_type.into_type();
        let conversion_type = conversion_type
            .as_ref()
            .map(|ct| ct.into_type(name))
            .unwrap_or(register_type.clone());
        let start = proc_macro2::Literal::u32_unsuffixed(*start);
        let end = proc_macro2::Literal::u32_unsuffixed(*end);

        quote! {
            pub fn #snake_case_name(&mut self, data: #conversion_type) -> &mut Self {
                device_driver::write_field::<Self, _, #conversion_type, #register_type, #start, #end, { Self::SIZE_BYTES }>(self, data)
            }
        }.to_token_stream()
    }
}
