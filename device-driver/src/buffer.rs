use crate::{Address, Block, ReadCapability, WriteCapability};
use core::marker::PhantomData;

/// Common properties shared by [`BufferInterface`] & [`AsyncBufferInterface`]
pub trait BufferInterfaceBase {
    /// The error type
    type Error;
    /// The address type used by this interface
    type AddressType: Address;
}

/// A trait to represent the interface to the device.
///
/// This is called to read from and write to buffers.
pub trait BufferInterface: BufferInterfaceBase {
    /// Write to the buffer with the given address.
    ///
    /// This interface must adhere to [`embedded_io::Write::write`].
    fn write(&mut self, address: Self::AddressType, buf: &[u8]) -> Result<usize, Self::Error>;
    /// Flush this output stream with the given address.
    ///
    /// This interface must adhere to [`embedded_io::Write::flush`].
    fn flush(&mut self, address: Self::AddressType) -> Result<(), Self::Error>;
    /// Read from the buffer with the given address.
    ///
    /// This interface must adhere to [`embedded_io::Read::read`].
    fn read(&mut self, address: Self::AddressType, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

/// A trait to represent the interface to the device.
///
/// This is called to read from and write to buffers.
pub trait AsyncBufferInterface: BufferInterfaceBase {
    /// Write to the buffer with the given address.
    ///
    /// This interface must adhere to [`embedded_io_async::Write::write`].
    async fn write(&mut self, address: Self::AddressType, buf: &[u8])
    -> Result<usize, Self::Error>;
    /// Flush this output stream with the given address.
    ///
    /// This interface must adhere to [`embedded_io_async::Write::flush`].
    async fn flush(&mut self, address: Self::AddressType) -> Result<(), Self::Error>;
    /// Read from the buffer with the given address.
    ///
    /// This interface must adhere to [`embedded_io_async::Read::read`].
    async fn read(
        &mut self,
        address: Self::AddressType,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error>;
}

/// Intermediate type for doing buffer operations
///
/// If the interface error implements [`embedded_io::Error`],
/// then this operation type also implements the [`embedded_io`] traits
pub struct BufferOperation<'b, B, AddressType, Access>
where
    B: Block,
    B::Interface: BufferInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
{
    block: &'b mut B,
    address: AddressType,
    _phantom: PhantomData<Access>,
}

impl<'b, B, AddressType, Access> BufferOperation<'b, B, AddressType, Access>
where
    B: Block,
    B::Interface: BufferInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
{
    #[doc(hidden)]
    pub fn new(
        interface: &'b mut B,
        address: <B::Interface as BufferInterfaceBase>::AddressType,
    ) -> Self {
        Self {
            block: interface,
            address,
            _phantom: PhantomData,
        }
    }
    /// Write a buffer into this writer, returning how many bytes were written.
    ///
    /// Mirror function of [`embedded_io::Write::write`].
    pub fn write(
        &mut self,
        buf: &[u8],
    ) -> Result<usize, <B::Interface as BufferInterfaceBase>::Error>
    where
        B::Interface: BufferInterface,
        Access: WriteCapability,
    {
        self.block.interface().write(self.address, buf)
    }

    /// Write a buffer into this writer, returning how many bytes were written.
    ///
    /// Mirror function of [`embedded_io_async::Write::write`].
    pub async fn write_async(
        &mut self,
        buf: &[u8],
    ) -> Result<usize, <B::Interface as BufferInterfaceBase>::Error>
    where
        B::Interface: AsyncBufferInterface,
        Access: WriteCapability,
    {
        self.block.interface().write(self.address, buf).await
    }

    /// Write an entire buffer into this writer.
    ///
    /// This function calls `write()` in a loop until exactly `buf.len()` bytes have been written, blocking if needed.
    ///
    /// Mirror function of [`embedded_io::Write::write_all`].
    pub fn write_all(
        &mut self,
        mut buf: &[u8],
    ) -> Result<(), <B::Interface as BufferInterfaceBase>::Error>
    where
        B::Interface: BufferInterface,
        Access: WriteCapability,
    {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => panic!("write() returned Ok(0)"),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Write an entire buffer into this writer.
    ///
    /// This function calls `write()` in a loop until exactly `buf.len()` bytes have been written, blocking if needed.
    ///
    /// Mirror function of [`embedded_io_async::Write::write_all`].
    pub async fn write_all_async(
        &mut self,
        mut buf: &[u8],
    ) -> Result<(), <B::Interface as BufferInterfaceBase>::Error>
    where
        B::Interface: AsyncBufferInterface,
        Access: WriteCapability,
    {
        while !buf.is_empty() {
            match self.write_async(buf).await {
                Ok(0) => panic!("write() returned Ok(0)"),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Flush this output stream, blocking until all intermediately buffered contents reach their destination.
    ///
    /// Mirror function of [`embedded_io::Write::flush`].
    pub fn flush(&mut self) -> Result<(), <B::Interface as BufferInterfaceBase>::Error>
    where
        B::Interface: BufferInterface,
        Access: WriteCapability,
    {
        self.block.interface().flush(self.address)
    }

    /// Flush this output stream, blocking until all intermediately buffered contents reach their destination.
    ///
    /// Mirror function of [`embedded_io_async::Write::flush`].
    pub async fn flush_async(&mut self) -> Result<(), <B::Interface as BufferInterfaceBase>::Error>
    where
        B::Interface: AsyncBufferInterface,
        Access: WriteCapability,
    {
        self.block.interface().flush(self.address).await
    }

    /// Read some bytes from this source into the specified buffer, returning how many bytes were read.
    ///
    /// Mirror function of [`embedded_io::Read::read`].
    pub fn read(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, <B::Interface as BufferInterfaceBase>::Error>
    where
        B::Interface: BufferInterface,
        Access: ReadCapability,
    {
        self.block.interface().read(self.address, buf)
    }

    /// Read some bytes from this source into the specified buffer, returning how many bytes were read.
    ///
    /// Mirror function of [`embedded_io_async::Read::read`].
    pub async fn read_async(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, <B::Interface as BufferInterfaceBase>::Error>
    where
        B::Interface: AsyncBufferInterface,
        Access: ReadCapability,
    {
        self.block.interface().read(self.address, buf).await
    }
}

// ------- embedded-io impls -------

#[cfg(feature = "embedded-io-07")]
impl<B, AddressType, Access> embedded_io::ErrorType for BufferOperation<'_, B, AddressType, Access>
where
    B: Block,
    B::Interface: BufferInterfaceBase<AddressType = AddressType>,
    <B::Interface as BufferInterfaceBase>::Error: embedded_io::Error,
    AddressType: Address,
{
    type Error = <B::Interface as BufferInterfaceBase>::Error;
}

#[cfg(feature = "embedded-io-07")]
impl<B, AddressType, Access> embedded_io::Write for BufferOperation<'_, B, AddressType, Access>
where
    B: Block,
    B::Interface: BufferInterface<AddressType = AddressType>,
    <B::Interface as BufferInterfaceBase>::Error: embedded_io::Error,
    Access: WriteCapability,
    AddressType: Address,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush()
    }
}

#[cfg(feature = "embedded-io-07")]
impl<B, AddressType, Access> embedded_io::Read for BufferOperation<'_, B, AddressType, Access>
where
    B: Block,
    B::Interface: BufferInterface<AddressType = AddressType>,
    <B::Interface as BufferInterfaceBase>::Error: embedded_io::Error,
    Access: ReadCapability,
    AddressType: Address,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(buf)
    }
}

#[cfg(feature = "embedded-io-07")]
impl<B, AddressType, Access> embedded_io_async::Write
    for BufferOperation<'_, B, AddressType, Access>
where
    B: Block,
    B::Interface: AsyncBufferInterface<AddressType = AddressType>,
    <B::Interface as BufferInterfaceBase>::Error: embedded_io::Error,
    Access: WriteCapability,
    AddressType: Address,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write_async(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush_async().await
    }
}

#[cfg(feature = "embedded-io-07")]
impl<B, AddressType, Access> embedded_io_async::Read for BufferOperation<'_, B, AddressType, Access>
where
    B: Block,
    B::Interface: AsyncBufferInterface<AddressType = AddressType>,
    <B::Interface as BufferInterfaceBase>::Error: embedded_io::Error,
    Access: ReadCapability,
    AddressType: Address,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_async(buf).await
    }
}
