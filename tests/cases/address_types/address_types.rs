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
    #[doc(hidden)]
    interface: I,
    #[doc(hidden)]
    base_address: u8,
}
impl<I> Device<I> {
    /// Create a new instance of the device, using the interface
    pub const fn new(interface: I) -> Self {
        Self { interface, base_address: 0 }
    }
    pub fn foo(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        I,
        u16,
        FooFieldSet,
        ::device_driver::RW,
        (),
    > {
        use ::device_driver::Block;
        let address = self.base_address + 0;
        ::device_driver::RegisterOperation::new(
            self.interface(),
            address as u16,
            FooFieldSet::new,
        )
    }
    pub fn bar(&mut self) -> ::device_driver::CommandOperation<'_, I, i32, (), (), ()> {
        use ::device_driver::Block;
        let address = self.base_address + 0;
        ::device_driver::CommandOperation::new(self.interface(), address as i32)
    }
    pub fn quux(
        &mut self,
    ) -> ::device_driver::BufferOperation<'_, I, i8, ::device_driver::RW> {
        use ::device_driver::Block;
        let address = self.base_address + 0;
        ::device_driver::BufferOperation::new(self.interface(), address as i8)
    }
}
impl<I> ::device_driver::Block for Device<I> {
    type Interface = I;
    type RegisterAddressType = u16;
    type CommandAddressType = i32;
    type BufferAddressType = i8;
    fn interface(&mut self) -> &mut Self::Interface {
        &mut self.interface
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct FooFieldSet {
    /// The internal bits
    bits: [u8; 0],
}
unsafe impl ::device_driver::FieldSet for FooFieldSet {
    type Unpacked = Self;
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
    fn unpack(self) -> Self::Unpacked {
        self
    }
}
impl ::device_driver::UnpackedFieldSet for FooFieldSet {
    type Packed = Self;
    fn pack(self) -> Self::Packed {
        self
    }
}
impl FooFieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}
impl Default for FooFieldSet {
    fn default() -> Self {
        Self::new()
    }
}
impl From<[u8; 0]> for FooFieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}
impl From<FooFieldSet> for [u8; 0] {
    fn from(val: FooFieldSet) -> Self {
        val.bits
    }
}
impl core::fmt::Debug for FooFieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("FooFieldSet");
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
