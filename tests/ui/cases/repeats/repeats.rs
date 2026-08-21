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

/// Root block of the Repeats driver
#[derive(Debug)]
pub struct Repeats<I> {
    interface: I,
    #[doc(hidden)]
    #[allow(unused)]
    base_address: u8,
}
impl<I> Repeats<I> {
    /// Create a new instance of the device
    pub const fn new(interface: I) -> Self {
        Self { interface, base_address: 0 }
    }
    /// Drop the driver instance and reclaim the interface
    pub fn free(self) -> I {
        self.interface
    }
    /// Block operation:
    /// - Address: `0`
    /// - Index range: `0..1`
    pub fn foo(&mut self, index: usize) -> Foo<'_, I> {
        let address = {
            assert!(index < 1);
            self.base_address + 0 + index as u8 * 1
        };
        Foo::<'_, I>::new(::device_driver::Block::interface(self), address)
    }
    /// Block operation:
    /// - Address: `0`
    /// - Index range: `0..1`
    pub fn bar(&mut self, index: usize) -> Bar<'_, I> {
        let address = {
            assert!(index < 1);
            self.base_address + 0 + index as u8 * 1
        };
        Bar::<'_, I>::new(::device_driver::Block::interface(self), address)
    }
    /// Block operation:
    /// - Address: `0`
    /// - Index range: `0..1`
    pub fn quux(&mut self, index: usize) -> Quux<'_, I> {
        let address = {
            assert!(index < 1);
            self.base_address + 0 + index as u8 * 1
        };
        Quux::<'_, I>::new(::device_driver::Block::interface(self), address)
    }
}
impl<I> ::device_driver::Block for Repeats<I> {
    type Interface = I;
    type RegisterAddressType = u8;
    type CommandAddressType = u8;
    type BufferAddressType = u8;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        &mut self.interface
    }
}
#[doc(alias = "foo")]
#[derive(Debug)]
pub struct Foo<'i, I> {
    #[doc(hidden)]
    interface: &'i mut I,
    #[doc(hidden)]
    #[allow(unused)]
    base_address: u8,
}
impl<'i, I> Foo<'i, I> {
    /// Create a new instance of the block based on device interface
    #[doc(hidden)]
    fn new(interface: &'i mut I, base_address: u8) -> Self {
        Self {
            interface,
            base_address: base_address,
        }
    }
}
impl<'i, I> ::device_driver::Block for Foo<'i, I> {
    type Interface = I;
    type RegisterAddressType = u8;
    type CommandAddressType = u8;
    type BufferAddressType = u8;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        self.interface
    }
}
#[doc(alias = "bar")]
#[derive(Debug)]
pub struct Bar<'i, I> {
    #[doc(hidden)]
    interface: &'i mut I,
    #[doc(hidden)]
    #[allow(unused)]
    base_address: u8,
}
impl<'i, I> Bar<'i, I> {
    /// Create a new instance of the block based on device interface
    #[doc(hidden)]
    fn new(interface: &'i mut I, base_address: u8) -> Self {
        Self {
            interface,
            base_address: base_address,
        }
    }
}
impl<'i, I> ::device_driver::Block for Bar<'i, I> {
    type Interface = I;
    type RegisterAddressType = u8;
    type CommandAddressType = u8;
    type BufferAddressType = u8;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        self.interface
    }
}
#[doc(alias = "quux")]
#[derive(Debug)]
pub struct Quux<'i, I> {
    #[doc(hidden)]
    interface: &'i mut I,
    #[doc(hidden)]
    #[allow(unused)]
    base_address: u8,
}
impl<'i, I> Quux<'i, I> {
    /// Create a new instance of the block based on device interface
    #[doc(hidden)]
    fn new(interface: &'i mut I, base_address: u8) -> Self {
        Self {
            interface,
            base_address: base_address,
        }
    }
}
impl<'i, I> ::device_driver::Block for Quux<'i, I> {
    type Interface = I;
    type RegisterAddressType = u8;
    type CommandAddressType = u8;
    type BufferAddressType = u8;
    type RegisterAddressMode = ();
    fn interface(&mut self) -> &mut Self::Interface {
        self.interface
    }
}
#[doc(alias = "too_big")]
#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TooBig {
    JustOk = 2147483647,
    TooBig = 2147483648,
}
impl core::convert::TryFrom<u32> for TooBig {
    type Error = ::device_driver::ConversionError<u32>;
    fn try_from(val: u32) -> Result<Self, Self::Error> {
        match val {
            2147483647 => Ok(Self::JustOk),
            2147483648 => Ok(Self::TooBig),
            val => {
                Err(::device_driver::ConversionError {
                    source: val,
                    target: "TooBig",
                })
            }
        }
    }
}
impl From<TooBig> for u32 {
    fn from(val: TooBig) -> Self {
        match val {
            TooBig::JustOk => 2147483647,
            TooBig::TooBig => 2147483648,
        }
    }
}
#[doc(hidden)]
impl ::device_driver::EnumIndex for TooBig {
    #[track_caller]
    fn index(&self) -> i32 {
        let index = u32::from(*self);
        index.try_into().unwrap()
    }
}
#[doc(alias = "small")]
#[repr(i8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Small {
    #[doc(alias = "a")]
    A = -3,
    #[doc(alias = "b")]
    B = -2,
    #[doc(alias = "c")]
    C = -1,
    #[doc(alias = "d")]
    D = 0,
}
impl core::convert::TryFrom<i8> for Small {
    type Error = ::device_driver::ConversionError<i8>;
    fn try_from(val: i8) -> Result<Self, Self::Error> {
        match val {
            -3 => Ok(Self::A),
            -2 => Ok(Self::B),
            -1 => Ok(Self::C),
            0 => Ok(Self::D),
            val => {
                Err(::device_driver::ConversionError {
                    source: val,
                    target: "Small",
                })
            }
        }
    }
}
impl From<Small> for i8 {
    fn from(val: Small) -> Self {
        match val {
            Small::A => -3,
            Small::B => -2,
            Small::C => -1,
            Small::D => 0,
        }
    }
}
#[doc(hidden)]
impl ::device_driver::EnumIndex for Small {
    #[track_caller]
    fn index(&self) -> i32 {
        let index = i8::from(*self);
        index.try_into().unwrap()
    }
}
compile_error!("The device driver input has errors that need to be solved!");
