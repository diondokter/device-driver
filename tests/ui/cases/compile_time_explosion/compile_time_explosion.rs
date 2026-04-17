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
    ///
    /// Valid index range: `0..100`
    pub fn foo_0(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        Self,
        Foo,
        u32,
        ::device_driver::RW,
        ::device_driver::ArrayRepeat<100, 1000>,
    >
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u32>,
    {
        let address = self.base_address + 0;
        ::device_driver::RegisterOperation::new(self, address as u32, Foo::default)
    }
    ///
    /// Valid index range: `0..100`
    pub fn foo_1(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        Self,
        Foo,
        u32,
        ::device_driver::RW,
        ::device_driver::ArrayRepeat<100, 1000>,
    >
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u32>,
    {
        let address = self.base_address + 1;
        ::device_driver::RegisterOperation::new(self, address as u32, Foo::default)
    }
    ///
    /// Valid index range: `0..100`
    pub fn foo_2(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        Self,
        Foo,
        u32,
        ::device_driver::RW,
        ::device_driver::ArrayRepeat<100, 1000>,
    >
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u32>,
    {
        let address = self.base_address + 2;
        ::device_driver::RegisterOperation::new(self, address as u32, Foo::default)
    }
    ///
    /// Valid index range: `0..100`
    pub fn foo_3(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        Self,
        Foo,
        u32,
        ::device_driver::RW,
        ::device_driver::ArrayRepeat<100, 1000>,
    >
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u32>,
    {
        let address = self.base_address + 3;
        ::device_driver::RegisterOperation::new(self, address as u32, Foo::default)
    }
    ///
    /// Valid index range: `0..100`
    pub fn foo_4(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        Self,
        Foo,
        u32,
        ::device_driver::RW,
        ::device_driver::ArrayRepeat<100, 1000>,
    >
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u32>,
    {
        let address = self.base_address + 4;
        ::device_driver::RegisterOperation::new(self, address as u32, Foo::default)
    }
    ///
    /// Valid index range: `0..100`
    pub fn foo_5(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        Self,
        Foo,
        u32,
        ::device_driver::RW,
        ::device_driver::ArrayRepeat<100, 1000>,
    >
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u32>,
    {
        let address = self.base_address + 5;
        ::device_driver::RegisterOperation::new(self, address as u32, Foo::default)
    }
    ///
    /// Valid index range: `0..100`
    pub fn foo_6(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        Self,
        Foo,
        u32,
        ::device_driver::RW,
        ::device_driver::ArrayRepeat<100, 1000>,
    >
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u32>,
    {
        let address = self.base_address + 6;
        ::device_driver::RegisterOperation::new(self, address as u32, Foo::default)
    }
    ///
    /// Valid index range: `0..100`
    pub fn foo_7(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        Self,
        Foo,
        u32,
        ::device_driver::RW,
        ::device_driver::ArrayRepeat<100, 1000>,
    >
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u32>,
    {
        let address = self.base_address + 7;
        ::device_driver::RegisterOperation::new(self, address as u32, Foo::default)
    }
    ///
    /// Valid index range: `0..100`
    pub fn foo_8(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        Self,
        Foo,
        u32,
        ::device_driver::RW,
        ::device_driver::ArrayRepeat<100, 1000>,
    >
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u32>,
    {
        let address = self.base_address + 8;
        ::device_driver::RegisterOperation::new(self, address as u32, Foo::default)
    }
    ///
    /// Valid index range: `0..100`
    pub fn foo_9(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        Self,
        Foo,
        u32,
        ::device_driver::RW,
        ::device_driver::ArrayRepeat<100, 1000>,
    >
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u32>,
    {
        let address = self.base_address + 9;
        ::device_driver::RegisterOperation::new(self, address as u32, Foo::default)
    }
}
impl<I> ::device_driver::Block for Device<I> {
    type Interface = I;
    type RegisterAddressType = u32;
    type CommandAddressType = u8;
    type BufferAddressType = u8;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        &mut self.interface
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Foo {
    /// The internal bits
    bits: [u8; 0],
}
unsafe impl ::device_driver::Fieldset for Foo {
    const METADATA: ::device_driver::FieldsetMetadata = ::device_driver::FieldsetMetadata::new()
        .with_byte_order(::device_driver::ByteOrder::LE);
    const ZERO: Self = Self { bits: [0; 0] };
}
impl Foo {}
impl Default for Foo {
    fn default() -> Self {
        <Self as ::device_driver::Fieldset>::ZERO
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
