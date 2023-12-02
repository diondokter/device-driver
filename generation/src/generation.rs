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
            name, size_bytes, ..
        } = self;

        let pascal_case_name = syn::Ident::new(&name.to_case(Case::Pascal), Span::call_site());

        let mut field_functions = TokenStream::new();
        field_functions.append_all(
            self.fields
                .iter()
                .map(|field| field.generate_field_function()),
        );

        let mut field_types = TokenStream::new();
        field_types.append_all(self.fields.iter().filter_map(|field| {
            field
                .conversion_type
                .as_ref()
                .map(|ct| ct.generate_type_definition(field.register_type.into_type(), &field.name))
        }));

        let address_type = device.address_type.into_type();
        let address = proc_macro2::Literal::u64_unsuffixed(self.address);
        let rw_capability = self.rw_capability.into_type();

        quote! {
            struct #pascal_case_name {
                bits: device_driver::bitvec::BitArray<[u8; Self::SIZE_BYTES]>,
            }

            impl device_driver::Register<{ Self::SIZE_BYTES }> for #pascal_case_name {
                const ZERO: Self = Self {
                    bits: BitArray::ZERO,
                };

                type AddressType = #address_type;
                const ADDRESS: Self::AddressType = #address;

                type RWCapability = #rw_capability;

                fn bits(&mut self) -> &mut BitArray<[u8; Self::SIZE_BYTES]> {
                    &mut self.bits
                }
            }

            impl #pascal_case_name {
                pub const SIZE_BYTES: usize = #size_bytes;

                #field_functions
            }

            #field_types
        }
        .into_token_stream()
    }
}

impl Field {
    fn generate_field_function(&self) -> TokenStream {
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
            pub fn #snake_case_name(&mut self) -> device_driver::Field<'_, Self, #conversion_type, #register_type, #start, #end, { Self::SIZE_BYTES }> {
                Field::new(self)
            }
        }.to_token_stream()
    }
}
