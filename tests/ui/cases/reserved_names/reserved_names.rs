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

// This code was generated using device-driver `2.0.0-alpha.1` (xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx),
// a tool distributed under MIT OR Apache-2.0 by Dion Dokter <dev@diondokter.nl>
// This version was built for xxxx-xxxx-xxxx using rustc 1.xx.x (xxxxxxxxx xxxx-xx-xx)
// 
// For more information about device-driver, visit the website: https://device-driver.com

/// Root block of the Foo driver
#[derive(Debug)]
pub struct Foo<I> {
    interface: I,
    #[doc(hidden)]
    #[allow(unused)]
    base_address: u8,
}
impl<I> Foo<I> {
    /// Create a new instance of the device
    pub const fn new(interface: I) -> Self {
        Self { interface, base_address: 0 }
    }
    /// Drop the driver instance and reclaim the interface
    pub fn free(self) -> I {
        self.interface
    }
}
impl<I> ::device_driver::Block for Foo<I> {
    type Interface = I;
    type RegisterAddressType = u8;
    type CommandAddressType = u8;
    type BufferAddressType = u16;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        &mut self.interface
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Bar {
    #[doc(hidden)]
    /// The internal bits
    bits: [u8; 1],
}
unsafe impl ::device_driver::Fieldset for Bar {
    const METADATA: ::device_driver::FieldsetMetadata = ::device_driver::FieldsetMetadata::new()
        .with_byte_order(::device_driver::ByteOrder::LE);
    const ZERO: Self = Self { bits: [0; 1] };
}
impl Bar {
    /// `bit 0` - Read the `my_field` field.
    ///
    #[must_use]
    pub fn my_field(&self) -> bool {
        let start = 0;
        let end = 0;
        let raw = unsafe {
            ::device_driver::ops::load::<
                u8,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw > 0
    }
    /// `bit 1` - Read the `set_my_field` field.
    ///
    #[must_use]
    pub fn set_my_field(&self) -> bool {
        let start = 1;
        let end = 1;
        let raw = unsafe {
            ::device_driver::ops::load::<
                u8,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw > 0
    }
    /// `bit 2` - Read the `other_field` field.
    ///
    #[must_use]
    pub fn other_field(&self) -> bool {
        let start = 2;
        let end = 2;
        let raw = unsafe {
            ::device_driver::ops::load::<
                u8,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw > 0
    }
    /// `bit 3` - Read the `set_other_field` field.
    ///
    #[must_use]
    pub fn set_other_field(&self) -> bool {
        let start = 3;
        let end = 3;
        let raw = unsafe {
            ::device_driver::ops::load::<
                u8,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw > 0
    }
    /// `bit 1` - Set the `set_my_field` field.
    ///
    pub fn set_set_my_field(&mut self, value: bool) {
        let start = 1;
        let end = 1;
        let raw = value as _;
        unsafe {
            ::device_driver::ops::store::<
                u8,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
    /// `bit 3` - Set the `set_other_field` field.
    ///
    pub fn set_set_other_field(&mut self, value: bool) {
        let start = 3;
        let end = 3;
        let raw = value as _;
        unsafe {
            ::device_driver::ops::store::<
                u8,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
}
impl Default for Bar {
    fn default() -> Self {
        <Self as ::device_driver::Fieldset>::ZERO
    }
}
impl From<[u8; 1]> for Bar {
    fn from(bits: [u8; 1]) -> Self {
        Self { bits }
    }
}
impl From<Bar> for [u8; 1] {
    fn from(val: Bar) -> Self {
        val.bits
    }
}
impl core::fmt::Debug for Bar {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Bar");
        d.field("my_field", &self.my_field());
        d.field("set_my_field", &self.set_my_field());
        d.field("other_field", &self.other_field());
        d.field("set_other_field", &self.set_other_field());
        d.finish()
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for Bar {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Bar {{ ");
        defmt::write!(f, "my_field: {=bool}, ", & self.my_field());
        defmt::write!(f, "set_my_field: {=bool}, ", & self.set_my_field());
        defmt::write!(f, "other_field: {=bool}, ", & self.other_field());
        defmt::write!(f, "set_other_field: {=bool}, ", & self.set_other_field());
        defmt::write!(f, "}}");
    }
}
impl core::ops::BitAnd for Bar {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Bar {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Bar {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Bar {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Bar {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Bar {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Bar {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}
compile_error!("The device driver input has errors that need to be solved!");
