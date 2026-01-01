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
        Self { interface, base_address: 0 }
    }
    /// A reference to the interface used to communicate with the device
    pub(crate) fn interface(&mut self) -> &mut I {
        &mut self.interface
    }
    /// \\\\\\/\/\/\////\/\/;{}'"`'
    pub fn foo(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        I,
        u8,
        FooFieldSet,
        ::device_driver::RW,
    > {
        let address = self.base_address + 0;
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u8,
            FooFieldSet,
            ::device_driver::RW,
        >::new(self.interface(), address as u8, FooFieldSet::new)
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FooFieldSet {
    /// The internal bits
    bits: [u8; 3],
}
impl ::device_driver::FieldSet for FooFieldSet {
    const SIZE_BITS: u32 = 24;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}
impl FooFieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 3] }
    }
    /// `@23:0` - Read the `value` field.
    ///
    /// \\\"#{
    /// %@&\n
    pub fn value(&self) -> u32 {
        let start = 0;
        let end = 24;
        let raw = unsafe {
            ::device_driver::ops::load_lsb0::<
                u32,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw
    }
    /// `@23:0` - Set the `value` field.
    ///
    /// \\\"#{
    /// %@&\n
    pub fn set_value(&mut self, value: u32) {
        let start = 0;
        let end = 24;
        let raw = value;
        unsafe {
            ::device_driver::ops::store_lsb0::<
                u32,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
}
impl Default for FooFieldSet {
    fn default() -> Self {
        Self::new()
    }
}
impl From<[u8; 3]> for FooFieldSet {
    fn from(bits: [u8; 3]) -> Self {
        Self { bits }
    }
}
impl From<FooFieldSet> for [u8; 3] {
    fn from(val: FooFieldSet) -> Self {
        val.bits
    }
}
impl core::fmt::Debug for FooFieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("FooFieldSet");
        d.field("value", &self.value());
        d.finish()
    }
}
impl core::ops::BitAnd for FooFieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for FooFieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for FooFieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for FooFieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for FooFieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for FooFieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for FooFieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}
