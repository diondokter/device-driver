use crate::{BaseType, Command, Device, Field, Register};

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};

impl Device {
    pub fn generate_device_impl(&self, mut existing_impl: syn::ItemImpl) -> TokenStream {
        assert!(
            existing_impl.trait_.is_none(),
            "Device impl must not be a block that impl's a trait"
        );
        existing_impl.items = self.generate_device_functions();

        existing_impl.into_token_stream()
    }

    pub fn generate_definitions(&self) -> TokenStream {
        let mut stream = TokenStream::new();

        if let Some(registers) = self.registers.as_ref() {
            stream.append_all(
                registers
                    .iter()
                    .map(|register| register.generate_definition(self)),
            );
        }

        stream
    }

    fn generate_device_functions(&self) -> Vec<syn::ImplItem> {
        let mut result = Vec::new();

        if let Some(registers) = self.registers.as_ref() {
            result.extend(
                registers
                    .iter()
                    .map(|register| register.generate_register_function()),
            );
        }

        if let Some(commands) = self.commands.as_ref() {
            result.extend(
                commands
                    .iter()
                    .flat_map(|command| command.generate_command_function()),
            );
        }

        result
    }
}

impl Command {
    fn generate_command_function(&self) -> Vec<syn::ImplItem> {
        let function_name = syn::Ident::new(
            &format!("dispatch_{}", self.name).to_case(Case::Snake),
            Span::call_site(),
        );
        let async_function_name = syn::Ident::new(
            &format!("dispatch_{}_async", self.name).to_case(Case::Snake),
            Span::call_site(),
        );

        let doc_attribute = if let Some(description) = self.description.as_ref() {
            quote! { #[doc = #description] }
        } else {
            quote!()
        };

        let value = proc_macro2::Literal::u64_unsuffixed(self.id);

        [
            syn::parse_quote! {
                #doc_attribute
                pub fn #function_name(&mut self) -> Result<(), <Self as device_driver::CommandDevice>::Error> {
                    device_driver::CommandDevice::dispatch_command(self, #value)
                }
            },

            syn::parse_quote! {
                #doc_attribute
                pub async fn #async_function_name(&mut self) -> Result<(), <Self as device_driver::CommandDevice>::Error> {
                    device_driver::AsyncCommandDevice::dispatch_command(self, #value).await
                }
            }
        ].to_vec()
    }
}

impl Register {
    fn generate_register_function(&self) -> syn::ImplItem {
        let snake_case_name = syn::Ident::new(&self.name.to_case(Case::Snake), Span::call_site());
        let pascal_case_name = syn::Ident::new(&self.name.to_case(Case::Pascal), Span::call_site());

        let doc_attribute = if let Some(description) = self.description.as_ref() {
            quote! { #[doc = #description] }
        } else {
            quote!()
        };

        syn::parse_quote! {
            #doc_attribute
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
            rw_type,
            description,
            reset_value,
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

        let address_type = if let Some(register_address_type) = device.register_address_type {
            register_address_type.into_type()
        } else {
            return syn::Error::new(Span::call_site(), "register_address_type is not specified")
                .to_compile_error();
        };
        let address = proc_macro2::Literal::u64_unsuffixed(*address);
        let rw_type = rw_type.into_type();
        let size_bits_lit = proc_macro2::Literal::u64_unsuffixed(*size_bits);
        let size_bytes_lit = proc_macro2::Literal::u64_unsuffixed(size_bits.div_ceil(8));

        let debug_field_calls = TokenStream::from_iter(fields.iter().map(|field| {
            let name_string = field.name.to_case(Case::Snake);
            let name = syn::Ident::new(&name_string, Span::call_site());
            quote!(.field(#name_string, &self.#name()))
        }));

        let doc_attribute = if let Some(description) = description {
            quote! { #[doc = #description] }
        } else {
            quote!()
        };

        let module_doc_string = format!("Implementation of R and W types for [{pascal_case_name}]");
        let w_doc_string = format!("Write struct for [{pascal_case_name}]");
        let r_doc_string = format!("Read struct for [{pascal_case_name}]");

        let reset_value = if let Some(reset_value) = reset_value {
            let value =
                reset_value.get_data(size_bits.div_ceil(8) as usize, &pascal_case_name_string);

            match value {
                Ok(value) => {
                    quote! {
                        fn reset_value() -> Self
                        where
                            Self: Sized,
                        {
                            Self {
                                bits: device_driver::bitvec::array::BitArray::new([#(#value),*]),
                            }
                        }
                    }
                }
                Err(e) => e.to_compile_error(),
            }
        } else {
            quote!()
        };

        quote! {
            #doc_attribute
            pub struct #pascal_case_name {
                bits: device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]>,
            }

            impl device_driver::Register<{ Self::SIZE_BYTES }> for #pascal_case_name {
                const ZERO: Self = Self {
                    bits: device_driver::bitvec::array::BitArray::ZERO,
                };

                type AddressType = #address_type;
                const ADDRESS: Self::AddressType = #address;

                type RWType = #rw_type;
                const SIZE_BITS: usize = #size_bits_lit;

                type WriteFields = #snake_case_name::W;
                type ReadFields = #snake_case_name::R;

                fn bits_mut(&mut self) -> &mut device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]> {
                    &mut self.bits
                }
                fn bits(&self) -> &device_driver::bitvec::array::BitArray<[u8; Self::SIZE_BYTES]> {
                    &self.bits
                }
                #reset_value
            }

            impl #pascal_case_name {
                pub const SIZE_BYTES: usize = #size_bytes_lit;
            }

            #[doc = #module_doc_string]
            pub mod #snake_case_name {
                use super::*;

                #[doc = #w_doc_string]
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

                #[doc = #r_doc_string]
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
            description,
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
        let start = proc_macro2::Literal::u32_unsuffixed(*start);

        let doc_attribute = if let Some(description) = description {
            quote! { #[doc = #description] }
        } else {
            quote!()
        };

        match (self.register_type, *end, conversion_type) {
            (BaseType::Bool, Some(end), Some(conversion_type)) if end == self.start + 1 => {
                let conversion_type = conversion_type.into_type(name);
                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&self) -> Result<#conversion_type, <#conversion_type as TryFrom<#register_type>>::Error> {
                        device_driver::read_field_bool::<Self, _, #conversion_type, #start, { Self::SIZE_BYTES }>(self)
                    }
                }.to_token_stream()
            }
            (BaseType::Bool, Some(end), None) if end == self.start + 1 => {
                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&self) -> #register_type {
                        device_driver::read_field_bool_no_convert::<Self, _, #start, { Self::SIZE_BYTES }>(self)
                    }
                }.to_token_stream()
            }
            (BaseType::Bool, None, Some(conversion_type)) => {
                let conversion_type = conversion_type.into_type(name);
                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&self) -> Result<#conversion_type, <#conversion_type as TryFrom<#register_type>>::Error> {
                        device_driver::read_field_bool::<Self, _, #conversion_type, #start, { Self::SIZE_BYTES }>(self)
                    }
                }.to_token_stream()
            }
            (BaseType::Bool, None, None) => {
                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&self) -> #register_type {
                        device_driver::read_field_bool_no_convert::<Self, _, #start, { Self::SIZE_BYTES }>(self)
                    }
                }.to_token_stream()
            }
            (BaseType::Bool, _, _) => {
                syn::Error::new(Span::call_site(), format!("Registers {snake_case_name} is a bool and must have no end or an end that is only 1 more than start")).into_compile_error()       
            }
            (_, Some(end), Some(conversion_type)) => {
                let end = proc_macro2::Literal::u32_unsuffixed(end);
                let conversion_type = conversion_type.into_type(name);

                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&self) -> Result<#conversion_type, <#conversion_type as TryFrom<#register_type>>::Error> {
                        device_driver::read_field::<Self, _, #conversion_type, #register_type, #start, #end, { Self::SIZE_BYTES }>(self)
                    }
                }.to_token_stream()
            }
            (_, Some(end), None) => {
                let end = proc_macro2::Literal::u32_unsuffixed(end);

                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&self) -> #register_type {
                        device_driver::read_field_no_convert::<Self, _, #register_type, #start, #end, { Self::SIZE_BYTES }>(self)
                    }
                }.to_token_stream()
            }
            (_, None, _) => {
                syn::Error::new(Span::call_site(), format!("Registers {snake_case_name} has no end specified")).into_compile_error()       
            }
        }
    }

    fn generate_field_function_write(&self) -> TokenStream {
        let Self {
            name,
            description,
            conversion_type,
            register_type,
            start,
            end,
        } = self;

        let snake_case_name = syn::Ident::new(&name.to_case(Case::Snake), Span::call_site());
        let register_type = register_type.into_type();
        let start = proc_macro2::Literal::u32_unsuffixed(*start);

        let doc_attribute = if let Some(description) = description {
            quote! { #[doc = #description] }
        } else {
            quote!()
        };

        match (self.register_type, *end, conversion_type .as_ref()) {
            (BaseType::Bool, Some(end), Some(conversion_type)) if end == self.start + 1 => {
                let conversion_type = conversion_type.into_type(name);
                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&mut self, data: #conversion_type) -> &mut Self {
                        device_driver::write_field_bool::<Self, _, #conversion_type, #start, { Self::SIZE_BYTES }>(self, data)
                    }
                }.to_token_stream()
            }
            (BaseType::Bool, Some(end), None) if end == self.start + 1 => {
                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&mut self, data: #register_type) -> &mut Self {
                        device_driver::write_field_bool_no_convert::<Self, _, #start, { Self::SIZE_BYTES }>(self, data)
                    }
                }.to_token_stream()
            }
            (BaseType::Bool, None, Some(conversion_type)) => {
                let conversion_type = conversion_type.into_type(name);
                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&mut self, data: #conversion_type) -> &mut Self {
                        device_driver::write_field_bool::<Self, _, #conversion_type, #start, { Self::SIZE_BYTES }>(self, data)
                    }
                }.to_token_stream()
            }
            (BaseType::Bool, None, None) => {
                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&mut self, data: #register_type) -> &mut Self {
                        device_driver::write_field_bool_no_convert::<Self, _, #start, { Self::SIZE_BYTES }>(self, data)
                    }
                }.to_token_stream()
            }
            (BaseType::Bool, _, _) => {
                syn::Error::new(Span::call_site(), format!("Registers {snake_case_name} is a bool and must have no end or an end that is only 1 more than start")).into_compile_error()       
            }
            (_, Some(end), Some(conversion_type)) => {
                let end = proc_macro2::Literal::u32_unsuffixed(end);
                let conversion_type = conversion_type.into_type(name);

                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&mut self, data: #conversion_type) -> &mut Self {
                        device_driver::write_field::<Self, _, #conversion_type, #register_type, #start, #end, { Self::SIZE_BYTES }>(self, data)
                    }
                }.to_token_stream()
            }
            (_, Some(end), None) => {
                let end = proc_macro2::Literal::u32_unsuffixed(end);

                quote! {
                    #doc_attribute
                    pub fn #snake_case_name(&mut self, data: #register_type) -> &mut Self {
                        device_driver::write_field_no_convert::<Self, _, #register_type, #start, #end, { Self::SIZE_BYTES }>(self, data)
                    }
                }.to_token_stream()
            }
            (_, None, _) => {
                syn::Error::new(Span::call_site(), format!("Registers {snake_case_name} has no end specified")).into_compile_error()       
            }
        }
    }
}
