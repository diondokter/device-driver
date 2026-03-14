#!/usr/bin/env cargo
---
[package]
edition = "2024"
[dependencies]
device-driver = { path="../../../../device-driver", default-features=false }
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
        Self { interface, base_address: 0 }
    }
    /// A reference to the interface used to communicate with the device
    pub(crate) fn interface(&mut self) -> &mut I {
        &mut self.interface
    }
    ///
    /// Valid index range: 0..100
    pub fn foo_0(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 0 + index as u32 * 1000
        };
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u32,
            Foo,
            ::device_driver::RW,
        >::new(self.interface(), address as u32, Foo::new)
    }
    ///
    /// Valid index range: 0..100
    pub fn foo_1(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 1 + index as u32 * 1000
        };
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u32,
            Foo,
            ::device_driver::RW,
        >::new(self.interface(), address as u32, Foo::new)
    }
    ///
    /// Valid index range: 0..100
    pub fn foo_2(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 2 + index as u32 * 1000
        };
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u32,
            Foo,
            ::device_driver::RW,
        >::new(self.interface(), address as u32, Foo::new)
    }
    ///
    /// Valid index range: 0..100
    pub fn foo_3(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 3 + index as u32 * 1000
        };
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u32,
            Foo,
            ::device_driver::RW,
        >::new(self.interface(), address as u32, Foo::new)
    }
    ///
    /// Valid index range: 0..100
    pub fn foo_4(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 4 + index as u32 * 1000
        };
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u32,
            Foo,
            ::device_driver::RW,
        >::new(self.interface(), address as u32, Foo::new)
    }
    ///
    /// Valid index range: 0..100
    pub fn foo_5(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 5 + index as u32 * 1000
        };
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u32,
            Foo,
            ::device_driver::RW,
        >::new(self.interface(), address as u32, Foo::new)
    }
    ///
    /// Valid index range: 0..100
    pub fn foo_6(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 6 + index as u32 * 1000
        };
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u32,
            Foo,
            ::device_driver::RW,
        >::new(self.interface(), address as u32, Foo::new)
    }
    ///
    /// Valid index range: 0..100
    pub fn foo_7(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 7 + index as u32 * 1000
        };
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u32,
            Foo,
            ::device_driver::RW,
        >::new(self.interface(), address as u32, Foo::new)
    }
    ///
    /// Valid index range: 0..100
    pub fn foo_8(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 8 + index as u32 * 1000
        };
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u32,
            Foo,
            ::device_driver::RW,
        >::new(self.interface(), address as u32, Foo::new)
    }
    ///
    /// Valid index range: 0..100
    pub fn foo_9(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 9 + index as u32 * 1000
        };
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u32,
            Foo,
            ::device_driver::RW,
        >::new(self.interface(), address as u32, Foo::new)
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo {
    /// The internal bits
    bits: [u8; 0],
}
impl ::device_driver::FieldSet for Foo {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}
impl Foo {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}
impl Default for Foo {
    fn default() -> Self {
        Self::new()
    }
}
impl From<[u8; 0]> for Foo {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}
impl From<Foo> for [u8; 0] {
    fn from(val: Foo) -> Self {
        val.bits
    }
}
impl core::fmt::Debug for Foo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo");
        d.finish()
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for Foo {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Foo {{ ");
        defmt::write!(f, "}}");
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
