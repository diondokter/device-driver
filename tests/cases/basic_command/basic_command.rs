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

    pub fn foo(&mut self) -> ::device_driver::CommandOperation<'_, I, u8, FooFieldSetIn, ()> {
        let address = self.base_address + 0;

        ::device_driver::CommandOperation::<'_, I, u8, FooFieldSetIn, ()>::new(
            self.interface(),
            address as u8,
        )
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FooFieldSetIn {
    /// The internal bits
    bits: [u8; 3],
}

impl ::device_driver::FieldSet for FooFieldSetIn {
    const SIZE_BITS: u32 = 24;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl FooFieldSetIn {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 3] }
    }

    ///Read the `value` field of the register.
    ///

    pub fn value(&self) -> u32 {
        let start = 0;
        let end = 24;

        let raw = unsafe {
            ::device_driver::ops::load_lsb0::<u32, ::device_driver::ops::LE>(&self.bits, start, end)
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

impl Default for FooFieldSetIn {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 3]> for FooFieldSetIn {
    fn from(bits: [u8; 3]) -> Self {
        Self { bits }
    }
}

impl From<FooFieldSetIn> for [u8; 3] {
    fn from(val: FooFieldSetIn) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for FooFieldSetIn {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("FooFieldSetIn");

        d.field("value", &self.value());

        d.finish()
    }
}

impl core::ops::BitAnd for FooFieldSetIn {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for FooFieldSetIn {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for FooFieldSetIn {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for FooFieldSetIn {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for FooFieldSetIn {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for FooFieldSetIn {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for FooFieldSetIn {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}
