use std::iter::once;

use proc_macro2::TokenStream;
use quote::quote;

use crate::lir::FieldSet;

pub fn generate_field_set_enum(
    field_sets: &[FieldSet],
    defmt_feature: Option<&str>,
) -> TokenStream {
    let filter = |fs: &&FieldSet| fs.size_bits > 0;

    let fields = field_sets.iter().filter(filter).map(|fs| {
        let name = &fs.name;
        let cfg = &fs.cfg_attr;
        let doc = &fs.doc_attr;
        quote! {
            #cfg
            #doc
            #name(#name)
        }
    });

    let from_impls = field_sets.iter().filter(filter).map(|fs| {
        let name = &fs.name;
        let cfg = &fs.cfg_attr;
        quote! {
            #cfg
            impl From<#name> for FieldSetValue {
                fn from(val: #name) -> Self {
                    Self::#name(val)
                }
            }
        }
    });

    let debug_forward_calls = field_sets
        .iter()
        .filter(filter)
        .map(|fs| {
            let name = &fs.name;
            let cfg = &fs.cfg_attr;
            quote! {
                #cfg
                Self::#name(val) => core::fmt::Debug::fmt(val, f)
            }
        })
        .chain(once(quote! { _ => unreachable!() }));

    let debug_impl = quote! {
        impl core::fmt::Debug for FieldSetValue {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #(#debug_forward_calls),*
                }
            }
        }
    };

    let defmt_forward_calls = field_sets
        .iter()
        .filter(filter)
        .map(|fs| {
            let name = &fs.name;
            let cfg = &fs.cfg_attr;
            quote! {
                #cfg
                Self::#name(val) => defmt::Format::format(val, f)
            }
        })
        .chain(once(quote! { _ => unreachable!() }));

    let defmt_impl = if let Some(defmt_feature) = defmt_feature {
        quote! {
            #[cfg(feature = #defmt_feature)]
            impl defmt::Format for FieldSetValue {
                fn format(&self, f: defmt::Formatter) {
                    match self {
                        #(#defmt_forward_calls),*
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        /// Enum containing all possible field set types
        pub enum FieldSetValue {
            #(#fields),*
        }

        #debug_impl
        #defmt_impl

        #(#from_impls)*
    }
}
