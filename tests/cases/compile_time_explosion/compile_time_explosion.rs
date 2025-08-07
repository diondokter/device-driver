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
    base_address: u32,
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
        mut callback: impl FnMut(u32, &'static str, field_sets::FieldSetValue),
    ) -> Result<(), I::Error>
    where
        I: ::device_driver::RegisterInterface<AddressType = u32>,
    {
        for index in 0..100 {
            let reg = self.foo_0(index).read()?;
            callback(0 + index as u32 * 1000, "foo_0", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_1(index).read()?;
            callback(1 + index as u32 * 1000, "foo_1", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_2(index).read()?;
            callback(2 + index as u32 * 1000, "foo_2", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_3(index).read()?;
            callback(3 + index as u32 * 1000, "foo_3", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_4(index).read()?;
            callback(4 + index as u32 * 1000, "foo_4", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_5(index).read()?;
            callback(5 + index as u32 * 1000, "foo_5", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_6(index).read()?;
            callback(6 + index as u32 * 1000, "foo_6", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_7(index).read()?;
            callback(7 + index as u32 * 1000, "foo_7", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_8(index).read()?;
            callback(8 + index as u32 * 1000, "foo_8", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_9(index).read()?;
            callback(9 + index as u32 * 1000, "foo_9", reg.into());
        }

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
        mut callback: impl FnMut(u32, &'static str, field_sets::FieldSetValue),
    ) -> Result<(), I::Error>
    where
        I: ::device_driver::AsyncRegisterInterface<AddressType = u32>,
    {
        for index in 0..100 {
            let reg = self.foo_0(index).read_async().await?;
            callback(0 + index as u32 * 1000, "foo_0", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_1(index).read_async().await?;
            callback(1 + index as u32 * 1000, "foo_1", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_2(index).read_async().await?;
            callback(2 + index as u32 * 1000, "foo_2", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_3(index).read_async().await?;
            callback(3 + index as u32 * 1000, "foo_3", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_4(index).read_async().await?;
            callback(4 + index as u32 * 1000, "foo_4", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_5(index).read_async().await?;
            callback(5 + index as u32 * 1000, "foo_5", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_6(index).read_async().await?;
            callback(6 + index as u32 * 1000, "foo_6", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_7(index).read_async().await?;
            callback(7 + index as u32 * 1000, "foo_7", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_8(index).read_async().await?;
            callback(8 + index as u32 * 1000, "foo_8", reg.into());
        }

        for index in 0..100 {
            let reg = self.foo_9(index).read_async().await?;
            callback(9 + index as u32 * 1000, "foo_9", reg.into());
        }

        Ok(())
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_0(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, field_sets::Foo0, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 0 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, field_sets::Foo0, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            field_sets::Foo0::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_1(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, field_sets::Foo1, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 1 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, field_sets::Foo1, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            field_sets::Foo1::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_2(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, field_sets::Foo2, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 2 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, field_sets::Foo2, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            field_sets::Foo2::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_3(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, field_sets::Foo3, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 3 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, field_sets::Foo3, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            field_sets::Foo3::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_4(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, field_sets::Foo4, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 4 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, field_sets::Foo4, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            field_sets::Foo4::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_5(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, field_sets::Foo5, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 5 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, field_sets::Foo5, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            field_sets::Foo5::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_6(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, field_sets::Foo6, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 6 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, field_sets::Foo6, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            field_sets::Foo6::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_7(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, field_sets::Foo7, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 7 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, field_sets::Foo7, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            field_sets::Foo7::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_8(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, field_sets::Foo8, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 8 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, field_sets::Foo8, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            field_sets::Foo8::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_9(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, field_sets::Foo9, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 9 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, field_sets::Foo9, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            field_sets::Foo9::new,
        )
    }
}

/// Module containing the generated fieldsets of the registers and commands
pub mod field_sets {
    #[allow(unused_imports)]
    use super::*;

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo0 {
        /// The internal bits
        bits: [u8; 0],
    }

    impl ::device_driver::FieldSet for Foo0 {
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

    impl Foo0 {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 0] }
        }
    }

    impl From<[u8; 0]> for Foo0 {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo0> for [u8; 0] {
        fn from(val: Foo0) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo0 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo0");

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo0 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for Foo0 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for Foo0 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for Foo0 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for Foo0 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for Foo0 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for Foo0 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo1 {
        /// The internal bits
        bits: [u8; 0],
    }

    impl ::device_driver::FieldSet for Foo1 {
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

    impl Foo1 {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 0] }
        }
    }

    impl From<[u8; 0]> for Foo1 {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo1> for [u8; 0] {
        fn from(val: Foo1) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo1 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo1");

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo1 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for Foo1 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for Foo1 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for Foo1 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for Foo1 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for Foo1 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for Foo1 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo2 {
        /// The internal bits
        bits: [u8; 0],
    }

    impl ::device_driver::FieldSet for Foo2 {
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

    impl Foo2 {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 0] }
        }
    }

    impl From<[u8; 0]> for Foo2 {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo2> for [u8; 0] {
        fn from(val: Foo2) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo2 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo2");

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo2 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for Foo2 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for Foo2 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for Foo2 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for Foo2 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for Foo2 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for Foo2 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo3 {
        /// The internal bits
        bits: [u8; 0],
    }

    impl ::device_driver::FieldSet for Foo3 {
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

    impl Foo3 {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 0] }
        }
    }

    impl From<[u8; 0]> for Foo3 {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo3> for [u8; 0] {
        fn from(val: Foo3) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo3 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo3");

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo3 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for Foo3 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for Foo3 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for Foo3 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for Foo3 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for Foo3 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for Foo3 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo4 {
        /// The internal bits
        bits: [u8; 0],
    }

    impl ::device_driver::FieldSet for Foo4 {
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

    impl Foo4 {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 0] }
        }
    }

    impl From<[u8; 0]> for Foo4 {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo4> for [u8; 0] {
        fn from(val: Foo4) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo4 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo4");

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo4 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for Foo4 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for Foo4 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for Foo4 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for Foo4 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for Foo4 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for Foo4 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo5 {
        /// The internal bits
        bits: [u8; 0],
    }

    impl ::device_driver::FieldSet for Foo5 {
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

    impl Foo5 {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 0] }
        }
    }

    impl From<[u8; 0]> for Foo5 {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo5> for [u8; 0] {
        fn from(val: Foo5) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo5 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo5");

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo5 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for Foo5 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for Foo5 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for Foo5 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for Foo5 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for Foo5 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for Foo5 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo6 {
        /// The internal bits
        bits: [u8; 0],
    }

    impl ::device_driver::FieldSet for Foo6 {
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

    impl Foo6 {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 0] }
        }
    }

    impl From<[u8; 0]> for Foo6 {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo6> for [u8; 0] {
        fn from(val: Foo6) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo6 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo6");

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo6 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for Foo6 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for Foo6 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for Foo6 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for Foo6 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for Foo6 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for Foo6 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo7 {
        /// The internal bits
        bits: [u8; 0],
    }

    impl ::device_driver::FieldSet for Foo7 {
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

    impl Foo7 {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 0] }
        }
    }

    impl From<[u8; 0]> for Foo7 {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo7> for [u8; 0] {
        fn from(val: Foo7) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo7 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo7");

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo7 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for Foo7 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for Foo7 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for Foo7 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for Foo7 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for Foo7 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for Foo7 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo8 {
        /// The internal bits
        bits: [u8; 0],
    }

    impl ::device_driver::FieldSet for Foo8 {
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

    impl Foo8 {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 0] }
        }
    }

    impl From<[u8; 0]> for Foo8 {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo8> for [u8; 0] {
        fn from(val: Foo8) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo8 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo8");

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo8 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for Foo8 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for Foo8 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for Foo8 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for Foo8 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for Foo8 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for Foo8 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foo9 {
        /// The internal bits
        bits: [u8; 0],
    }

    impl ::device_driver::FieldSet for Foo9 {
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

    impl Foo9 {
        /// Create a new instance, loaded with the reset value (if any)
        pub const fn new() -> Self {
            Self { bits: [] }
        }
        /// Create a new instance, loaded with all zeroes
        pub const fn new_zero() -> Self {
            Self { bits: [0; 0] }
        }
    }

    impl From<[u8; 0]> for Foo9 {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }

    impl From<Foo9> for [u8; 0] {
        fn from(val: Foo9) -> Self {
            val.bits
        }
    }

    impl core::fmt::Debug for Foo9 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foo9");

            d.finish()
        }
    }

    impl core::ops::BitAnd for Foo9 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }

    impl core::ops::BitAndAssign for Foo9 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }

    impl core::ops::BitOr for Foo9 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }

    impl core::ops::BitOrAssign for Foo9 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }

    impl core::ops::BitXor for Foo9 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }

    impl core::ops::BitXorAssign for Foo9 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }

    impl core::ops::Not for Foo9 {
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
        Foo0(Foo0),

        Foo1(Foo1),

        Foo2(Foo2),

        Foo3(Foo3),

        Foo4(Foo4),

        Foo5(Foo5),

        Foo6(Foo6),

        Foo7(Foo7),

        Foo8(Foo8),

        Foo9(Foo9),
    }
    impl core::fmt::Debug for FieldSetValue {
        fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::Foo0(val) => core::fmt::Debug::fmt(val, _f),

                Self::Foo1(val) => core::fmt::Debug::fmt(val, _f),

                Self::Foo2(val) => core::fmt::Debug::fmt(val, _f),

                Self::Foo3(val) => core::fmt::Debug::fmt(val, _f),

                Self::Foo4(val) => core::fmt::Debug::fmt(val, _f),

                Self::Foo5(val) => core::fmt::Debug::fmt(val, _f),

                Self::Foo6(val) => core::fmt::Debug::fmt(val, _f),

                Self::Foo7(val) => core::fmt::Debug::fmt(val, _f),

                Self::Foo8(val) => core::fmt::Debug::fmt(val, _f),

                Self::Foo9(val) => core::fmt::Debug::fmt(val, _f),

                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }
    }

    impl From<Foo0> for FieldSetValue {
        fn from(val: Foo0) -> Self {
            Self::Foo0(val)
        }
    }

    impl From<Foo1> for FieldSetValue {
        fn from(val: Foo1) -> Self {
            Self::Foo1(val)
        }
    }

    impl From<Foo2> for FieldSetValue {
        fn from(val: Foo2) -> Self {
            Self::Foo2(val)
        }
    }

    impl From<Foo3> for FieldSetValue {
        fn from(val: Foo3) -> Self {
            Self::Foo3(val)
        }
    }

    impl From<Foo4> for FieldSetValue {
        fn from(val: Foo4) -> Self {
            Self::Foo4(val)
        }
    }

    impl From<Foo5> for FieldSetValue {
        fn from(val: Foo5) -> Self {
            Self::Foo5(val)
        }
    }

    impl From<Foo6> for FieldSetValue {
        fn from(val: Foo6) -> Self {
            Self::Foo6(val)
        }
    }

    impl From<Foo7> for FieldSetValue {
        fn from(val: Foo7) -> Self {
            Self::Foo7(val)
        }
    }

    impl From<Foo8> for FieldSetValue {
        fn from(val: Foo8) -> Self {
            Self::Foo8(val)
        }
    }

    impl From<Foo9> for FieldSetValue {
        fn from(val: Foo9) -> Self {
            Self::Foo9(val)
        }
    }
}
