use proc_macro2::TokenStream;
use quote::quote;

use crate::lir::EnumVariant;

use super::Enum;

pub fn generate_enum(value: &Enum) -> TokenStream {
    let Enum {
        cfg_attr,
        doc_attr,
        name,
        base_type,
        variants,
    } = value;
    let default_variant = variants.iter().find(|v| v.default);
    let catch_all_variant = variants.iter().find(|v| v.catch_all);

    let variant_quotes = variants.iter().map(|var| {
        let EnumVariant {
            cfg_attr,
            doc_attr,
            name,
            number,
            catch_all,
            ..
        } = var;

        let enum_field = if *catch_all {
            quote! {
                (#base_type)
            }
        } else {
            quote! {}
        };

        quote! {
            #doc_attr
            #cfg_attr
            #name #enum_field = #number
        }
    });

    let default_impl = if let Some(EnumVariant {
        name: var_name,
        number,
        catch_all,
        ..
    }) = default_variant
    {
        let catch_all_extension = if *catch_all {
            quote! { (#number) }
        } else {
            quote! {}
        };

        quote! {
            impl Default for #name {
                fn default() -> Self {
                    Self::#var_name #catch_all_extension
                }
            }
        }
    } else {
        quote! {}
    };

    let try_from_fallback_variant = match (catch_all_variant, default_variant) {
        (None, None) => quote! { val => Err(val) },
        (None, Some(_)) => quote! { _ => Ok(Self::default()) },
        (Some(EnumVariant { name, .. }), _) => quote! { val => Ok(Self::#name(val)) },
    };
    let try_from_variants = variants
        .iter()
        .filter(|v| !v.catch_all)
        .map(|EnumVariant { name, number, .. }| {
            quote! {
                #number => Ok(Self::#name)
            }
        })
        .chain(Some(try_from_fallback_variant));
    let try_from_impl = quote! {
        impl core::convert::TryFrom<#base_type> for #name {
            type Error = #base_type;
            fn try_from(val: #base_type) -> Result<Self, Self::Error> {
                match val {
                    #(#try_from_variants),*
                }
            }
        }
    };

    let from_impl = if catch_all_variant.is_some() || default_variant.is_some() {
        quote! {
            impl From<#base_type> for #name {
                fn from(val: #base_type) -> Self {
                    use core::convert::TryInto;
                    val.try_into().unwrap()
                }
            }
        }
    } else {
        quote! {}
    };

    let into_impl = {
        let into_variants = variants.iter().map(
            |EnumVariant {
                 name: var_name,
                 number,
                 catch_all,
                 ..
             }| {
                if *catch_all {
                    quote! {
                        #name::#var_name(num) => num
                    }
                } else {
                    quote! {
                        #name::#var_name => #number
                    }
                }
            },
        );

        quote! {
            impl From<#name> for #base_type {
                fn from(val: #name) -> Self {
                    match val {
                        #(#into_variants),*
                    }
                }
            }
        }
    };

    // TODO: Add defmt
    quote! {
        #doc_attr
        #cfg_attr
        #[repr(#base_type)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub enum #name {
            #(#variant_quotes),*
        }

        #default_impl

        #try_from_impl

        #from_impl

        #into_impl
    }
}

#[cfg(test)]
mod tests {
    use crate::lir::EnumVariant;

    use super::*;
    use indoc::indoc;
    use proc_macro2::Literal;
    use quote::format_ident;

    #[test]
    fn enum_correct() {
        let output = generate_enum(&Enum {
            cfg_attr: quote! { #[cfg(windows)] },
            doc_attr: quote! { #[doc = "Docs are important!"] },
            name: format_ident!("MyEnum"),
            base_type: format_ident!("u8"),
            variants: vec![
                EnumVariant {
                    cfg_attr: quote! {#[cfg(unix)]},
                    doc_attr: quote! {#[doc="Field!"]},
                    name: format_ident!("MyField"),
                    number: Literal::u8_unsuffixed(0),
                    default: false,
                    catch_all: false,
                },
                EnumVariant {
                    cfg_attr: quote! {},
                    doc_attr: quote! {},
                    name: format_ident!("MyField1"),
                    number: Literal::u8_unsuffixed(1),
                    default: true,
                    catch_all: false,
                },
                EnumVariant {
                    cfg_attr: quote! {},
                    doc_attr: quote! {},
                    name: format_ident!("MyField2"),
                    number: Literal::u8_unsuffixed(4),
                    default: false,
                    catch_all: true,
                },
            ],
        });

        pretty_assertions::assert_eq!(
            prettyplease::unparse(&syn::parse2(output).unwrap()),
            indoc! {"
                ///Docs are important!
                #[cfg(windows)]
                #[repr(u8)]
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
                pub enum MyEnum {
                    ///Field!
                    #[cfg(unix)]
                    MyField = 0,
                    MyField1 = 1,
                    MyField2(u8) = 4,
                }
                impl Default for MyEnum {
                    fn default() -> Self {
                        Self::MyField1
                    }
                }
                impl core::convert::TryFrom<u8> for MyEnum {
                    type Error = u8;
                    fn try_from(val: u8) -> Result<Self, Self::Error> {
                        match val {
                            0 => Ok(Self::MyField),
                            1 => Ok(Self::MyField1),
                            val => Ok(Self::MyField2(val)),
                        }
                    }
                }
                impl From<u8> for MyEnum {
                    fn from(val: u8) -> Self {
                        use core::convert::TryInto;
                        val.try_into().unwrap()
                    }
                }
                impl From<MyEnum> for u8 {
                    fn from(val: MyEnum) -> Self {
                        match val {
                            MyEnum::MyField => 0,
                            MyEnum::MyField1 => 1,
                            MyEnum::MyField2(num) => num,
                        }
                    }
                }
            "}
        )
    }
}
