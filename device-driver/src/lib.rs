#![allow(async_fn_in_trait)]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

pub use bitvec;
pub use device_driver_macros::*;
pub use embedded_io;
pub use funty;
pub use num_enum;

mod register;
pub use register::*;
mod command;
pub use command::*;
mod buffer;
pub use buffer::*;

#[doc(hidden)]
pub struct WriteOnly;
#[doc(hidden)]
pub struct ReadOnly;
#[doc(hidden)]
pub struct ReadWrite;
#[doc(hidden)]
pub struct ReadClear;
#[doc(hidden)]
pub struct ClearOnly;

#[doc(hidden)]
pub trait ReadCapability {}
#[doc(hidden)]
pub trait WriteCapability {}
#[doc(hidden)]
pub trait ClearCapability {}

impl WriteCapability for WriteOnly {}
impl ClearCapability for WriteOnly {}

impl ReadCapability for ReadOnly {}

impl WriteCapability for ReadWrite {}
impl ReadCapability for ReadWrite {}
impl ClearCapability for ReadWrite {}

impl ReadCapability for ReadClear {}
impl ClearCapability for ReadClear {}

impl ClearCapability for ClearOnly {}
