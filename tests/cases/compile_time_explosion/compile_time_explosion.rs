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

    ///
    /// Valid index range: 0..100

    pub fn foo_0(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo0FieldSet, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 0 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, Foo0FieldSet, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            Foo0FieldSet::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_1(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo1FieldSet, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 1 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, Foo1FieldSet, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            Foo1FieldSet::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_2(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo2FieldSet, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 2 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, Foo2FieldSet, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            Foo2FieldSet::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_3(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo3FieldSet, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 3 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, Foo3FieldSet, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            Foo3FieldSet::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_4(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo4FieldSet, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 4 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, Foo4FieldSet, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            Foo4FieldSet::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_5(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo5FieldSet, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 5 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, Foo5FieldSet, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            Foo5FieldSet::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_6(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo6FieldSet, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 6 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, Foo6FieldSet, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            Foo6FieldSet::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_7(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo7FieldSet, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 7 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, Foo7FieldSet, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            Foo7FieldSet::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_8(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo8FieldSet, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 8 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, Foo8FieldSet, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            Foo8FieldSet::new,
        )
    }

    ///
    /// Valid index range: 0..100

    pub fn foo_9(
        &mut self,
        index: usize,
    ) -> ::device_driver::RegisterOperation<'_, I, u32, Foo9FieldSet, ::device_driver::RW> {
        let address = {
            assert!(index < 100);
            self.base_address + 9 + index as u32 * 1000
        };

        ::device_driver::RegisterOperation::<'_, I, u32, Foo9FieldSet, ::device_driver::RW>::new(
            self.interface(),
            address as u32,
            Foo9FieldSet::new,
        )
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo0FieldSet {
    /// The internal bits
    bits: [u8; 0],
}

impl ::device_driver::FieldSet for Foo0FieldSet {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl Foo0FieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}

impl Default for Foo0FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 0]> for Foo0FieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}

impl From<Foo0FieldSet> for [u8; 0] {
    fn from(val: Foo0FieldSet) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for Foo0FieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo0FieldSet");

        d.finish()
    }
}

impl core::ops::BitAnd for Foo0FieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Foo0FieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Foo0FieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Foo0FieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Foo0FieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Foo0FieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Foo0FieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo1FieldSet {
    /// The internal bits
    bits: [u8; 0],
}

impl ::device_driver::FieldSet for Foo1FieldSet {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl Foo1FieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}

impl Default for Foo1FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 0]> for Foo1FieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}

impl From<Foo1FieldSet> for [u8; 0] {
    fn from(val: Foo1FieldSet) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for Foo1FieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo1FieldSet");

        d.finish()
    }
}

impl core::ops::BitAnd for Foo1FieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Foo1FieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Foo1FieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Foo1FieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Foo1FieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Foo1FieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Foo1FieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo2FieldSet {
    /// The internal bits
    bits: [u8; 0],
}

impl ::device_driver::FieldSet for Foo2FieldSet {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl Foo2FieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}

impl Default for Foo2FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 0]> for Foo2FieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}

impl From<Foo2FieldSet> for [u8; 0] {
    fn from(val: Foo2FieldSet) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for Foo2FieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo2FieldSet");

        d.finish()
    }
}

impl core::ops::BitAnd for Foo2FieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Foo2FieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Foo2FieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Foo2FieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Foo2FieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Foo2FieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Foo2FieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo3FieldSet {
    /// The internal bits
    bits: [u8; 0],
}

impl ::device_driver::FieldSet for Foo3FieldSet {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl Foo3FieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}

impl Default for Foo3FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 0]> for Foo3FieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}

impl From<Foo3FieldSet> for [u8; 0] {
    fn from(val: Foo3FieldSet) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for Foo3FieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo3FieldSet");

        d.finish()
    }
}

impl core::ops::BitAnd for Foo3FieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Foo3FieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Foo3FieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Foo3FieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Foo3FieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Foo3FieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Foo3FieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo4FieldSet {
    /// The internal bits
    bits: [u8; 0],
}

impl ::device_driver::FieldSet for Foo4FieldSet {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl Foo4FieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}

impl Default for Foo4FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 0]> for Foo4FieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}

impl From<Foo4FieldSet> for [u8; 0] {
    fn from(val: Foo4FieldSet) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for Foo4FieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo4FieldSet");

        d.finish()
    }
}

impl core::ops::BitAnd for Foo4FieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Foo4FieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Foo4FieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Foo4FieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Foo4FieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Foo4FieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Foo4FieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo5FieldSet {
    /// The internal bits
    bits: [u8; 0],
}

impl ::device_driver::FieldSet for Foo5FieldSet {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl Foo5FieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}

impl Default for Foo5FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 0]> for Foo5FieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}

impl From<Foo5FieldSet> for [u8; 0] {
    fn from(val: Foo5FieldSet) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for Foo5FieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo5FieldSet");

        d.finish()
    }
}

impl core::ops::BitAnd for Foo5FieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Foo5FieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Foo5FieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Foo5FieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Foo5FieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Foo5FieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Foo5FieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo6FieldSet {
    /// The internal bits
    bits: [u8; 0],
}

impl ::device_driver::FieldSet for Foo6FieldSet {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl Foo6FieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}

impl Default for Foo6FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 0]> for Foo6FieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}

impl From<Foo6FieldSet> for [u8; 0] {
    fn from(val: Foo6FieldSet) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for Foo6FieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo6FieldSet");

        d.finish()
    }
}

impl core::ops::BitAnd for Foo6FieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Foo6FieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Foo6FieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Foo6FieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Foo6FieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Foo6FieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Foo6FieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo7FieldSet {
    /// The internal bits
    bits: [u8; 0],
}

impl ::device_driver::FieldSet for Foo7FieldSet {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl Foo7FieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}

impl Default for Foo7FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 0]> for Foo7FieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}

impl From<Foo7FieldSet> for [u8; 0] {
    fn from(val: Foo7FieldSet) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for Foo7FieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo7FieldSet");

        d.finish()
    }
}

impl core::ops::BitAnd for Foo7FieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Foo7FieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Foo7FieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Foo7FieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Foo7FieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Foo7FieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Foo7FieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo8FieldSet {
    /// The internal bits
    bits: [u8; 0],
}

impl ::device_driver::FieldSet for Foo8FieldSet {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl Foo8FieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}

impl Default for Foo8FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 0]> for Foo8FieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}

impl From<Foo8FieldSet> for [u8; 0] {
    fn from(val: Foo8FieldSet) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for Foo8FieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo8FieldSet");

        d.finish()
    }
}

impl core::ops::BitAnd for Foo8FieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Foo8FieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Foo8FieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Foo8FieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Foo8FieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Foo8FieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Foo8FieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Foo9FieldSet {
    /// The internal bits
    bits: [u8; 0],
}

impl ::device_driver::FieldSet for Foo9FieldSet {
    const SIZE_BITS: u32 = 0;
    fn get_inner_buffer(&self) -> &[u8] {
        &self.bits
    }
    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.bits
    }
}

impl Foo9FieldSet {
    /// Create a new instance, loaded with all zeroes
    pub const fn new() -> Self {
        Self { bits: [0; 0] }
    }
}

impl Default for Foo9FieldSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<[u8; 0]> for Foo9FieldSet {
    fn from(bits: [u8; 0]) -> Self {
        Self { bits }
    }
}

impl From<Foo9FieldSet> for [u8; 0] {
    fn from(val: Foo9FieldSet) -> Self {
        val.bits
    }
}

impl core::fmt::Debug for Foo9FieldSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Foo9FieldSet");

        d.finish()
    }
}

impl core::ops::BitAnd for Foo9FieldSet {
    type Output = Self;
    fn bitand(mut self, rhs: Self) -> Self::Output {
        self &= rhs;
        self
    }
}
impl core::ops::BitAndAssign for Foo9FieldSet {
    fn bitand_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l &= *r;
        }
    }
}
impl core::ops::BitOr for Foo9FieldSet {
    type Output = Self;
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}
impl core::ops::BitOrAssign for Foo9FieldSet {
    fn bitor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l |= *r;
        }
    }
}
impl core::ops::BitXor for Foo9FieldSet {
    type Output = Self;
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self ^= rhs;
        self
    }
}
impl core::ops::BitXorAssign for Foo9FieldSet {
    fn bitxor_assign(&mut self, rhs: Self) {
        for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
            *l ^= *r;
        }
    }
}
impl core::ops::Not for Foo9FieldSet {
    type Output = Self;
    fn not(mut self) -> Self::Output {
        for val in self.bits.iter_mut() {
            *val = !*val;
        }
        self
    }
}
