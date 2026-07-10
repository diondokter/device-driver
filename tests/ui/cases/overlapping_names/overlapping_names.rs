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
    pub(crate) interface: I,
    #[doc(hidden)]
    base_address: u8,
}
impl<I> Foo<I> {
    /// Create a new instance of the block based on device interface
    pub const fn new(interface: I) -> Self {
        Self { interface, base_address: 0 }
    }
    #[doc(alias = "Bar")]
    pub fn bar(
        &mut self,
    ) -> ::device_driver::RegisterOperation<'_, Self, Bar, u8, ::device_driver::RW, ()>
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u8>,
    {
        let address = self.base_address + 0;
        ::device_driver::RegisterOperation::new(self, address as u8, Bar::default)
    }
}
impl<I> ::device_driver::Block for Foo<I> {
    type Interface = I;
    type RegisterAddressType = u8;
    type CommandAddressType = u8;
    type BufferAddressType = u8;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        &mut self.interface
    }
}
/// Root block of the FooDup1 driver
#[doc(alias = "Foo")]
#[derive(Debug)]
pub struct FooDup1<I> {
    pub(crate) interface: I,
    #[doc(hidden)]
    base_address: u8,
}
impl<I> FooDup1<I> {
    /// Create a new instance of the block based on device interface
    pub const fn new(interface: I) -> Self {
        Self { interface, base_address: 0 }
    }
    #[doc(alias = "Bar")]
    pub fn bar_dup_2(
        &mut self,
    ) -> ::device_driver::RegisterOperation<'_, Self, Bar, u8, ::device_driver::RW, ()>
    where
        I: ::device_driver::RegisterInterfaceBase<AddressType = u8>,
    {
        let address = self.base_address + 0;
        ::device_driver::RegisterOperation::new(self, address as u8, Bar::default)
    }
}
impl<I> ::device_driver::Block for FooDup1<I> {
    type Interface = I;
    type RegisterAddressType = u8;
    type CommandAddressType = u8;
    type BufferAddressType = u8;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        &mut self.interface
    }
}
/// Root block of the Blah driver
#[derive(Debug)]
pub struct Blah<I> {
    pub(crate) interface: I,
    #[doc(hidden)]
    base_address: u8,
}
impl<I> Blah<I> {
    /// Create a new instance of the block based on device interface
    pub const fn new(interface: I) -> Self {
        Self { interface, base_address: 0 }
    }
    #[doc(alias = "Wheee")]
    pub fn wheee(&mut self) -> Wheee<'_, I> {
        let address = self.base_address + 0;
        Wheee::<'_, I>::new(::device_driver::Block::interface(self), address)
    }
    #[doc(alias = "Wheee")]
    pub fn wheee_dup_5(
        &mut self,
    ) -> ::device_driver::BufferOperation<'_, Self, u8, ::device_driver::RW>
    where
        I: ::device_driver::BufferInterfaceBase<AddressType = u8>,
    {
        let address = self.base_address + 0;
        ::device_driver::BufferOperation::new(self, address as u8)
    }
    #[doc(alias = "Wheee2")]
    pub fn wheee_2(&mut self) -> Wheee2<'_, I> {
        let address = self.base_address + 0;
        Wheee2::<'_, I>::new(::device_driver::Block::interface(self), address)
    }
}
impl<I> ::device_driver::Block for Blah<I> {
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
pub struct Wheee<'i, I> {
    pub(crate) interface: &'i mut I,
    #[doc(hidden)]
    base_address: u8,
}
impl<'i, I> Wheee<'i, I> {
    /// Create a new instance of the block based on device interface
    #[doc(hidden)]
    fn new(interface: &'i mut I, base_address: u8) -> Self {
        Self {
            interface,
            base_address: base_address,
        }
    }
}
impl<'i, I> ::device_driver::Block for Wheee<'i, I> {
    type Interface = I;
    type RegisterAddressType = u8;
    type CommandAddressType = u8;
    type BufferAddressType = u8;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        self.interface
    }
}
#[derive(Debug)]
pub struct Wheee2<'i, I> {
    pub(crate) interface: &'i mut I,
    #[doc(hidden)]
    base_address: u8,
}
impl<'i, I> Wheee2<'i, I> {
    /// Create a new instance of the block based on device interface
    #[doc(hidden)]
    fn new(interface: &'i mut I, base_address: u8) -> Self {
        Self {
            interface,
            base_address: base_address,
        }
    }
}
impl<'i, I> ::device_driver::Block for Wheee2<'i, I> {
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
pub struct Bar {
    /// The internal bits
    bits: [u8; 1],
}
unsafe impl ::device_driver::Fieldset for Bar {
    const METADATA: ::device_driver::FieldsetMetadata = ::device_driver::FieldsetMetadata::new()
        .with_byte_order(::device_driver::ByteOrder::LE);
    const ZERO: Self = Self { bits: [0; 1] };
}
impl Bar {
    /// `7:0` - Read the `quux` field.
    ///
    #[must_use]
    pub fn quux(&self) -> Result<Quux, <Quux as TryFrom<u8>>::Error> {
        let start = 0;
        let end = 7;
        let raw = unsafe {
            ::device_driver::ops::load::<
                u8,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw.try_into()
    }
    /// `7:0` - Set the `quux` field.
    ///
    pub fn set_quux(&mut self, value: Quux) {
        let start = 0;
        let end = 7;
        let raw = value.into();
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
        d.field("quux", &self.quux());
        d.finish()
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for Bar {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Bar {{ ");
        defmt::write!(f, "quux: {}, ", & self.quux());
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
#[doc(alias = "Bar")]
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct BarDup4 {
    /// The internal bits
    bits: [u8; 1],
}
unsafe impl ::device_driver::Fieldset for BarDup4 {
    const METADATA: ::device_driver::FieldsetMetadata = ::device_driver::FieldsetMetadata::new()
        .with_byte_order(::device_driver::ByteOrder::LE);
    const ZERO: Self = Self { bits: [0; 1] };
}
impl BarDup4 {
    /// `7:0` - Read the `quux` field.
    ///
    #[must_use]
    pub fn quux(&self) -> Result<Quux, <Quux as TryFrom<u8>>::Error> {
        let start = 0;
        let end = 7;
        let raw = unsafe {
            ::device_driver::ops::load::<
                u8,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw.try_into()
    }
    /// `7:0` - Set the `quux` field.
    ///
    pub fn set_quux(&mut self, value: Quux) {
        let start = 0;
        let end = 7;
        let raw = value.into();
        unsafe {
            ::device_driver::ops::store::<
                u8,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
}
impl Default for BarDup4 {
    fn default() -> Self {
        <Self as ::device_driver::Fieldset>::ZERO
    }
}
impl From<[u8; 1]> for BarDup4 {
    fn from(bits: [u8; 1]) -> Self {
        Self { bits }
    }
}
impl From<BarDup4> for [u8; 1] {
    fn from(val: BarDup4) -> Self {
        val.bits
    }
}
impl core::fmt::Debug for BarDup4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("BarDup4");
        d.field("quux", &self.quux());
        d.finish()
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for BarDup4 {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "BarDup4 {{ ");
        defmt::write!(f, "quux: {}, ", & self.quux());
        defmt::write!(f, "}}");
    }
}
impl core::ops::BitAnd for BarDup4 {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for BarDup4 {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for BarDup4 {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for BarDup4 {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for BarDup4 {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for BarDup4 {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for BarDup4 {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}
#[doc(alias = "quux")]
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Quux {
    #[doc(alias = "quux")]
    Quux = 0,
    #[doc(alias = "bar")]
    Bar = 1,
    #[doc(alias = "foo")]
    Foo = 2,
}
impl core::convert::TryFrom<u8> for Quux {
    type Error = ::device_driver::ConversionError<u8>;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::Quux),
            1 => Ok(Self::Bar),
            2 => Ok(Self::Foo),
            val => {
                Err(::device_driver::ConversionError {
                    source: val,
                    target: "Quux",
                })
            }
        }
    }
}
impl From<Quux> for u8 {
    fn from(val: Quux) -> Self {
        match val {
            Quux::Quux => 0,
            Quux::Bar => 1,
            Quux::Foo => 2,
        }
    }
}
impl ::device_driver::EnumIndex for Quux {
    #[track_caller]
    fn index(&self) -> i32 {
        let index = u8::from(*self);
        index.try_into().unwrap()
    }
}
#[doc(alias = "quux")]
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum QuuxDup3 {
    #[doc(alias = "quux")]
    Quux = 0,
    #[doc(alias = "bar")]
    Bar = 1,
    #[doc(alias = "foo")]
    Foo = 2,
}
impl core::convert::TryFrom<u8> for QuuxDup3 {
    type Error = ::device_driver::ConversionError<u8>;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::Quux),
            1 => Ok(Self::Bar),
            2 => Ok(Self::Foo),
            val => {
                Err(::device_driver::ConversionError {
                    source: val,
                    target: "QuuxDup3",
                })
            }
        }
    }
}
impl From<QuuxDup3> for u8 {
    fn from(val: QuuxDup3) -> Self {
        match val {
            QuuxDup3::Quux => 0,
            QuuxDup3::Bar => 1,
            QuuxDup3::Foo => 2,
        }
    }
}
impl ::device_driver::EnumIndex for QuuxDup3 {
    #[track_caller]
    fn index(&self) -> i32 {
        let index = u8::from(*self);
        index.try_into().unwrap()
    }
}
#[doc(alias = "Wheee2")]
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Wheee2Dup6 {
    A = 0,
}
impl core::convert::TryFrom<u8> for Wheee2Dup6 {
    type Error = ::device_driver::ConversionError<u8>;
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::A),
            val => {
                Err(::device_driver::ConversionError {
                    source: val,
                    target: "Wheee2Dup6",
                })
            }
        }
    }
}
impl From<Wheee2Dup6> for u8 {
    fn from(val: Wheee2Dup6) -> Self {
        match val {
            Wheee2Dup6::A => 0,
        }
    }
}
impl ::device_driver::EnumIndex for Wheee2Dup6 {
    #[track_caller]
    fn index(&self) -> i32 {
        let index = u8::from(*self);
        index.try_into().unwrap()
    }
}
compile_error!("The device driver input has errors that need to be solved!");
