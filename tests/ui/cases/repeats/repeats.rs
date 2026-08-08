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
    ///
    /// Valid index range: `0..1`
    pub fn foo(&mut self, index: usize) -> Foo<'_, I> {
        let address = {
            assert!(index < 1);
            self.base_address + 0 + index as u8 * 1
        };
        Foo::<'_, I>::new(::device_driver::Block::interface(self), address)
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
compile_error!("The device driver input has errors that need to be solved!");
