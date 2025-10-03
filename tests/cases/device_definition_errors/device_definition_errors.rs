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

pub mod foo_d_0 {
    /// Root block of the FooD0 driver
    #[derive(Debug)]
    pub struct FooD0<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD0<I> {
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
    }
}

pub mod foo_d_1 {
    /// Root block of the FooD1 driver
    #[derive(Debug)]
    pub struct FooD1<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD1<I> {
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
    }
}

pub mod foo_d_2 {
    /// Root block of the FooD2 driver
    #[derive(Debug)]
    pub struct FooD2<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD2<I> {
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
    }
}

pub mod foo_d_3 {
    /// Root block of the FooD3 driver
    #[derive(Debug)]
    pub struct FooD3<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD3<I> {
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
    }
}

pub mod foo_d_4 {
    /// Root block of the FooD4 driver
    #[derive(Debug)]
    pub struct FooD4<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD4<I> {
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
    }
}

pub mod foo_d_5 {
    /// Root block of the FooD5 driver
    #[derive(Debug)]
    pub struct FooD5<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD5<I> {
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
    }
}

pub mod foo_d_6 {
    /// Root block of the FooD6 driver
    #[derive(Debug)]
    pub struct FooD6<I> {
        pub(crate) interface: I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<I> FooD6<I> {
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
    
        pub fn foor_4(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, Foor4FieldSet, ::device_driver::RW> {
            let address = self.base_address + 0;
    
            ::device_driver::RegisterOperation::<'_, I, u8, Foor4FieldSet, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                Foor4FieldSet::new,
            )
        }
    
        pub fn foor_5(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, Foor5FieldSet, ::device_driver::RW> {
            let address = self.base_address + 1;
    
            ::device_driver::RegisterOperation::<'_, I, u8, Foor5FieldSet, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                Foor5FieldSet::new,
            )
        }
    
        pub fn foor_6(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, Foor6FieldSet, ::device_driver::RW> {
            let address = self.base_address + 2;
    
            ::device_driver::RegisterOperation::<'_, I, u8, Foor6FieldSet, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                Foor6FieldSet::new,
            )
        }
    
        pub fn foor_7(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, Foor7FieldSet, ::device_driver::RW> {
            let address = self.base_address + 3;
    
            ::device_driver::RegisterOperation::<'_, I, u8, Foor7FieldSet, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                Foor7FieldSet::new,
            )
        }
    
        /// Hello!
    
        pub fn foor_8(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, CustomFieldSetName, ::device_driver::RW>
        {
            let address = self.base_address + 4;
    
            ::device_driver::RegisterOperation::<
                            '_,
                            I,
    u8,
    CustomFieldSetName,
                            ::device_driver::RW,
                        >::new(
                            self.interface(),
                            address as u8,
    
    CustomFieldSetName::new,
    
    )
        }
    
        pub fn foor_9(
            &mut self,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, Foor9FieldSet, ::device_driver::RW> {
            let address = self.base_address + 5;
    
            ::device_driver::RegisterOperation::<'_, I, u8, Foor9FieldSet, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                Foor9FieldSet::new,
            )
        }
    
        pub fn foor_10(
            &mut self,
            index: Foo10E1,
        ) -> ::device_driver::RegisterOperation<'_, I, u8, Foor10FieldSet, ::device_driver::RW> {
            let address = self.base_address + 6 + u16::from(index) as u8 * 2;
    
            ::device_driver::RegisterOperation::<'_, I, u8, Foor10FieldSet, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
                Foor10FieldSet::new,
            )
        }
    
        pub fn fooc_1(
            &mut self,
        ) -> ::device_driver::CommandOperation<'_, I, u8, Fooc1FieldSetIn, Fooc1FieldSetOut> {
            let address = self.base_address + 0;
    
            ::device_driver::CommandOperation::<'_, I, u8, Fooc1FieldSetIn, Fooc1FieldSetOut>::new(
                self.interface(),
                address as u8,
            )
        }
    
        pub fn foob_1(&mut self) -> ::device_driver::BufferOperation<'_, I, u8, ::device_driver::RW> {
            let address = self.base_address + 0;
    
            ::device_driver::BufferOperation::<'_, I, u8, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
            )
        }
    
        pub fn foob_2(&mut self) -> ::device_driver::BufferOperation<'_, I, u8, ::device_driver::RO> {
            let address = self.base_address + 2;
    
            ::device_driver::BufferOperation::<'_, I, u8, ::device_driver::RO>::new(
                self.interface(),
                address as u8,
            )
        }
    
        /// This is a block
    
        pub fn b_1(&mut self) -> B1<'_, I> {
            let address = self.base_address + 5;
    
            B1::<'_, I>::new(self.interface(), address)
        }
    
        ///
        /// Valid index range: 0..2
    
        pub fn b_2(&mut self, index: usize) -> B2<'_, I> {
            let address = {
                assert!(index < 2);
                self.base_address + 0 + index as u8 * 4
            };
    
            B2::<'_, I>::new(self.interface(), address)
        }
    }
    
    /// This is a block
    #[derive(Debug)]
    pub struct B1<'i, I> {
        pub(crate) interface: &'i mut I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<'i, I> B1<'i, I> {
        /// Create a new instance of the block based on device interface
        #[doc(hidden)]
        fn new(interface: &'i mut I, base_address: u8) -> Self {
            Self {
                interface,
                base_address: base_address,
            }
        }
    
        /// A reference to the interface used to communicate with the device
        pub(crate) fn interface(&mut self) -> &mut I {
            self.interface
        }
    }
    
    #[derive(Debug)]
    pub struct B2<'i, I> {
        pub(crate) interface: &'i mut I,
    
        #[doc(hidden)]
        base_address: u8,
    }
    
    impl<'i, I> B2<'i, I> {
        /// Create a new instance of the block based on device interface
        #[doc(hidden)]
        fn new(interface: &'i mut I, base_address: u8) -> Self {
            Self {
                interface,
                base_address: base_address,
            }
        }
    
        /// A reference to the interface used to communicate with the device
        pub(crate) fn interface(&mut self) -> &mut I {
            self.interface
        }
    
        pub fn b_2_foo(&mut self) -> ::device_driver::BufferOperation<'_, I, u8, ::device_driver::RW> {
            let address = self.base_address + 42;
    
            ::device_driver::BufferOperation::<'_, I, u8, ::device_driver::RW>::new(
                self.interface(),
                address as u8,
            )
        }
    }
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foor4FieldSet {
        /// The internal bits
        bits: [u8; 0],
    }
    
    impl ::device_driver::FieldSet for Foor4FieldSet {
        const SIZE_BITS: u32 = 0;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl Foor4FieldSet {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 0] }
        }
    }
    
    impl Default for Foor4FieldSet {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 0]> for Foor4FieldSet {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }
    
    impl From<Foor4FieldSet> for [u8; 0] {
        fn from(val: Foor4FieldSet) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for Foor4FieldSet {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foor4FieldSet");
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for Foor4FieldSet {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Foor4FieldSet {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Foor4FieldSet {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Foor4FieldSet {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Foor4FieldSet {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Foor4FieldSet {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Foor4FieldSet {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foor5FieldSet {
        /// The internal bits
        bits: [u8; 0],
    }
    
    impl ::device_driver::FieldSet for Foor5FieldSet {
        const SIZE_BITS: u32 = 0;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl Foor5FieldSet {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 0] }
        }
    }
    
    impl Default for Foor5FieldSet {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 0]> for Foor5FieldSet {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }
    
    impl From<Foor5FieldSet> for [u8; 0] {
        fn from(val: Foor5FieldSet) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for Foor5FieldSet {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foor5FieldSet");
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for Foor5FieldSet {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Foor5FieldSet {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Foor5FieldSet {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Foor5FieldSet {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Foor5FieldSet {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Foor5FieldSet {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Foor5FieldSet {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foor6FieldSet {
        /// The internal bits
        bits: [u8; 0],
    }
    
    impl ::device_driver::FieldSet for Foor6FieldSet {
        const SIZE_BITS: u32 = 0;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl Foor6FieldSet {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 0] }
        }
    }
    
    impl Default for Foor6FieldSet {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 0]> for Foor6FieldSet {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }
    
    impl From<Foor6FieldSet> for [u8; 0] {
        fn from(val: Foor6FieldSet) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for Foor6FieldSet {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foor6FieldSet");
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for Foor6FieldSet {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Foor6FieldSet {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Foor6FieldSet {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Foor6FieldSet {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Foor6FieldSet {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Foor6FieldSet {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Foor6FieldSet {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foor7FieldSet {
        /// The internal bits
        bits: [u8; 0],
    }
    
    impl ::device_driver::FieldSet for Foor7FieldSet {
        const SIZE_BITS: u32 = 0;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl Foor7FieldSet {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 0] }
        }
    }
    
    impl Default for Foor7FieldSet {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 0]> for Foor7FieldSet {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }
    
    impl From<Foor7FieldSet> for [u8; 0] {
        fn from(val: Foor7FieldSet) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for Foor7FieldSet {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foor7FieldSet");
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for Foor7FieldSet {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Foor7FieldSet {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Foor7FieldSet {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Foor7FieldSet {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Foor7FieldSet {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Foor7FieldSet {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Foor7FieldSet {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    /// This fieldset has a custom name
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct CustomFieldSetName {
        /// The internal bits
        bits: [u8; 1],
    }
    
    impl ::device_driver::FieldSet for CustomFieldSetName {
        const SIZE_BITS: u32 = 8;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl CustomFieldSetName {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 1] }
        }
    
        ///Read the `bar` field of the register.
        ///
    
        pub fn bar(&self) -> bool {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 3, 4)
            };
            raw > 0
        }
    
        ///Read the `baz` field of the register.
        ///
    
        pub fn baz(&self) -> u8 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 4, 8)
            };
            raw
        }
    
        ///Write the `bar` field of the register.
        ///
    
        pub fn set_bar(&mut self, value: bool) {
            let raw = value as _;
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    3,
                    4,
                    &mut self.bits,
                )
            };
        }
    
        ///Write the `baz` field of the register.
        ///
    
        pub fn set_baz(&mut self, value: u8) {
            let raw = value;
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    4,
                    8,
                    &mut self.bits,
                )
            };
        }
    }
    
    impl Default for CustomFieldSetName {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 1]> for CustomFieldSetName {
        fn from(bits: [u8; 1]) -> Self {
            Self { bits }
        }
    }
    
    impl From<CustomFieldSetName> for [u8; 1] {
        fn from(val: CustomFieldSetName) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for CustomFieldSetName {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("CustomFieldSetName");
    
            d.field("bar", &self.bar());
    
            d.field("baz", &self.baz());
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for CustomFieldSetName {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for CustomFieldSetName {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for CustomFieldSetName {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for CustomFieldSetName {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for CustomFieldSetName {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for CustomFieldSetName {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for CustomFieldSetName {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foor9FieldSet {
        /// The internal bits
        bits: [u8; 1],
    }
    
    impl ::device_driver::FieldSet for Foor9FieldSet {
        const SIZE_BITS: u32 = 8;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl Foor9FieldSet {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 1] }
        }
    
        ///Read the `bar` field of the register.
        ///
    
        pub fn bar(&self) -> bool {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 0, 1)
            };
            raw > 0
        }
    
        ///Read the `baz` field of the register.
        ///
    
        pub fn baz(&self) -> u8 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 1, 3)
            };
            raw
        }
    
        ///Read the `quux` field of the register.
        ///
    
        pub fn quux(&self) -> u8 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 3, 5)
            };
            raw
        }
    
        ///Read the `qus` field of the register.
        ///
    
        pub fn qus(&self) -> Result<E3, <E3 as TryFrom<i32>>::Error> {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<i32, ::device_driver::ops::LE>(&self.bits, 5, 7)
            };
            raw.try_into()
        }
    
        ///Write the `bar` field of the register.
        ///
    
        pub fn set_bar(&mut self, value: bool) {
            let raw = value as _;
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    0,
                    1,
                    &mut self.bits,
                )
            };
        }
    
        ///Write the `baz` field of the register.
        ///
    
        pub fn set_baz(&mut self, value: u8) {
            let raw = value;
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    1,
                    3,
                    &mut self.bits,
                )
            };
        }
    
        ///Write the `quux` field of the register.
        ///
    
        pub fn set_quux(&mut self, value: u8) {
            let raw = value;
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    3,
                    5,
                    &mut self.bits,
                )
            };
        }
    
        ///Write the `qus` field of the register.
        ///
    
        pub fn set_qus(&mut self, value: E3) {
            let raw = value.into();
    
            unsafe {
                ::device_driver::ops::store_lsb0::<i32, ::device_driver::ops::LE>(
                    raw,
                    5,
                    7,
                    &mut self.bits,
                )
            };
        }
    }
    
    impl Default for Foor9FieldSet {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 1]> for Foor9FieldSet {
        fn from(bits: [u8; 1]) -> Self {
            Self { bits }
        }
    }
    
    impl From<Foor9FieldSet> for [u8; 1] {
        fn from(val: Foor9FieldSet) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for Foor9FieldSet {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foor9FieldSet");
    
            d.field("bar", &self.bar());
    
            d.field("baz", &self.baz());
    
            d.field("quux", &self.quux());
    
            d.field("qus", &self.qus());
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for Foor9FieldSet {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Foor9FieldSet {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Foor9FieldSet {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Foor9FieldSet {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Foor9FieldSet {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Foor9FieldSet {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Foor9FieldSet {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Foor10FieldSet {
        /// The internal bits
        bits: [u8; 1],
    }
    
    impl ::device_driver::FieldSet for Foor10FieldSet {
        const SIZE_BITS: u32 = 8;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl Foor10FieldSet {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 1] }
        }
    
        ///Read the `bar` field of the register.
        ///
    
        pub fn bar(&self) -> Foo10E1 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u16, ::device_driver::ops::LE>(&self.bits, 0, 2)
            };
            raw.into()
        }
    
        ///Read the `baz` field of the register.
        ///
    
        pub fn baz(&self) -> u8 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 2, 4)
            };
            raw
        }
    
        ///Read the `bam` field of the register.
        ///
    
        pub fn bam(&self) -> u8 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 4, 6)
            };
            raw
        }
    
        ///Read the `bat` field of the register.
        ///
    
        pub fn bat(&self) -> Result<Foo10E2, <Foo10E2 as TryFrom<u8>>::Error> {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 6, 8)
            };
            raw.try_into()
        }
    
        ///Write the `bar` field of the register.
        ///
    
        pub fn set_bar(&mut self, value: Foo10E1) {
            let raw = value.into();
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u16, ::device_driver::ops::LE>(
                    raw,
                    0,
                    2,
                    &mut self.bits,
                )
            };
        }
    
        ///Write the `baz` field of the register.
        ///
    
        pub fn set_baz(&mut self, value: u8) {
            let raw = value;
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    2,
                    4,
                    &mut self.bits,
                )
            };
        }
    
        ///Write the `bam` field of the register.
        ///
    
        pub fn set_bam(&mut self, value: u8) {
            let raw = value;
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    4,
                    6,
                    &mut self.bits,
                )
            };
        }
    
        ///Write the `bat` field of the register.
        ///
    
        pub fn set_bat(&mut self, value: Foo10E2) {
            let raw = value.into();
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    6,
                    8,
                    &mut self.bits,
                )
            };
        }
    }
    
    impl Default for Foor10FieldSet {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 1]> for Foor10FieldSet {
        fn from(bits: [u8; 1]) -> Self {
            Self { bits }
        }
    }
    
    impl From<Foor10FieldSet> for [u8; 1] {
        fn from(val: Foor10FieldSet) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for Foor10FieldSet {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Foor10FieldSet");
    
            d.field("bar", &self.bar());
    
            d.field("baz", &self.baz());
    
            d.field("bam", &self.bam());
    
            d.field("bat", &self.bat());
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for Foor10FieldSet {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Foor10FieldSet {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Foor10FieldSet {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Foor10FieldSet {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Foor10FieldSet {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Foor10FieldSet {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Foor10FieldSet {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Fooc1FieldSetIn {
        /// The internal bits
        bits: [u8; 0],
    }
    
    impl ::device_driver::FieldSet for Fooc1FieldSetIn {
        const SIZE_BITS: u32 = 0;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl Fooc1FieldSetIn {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 0] }
        }
    }
    
    impl Default for Fooc1FieldSetIn {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 0]> for Fooc1FieldSetIn {
        fn from(bits: [u8; 0]) -> Self {
            Self { bits }
        }
    }
    
    impl From<Fooc1FieldSetIn> for [u8; 0] {
        fn from(val: Fooc1FieldSetIn) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for Fooc1FieldSetIn {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Fooc1FieldSetIn");
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for Fooc1FieldSetIn {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Fooc1FieldSetIn {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Fooc1FieldSetIn {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Fooc1FieldSetIn {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Fooc1FieldSetIn {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Fooc1FieldSetIn {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Fooc1FieldSetIn {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Fooc1FieldSetOut {
        /// The internal bits
        bits: [u8; 1],
    }
    
    impl ::device_driver::FieldSet for Fooc1FieldSetOut {
        const SIZE_BITS: u32 = 8;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl Fooc1FieldSetOut {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 1] }
        }
    
        ///Read the `b` field of the register.
        ///
    
        pub fn b(&self) -> bool {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 0, 1)
            };
            raw > 0
        }
    
        ///Write the `b` field of the register.
        ///
    
        pub fn set_b(&mut self, value: bool) {
            let raw = value as _;
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    0,
                    1,
                    &mut self.bits,
                )
            };
        }
    }
    
    impl Default for Fooc1FieldSetOut {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 1]> for Fooc1FieldSetOut {
        fn from(bits: [u8; 1]) -> Self {
            Self { bits }
        }
    }
    
    impl From<Fooc1FieldSetOut> for [u8; 1] {
        fn from(val: Fooc1FieldSetOut) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for Fooc1FieldSetOut {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Fooc1FieldSetOut");
    
            d.field("b", &self.b());
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for Fooc1FieldSetOut {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Fooc1FieldSetOut {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Fooc1FieldSetOut {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Fooc1FieldSetOut {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Fooc1FieldSetOut {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Fooc1FieldSetOut {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Fooc1FieldSetOut {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Fs1 {
        /// The internal bits
        bits: [u8; 2],
    }
    
    impl ::device_driver::FieldSet for Fs1 {
        const SIZE_BITS: u32 = 16;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl Fs1 {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 2] }
        }
    
        ///Read the `value` field of the register.
        ///
    
        pub fn value(&self) -> u16 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u16, ::device_driver::ops::LE>(&self.bits, 0, 16)
            };
            raw
        }
    
        ///Write the `value` field of the register.
        ///
    
        pub fn set_value(&mut self, value: u16) {
            let raw = value;
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u16, ::device_driver::ops::LE>(
                    raw,
                    0,
                    16,
                    &mut self.bits,
                )
            };
        }
    }
    
    impl Default for Fs1 {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 2]> for Fs1 {
        fn from(bits: [u8; 2]) -> Self {
            Self { bits }
        }
    }
    
    impl From<Fs1> for [u8; 2] {
        fn from(val: Fs1) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for Fs1 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Fs1");
    
            d.field("value", &self.value());
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for Fs1 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Fs1 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Fs1 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Fs1 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Fs1 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Fs1 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Fs1 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct Fs2 {
        /// The internal bits
        bits: [u8; 4],
    }
    
    impl ::device_driver::FieldSet for Fs2 {
        const SIZE_BITS: u32 = 32;
        fn get_inner_buffer(&self) -> &[u8] {
            &self.bits
        }
        fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
            &mut self.bits
        }
    }
    
    impl Fs2 {
        /// Create a new instance, loaded with all zeroes
        pub const fn new() -> Self {
            Self { bits: [0; 4] }
        }
    
        ///Read the `value` field of the register.
        ///
    
        pub fn value(&self) -> Etype2 {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 0, 7)
            };
            raw.into()
        }
    
        ///Read the `value_2` field of the register.
        ///
    
        pub fn value_2(&self) -> Result<Etype3, <Etype3 as TryFrom<u8>>::Error> {
            let raw = unsafe {
                ::device_driver::ops::load_lsb0::<u8, ::device_driver::ops::LE>(&self.bits, 7, 11)
            };
            raw.try_into()
        }
    
        ///Write the `value` field of the register.
        ///
    
        pub fn set_value(&mut self, value: Etype2) {
            let raw = value.into();
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    0,
                    7,
                    &mut self.bits,
                )
            };
        }
    
        ///Write the `value_2` field of the register.
        ///
    
        pub fn set_value_2(&mut self, value: Etype3) {
            let raw = value.into();
    
            unsafe {
                ::device_driver::ops::store_lsb0::<u8, ::device_driver::ops::LE>(
                    raw,
                    7,
                    11,
                    &mut self.bits,
                )
            };
        }
    }
    
    impl Default for Fs2 {
        fn default() -> Self {
            Self::new()
        }
    }
    
    impl From<[u8; 4]> for Fs2 {
        fn from(bits: [u8; 4]) -> Self {
            Self { bits }
        }
    }
    
    impl From<Fs2> for [u8; 4] {
        fn from(val: Fs2) -> Self {
            val.bits
        }
    }
    
    impl core::fmt::Debug for Fs2 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
            let mut d = f.debug_struct("Fs2");
    
            d.field("value", &self.value());
    
            d.field("value_2", &self.value_2());
    
            d.finish()
        }
    }
    
    impl core::ops::BitAnd for Fs2 {
        type Output = Self;
        fn bitand(mut self, rhs: Self) -> Self::Output {
            self &= rhs;
            self
        }
    }
    impl core::ops::BitAndAssign for Fs2 {
        fn bitand_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l &= *r;
            }
        }
    }
    impl core::ops::BitOr for Fs2 {
        type Output = Self;
        fn bitor(mut self, rhs: Self) -> Self::Output {
            self |= rhs;
            self
        }
    }
    impl core::ops::BitOrAssign for Fs2 {
        fn bitor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l |= *r;
            }
        }
    }
    impl core::ops::BitXor for Fs2 {
        type Output = Self;
        fn bitxor(mut self, rhs: Self) -> Self::Output {
            self ^= rhs;
            self
        }
    }
    impl core::ops::BitXorAssign for Fs2 {
        fn bitxor_assign(&mut self, rhs: Self) {
            for (l, r) in self.bits.iter_mut().zip(&rhs.bits) {
                *l ^= *r;
            }
        }
    }
    impl core::ops::Not for Fs2 {
        type Output = Self;
        fn not(mut self) -> Self::Output {
            for val in self.bits.iter_mut() {
                *val = !*val;
            }
            self
        }
    }
    
    #[repr(u16)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    
    pub enum Foo10E1 {
        A = 0,
    
        B = 1,
    
        C = 2,
    
        D(u16) = 3,
    }
    
    impl Default for Foo10E1 {
        fn default() -> Self {
            Self::C
        }
    }
    
    impl From<u16> for Foo10E1 {
        fn from(val: u16) -> Self {
            match val {
                0 => Self::A,
    
                1 => Self::B,
    
                val => Self::D(val),
            }
        }
    }
    
    impl From<Foo10E1> for u16 {
        fn from(val: Foo10E1) -> Self {
            match val {
                Foo10E1::A => 0,
    
                Foo10E1::B => 1,
    
                Foo10E1::C => 2,
    
                Foo10E1::D(num) => num,
            }
        }
    }
    
    #[repr(u8)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    
    pub enum Foo10E2 {
        A = 0,
    }
    
    impl core::convert::TryFrom<u8> for Foo10E2 {
        type Error = ::device_driver::ConversionError<u8>;
        fn try_from(val: u8) -> Result<Self, Self::Error> {
            match val {
                0 => Ok(Self::A),
    
                val => Err(::device_driver::ConversionError {
                    source: val,
                    target: "Foo10E2",
                }),
            }
        }
    }
    
    impl From<Foo10E2> for u8 {
        fn from(val: Foo10E2) -> Self {
            match val {
                Foo10E2::A => 0,
            }
        }
    }
    
    #[repr(u8)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    
    pub enum E1 {
        A = 0,
    }
    
    impl core::convert::TryFrom<u8> for E1 {
        type Error = ::device_driver::ConversionError<u8>;
        fn try_from(val: u8) -> Result<Self, Self::Error> {
            match val {
                0 => Ok(Self::A),
    
                val => Err(::device_driver::ConversionError {
                    source: val,
                    target: "E1",
                }),
            }
        }
    }
    
    impl From<E1> for u8 {
        fn from(val: E1) -> Self {
            match val {
                E1::A => 0,
            }
        }
    }
    
    #[repr(u8)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    
    pub enum E2 {
        A = 0,
    
        B = 1,
    }
    
    impl core::convert::TryFrom<u8> for E2 {
        type Error = ::device_driver::ConversionError<u8>;
        fn try_from(val: u8) -> Result<Self, Self::Error> {
            match val {
                0 => Ok(Self::A),
    
                1 => Ok(Self::B),
    
                val => Err(::device_driver::ConversionError {
                    source: val,
                    target: "E2",
                }),
            }
        }
    }
    
    impl From<E2> for u8 {
        fn from(val: E2) -> Self {
            match val {
                E2::A => 0,
    
                E2::B => 1,
            }
        }
    }
    
    /// You can document enums too!
    #[repr(i32)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
    
    pub enum E3 {
        A = 0,
    
        /// Wow, it's a B!
        B = 1,
    }
    
    impl core::convert::TryFrom<i32> for E3 {
        type Error = ::device_driver::ConversionError<i32>;
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            match val {
                0 => Ok(Self::A),
    
                1 => Ok(Self::B),
    
                val => Err(::device_driver::ConversionError {
                    source: val,
                    target: "E3",
                }),
            }
        }
    }
    
    impl From<E3> for i32 {
        fn from(val: E3) -> Self {
            match val {
                E3::A => 0,
    
                E3::B => 1,
            }
        }
    }
}

compile_error!("The device driver input has errors that need to be solved!");
