use crate::{ReadCapability, WriteCapability};
use core::marker::PhantomData;
use embedded_io::ErrorKind;

/// A trait to represent the interface to the device.
///
/// This is called to read from and write to buffers.
pub trait BufferDevice {
    /// Write to the buffer with the given id.
    ///
    /// This interface should adhere to [embedded_io::Write::write].
    fn write(&mut self, id: u32, buf: &[u8]) -> Result<usize, ErrorKind>;
    /// Read from the buffer with the given id.
    ///
    /// This interface should adhere to [embedded_io::Read::read].
    fn read(&mut self, id: u32, buf: &mut [u8]) -> Result<usize, ErrorKind>;
}

/// A trait to represent the interface to the device.
///
/// This is called to read from and write to buffers.
pub trait AsyncBufferDevice {
    /// Write to the buffer with the given id.
    ///
    /// This interface should adhere to [embedded_io_async::Write::write].
    async fn write(&mut self, id: u32, buf: &[u8]) -> Result<usize, ErrorKind>;
    /// Read from the buffer with the given id.
    ///
    /// This interface should adhere to [embedded_io_async::Read::read].
    async fn read(&mut self, id: u32, buf: &mut [u8]) -> Result<usize, ErrorKind>;
}

/// Intermediate type for doing buffer operations
pub struct BufferOperation<'a, D, RWType> {
    device: &'a mut D,
    id: u32,
    _phantom: PhantomData<RWType>,
}

impl<'a, D, RWType> BufferOperation<'a, D, RWType> {
    #[doc(hidden)]
    pub fn new(device: &'a mut D, id: u32) -> Self {
        Self {
            device,
            id,
            _phantom: PhantomData,
        }
    }
}

impl<'a, D, RWType> embedded_io::ErrorType for BufferOperation<'a, D, RWType> {
    type Error = ErrorKind;
}

impl<'a, D, RWType> embedded_io::Read for BufferOperation<'a, D, RWType>
where
    D: BufferDevice,
    RWType: ReadCapability,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.device.read(self.id, buf)
    }
}

impl<'a, D, RWType> embedded_io_async::Read for BufferOperation<'a, D, RWType>
where
    D: AsyncBufferDevice,
    RWType: ReadCapability,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.device.read(self.id, buf).await
    }
}

impl<'a, D, RWType> embedded_io::Write for BufferOperation<'a, D, RWType>
where
    D: BufferDevice,
    RWType: WriteCapability,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.device.write(self.id, buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, D, RWType> embedded_io_async::Write for BufferOperation<'a, D, RWType>
where
    D: AsyncBufferDevice,
    RWType: WriteCapability,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.device.write(self.id, buf).await
    }
}
