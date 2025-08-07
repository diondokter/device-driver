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
        let reg = self.foo_ro().read()?;

        callback(0 + 0 * 0, "foo_ro", reg.into());

        let reg = self.foo_rw().read()?;

        callback(1 + 0 * 0, "foo_rw", reg.into());

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
        let reg = self.foo_ro().read_async().await?;

        callback(0 + 0 * 0, "foo_ro", reg.into());

        let reg = self.foo_rw().read_async().await?;

        callback(1 + 0 * 0, "foo_rw", reg.into());

        Ok(())
    }

    pub fn foo_ro(
        &mut self,
    ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::FooRo, ::device_driver::RO> {
        let address = self.base_address + 0;

        ::device_driver::RegisterOperation::<'_, I, u8, field_sets::FooRo, ::device_driver::RO>::new(
            self.interface(),
            address as u8,
            field_sets::FooRo::new,
        )
    }

    pub fn foo_rw(
        &mut self,
    ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::FooRw, ::device_driver::RW> {
        let address = self.base_address + 1;

        ::device_driver::RegisterOperation::<'_, I, u8, field_sets::FooRw, ::device_driver::RW>::new(
            self.interface(),
            address as u8,
            field_sets::FooRw::new,
        )
    }

    pub fn foo_wo(
        &mut self,
    ) -> ::device_driver::RegisterOperation<'_, I, u8, field_sets::FooWo, ::device_driver::WO> {
        let address = self.base_address + 2;

        ::device_driver::RegisterOperation::<'_, I, u8, field_sets::FooWo, ::device_driver::WO>::new(
            self.interface(),
            address as u8,
            field_sets::FooWo::new,
        )
    }
}

/// Module containing the generated fieldsets of the registers and commands
pub mod field_sets {
    #[allow(unused_imports)]
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct FooRo {
        /// The internal bits
        bits: [u8; 8],
    }

    impl ::device_driver::FieldSet for FooRo {
        const SIZE_BITS: u32 = 64;
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

    impl FooRo {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self {
                bits: [0, 0, 0, 0, 0, 0, 0, 0],
            }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 8] }
        }

        ///Read the `value_ro` field of the register.
        ///

        pub fn value_ro(&self) -> u16 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u16, ::device_driver::ops::LE>(&self.bits, 0, 16)
            };

            raw
        }

        ///Read the `value_rw` field of the register.
        ///

        pub fn value_rw(&self) -> i16 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<i16, ::device_driver::ops::LE>(&self.bits, 16, 32)
            };

            raw
        }

        ///Write the `value_rw` field of the register.
        ///

        pub fn set_value_rw(&mut self, value: i16) {
            let raw = value;

            unsafe {
                ::device_driver::ops::store_lsb0::<i16, ::device_driver::ops::LE>(
                    raw,
                    16,
                    32,
                    &mut self.bits,
                )
            };
        }

        ///Write the `value_wo` field of the register.
        ///

        pub fn set_value_wo(&mut self, value: bool) {
            let raw = value as _;

            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    32,
                    33,
                    &mut self.bits,
                )
            };
        }
    }

    impl From<[u8; 8]> for FooRo {
        fn from(bits: [u8; 8]) -> Self {
            Self { bits }
        }
    }

    impl From<FooRo> for [u8; 8] {
        fn from(val: FooRo) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for FooRo {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("FooRo");

            d.field("value_ro", &self.value_ro());

            d.field("value_rw", &self.value_rw());

            d.finish()
        }
    }

    impl core::ops::BitAnd for FooRo {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for FooRo {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for FooRo {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for FooRo {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for FooRo {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for FooRo {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for FooRo {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct FooRw {
        /// The internal bits
        bits: [u8; 8],
    }

    impl ::device_driver::FieldSet for FooRw {
        const SIZE_BITS: u32 = 64;
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

    impl FooRw {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self {
                bits: [0, 0, 0, 0, 0, 0, 0, 0],
            }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 8] }
        }

        ///Read the `value_ro` field of the register.
        ///

        pub fn value_ro(&self) -> u16 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u16, ::device_driver::ops::LE>(&self.bits, 0, 16)
            };

            raw
        }

        ///Read the `value_rw` field of the register.
        ///

        pub fn value_rw(&self) -> i16 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<i16, ::device_driver::ops::LE>(&self.bits, 16, 32)
            };

            raw
        }

        ///Write the `value_rw` field of the register.
        ///

        pub fn set_value_rw(&mut self, value: i16) {
            let raw = value;

            unsafe {
                ::device_driver::ops::store_lsb0::<i16, ::device_driver::ops::LE>(
                    raw,
                    16,
                    32,
                    &mut self.bits,
                )
            };
        }

        ///Write the `value_wo` field of the register.
        ///

        pub fn set_value_wo(&mut self, value: bool) {
            let raw = value as _;

            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    32,
                    33,
                    &mut self.bits,
                )
            };
        }
    }

    impl From<[u8; 8]> for FooRw {
        fn from(bits: [u8; 8]) -> Self {
            Self { bits }
        }
    }

    impl From<FooRw> for [u8; 8] {
        fn from(val: FooRw) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for FooRw {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("FooRw");

            d.field("value_ro", &self.value_ro());

            d.field("value_rw", &self.value_rw());

            d.finish()
        }
    }

    impl core::ops::BitAnd for FooRw {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for FooRw {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for FooRw {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for FooRw {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for FooRw {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for FooRw {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for FooRw {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct FooWo {
        /// The internal bits
        bits: [u8; 8],
    }

    impl ::device_driver::FieldSet for FooWo {
        const SIZE_BITS: u32 = 64;
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

    impl FooWo {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self {
                bits: [0, 0, 0, 0, 0, 0, 0, 0],
            }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 8] }
        }

        ///Read the `value_ro` field of the register.
        ///

        pub fn value_ro(&self) -> u16 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u16, ::device_driver::ops::LE>(&self.bits, 0, 16)
            };

            raw
        }

        ///Read the `value_rw` field of the register.
        ///

        pub fn value_rw(&self) -> i16 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<i16, ::device_driver::ops::LE>(&self.bits, 16, 32)
            };

            raw
        }

        ///Write the `value_rw` field of the register.
        ///

        pub fn set_value_rw(&mut self, value: i16) {
            let raw = value;

            unsafe {
                ::device_driver::ops::store_lsb0::<i16, ::device_driver::ops::LE>(
                    raw,
                    16,
                    32,
                    &mut self.bits,
                )
            };
        }

        ///Write the `value_wo` field of the register.
        ///

        pub fn set_value_wo(&mut self, value: bool) {
            let raw = value as _;

            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    32,
                    33,
                    &mut self.bits,
                )
            };
        }
    }

    impl From<[u8; 8]> for FooWo {
        fn from(bits: [u8; 8]) -> Self {
            Self { bits }
        }
    }

    impl From<FooWo> for [u8; 8] {
        fn from(val: FooWo) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for FooWo {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("FooWo");

            d.field("value_ro", &self.value_ro());

            d.field("value_rw", &self.value_rw());

            d.finish()
        }
    }

    impl core::ops::BitAnd for FooWo {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for FooWo {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for FooWo {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for FooWo {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for FooWo {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for FooWo {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for FooWo {
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
        FooRo(FooRo),

        FooRw(FooRw),

        FooWo(FooWo),
    }
    impl core::fmt::Debug for FieldSetValue {
        fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::FooRo(val) => core::fmt::Debug::fmt(val, _f),

                Self::FooRw(val) => core::fmt::Debug::fmt(val, _f),

                Self::FooWo(val) => core::fmt::Debug::fmt(val, _f),

                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }
    }

    impl From<FooRo> for FieldSetValue {
        fn from(val: FooRo) -> Self {
            Self::FooRo(val)
        }
    }

    impl From<FooRw> for FieldSetValue {
        fn from(val: FooRw) -> Self {
            Self::FooRw(val)
        }
    }

    impl From<FooWo> for FieldSetValue {
        fn from(val: FooWo) -> Self {
            Self::FooWo(val)
        }
    }
}
