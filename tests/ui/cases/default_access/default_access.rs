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

// This code was generated using device-driver `xx.xx.xx` (xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx),
// a tool distributed under MIT OR Apache-2.0 by Dion Dokter <dev@diondokter.nl>
// This version was built for xxxx-xxxx-xxxx using rustc 1.xx.x (xxxxxxxxx xxxx-xx-xx)
// 
// For more information about device-driver, visit the website: https://device-driver.com

/// Root block of the Y driver
#[derive(Debug)]
pub struct Y<I> {
    interface: I,
    #[doc(hidden)]
    #[allow(unused)]
    base_address: u8,
}
impl<I> Y<I> {
    /// Create a new instance of the device
    pub const fn new(interface: I) -> Self {
        Self { interface, base_address: 0 }
    }
    /// Drop the driver instance and reclaim the interface
    pub fn free(self) -> I {
        self.interface
    }
    /// Register operation:
    /// - Address: `0`
    /// - Reset value: `0`
    #[doc(alias = "Z")]
    pub fn z(
        &mut self,
    ) -> ::device_driver::RegisterOperation<'_, Self, A, u8, ::device_driver::RO, ()>
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u8>,
    {
        let address = self.base_address + 0;
        ::device_driver::RegisterOperation::new(self, address as u8, A::default)
    }
    /// Block operation:
    /// - Address: `0`
    #[doc(alias = "B")]
    pub fn b(&mut self) -> B<'_, I> {
        let address = self.base_address + 0;
        B::<'_, I>::new(::device_driver::Block::interface(self), address)
    }
}
impl<I> ::device_driver::Block for Y<I> {
    type Interface = I;
    type RegisterAddressType = u8;
    type CommandAddressType = u8;
    type BufferAddressType = u8;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        &mut self.interface
    }
}
#[derive(Debug)]
pub struct B<'i, I> {
    #[doc(hidden)]
    interface: &'i mut I,
    #[doc(hidden)]
    #[allow(unused)]
    base_address: u8,
}
impl<'i, I> B<'i, I> {
    /// Create a new instance of the block based on device interface
    #[doc(hidden)]
    fn new(interface: &'i mut I, base_address: u8) -> Self {
        Self {
            interface,
            base_address: base_address,
        }
    }
    /// Register operation:
    /// - Address: `1`
    /// - Reset value: `0`
    #[doc(alias = "C")]
    pub fn c(
        &mut self,
    ) -> ::device_driver::RegisterOperation<'_, Self, A, u8, ::device_driver::WO, ()>
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u8>,
    {
        let address = self.base_address + 1;
        ::device_driver::RegisterOperation::new(self, address as u8, A::default)
    }
}
impl<'i, I> ::device_driver::Block for B<'i, I> {
    type Interface = I;
    type RegisterAddressType = u8;
    type CommandAddressType = u8;
    type BufferAddressType = u8;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        self.interface
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct A {
    #[doc(hidden)]
    /// The internal bits
    bits: [u8; 1],
}
unsafe impl ::device_driver::Fieldset for A {
    const METADATA: ::device_driver::FieldsetMetadata = ::device_driver::FieldsetMetadata::new()
        .with_byte_order(::device_driver::ByteOrder::LE);
    const ZERO: Self = Self { bits: [0; 1] };
}
impl A {
    /// `bit 0` - Set the `a` field.
    ///
    pub fn set_a(&mut self, value: bool) {
        let start = 0;
        let end = 0;
        let raw = value as _;
        unsafe {
            ::device_driver::ops::store::<
                u8,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
}
impl Default for A {
    fn default() -> Self {
        <Self as ::device_driver::Fieldset>::ZERO
    }
}
impl From<[u8; 1]> for A {
    fn from(bits: [u8; 1]) -> Self {
        Self { bits }
    }
}
impl From<A> for [u8; 1] {
    fn from(val: A) -> Self {
        val.bits
    }
}
impl core::fmt::Debug for A {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("A");
        d.finish()
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for A {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "A {{ ");
        defmt::write!(f, "}}");
    }
}
impl core::ops::BitAnd for A {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for A {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for A {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for A {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for A {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for A {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for A {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}
