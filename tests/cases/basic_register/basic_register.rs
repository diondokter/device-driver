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

/// Root block of the Device driver
#[derive(Debug)]
pub struct Device<I> {
    pub(crate) interface: I,

    #[doc(hidden)]
    base_address: u8,
}

impl<I> Device<I> {
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
        let reg = self.foo().read()?;
        callback(0 + 0 * 0, "foo", reg.into());

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
        let reg = self.foo().read_async().await?;
        callback(0 + 0 * 0, "foo", reg.into());

        Ok(())
    }

    pub fn foo(
        &mut self,
    ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::Foo, ::device_driver::RW> {
        let address = self.base_address + 0;

        ::device_driver::RegisterOperation::<'_, I, u8, field_sets::Foo, ::device_driver::RW>::new(
            self.interface(),
            address as u8,
            field_sets::Foo::new,
        )
    }
}

/// Module containing the generated fieldsets of the registers and commands
pub mod field_sets {
    #[allow(unused_imports)]
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo {
        /// The internal bits
        bits: [u8; 3],
    }

    impl ::device_driver::FieldSet for Foo {
        const SIZE_BITS: u32 = 24;
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

    impl Foo {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [0, 0, 0] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 3] }
        }

        ///Read the `value` field of the register.
        ///

        pub fn value(&self) -> u32 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u32, ::device_driver::ops::LE>(&self.bits, 0, 24)
            };

            raw
        }

        ///Write the `value` field of the register.
        ///

        pub fn set_value(&mut self, value: u32) {
            let raw = value;

            unsafe {
                ::device_driver::ops::store_lsb0::<u32, ::device_driver::ops::LE>(
                    raw,
                    0,
                    24,
                    &mut self.bits,
                )
            };
        }
    }

    impl From<[u8; 3]> for Foo {
        fn from(bits: [u8; 3]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo> for [u8; 3] {
        fn from(val: Foo) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo");

            d.field("value", &self.value());

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Foo {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Foo {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Foo {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Foo {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Foo {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Foo {
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
        Foo(Foo),
    }
    impl core::fmt::Debug for FieldSetValue {
        fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::Foo(val) => core::fmt::Debug::fmt(val, _f),

                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }
    }

    impl From<Foo> for FieldSetValue {
        fn from(val: Foo) -> Self {
            Self::Foo(val)
        }
    }
}
