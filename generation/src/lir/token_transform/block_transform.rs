use itertools::Itertools;

use crate::{
    lir::{Block, BlockMethod, BlockMethodKind, BlockMethodType},
    mir::{self, Integer},
};

pub fn generate_block(
    value: &Block,
    internal_address_type: &Integer,
    register_address_type: &Integer,
) -> String {
    let Block {
        cfg_attr,
        doc_attr,
        root,
        name,
        methods,
    } = value;

    let (generics, interface_declaration, address_param, address_specifier, interface_borrow) =
        if *root {
            (
                "I".to_string(),
                "I".to_string(),
                String::new(),
                "0".to_string(),
                "&mut self.interface".to_string(),
            )
        } else {
            (
                "'i, I".to_string(),
                "&'i mut I".to_string(),
                format!("base_address: {internal_address_type}"),
                "base_address".to_string(),
                "self.interface".to_string(),
            )
        };

    let method_impls = methods
        .iter()
        .map(|m| generate_method(m, internal_address_type))
        .join("\n");

    let (new_hidden_if_not_root, new_access, new_const) = if *root {
        ("", "pub", "const")
    } else {
        ("#[doc(hidden)]", "", "")
    };

    let generate_read_all_registers_items = |use_async: bool| {
        methods
            .iter()
            .filter_map(move |m| match &m.method_type {
                BlockMethodType::Register {
                    access: mir::Access::RO | mir::Access::RW,
                    ..
                } => {
                    let register_name = &m.name;
                    let address = &m.address;
                    let cfg_attr = &m.cfg_attr;

                    let read_function = match use_async {
                        true => "read_async().await",
                        false => "read()",
                    };

                    let (count, stride, index_required) = match &m.kind {
                        BlockMethodKind::Normal => (1i64, 0, false),
                        BlockMethodKind::Repeated { count, stride } => {
                            (count.to_string().parse().unwrap(), *stride, true)
                        }
                    };

                    Some((0..count).map(move |index| {
                        let (index_param, register_display_name) = match index_required {
                            true => {
                                (format!("{index}"), format!("{}[{index}]", m.name))
                            }
                            false => (String::new(), m.name.to_string()),
                        };
                        format!("
                            {cfg_attr}
                            let reg = self.{register_name}({index_param}).{read_function}?;
                            {cfg_attr}
                            callback({address} + {index} * {stride}, \"{register_display_name}\", reg.into());
                        ")
                    }))
                }
                _ => None,
            })
            .flatten()
    };

    let read_all_registers_items = generate_read_all_registers_items(false).join("\n");
    let read_async_all_registers_items = generate_read_all_registers_items(true).join("\n");

    let read_all_docs = "
        /// Read all readable register values in this block from the device.
        /// The callback is called for each of them.
        /// Any registers in child blocks are not included.
        ///
        /// The callback has three arguments:
        ///
        /// - The address of the register
        /// - The name of the register (with index for repeated registers)
        /// - The read value from the register
        ///
        /// This is useful for e.g. debug printing all values.
        /// The given [field_sets::FieldSetValue] has a Debug and Format implementation that forwards to the concrete type
        /// the lies within so it can be printed without matching on it.
    ".to_string();

    format!("
        {doc_attr}
        {cfg_attr}
        #[derive(Debug)]
        pub struct {name}<{generics}> {{
            pub(crate) interface: {interface_declaration},
            #[doc(hidden)]
            base_address: {internal_address_type},
        }}

        {cfg_attr}
        impl<{generics}> {name}<{generics}> {{
            /// Create a new instance of the block based on device interface
            {new_hidden_if_not_root}
            {new_access} {new_const} fn new(interface: {interface_declaration}, {address_param}) -> Self {{
                Self {{
                    interface,
                    base_address: {address_specifier},
                }}
            }}

            /// A reference to the interface used to communicate with the device
            pub(crate) fn interface(&mut self) -> &mut I {{
                {interface_borrow}
            }}

            {read_all_docs}
            pub fn read_all_registers(
                &mut self,
                mut callback: impl FnMut({register_address_type}, &'static str, field_sets::FieldSetValue)
            ) -> Result<(), I::Error>
                where I: ::device_driver::RegisterInterface<AddressType = {register_address_type}>
            {{
                {read_all_registers_items}
                Ok(())
            }}

            {read_all_docs}
            pub async fn read_all_registers_async(
                &mut self,
                mut callback: impl FnMut({register_address_type}, &'static str, field_sets::FieldSetValue)
            ) -> Result<(), I::Error>
                where I: ::device_driver::AsyncRegisterInterface<AddressType = {register_address_type}>
            {{
                {read_async_all_registers_items}
                Ok(())
            }}

            {method_impls}
        }}
    ")
}

fn generate_method(method: &BlockMethod, internal_address_type: &Integer) -> String {
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
        BlockMethodType::Block { name } => {
            (format!("{name}::<'_, I>"), String::new(), String::new())
        }
        BlockMethodType::Register {
            field_set_name,
            access,
            address_type,
            reset_value_function: default_value_function_name,
        } => (
            format!(
                "::device_driver::RegisterOperation::<'_, I, {address_type}, field_sets::{field_set_name}, ::device_driver::{access}>"
            ),
            format!("as {address_type}"),
            format!(", field_sets::{field_set_name}::{default_value_function_name}"),
        ),
        BlockMethodType::Command {
            field_set_name_in,
            field_set_name_out,
            address_type,
        } => {
            let field_set_name_in = match field_set_name_in {
                Some(val) => format!("field_sets::{val}"),
                None => "()".to_string(),
            };
            let field_set_name_out = match field_set_name_out {
                Some(val) => format!("field_sets::{val}"),
                None => "()".to_string(),
            };
            (
                format!(
                    "::device_driver::CommandOperation::<'_, I, {address_type}, {field_set_name_in}, {field_set_name_out}>"
                ),
                format!("as {address_type}"),
                String::new(),
            )
        }
        BlockMethodType::Buffer {
            access,
            address_type,
        } => (
            format!(
                "::device_driver::BufferOperation::<'_, I, {address_type}, ::device_driver::{access}>"
            ),
            format!("as {address_type}"),
            String::new(),
        ),
    };

    let (index_param, address_calc, index_doc) = match kind {
        BlockMethodKind::Normal => (
            String::new(),
            format!("self.base_address + {address}"),
            String::new(),
        ),
        BlockMethodKind::Repeated { count, stride } => {
            let doc = format!("Valid index range: 0..{count}");

            let stride = stride.to_string().parse::<i64>().unwrap();

            let operator = if stride.is_negative() { "-" } else { "+" };

            let stride = stride.unsigned_abs();

            (
                "index: usize,".to_string(),
                format!("{{
                    assert!(index < {count});
                    self.base_address + {address} {operator} index as {internal_address_type} * {stride}
                }}"),
                format!("
                    #[doc = \"\"]
                    #[doc = \"{doc}\"]
                "),
            )
        }
    };

    format!(
        "
        {doc_attr}
        {index_doc}
        {cfg_attr}
        pub fn {name}(&mut self, {index_param}) -> {return_type} {{
            let address = {address_calc};
            {return_type}::new(self.interface(), address {address_conversion} {default_arg})
        }}
    "
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use quote::quote;

    #[test]
    fn root_block_correct() {
        let output = generate_block(
            &Block {
                cfg_attr: quote! { #[cfg(unix)] },
                doc_attr: quote! { #[doc = "Hello!"] },
                root: true,
                name: "RootBlock".to_string(),
                methods: vec![BlockMethod {
                    cfg_attr: quote! { #[cfg(unix)] },
                    doc_attr: quote! { #[doc = "42 is the answer"] },
                    name: "my_register1".to_string(),
                    address: 5,
                    allow_address_overlap: false,
                    kind: BlockMethodKind::Normal,
                    method_type: BlockMethodType::Register {
                        field_set_name: "MyRegister".to_string(),
                        access: crate::mir::Access::RW,
                        address_type: mir::Integer::U8,
                        reset_value_function: "new".to_string(),
                    },
                }],
            },
            &mir::Integer::U8,
            &mir::Integer::U8,
        );

        pretty_assertions::assert_eq!(
            prettyplease::unparse(&syn::parse_str(&output).unwrap()),
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
                    /// A reference to the interface used to communicate with the device
                    pub(crate) fn interface(&mut self) -> &mut I {
                        &mut self.interface
                    }
                    /// Read all readable register values in this block from the device.
                    /// The callback is called for each of them.
                    /// Any registers in child blocks are not included.
                    ///
                    /// The callback has three arguments:
                    ///
                    /// - The address of the register
                    /// - The name of the register (with index for repeated registers)
                    /// - The read value from the register
                    ///
                    /// This is useful for e.g. debug printing all values.
                    /// The given [field_sets::FieldSetValue] has a Debug and Format implementation that forwards to the concrete type
                    /// the lies within so it can be printed without matching on it.
                    pub fn read_all_registers(
                        &mut self,
                        mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
                    ) -> Result<(), I::Error>
                    where
                        I: ::device_driver::RegisterInterface<AddressType = u8>,
                    {
                        #[cfg(unix)]
                        let reg = self.my_register1().read()?;
                        #[cfg(unix)] callback(5 + 0 * 0, \"my_register1\", reg.into());
                        Ok(())
                    }
                    /// Read all readable register values in this block from the device.
                    /// The callback is called for each of them.
                    /// Any registers in child blocks are not included.
                    ///
                    /// The callback has three arguments:
                    ///
                    /// - The address of the register
                    /// - The name of the register (with index for repeated registers)
                    /// - The read value from the register
                    ///
                    /// This is useful for e.g. debug printing all values.
                    /// The given [field_sets::FieldSetValue] has a Debug and Format implementation that forwards to the concrete type
                    /// the lies within so it can be printed without matching on it.
                    pub async fn read_all_registers_async(
                        &mut self,
                        mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
                    ) -> Result<(), I::Error>
                    where
                        I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
                    {
                        #[cfg(unix)]
                        let reg = self.my_register1().read_async().await?;
                        #[cfg(unix)] callback(5 + 0 * 0, \"my_register1\", reg.into());
                        Ok(())
                    }
                    ///42 is the answer
                    #[cfg(unix)]
                    pub fn my_register1(
                        &mut self,
                    ) -> ::device_driver::RegisterOperation<
                        '_,
                        I,
                        u8,
                        field_sets::MyRegister,
                        ::device_driver::RW,
                    > {
                        let address = self.base_address + 5;
                        ::device_driver::RegisterOperation::<
                            '_,
                            I,
                            u8,
                            field_sets::MyRegister,
                            ::device_driver::RW,
                        >::new(self.interface(), address as u8, field_sets::MyRegister::new)
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
                name: "AnyBlock".to_string(),
                methods: vec![BlockMethod {
                    cfg_attr: quote! { #[cfg(unix)] },
                    doc_attr: quote! { #[doc = "42 is the answer"] },
                    name: "my_buffer".to_string(),
                    address: 5,
                    allow_address_overlap: false,
                    kind: BlockMethodKind::Repeated {
                        count: 4,
                        stride: 1,
                    },
                    method_type: BlockMethodType::Buffer {
                        access: crate::mir::Access::RO,
                        address_type: Integer::I16,
                    },
                }],
            },
            &Integer::U8,
            &Integer::U8,
        );

        pretty_assertions::assert_eq!(
            prettyplease::unparse(&syn::parse_str(&output).unwrap()),
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
                    fn new(interface: &'i mut I, base_address: u8) -> Self {
                        Self {
                            interface,
                            base_address: base_address,
                        }
                    }
                    /// A reference to the interface used to communicate with the device
                    pub(crate) fn interface(&mut self) -> &mut I {
                        self.interface
                    }
                    /// Read all readable register values in this block from the device.
                    /// The callback is called for each of them.
                    /// Any registers in child blocks are not included.
                    ///
                    /// The callback has three arguments:
                    ///
                    /// - The address of the register
                    /// - The name of the register (with index for repeated registers)
                    /// - The read value from the register
                    ///
                    /// This is useful for e.g. debug printing all values.
                    /// The given [field_sets::FieldSetValue] has a Debug and Format implementation that forwards to the concrete type
                    /// the lies within so it can be printed without matching on it.
                    pub fn read_all_registers(
                        &mut self,
                        mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
                    ) -> Result<(), I::Error>
                    where
                        I: ::device_driver::RegisterInterface<AddressType = u8>,
                    {
                        Ok(())
                    }
                    /// Read all readable register values in this block from the device.
                    /// The callback is called for each of them.
                    /// Any registers in child blocks are not included.
                    ///
                    /// The callback has three arguments:
                    ///
                    /// - The address of the register
                    /// - The name of the register (with index for repeated registers)
                    /// - The read value from the register
                    ///
                    /// This is useful for e.g. debug printing all values.
                    /// The given [field_sets::FieldSetValue] has a Debug and Format implementation that forwards to the concrete type
                    /// the lies within so it can be printed without matching on it.
                    pub async fn read_all_registers_async(
                        &mut self,
                        mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
                    ) -> Result<(), I::Error>
                    where
                        I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
                    {
                        Ok(())
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
