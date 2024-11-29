use crate::{ReadCapability, WriteCapability};
use core::marker::PhantomData;

/// Common error definition for (async) [BufferInterface]
pub trait BufferInterfaceError {
    /// The error type
    type Error;
}

/// A trait to represent the interface to the device.
///
/// This is called to read from and write to buffers.
pub trait BufferInterface: BufferInterfaceError {
    /// The address type used by this interface. Should likely be an integer.
    type AddressType: Copy;

    /// Write to the buffer with the given address.
    ///
    /// This interface must adhere to [embedded_io::Write::write].
    fn write(&mut self, address: Self::AddressType, buf: &[u8]) -> Result<usize, Self::Error>;
    /// Flush this output stream with the given address.
    ///
    /// This interface must adhere to [embedded_io::Write::flush].
    fn flush(&mut self, address: Self::AddressType) -> Result<(), Self::Error>;
    /// Read from the buffer with the given address.
    ///
    /// This interface must adhere to [embedded_io::Read::read].
    fn read(&mut self, address: Self::AddressType, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

/// A trait to represent the interface to the device.
///
/// This is called to read from and write to buffers.
pub trait AsyncBufferInterface: BufferInterfaceError {
    /// The address type used by this interface. Should likely be an integer.
    type AddressType: Copy;

    /// Write to the buffer with the given address.
    ///
    /// This interface must adhere to [embedded_io_async::Write::write].
    async fn write(&mut self, address: Self::AddressType, buf: &[u8])
        -> Result<usize, Self::Error>;
    /// Flush this output stream with the given address.
    ///
    /// This interface must adhere to [embedded_io_async::Write::flush].
    async fn flush(&mut self, address: Self::AddressType) -> Result<(), Self::Error>;
    /// Read from the buffer with the given address.
    ///
    /// This interface must adhere to [embedded_io_async::Read::read].
    async fn read(
        &mut self,
        address: Self::AddressType,
        buf: &mut [u8],
    ) -> Result<usize, Self::Error>;
}

/// Intermediate type for doing buffer operations
///
/// If the interface error implements [embedded_io::Error],
/// then this operation type also implements the [embedded_io] traits
pub struct BufferOperation<'i, Interface, AddressType: Copy, Access> {
    interface: &'i mut Interface,
    address: AddressType,
    _phantom: PhantomData<Access>,
}

impl<'i, Interface, AddressType: Copy, Access> BufferOperation<'i, Interface, AddressType, Access> {
    #[doc(hidden)]
    pub fn new(interface: &'i mut Interface, address: AddressType) -> Self {
        Self {
            interface,
            address,
            _phantom: PhantomData,
        }
    }
}

impl<Interface, AddressType: Copy, Access> BufferOperation<'_, Interface, AddressType, Access>
where
    Interface: BufferInterface<AddressType = AddressType>,
    Access: WriteCapability,
{
    /// Write a buffer into this writer, returning how many bytes were written.
    ///
    /// Mirror function of [embedded_io::Write::write].
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, Interface::Error> {
        self.interface.write(self.address, buf)
    }

    /// Write an entire buffer into this writer.
    ///
    /// This function calls write() in a loop until exactly buf.len() bytes have been written, blocking if needed.
    ///
    /// Mirror function of [embedded_io::Write::write_all].
    pub fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Interface::Error> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => panic!("write() returned Ok(0)"),
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Flush this output stream, blocking until all intermediately buffered contents reach their destination.
    ///
    /// Mirror function of [embedded_io::Write::flush].
    pub fn flush(&mut self) -> Result<(), Interface::Error> {
        self.interface.flush(self.address)
    }
}

impl<Interface, AddressType: Copy, Access> BufferOperation<'_, Interface, AddressType, Access>
where
    Interface: BufferInterface<AddressType = AddressType>,
    Access: ReadCapability,
{
    /// Read some bytes from this source into the specified buffer, returning how many bytes were read.
    ///
    /// Mirror function of [embedded_io::Read::read].
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, Interface::Error> {
        self.interface.read(self.address, buf)
    }

    /// Read the exact number of bytes required to fill buf.
    /// This function calls read() in a loop until exactly buf.len() bytes have been read, blocking if needed.
    ///
    /// Mirror function of [embedded_io::Read::read_exact].
    pub fn read_exact(
        &mut self,
        mut buf: &mut [u8],
    ) -> Result<(), embedded_io::ReadExactError<Interface::Error>> {
        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => break,
                Ok(n) => buf = &mut buf[n..],
                Err(e) => return Err(embedded_io::ReadExactError::Other(e)),
            }
        }
        if buf.is_empty() {
            Ok(())
        } else {
            Err(embedded_io::ReadExactError::UnexpectedEof)
        }
    }
}

impl<'i, Interface, AddressType: Copy, Access> BufferOperation<'i, Interface, AddressType, Access>
where
    Interface: AsyncBufferInterface<AddressType = AddressType>,
    Access: WriteCapability,
{
    /// Write a buffer into this writer, returning how many bytes were written.
    ///
    /// Mirror function of [embedded_io_async::Write::write].
    pub async fn write_async(&mut self, buf: &[u8]) -> Result<usize, Interface::Error> {
        self.interface.write(self.address, buf).await
    }

    /// Write an entire buffer into this writer.
    ///
    /// This function calls write() in a loop until exactly buf.len() bytes have been written, blocking if needed.
    ///
    /// Mirror function of [embedded_io_async::Write::write_all].
    pub async fn write_all_async(&mut self, mut buf: &[u8]) -> Result<(), Interface::Error> {
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
    /// Mirror function of [embedded_io_async::Write::flush].
    pub async fn flush_async(&mut self) -> Result<(), Interface::Error> {
        self.interface.flush(self.address).await
    }
}

impl<'i, Interface, AddressType: Copy, Access> BufferOperation<'i, Interface, AddressType, Access>
where
    Interface: AsyncBufferInterface<AddressType = AddressType>,
    Access: ReadCapability,
{
    /// Read some bytes from this source into the specified buffer, returning how many bytes were read.
    ///
    /// Mirror function of [embedded_io_async::Read::read].
    pub async fn read_async(&mut self, buf: &mut [u8]) -> Result<usize, Interface::Error> {
        self.interface.read(self.address, buf).await
    }

    /// Read the exact number of bytes required to fill buf.
    ///
    /// This function calls read() in a loop until exactly buf.len() bytes have been read, waiting if needed.
    ///
    /// Mirror function of [embedded_io_async::Read::read_exact].
    pub async fn read_exact_async(
        &mut self,
        mut buf: &mut [u8],
    ) -> Result<(), embedded_io::ReadExactError<Interface::Error>> {
        while !buf.is_empty() {
            match self.read_async(buf).await {
                Ok(0) => break,
                Ok(n) => buf = &mut buf[n..],
                Err(e) => return Err(embedded_io::ReadExactError::Other(e)),
            }
        }
        if buf.is_empty() {
            Ok(())
        } else {
            Err(embedded_io::ReadExactError::UnexpectedEof)
        }
    }
}

// ------- embedded-io impls -------

impl<Interface, AddressType: Copy, Access> embedded_io::ErrorType
    for BufferOperation<'_, Interface, AddressType, Access>
where
    Interface: BufferInterfaceError,
    Interface::Error: embedded_io::Error,
{
    type Error = Interface::Error;
}

impl<Interface, AddressType: Copy, Access> embedded_io::Write
    for BufferOperation<'_, Interface, AddressType, Access>
where
    Interface: BufferInterface<AddressType = AddressType>,
    Interface::Error: embedded_io::Error,
    Access: WriteCapability,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush()
    }
}

impl<Interface, AddressType: Copy, Access> embedded_io::Read
    for BufferOperation<'_, Interface, AddressType, Access>
where
    Interface: BufferInterface<AddressType = AddressType>,
    Interface::Error: embedded_io::Error,
    Access: ReadCapability,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read(buf)
    }
}

impl<'i, Interface, AddressType: Copy, Access> embedded_io_async::Write
    for BufferOperation<'i, Interface, AddressType, Access>
where
    Interface: AsyncBufferInterface<AddressType = AddressType>,
    Interface::Error: embedded_io::Error,
    Access: WriteCapability,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.write_async(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush_async().await
    }
}

impl<'i, Interface, AddressType: Copy, Access> embedded_io_async::Read
    for BufferOperation<'i, Interface, AddressType, Access>
where
    Interface: AsyncBufferInterface<AddressType = AddressType>,
    Interface::Error: embedded_io::Error,
    Access: ReadCapability,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.read_async(buf).await
    }
}
