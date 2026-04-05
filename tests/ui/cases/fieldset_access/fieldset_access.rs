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
    pub fn foo_ro(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        I,
        u8,
        FooRoFieldSet,
        ::device_driver::RO,
    > {
        let address = self.base_address + 0;
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u8,
            FooRoFieldSet,
            ::device_driver::RO,
        >::new(self.interface(), address as u8, FooRoFieldSet::default)
    }
    pub fn foo_rw(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        I,
        u8,
        FooRwFieldSet,
        ::device_driver::RW,
    > {
        let address = self.base_address + 1;
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u8,
            FooRwFieldSet,
            ::device_driver::RW,
        >::new(self.interface(), address as u8, FooRwFieldSet::default)
    }
    pub fn foo_wo(
        &mut self,
    ) -> ::device_driver::RegisterOperation<
        '_,
        I,
        u8,
        FooWoFieldSet,
        ::device_driver::WO,
    > {
        let address = self.base_address + 2;
        ::device_driver::RegisterOperation::<
            '_,
            I,
            u8,
            FooWoFieldSet,
            ::device_driver::WO,
        >::new(self.interface(), address as u8, FooWoFieldSet::default)
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct FooWoFieldSet {
    /// The internal bits
    bits: [u8; 8],
}
unsafe impl ::device_driver::Fieldset for FooWoFieldSet {
    const METADATA: ::device_driver::FieldsetMetadata = ::device_driver::FieldsetMetadata::new()
        .with_byte_order(::device_driver::ByteOrder::LE);
    const ZERO: Self = Self { bits: [0; 8] };
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}
impl FooWoFieldSet {
    /// `15:0` - Read the `value_ro` field.
    ///
    pub fn value_ro(&self) -> u16 {
        let start = 0;
        let end = 15;
        let raw = unsafe {
            ::device_driver::ops::load::<
                u16,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw
    }
    /// `31:16` - Read the `value_rw` field.
    ///
    pub fn value_rw(&self) -> i16 {
        let start = 16;
        let end = 31;
        let raw = unsafe {
            ::device_driver::ops::load::<
                i16,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw
    }
    /// `31:16` - Set the `value_rw` field.
    ///
    pub fn set_value_rw(&mut self, value: i16) {
        let start = 16;
        let end = 31;
        let raw = value;
        unsafe {
            ::device_driver::ops::store::<
                i16,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
    /// `bit 32` - Set the `value_wo` field.
    ///
    pub fn set_value_wo(&mut self, value: bool) {
        let start = 32;
        let end = 32;
        let raw = value as _;
        unsafe {
            ::device_driver::ops::store::<
                u8,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
}
impl Default for FooWoFieldSet {
    fn default() -> Self {
        <Self as ::device_driver::Fieldset>::ZERO
    }
}
impl From<[u8; 8]> for FooWoFieldSet {
    fn from(bits: [u8; 8]) -> Self {
        Self { bits }
    }
}
impl From<FooWoFieldSet> for [u8; 8] {
    fn from(val: FooWoFieldSet) -> Self {
        val.bits
    }
}
impl core::fmt::Debug for FooWoFieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("FooWoFieldSet");
        d.field("value_ro", &self.value_ro());
        d.field("value_rw", &self.value_rw());
        d.finish()
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for FooWoFieldSet {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "FooWoFieldSet {{ ");
        defmt::write!(f, "value_ro: {=u16}, ", & self.value_ro());
        defmt::write!(f, "value_rw: {=i16}, ", & self.value_rw());
        defmt::write!(f, "}}");
    }
}
impl core::ops::BitAnd for FooWoFieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for FooWoFieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for FooWoFieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for FooWoFieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for FooWoFieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for FooWoFieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for FooWoFieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct FooRwFieldSet {
    /// The internal bits
    bits: [u8; 8],
}
unsafe impl ::device_driver::Fieldset for FooRwFieldSet {
    const METADATA: ::device_driver::FieldsetMetadata = ::device_driver::FieldsetMetadata::new()
        .with_byte_order(::device_driver::ByteOrder::LE);
    const ZERO: Self = Self { bits: [0; 8] };
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}
impl FooRwFieldSet {
    /// `15:0` - Read the `value_ro` field.
    ///
    pub fn value_ro(&self) -> u16 {
        let start = 0;
        let end = 15;
        let raw = unsafe {
            ::device_driver::ops::load::<
                u16,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw
    }
    /// `31:16` - Read the `value_rw` field.
    ///
    pub fn value_rw(&self) -> i16 {
        let start = 16;
        let end = 31;
        let raw = unsafe {
            ::device_driver::ops::load::<
                i16,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw
    }
    /// `31:16` - Set the `value_rw` field.
    ///
    pub fn set_value_rw(&mut self, value: i16) {
        let start = 16;
        let end = 31;
        let raw = value;
        unsafe {
            ::device_driver::ops::store::<
                i16,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
    /// `bit 32` - Set the `value_wo` field.
    ///
    pub fn set_value_wo(&mut self, value: bool) {
        let start = 32;
        let end = 32;
        let raw = value as _;
        unsafe {
            ::device_driver::ops::store::<
                u8,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
}
impl Default for FooRwFieldSet {
    fn default() -> Self {
        <Self as ::device_driver::Fieldset>::ZERO
    }
}
impl From<[u8; 8]> for FooRwFieldSet {
    fn from(bits: [u8; 8]) -> Self {
        Self { bits }
    }
}
impl From<FooRwFieldSet> for [u8; 8] {
    fn from(val: FooRwFieldSet) -> Self {
        val.bits
    }
}
impl core::fmt::Debug for FooRwFieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("FooRwFieldSet");
        d.field("value_ro", &self.value_ro());
        d.field("value_rw", &self.value_rw());
        d.finish()
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for FooRwFieldSet {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "FooRwFieldSet {{ ");
        defmt::write!(f, "value_ro: {=u16}, ", & self.value_ro());
        defmt::write!(f, "value_rw: {=i16}, ", & self.value_rw());
        defmt::write!(f, "}}");
    }
}
impl core::ops::BitAnd for FooRwFieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for FooRwFieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for FooRwFieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for FooRwFieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for FooRwFieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for FooRwFieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for FooRwFieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct FooRoFieldSet {
    /// The internal bits
    bits: [u8; 8],
}
unsafe impl ::device_driver::Fieldset for FooRoFieldSet {
    const METADATA: ::device_driver::FieldsetMetadata = ::device_driver::FieldsetMetadata::new()
        .with_byte_order(::device_driver::ByteOrder::LE);
    const ZERO: Self = Self { bits: [0; 8] };
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}
impl FooRoFieldSet {
    /// `15:0` - Read the `value_ro` field.
    ///
    pub fn value_ro(&self) -> u16 {
        let start = 0;
        let end = 15;
        let raw = unsafe {
            ::device_driver::ops::load::<
                u16,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw
    }
    /// `31:16` - Read the `value_rw` field.
    ///
    pub fn value_rw(&self) -> i16 {
        let start = 16;
        let end = 31;
        let raw = unsafe {
            ::device_driver::ops::load::<
                i16,
                ::device_driver::ops::LE,
            >(&self.bits, start, end)
        };
        raw
    }
    /// `31:16` - Set the `value_rw` field.
    ///
    pub fn set_value_rw(&mut self, value: i16) {
        let start = 16;
        let end = 31;
        let raw = value;
        unsafe {
            ::device_driver::ops::store::<
                i16,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
    /// `bit 32` - Set the `value_wo` field.
    ///
    pub fn set_value_wo(&mut self, value: bool) {
        let start = 32;
        let end = 32;
        let raw = value as _;
        unsafe {
            ::device_driver::ops::store::<
                u8,
                ::device_driver::ops::LE,
            >(raw, start, end, &mut self.bits)
        };
    }
}
impl Default for FooRoFieldSet {
    fn default() -> Self {
        <Self as ::device_driver::Fieldset>::ZERO
    }
}
impl From<[u8; 8]> for FooRoFieldSet {
    fn from(bits: [u8; 8]) -> Self {
        Self { bits }
    }
}
impl From<FooRoFieldSet> for [u8; 8] {
    fn from(val: FooRoFieldSet) -> Self {
        val.bits
    }
}
impl core::fmt::Debug for FooRoFieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("FooRoFieldSet");
        d.field("value_ro", &self.value_ro());
        d.field("value_rw", &self.value_rw());
        d.finish()
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for FooRoFieldSet {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "FooRoFieldSet {{ ");
        defmt::write!(f, "value_ro: {=u16}, ", & self.value_ro());
        defmt::write!(f, "value_rw: {=i16}, ", & self.value_rw());
        defmt::write!(f, "}}");
    }
}
impl core::ops::BitAnd for FooRoFieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for FooRoFieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for FooRoFieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for FooRoFieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for FooRoFieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for FooRoFieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for FooRoFieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}
