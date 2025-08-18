#!/usr/bin/env cargo
---
[package]
edition = "2024"
[dependencies]
device-driver = { path="../../../device-driver", default-features=false }
---
#![deny(warnings)]
#![allow(unexpected_cfgs)]
fn main() {}

pub mod foo_d_0 {
    /// Root block of the FooD0 driver
    
    #[derive(Debug)]
    pub struct FooD0<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD0<I> {
        /// Create a new instance of the block based on device interface
        pub const fn new(interface: I) -> Self {
            Self {
                interface,
                base_address: 0,
            }
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub async fn read_all_registers_async(
            &mut self,
            mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
        ) -> Result<(), I::Error>
        where
            I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
        {
            Ok(())
        }
    }
    
    /// Module containing the generated fieldsets of the registers and commands
    pub mod field_sets {
        #[allow(unused_imports)]
        use super::*;
    
        /// Enum containing all possible field set types
        pub enum FieldSetValue {}
        impl core::fmt::Debug for FieldSetValue {
            fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    }
}

pub mod foo_d_1 {
    /// Root block of the FooD1 driver
    
    #[derive(Debug)]
    pub struct FooD1<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD1<I> {
        /// Create a new instance of the block based on device interface
        pub const fn new(interface: I) -> Self {
            Self {
                interface,
                base_address: 0,
            }
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub async fn read_all_registers_async(
            &mut self,
            mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
        ) -> Result<(), I::Error>
        where
            I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
        {
            Ok(())
        }
    }
    
    /// Module containing the generated fieldsets of the registers and commands
    pub mod field_sets {
        #[allow(unused_imports)]
        use super::*;
    
        /// Enum containing all possible field set types
        pub enum FieldSetValue {}
        impl core::fmt::Debug for FieldSetValue {
            fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    }
}

pub mod foo_d_2 {
    /// Root block of the FooD2 driver
    
    #[derive(Debug)]
    pub struct FooD2<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD2<I> {
        /// Create a new instance of the block based on device interface
        pub const fn new(interface: I) -> Self {
            Self {
                interface,
                base_address: 0,
            }
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub async fn read_all_registers_async(
            &mut self,
            mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
        ) -> Result<(), I::Error>
        where
            I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
        {
            Ok(())
        }
    }
    
    /// Module containing the generated fieldsets of the registers and commands
    pub mod field_sets {
        #[allow(unused_imports)]
        use super::*;
    
        /// Enum containing all possible field set types
        pub enum FieldSetValue {}
        impl core::fmt::Debug for FieldSetValue {
            fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    }
}

pub mod foo_d_3 {
    /// Root block of the FooD3 driver
    
    #[derive(Debug)]
    pub struct FooD3<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD3<I> {
        /// Create a new instance of the block based on device interface
        pub const fn new(interface: I) -> Self {
            Self {
                interface,
                base_address: 0,
            }
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub async fn read_all_registers_async(
            &mut self,
            mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
        ) -> Result<(), I::Error>
        where
            I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
        {
            Ok(())
        }
    }
    
    /// Module containing the generated fieldsets of the registers and commands
    pub mod field_sets {
        #[allow(unused_imports)]
        use super::*;
    
        /// Enum containing all possible field set types
        pub enum FieldSetValue {}
        impl core::fmt::Debug for FieldSetValue {
            fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    }
}

pub mod foo_d_4 {
    /// Root block of the FooD4 driver
    
    #[derive(Debug)]
    pub struct FooD4<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD4<I> {
        /// Create a new instance of the block based on device interface
        pub const fn new(interface: I) -> Self {
            Self {
                interface,
                base_address: 0,
            }
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub async fn read_all_registers_async(
            &mut self,
            mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
        ) -> Result<(), I::Error>
        where
            I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
        {
            Ok(())
        }
    }
    
    /// Module containing the generated fieldsets of the registers and commands
    pub mod field_sets {
        #[allow(unused_imports)]
        use super::*;
    
        /// Enum containing all possible field set types
        pub enum FieldSetValue {}
        impl core::fmt::Debug for FieldSetValue {
            fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    }
}

pub mod foo_d_5 {
    /// Root block of the FooD5 driver
    
    #[derive(Debug)]
    pub struct FooD5<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD5<I> {
        /// Create a new instance of the block based on device interface
        pub const fn new(interface: I) -> Self {
            Self {
                interface,
                base_address: 0,
            }
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub async fn read_all_registers_async(
            &mut self,
            mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
        ) -> Result<(), I::Error>
        where
            I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
        {
            Ok(())
        }
    }
    
    /// Module containing the generated fieldsets of the registers and commands
    pub mod field_sets {
        #[allow(unused_imports)]
        use super::*;
    
        /// Enum containing all possible field set types
        pub enum FieldSetValue {}
        impl core::fmt::Debug for FieldSetValue {
            fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    
        #[cfg(feature = "lol")]
        impl defmt::Format for FieldSetValue {
            fn format(&self, f: defmt::Formatter) {
                match self {}
            }
        }
    }
}

pub mod foo_d_6 {
    /// Root block of the FooD6 driver
    
    #[derive(Debug)]
    pub struct FooD6<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD6<I> {
        /// Create a new instance of the block based on device interface
        pub const fn new(interface: I) -> Self {
            Self {
                interface,
                base_address: 0,
            }
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub fn read_all_registers(
            &mut self,
            mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
        ) -> Result<(), I::Error>
        where
            I: ::device_driver::RegisterInterface<AddressType = u8>,
        {
            let reg = self.foor_4().read()?;
    
            callback(0 + 0 * 0, "foor_4", reg.into());
    
            let reg = self.foor_5().read()?;
    
            callback(1 + 0 * 0, "foor_5", reg.into());
    
            let reg = self.foor_6().read()?;
    
            callback(2 + 0 * 0, "foor_6", reg.into());
    
            let reg = self.foor_7().read()?;
    
            callback(3 + 0 * 0, "foor_7", reg.into());
    
            let reg = self.foor_8().read()?;
    
            callback(4 + 0 * 0, "foor_8", reg.into());
    
            let reg = self.foor_9().read()?;
    
            callback(5 + 0 * 0, "foor_9", reg.into());
    
            let reg = self.foor_10().read()?;
    
            callback(6 + 0 * 0, "foor_10", reg.into());
    
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub async fn read_all_registers_async(
            &mut self,
            mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
        ) -> Result<(), I::Error>
        where
            I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
        {
            let reg = self.foor_4().read_async().await?;
    
            callback(0 + 0 * 0, "foor_4", reg.into());
    
            let reg = self.foor_5().read_async().await?;
    
            callback(1 + 0 * 0, "foor_5", reg.into());
    
            let reg = self.foor_6().read_async().await?;
    
            callback(2 + 0 * 0, "foor_6", reg.into());
    
            let reg = self.foor_7().read_async().await?;
    
            callback(3 + 0 * 0, "foor_7", reg.into());
    
            let reg = self.foor_8().read_async().await?;
    
            callback(4 + 0 * 0, "foor_8", reg.into());
    
            let reg = self.foor_9().read_async().await?;
    
            callback(5 + 0 * 0, "foor_9", reg.into());
    
            let reg = self.foor_10().read_async().await?;
    
            callback(6 + 0 * 0, "foor_10", reg.into());
    
            Ok(())
        }
    
        pub fn foor_4(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::Foor4, ::device_driver::RW> {
            let address = self.base_address + 0;
    
            ::device_driver::RegisterOperation::<'_, I, u8, field_sets::Foor4, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                field_sets::Foor4::new,
            )
        }
    
        pub fn foor_5(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::Foor5, ::device_driver::RW> {
            let address = self.base_address + 1;
    
            ::device_driver::RegisterOperation::<'_, I, u8, field_sets::Foor5, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                field_sets::Foor5::new,
            )
        }
    
        pub fn foor_6(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::Foor6, ::device_driver::RW> {
            let address = self.base_address + 2;
    
            ::device_driver::RegisterOperation::<'_, I, u8, field_sets::Foor6, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                field_sets::Foor6::new,
            )
        }
    
        pub fn foor_7(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::Foor7, ::device_driver::RW> {
            let address = self.base_address + 3;
    
            ::device_driver::RegisterOperation::<'_, I, u8, field_sets::Foor7, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                field_sets::Foor7::new,
            )
        }
    
        pub fn foor_8(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::Foor8, ::device_driver::RW> {
            let address = self.base_address + 4;
    
            ::device_driver::RegisterOperation::<'_, I, u8, field_sets::Foor8, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                field_sets::Foor8::new,
            )
        }
    
        pub fn foor_9(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::Foor9, ::device_driver::RW> {
            let address = self.base_address + 5;
    
            ::device_driver::RegisterOperation::<'_, I, u8, field_sets::Foor9, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                field_sets::Foor9::new,
            )
        }
    
        pub fn foor_10(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::Foor10, ::device_driver::RW>
        {
            let address = self.base_address + 6;
    
            ::device_driver::RegisterOperation::<
                            '_,
                            I,
                            u8,
                            field_sets::Foor10,
                            ::device_driver::RW,
                        >::new(self.interface(), address as u8, field_sets::Foor10::new)
        }
    
        pub fn fooc_1(
            &mut self,
        ) -> ::device_driver::CommandOperation<
            '_,
            I,
            u8,
            field_sets::Fooc1FieldsIn,
            field_sets::Fooc1FieldsOut,
        > {
            let address = self.base_address + 0;
    
            ::device_driver::CommandOperation::<
                '_,
                I,
                u8,
                field_sets::Fooc1FieldsIn,
                field_sets::Fooc1FieldsOut,
            >::new(self.interface(), address as u8)
        }
    
        pub fn foob_1(&mut self) -> ::device_driver::BufferOperation<'_, I, u8, ::device_driver::RW> {
            let address = self.base_address + 0;
    
            ::device_driver::BufferOperation::<'_, I, u8, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
            )
        }
    
        pub fn foob_2(&mut self) -> ::device_driver::BufferOperation<'_, I, u8, ::device_driver::RO> {
            let address = self.base_address + 2;
    
            ::device_driver::BufferOperation::<'_, I, u8, ::device_driver::RO>::new(
                self.interface(),
                address as u8,
            )
        }
    
        pub fn b_1(&mut self) -> B1<'_, I> {
            let address = self.base_address + 5;
    
            B1::<'_, I>::new(self.interface(), address)
        }
    
        ///
        /// Valid index range: 0..2
    
        pub fn b_2(&mut self, index: usize) -> B2<'_, I> {
            let address = {
                assert!(index < 2);
                self.base_address + 0 + index as u8 * 4
            };
    
            B2::<'_, I>::new(self.interface(), address)
        }
    }
    
    #[derive(Debug)]
    pub struct B1<'i, I> {
        pub(crate) interface: &'i mut I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<'i, I> B1<'i, I> {
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub async fn read_all_registers_async(
            &mut self,
            mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
        ) -> Result<(), I::Error>
        where
            I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
        {
            Ok(())
        }
    }
    
    #[derive(Debug)]
    pub struct B2<'i, I> {
        pub(crate) interface: &'i mut I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<'i, I> B2<'i, I> {
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
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
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        pub async fn read_all_registers_async(
            &mut self,
            mut callback: impl FnMut(u8, &'static str, field_sets::FieldSetValue),
        ) -> Result<(), I::Error>
        where
            I: ::device_driver::AsyncRegisterInterface<AddressType = u8>,
        {
            Ok(())
        }
    
        pub fn b_2_foo(&mut self) -> ::device_driver::BufferOperation<'_, I, u8, ::device_driver::RW> {
            let address = self.base_address + 42;
    
            ::device_driver::BufferOperation::<'_, I, u8, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
            )
        }
    }
    
    /// Module containing the generated fieldsets of the registers and commands
    pub mod field_sets {
        #[allow(unused_imports)]
        use super::*;
    
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct Foor4 {
            /// The internal bits
            bits: [u8; 0],
        }
    
        impl ::device_driver::FieldSet for Foor4 {
            const SIZE_BITS: u32 = 0;
            fn new_with_zero() -> Self {
                Self::new_zero()
            }
            fn get_inner_buffer(&self) -> &[u8] {
                &self.bits
            }
            fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
                &mut self.bits
            }
        }
    
        impl Foor4 {
            /// Create a new instance, loaded with the reset value (if any)
            pub const fn new() -> Self {
                Self { bits: [] }
            }
            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {
                Self { bits: [0; 0] }
            }
        }
    
        impl From<[u8; 0]> for Foor4 {
            fn from(bits: [u8; 0]) -> Self {
                Self { bits }
            }
        }
    
        impl From<Foor4> for [u8; 0] {
            fn from(val: Foor4) -> Self {
                val.bits
            }
        }
    
        impl core::fmt::Debug for Foor4 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                let mut d = f.debug_struct("Foor4");
    
                d.finish()
            }
        }
    
        impl core::ops::BitAnd for Foor4 {
            type Output = Self;
            fn bitand(mut self, rhs: Self) -> Self::Output {
                self &= rhs;
                self
            }
        }
    
        impl core::ops::BitAndAssign for Foor4 {
            fn bitand_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l &= *r;
                }
            }
        }
    
        impl core::ops::BitOr for Foor4 {
            type Output = Self;
            fn bitor(mut self, rhs: Self) -> Self::Output {
                self |= rhs;
                self
            }
        }
    
        impl core::ops::BitOrAssign for Foor4 {
            fn bitor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l |= *r;
                }
            }
        }
    
        impl core::ops::BitXor for Foor4 {
            type Output = Self;
            fn bitxor(mut self, rhs: Self) -> Self::Output {
                self ^= rhs;
                self
            }
        }
    
        impl core::ops::BitXorAssign for Foor4 {
            fn bitxor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l ^= *r;
                }
            }
        }
    
        impl core::ops::Not for Foor4 {
            type Output = Self;
            fn not(mut self) -> Self::Output {
                for val in self.bits.iter_mut() {
                    *val = !*val;
                }
                self
            }
        }
    
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct Foor5 {
            /// The internal bits
            bits: [u8; 0],
        }
    
        impl ::device_driver::FieldSet for Foor5 {
            const SIZE_BITS: u32 = 0;
            fn new_with_zero() -> Self {
                Self::new_zero()
            }
            fn get_inner_buffer(&self) -> &[u8] {
                &self.bits
            }
            fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
                &mut self.bits
            }
        }
    
        impl Foor5 {
            /// Create a new instance, loaded with the reset value (if any)
            pub const fn new() -> Self {
                Self { bits: [] }
            }
            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {
                Self { bits: [0; 0] }
            }
        }
    
        impl From<[u8; 0]> for Foor5 {
            fn from(bits: [u8; 0]) -> Self {
                Self { bits }
            }
        }
    
        impl From<Foor5> for [u8; 0] {
            fn from(val: Foor5) -> Self {
                val.bits
            }
        }
    
        impl core::fmt::Debug for Foor5 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                let mut d = f.debug_struct("Foor5");
    
                d.finish()
            }
        }
    
        impl core::ops::BitAnd for Foor5 {
            type Output = Self;
            fn bitand(mut self, rhs: Self) -> Self::Output {
                self &= rhs;
                self
            }
        }
    
        impl core::ops::BitAndAssign for Foor5 {
            fn bitand_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l &= *r;
                }
            }
        }
    
        impl core::ops::BitOr for Foor5 {
            type Output = Self;
            fn bitor(mut self, rhs: Self) -> Self::Output {
                self |= rhs;
                self
            }
        }
    
        impl core::ops::BitOrAssign for Foor5 {
            fn bitor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l |= *r;
                }
            }
        }
    
        impl core::ops::BitXor for Foor5 {
            type Output = Self;
            fn bitxor(mut self, rhs: Self) -> Self::Output {
                self ^= rhs;
                self
            }
        }
    
        impl core::ops::BitXorAssign for Foor5 {
            fn bitxor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l ^= *r;
                }
            }
        }
    
        impl core::ops::Not for Foor5 {
            type Output = Self;
            fn not(mut self) -> Self::Output {
                for val in self.bits.iter_mut() {
                    *val = !*val;
                }
                self
            }
        }
    
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct Foor6 {
            /// The internal bits
            bits: [u8; 0],
        }
    
        impl ::device_driver::FieldSet for Foor6 {
            const SIZE_BITS: u32 = 0;
            fn new_with_zero() -> Self {
                Self::new_zero()
            }
            fn get_inner_buffer(&self) -> &[u8] {
                &self.bits
            }
            fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
                &mut self.bits
            }
        }
    
        impl Foor6 {
            /// Create a new instance, loaded with the reset value (if any)
            pub const fn new() -> Self {
                Self { bits: [] }
            }
            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {
                Self { bits: [0; 0] }
            }
        }
    
        impl From<[u8; 0]> for Foor6 {
            fn from(bits: [u8; 0]) -> Self {
                Self { bits }
            }
        }
    
        impl From<Foor6> for [u8; 0] {
            fn from(val: Foor6) -> Self {
                val.bits
            }
        }
    
        impl core::fmt::Debug for Foor6 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                let mut d = f.debug_struct("Foor6");
    
                d.finish()
            }
        }
    
        impl core::ops::BitAnd for Foor6 {
            type Output = Self;
            fn bitand(mut self, rhs: Self) -> Self::Output {
                self &= rhs;
                self
            }
        }
    
        impl core::ops::BitAndAssign for Foor6 {
            fn bitand_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l &= *r;
                }
            }
        }
    
        impl core::ops::BitOr for Foor6 {
            type Output = Self;
            fn bitor(mut self, rhs: Self) -> Self::Output {
                self |= rhs;
                self
            }
        }
    
        impl core::ops::BitOrAssign for Foor6 {
            fn bitor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l |= *r;
                }
            }
        }
    
        impl core::ops::BitXor for Foor6 {
            type Output = Self;
            fn bitxor(mut self, rhs: Self) -> Self::Output {
                self ^= rhs;
                self
            }
        }
    
        impl core::ops::BitXorAssign for Foor6 {
            fn bitxor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l ^= *r;
                }
            }
        }
    
        impl core::ops::Not for Foor6 {
            type Output = Self;
            fn not(mut self) -> Self::Output {
                for val in self.bits.iter_mut() {
                    *val = !*val;
                }
                self
            }
        }
    
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct Foor7 {
            /// The internal bits
            bits: [u8; 0],
        }
    
        impl ::device_driver::FieldSet for Foor7 {
            const SIZE_BITS: u32 = 0;
            fn new_with_zero() -> Self {
                Self::new_zero()
            }
            fn get_inner_buffer(&self) -> &[u8] {
                &self.bits
            }
            fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
                &mut self.bits
            }
        }
    
        impl Foor7 {
            /// Create a new instance, loaded with the reset value (if any)
            pub const fn new() -> Self {
                Self { bits: [] }
            }
            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {
                Self { bits: [0; 0] }
            }
        }
    
        impl From<[u8; 0]> for Foor7 {
            fn from(bits: [u8; 0]) -> Self {
                Self { bits }
            }
        }
    
        impl From<Foor7> for [u8; 0] {
            fn from(val: Foor7) -> Self {
                val.bits
            }
        }
    
        impl core::fmt::Debug for Foor7 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                let mut d = f.debug_struct("Foor7");
    
                d.finish()
            }
        }
    
        impl core::ops::BitAnd for Foor7 {
            type Output = Self;
            fn bitand(mut self, rhs: Self) -> Self::Output {
                self &= rhs;
                self
            }
        }
    
        impl core::ops::BitAndAssign for Foor7 {
            fn bitand_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l &= *r;
                }
            }
        }
    
        impl core::ops::BitOr for Foor7 {
            type Output = Self;
            fn bitor(mut self, rhs: Self) -> Self::Output {
                self |= rhs;
                self
            }
        }
    
        impl core::ops::BitOrAssign for Foor7 {
            fn bitor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l |= *r;
                }
            }
        }
    
        impl core::ops::BitXor for Foor7 {
            type Output = Self;
            fn bitxor(mut self, rhs: Self) -> Self::Output {
                self ^= rhs;
                self
            }
        }
    
        impl core::ops::BitXorAssign for Foor7 {
            fn bitxor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l ^= *r;
                }
            }
        }
    
        impl core::ops::Not for Foor7 {
            type Output = Self;
            fn not(mut self) -> Self::Output {
                for val in self.bits.iter_mut() {
                    *val = !*val;
                }
                self
            }
        }
    
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct Foor8 {
            /// The internal bits
            bits: [u8; 1],
        }
    
        impl ::device_driver::FieldSet for Foor8 {
            const SIZE_BITS: u32 = 8;
            fn new_with_zero() -> Self {
                Self::new_zero()
            }
            fn get_inner_buffer(&self) -> &[u8] {
                &self.bits
            }
            fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
                &mut self.bits
            }
        }
    
        impl Foor8 {
            /// Create a new instance, loaded with the reset value (if any)
            pub const fn new() -> Self {
                Self { bits: [0] }
            }
            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {
                Self { bits: [0; 1] }
            }
    
            ///Read the `bar` field of the register.
            ///
    
            pub fn bar(&self) -> bool {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 3, 4)
                };
    
                raw > 0
            }
    
            ///Read the `baz` field of the register.
            ///
    
            pub fn baz(&self) -> u8 {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 4, 8)
                };
    
                raw
            }
    
            ///Write the `bar` field of the register.
            ///
    
            pub fn set_bar(&mut self, value: bool) {
                let raw = value as _;
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        3,
                        4,
                        &mut self.bits,
                    )
                };
            }
    
            ///Write the `baz` field of the register.
            ///
    
            pub fn set_baz(&mut self, value: u8) {
                let raw = value;
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        4,
                        8,
                        &mut self.bits,
                    )
                };
            }
        }
    
        impl From<[u8; 1]> for Foor8 {
            fn from(bits: [u8; 1]) -> Self {
                Self { bits }
            }
        }
    
        impl From<Foor8> for [u8; 1] {
            fn from(val: Foor8) -> Self {
                val.bits
            }
        }
    
        impl core::fmt::Debug for Foor8 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                let mut d = f.debug_struct("Foor8");
    
                d.field("bar", &self.bar());
    
                d.field("baz", &self.baz());
    
                d.finish()
            }
        }
    
        impl core::ops::BitAnd for Foor8 {
            type Output = Self;
            fn bitand(mut self, rhs: Self) -> Self::Output {
                self &= rhs;
                self
            }
        }
    
        impl core::ops::BitAndAssign for Foor8 {
            fn bitand_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l &= *r;
                }
            }
        }
    
        impl core::ops::BitOr for Foor8 {
            type Output = Self;
            fn bitor(mut self, rhs: Self) -> Self::Output {
                self |= rhs;
                self
            }
        }
    
        impl core::ops::BitOrAssign for Foor8 {
            fn bitor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l |= *r;
                }
            }
        }
    
        impl core::ops::BitXor for Foor8 {
            type Output = Self;
            fn bitxor(mut self, rhs: Self) -> Self::Output {
                self ^= rhs;
                self
            }
        }
    
        impl core::ops::BitXorAssign for Foor8 {
            fn bitxor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l ^= *r;
                }
            }
        }
    
        impl core::ops::Not for Foor8 {
            type Output = Self;
            fn not(mut self) -> Self::Output {
                for val in self.bits.iter_mut() {
                    *val = !*val;
                }
                self
            }
        }
    
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct Foor9 {
            /// The internal bits
            bits: [u8; 1],
        }
    
        impl ::device_driver::FieldSet for Foor9 {
            const SIZE_BITS: u32 = 8;
            fn new_with_zero() -> Self {
                Self::new_zero()
            }
            fn get_inner_buffer(&self) -> &[u8] {
                &self.bits
            }
            fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
                &mut self.bits
            }
        }
    
        impl Foor9 {
            /// Create a new instance, loaded with the reset value (if any)
            pub const fn new() -> Self {
                Self { bits: [0] }
            }
            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {
                Self { bits: [0; 1] }
            }
    
            ///Read the `bar` field of the register.
            ///
    
            pub fn bar(&self) -> bool {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 0, 1)
                };
    
                raw > 0
            }
    
            ///Read the `baz` field of the register.
            ///
    
            pub fn baz(&self) -> u8 {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 1, 3)
                };
    
                raw
            }
    
            ///Read the `quux` field of the register.
            ///
    
            pub fn quux(&self) -> u8 {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 3, 5)
                };
    
                raw
            }
    
            ///Read the `qus` field of the register.
            ///
    
            pub fn qus(
                &self,
            ) -> Result<super::MyCustomType, <super::MyCustomType as TryFrom<u8>>::Error> {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 5, 7)
                };
    
                raw.try_into()
            }
    
            ///Write the `bar` field of the register.
            ///
    
            pub fn set_bar(&mut self, value: bool) {
                let raw = value as _;
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        0,
                        1,
                        &mut self.bits,
                    )
                };
            }
    
            ///Write the `baz` field of the register.
            ///
    
            pub fn set_baz(&mut self, value: u8) {
                let raw = value;
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        1,
                        3,
                        &mut self.bits,
                    )
                };
            }
    
            ///Write the `quux` field of the register.
            ///
    
            pub fn set_quux(&mut self, value: u8) {
                let raw = value;
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        3,
                        5,
                        &mut self.bits,
                    )
                };
            }
    
            ///Write the `qus` field of the register.
            ///
    
            pub fn set_qus(&mut self, value: super::MyCustomType) {
                let raw = value.into();
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        5,
                        7,
                        &mut self.bits,
                    )
                };
            }
        }
    
        impl From<[u8; 1]> for Foor9 {
            fn from(bits: [u8; 1]) -> Self {
                Self { bits }
            }
        }
    
        impl From<Foor9> for [u8; 1] {
            fn from(val: Foor9) -> Self {
                val.bits
            }
        }
    
        impl core::fmt::Debug for Foor9 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                let mut d = f.debug_struct("Foor9");
    
                d.field("bar", &self.bar());
    
                d.field("baz", &self.baz());
    
                d.field("quux", &self.quux());
    
                d.field("qus", &self.qus());
    
                d.finish()
            }
        }
    
        impl core::ops::BitAnd for Foor9 {
            type Output = Self;
            fn bitand(mut self, rhs: Self) -> Self::Output {
                self &= rhs;
                self
            }
        }
    
        impl core::ops::BitAndAssign for Foor9 {
            fn bitand_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l &= *r;
                }
            }
        }
    
        impl core::ops::BitOr for Foor9 {
            type Output = Self;
            fn bitor(mut self, rhs: Self) -> Self::Output {
                self |= rhs;
                self
            }
        }
    
        impl core::ops::BitOrAssign for Foor9 {
            fn bitor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l |= *r;
                }
            }
        }
    
        impl core::ops::BitXor for Foor9 {
            type Output = Self;
            fn bitxor(mut self, rhs: Self) -> Self::Output {
                self ^= rhs;
                self
            }
        }
    
        impl core::ops::BitXorAssign for Foor9 {
            fn bitxor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l ^= *r;
                }
            }
        }
    
        impl core::ops::Not for Foor9 {
            type Output = Self;
            fn not(mut self) -> Self::Output {
                for val in self.bits.iter_mut() {
                    *val = !*val;
                }
                self
            }
        }
    
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct Foor10 {
            /// The internal bits
            bits: [u8; 1],
        }
    
        impl ::device_driver::FieldSet for Foor10 {
            const SIZE_BITS: u32 = 8;
            fn new_with_zero() -> Self {
                Self::new_zero()
            }
            fn get_inner_buffer(&self) -> &[u8] {
                &self.bits
            }
            fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
                &mut self.bits
            }
        }
    
        impl Foor10 {
            /// Create a new instance, loaded with the reset value (if any)
            pub const fn new() -> Self {
                Self { bits: [0] }
            }
            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {
                Self { bits: [0; 1] }
            }
    
            ///Read the `bar` field of the register.
            ///
    
            pub fn bar(&self) -> super::Foo10E1 {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 0, 2)
                };
    
                unsafe { raw.try_into().unwrap_unchecked() }
            }
    
            ///Read the `baz` field of the register.
            ///
    
            pub fn baz(&self) -> u8 {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 2, 4)
                };
    
                raw
            }
    
            ///Read the `bam` field of the register.
            ///
    
            pub fn bam(&self) -> u8 {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 4, 6)
                };
    
                raw
            }
    
            ///Read the `bat` field of the register.
            ///
    
            pub fn bat(&self) -> Result<super::Foo10E2, <super::Foo10E2 as TryFrom<u8>>::Error> {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 6, 8)
                };
    
                raw.try_into()
            }
    
            ///Write the `bar` field of the register.
            ///
    
            pub fn set_bar(&mut self, value: super::Foo10E1) {
                let raw = value.into();
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        0,
                        2,
                        &mut self.bits,
                    )
                };
            }
    
            ///Write the `baz` field of the register.
            ///
    
            pub fn set_baz(&mut self, value: u8) {
                let raw = value;
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        2,
                        4,
                        &mut self.bits,
                    )
                };
            }
    
            ///Write the `bam` field of the register.
            ///
    
            pub fn set_bam(&mut self, value: u8) {
                let raw = value;
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        4,
                        6,
                        &mut self.bits,
                    )
                };
            }
    
            ///Write the `bat` field of the register.
            ///
    
            pub fn set_bat(&mut self, value: super::Foo10E2) {
                let raw = value.into();
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        6,
                        8,
                        &mut self.bits,
                    )
                };
            }
        }
    
        impl From<[u8; 1]> for Foor10 {
            fn from(bits: [u8; 1]) -> Self {
                Self { bits }
            }
        }
    
        impl From<Foor10> for [u8; 1] {
            fn from(val: Foor10) -> Self {
                val.bits
            }
        }
    
        impl core::fmt::Debug for Foor10 {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                let mut d = f.debug_struct("Foor10");
    
                d.field("bar", &self.bar());
    
                d.field("baz", &self.baz());
    
                d.field("bam", &self.bam());
    
                d.field("bat", &self.bat());
    
                d.finish()
            }
        }
    
        impl core::ops::BitAnd for Foor10 {
            type Output = Self;
            fn bitand(mut self, rhs: Self) -> Self::Output {
                self &= rhs;
                self
            }
        }
    
        impl core::ops::BitAndAssign for Foor10 {
            fn bitand_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l &= *r;
                }
            }
        }
    
        impl core::ops::BitOr for Foor10 {
            type Output = Self;
            fn bitor(mut self, rhs: Self) -> Self::Output {
                self |= rhs;
                self
            }
        }
    
        impl core::ops::BitOrAssign for Foor10 {
            fn bitor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l |= *r;
                }
            }
        }
    
        impl core::ops::BitXor for Foor10 {
            type Output = Self;
            fn bitxor(mut self, rhs: Self) -> Self::Output {
                self ^= rhs;
                self
            }
        }
    
        impl core::ops::BitXorAssign for Foor10 {
            fn bitxor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l ^= *r;
                }
            }
        }
    
        impl core::ops::Not for Foor10 {
            type Output = Self;
            fn not(mut self) -> Self::Output {
                for val in self.bits.iter_mut() {
                    *val = !*val;
                }
                self
            }
        }
    
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct Fooc1FieldsIn {
            /// The internal bits
            bits: [u8; 0],
        }
    
        impl ::device_driver::FieldSet for Fooc1FieldsIn {
            const SIZE_BITS: u32 = 0;
            fn new_with_zero() -> Self {
                Self::new_zero()
            }
            fn get_inner_buffer(&self) -> &[u8] {
                &self.bits
            }
            fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
                &mut self.bits
            }
        }
    
        impl Fooc1FieldsIn {
            /// Create a new instance, loaded with the reset value (if any)
            pub const fn new() -> Self {
                Self { bits: [] }
            }
            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {
                Self { bits: [0; 0] }
            }
        }
    
        impl From<[u8; 0]> for Fooc1FieldsIn {
            fn from(bits: [u8; 0]) -> Self {
                Self { bits }
            }
        }
    
        impl From<Fooc1FieldsIn> for [u8; 0] {
            fn from(val: Fooc1FieldsIn) -> Self {
                val.bits
            }
        }
    
        impl core::fmt::Debug for Fooc1FieldsIn {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                let mut d = f.debug_struct("Fooc1FieldsIn");
    
                d.finish()
            }
        }
    
        impl core::ops::BitAnd for Fooc1FieldsIn {
            type Output = Self;
            fn bitand(mut self, rhs: Self) -> Self::Output {
                self &= rhs;
                self
            }
        }
    
        impl core::ops::BitAndAssign for Fooc1FieldsIn {
            fn bitand_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l &= *r;
                }
            }
        }
    
        impl core::ops::BitOr for Fooc1FieldsIn {
            type Output = Self;
            fn bitor(mut self, rhs: Self) -> Self::Output {
                self |= rhs;
                self
            }
        }
    
        impl core::ops::BitOrAssign for Fooc1FieldsIn {
            fn bitor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l |= *r;
                }
            }
        }
    
        impl core::ops::BitXor for Fooc1FieldsIn {
            type Output = Self;
            fn bitxor(mut self, rhs: Self) -> Self::Output {
                self ^= rhs;
                self
            }
        }
    
        impl core::ops::BitXorAssign for Fooc1FieldsIn {
            fn bitxor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l ^= *r;
                }
            }
        }
    
        impl core::ops::Not for Fooc1FieldsIn {
            type Output = Self;
            fn not(mut self) -> Self::Output {
                for val in self.bits.iter_mut() {
                    *val = !*val;
                }
                self
            }
        }
    
        #[derive(Copy, Clone, Eq, PartialEq)]
        pub struct Fooc1FieldsOut {
            /// The internal bits
            bits: [u8; 1],
        }
    
        impl ::device_driver::FieldSet for Fooc1FieldsOut {
            const SIZE_BITS: u32 = 8;
            fn new_with_zero() -> Self {
                Self::new_zero()
            }
            fn get_inner_buffer(&self) -> &[u8] {
                &self.bits
            }
            fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
                &mut self.bits
            }
        }
    
        impl Fooc1FieldsOut {
            /// Create a new instance, loaded with the reset value (if any)
            pub const fn new() -> Self {
                Self { bits: [0] }
            }
            /// Create a new instance, loaded with all zeroes
            pub const fn new_zero() -> Self {
                Self { bits: [0; 1] }
            }
    
            ///Read the `b` field of the register.
            ///
    
            pub fn b(&self) -> bool {
                let raw = unsafe {
                    ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 0, 1)
                };
    
                raw > 0
            }
    
            ///Write the `b` field of the register.
            ///
    
            pub fn set_b(&mut self, value: bool) {
                let raw = value as _;
    
                unsafe {
                    ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                        raw,
                        0,
                        1,
                        &mut self.bits,
                    )
                };
            }
        }
    
        impl From<[u8; 1]> for Fooc1FieldsOut {
            fn from(bits: [u8; 1]) -> Self {
                Self { bits }
            }
        }
    
        impl From<Fooc1FieldsOut> for [u8; 1] {
            fn from(val: Fooc1FieldsOut) -> Self {
                val.bits
            }
        }
    
        impl core::fmt::Debug for Fooc1FieldsOut {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                let mut d = f.debug_struct("Fooc1FieldsOut");
    
                d.field("b", &self.b());
    
                d.finish()
            }
        }
    
        impl core::ops::BitAnd for Fooc1FieldsOut {
            type Output = Self;
            fn bitand(mut self, rhs: Self) -> Self::Output {
                self &= rhs;
                self
            }
        }
    
        impl core::ops::BitAndAssign for Fooc1FieldsOut {
            fn bitand_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l &= *r;
                }
            }
        }
    
        impl core::ops::BitOr for Fooc1FieldsOut {
            type Output = Self;
            fn bitor(mut self, rhs: Self) -> Self::Output {
                self |= rhs;
                self
            }
        }
    
        impl core::ops::BitOrAssign for Fooc1FieldsOut {
            fn bitor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l |= *r;
                }
            }
        }
    
        impl core::ops::BitXor for Fooc1FieldsOut {
            type Output = Self;
            fn bitxor(mut self, rhs: Self) -> Self::Output {
                self ^= rhs;
                self
            }
        }
    
        impl core::ops::BitXorAssign for Fooc1FieldsOut {
            fn bitxor_assign(&mut self, rhs: Self) {
                for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                    *l ^= *r;
                }
            }
        }
    
        impl core::ops::Not for Fooc1FieldsOut {
            type Output = Self;
            fn not(mut self) -> Self::Output {
                for val in self.bits.iter_mut() {
                    *val = !*val;
                }
                self
            }
        }
    
        /// Enum containing all possible field set types
        pub enum FieldSetValue {
            Foor4(Foor4),
    
            Foor5(Foor5),
    
            Foor6(Foor6),
    
            Foor7(Foor7),
    
            Foor8(Foor8),
    
            Foor9(Foor9),
    
            Foor10(Foor10),
    
            Fooc1FieldsIn(Fooc1FieldsIn),
    
            Fooc1FieldsOut(Fooc1FieldsOut),
        }
        impl core::fmt::Debug for FieldSetValue {
            fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    Self::Foor4(val) => core::fmt::Debug::fmt(val, _f),
    
                    Self::Foor5(val) => core::fmt::Debug::fmt(val, _f),
    
                    Self::Foor6(val) => core::fmt::Debug::fmt(val, _f),
    
                    Self::Foor7(val) => core::fmt::Debug::fmt(val, _f),
    
                    Self::Foor8(val) => core::fmt::Debug::fmt(val, _f),
    
                    Self::Foor9(val) => core::fmt::Debug::fmt(val, _f),
    
                    Self::Foor10(val) => core::fmt::Debug::fmt(val, _f),
    
                    Self::Fooc1FieldsIn(val) => core::fmt::Debug::fmt(val, _f),
    
                    Self::Fooc1FieldsOut(val) => core::fmt::Debug::fmt(val, _f),
    
                    #[allow(unreachable_patterns)]
                    _ => unreachable!(),
                }
            }
        }
    
        impl From<Foor4> for FieldSetValue {
            fn from(val: Foor4) -> Self {
                Self::Foor4(val)
            }
        }
    
        impl From<Foor5> for FieldSetValue {
            fn from(val: Foor5) -> Self {
                Self::Foor5(val)
            }
        }
    
        impl From<Foor6> for FieldSetValue {
            fn from(val: Foor6) -> Self {
                Self::Foor6(val)
            }
        }
    
        impl From<Foor7> for FieldSetValue {
            fn from(val: Foor7) -> Self {
                Self::Foor7(val)
            }
        }
    
        impl From<Foor8> for FieldSetValue {
            fn from(val: Foor8) -> Self {
                Self::Foor8(val)
            }
        }
    
        impl From<Foor9> for FieldSetValue {
            fn from(val: Foor9) -> Self {
                Self::Foor9(val)
            }
        }
    
        impl From<Foor10> for FieldSetValue {
            fn from(val: Foor10) -> Self {
                Self::Foor10(val)
            }
        }
    
        impl From<Fooc1FieldsIn> for FieldSetValue {
            fn from(val: Fooc1FieldsIn) -> Self {
                Self::Fooc1FieldsIn(val)
            }
        }
    
        impl From<Fooc1FieldsOut> for FieldSetValue {
            fn from(val: Fooc1FieldsOut) -> Self {
                Self::Fooc1FieldsOut(val)
            }
        }
    }
    
    #[repr(u8)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    
    pub enum Foo10E1 {
        A = 0,
    
        B = 1,
    
        C = 2,
    
        D(u8) = 3,
    }
    
    impl Default for Foo10E1 {
        fn default() -> Self {
            Self::C
        }
    }
    
    impl From<u8> for Foo10E1 {
        fn from(val: u8) -> Self {
            match val {
                0 => Self::A,
    
                1 => Self::B,
    
                val => Self::D(val),
            }
        }
    }
    
    impl From<Foo10E1> for u8 {
        fn from(val: Foo10E1) -> Self {
            match val {
                Foo10E1::A => 0,
    
                Foo10E1::B => 1,
    
                Foo10E1::C => 2,
    
                Foo10E1::D(num) => num,
            }
        }
    }
    
    #[repr(u8)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    
    pub enum Foo10E2 {
        A = 0,
    }
    
    impl core::convert::TryFrom<u8> for Foo10E2 {
        type Error = ::device_driver::ConversionError<u8>;
        fn try_from(val: u8) -> Result<Self, Self::Error> {
            match val {
                0 => Ok(Self::A),
    
                val => Err(::device_driver::ConversionError {
                    source: val,
                    target: "Foo10E2",
                }),
            }
        }
    }
    
    impl From<Foo10E2> for u8 {
        fn from(val: Foo10E2) -> Self {
            match val {
                Foo10E2::A => 0,
            }
        }
    }
}

compile_error!("The device driver input has errors that need to be solved!");
