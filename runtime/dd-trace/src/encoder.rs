use crate::{MessageType, ObjectIdentifier, TraceInterface, rzcobs::Encoder};

pub struct Frame {
    encoder: Encoder,
    interface: &'static TraceInterface,
}

impl Frame {
    pub fn new(interface: &'static TraceInterface, source_crc: u32, id: u8) -> Self {
        (interface.acquire)();
        let mut encoder = Encoder::new();
        encoder.start_frame(|b| unsafe { (interface.write)(&[b]) });

        let source_crc = source_crc.to_le_bytes();
        encoder.write(
            &[
                source_crc[0],
                source_crc[1],
                source_crc[2],
                source_crc[3],
                id,
            ],
            |b| unsafe { (interface.write)(&[b]) },
        );

        Self { encoder, interface }
    }

    fn write(&mut self, bytes: &[u8]) {
        self.encoder
            .write(bytes, |b| unsafe { (self.interface.write)(&[b]) });
    }

    pub fn send_timestamp(&mut self) {
        if let Some(timestamp) = unsafe { (self.interface.timestamp)() } {
            let mut buffer = [0; 9];

            // | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
            // |         0         |   mtype   |
            buffer[0] = MessageType::Timestamp as u8;
            buffer[1..][..8].copy_from_slice(&timestamp.to_le_bytes());

            self.write(&buffer[..9]);
        }
    }

    pub fn send_object_identifier(&mut self, operation: &ObjectIdentifier) {
        let mut buffer = [0; 9];

        // | 7 | 6 | 5 | 4 |   3   | 2 | 1 | 0 |
        // |       0       | index |   mtype   |
        buffer[0] = (operation.index.is_some() as u8) << 3 | MessageType::ObjectIdentifier as u8;

        buffer[1..][..4].copy_from_slice(&operation.object_id.to_le_bytes());

        let mut buffer_pos = 5;

        if let Some(index) = operation.index {
            buffer[buffer_pos..][..4].copy_from_slice(&index.to_le_bytes());
            buffer_pos += 4;
        }

        self.write(&buffer[..buffer_pos]);
    }

    pub fn send_bytes(&mut self, write: bool, bytes: &[u8]) {
        for byte_chunk in bytes.chunks(16) {
            // | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
            // |      len      | w |   mtype   |
            let header = [(byte_chunk.len() as u8 - 1) << 4
                | (write as u8) << 3
                | MessageType::OperationData as u8];
            self.write(&header);
            self.write(byte_chunk);
        }
    }

    pub fn send_address(&mut self, address: &[u8]) {
        debug_assert!(!address.is_empty(), "empty address is not allowed");
        debug_assert!(address.len() <= 16, "max address size is 16 bytes (i/u128)");
        // | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
        // |      len      | 0 |   mtype   |
        let header = [(address.len() as u8 - 1) << 4 | MessageType::OperationAddress as u8];
        self.write(&header);
        self.write(address);
    }

    fn send_finalize(&mut self) {
        self.write(&[MessageType::Finalize as u8]);
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        self.encoder
            .end_frame(|b| unsafe { (self.interface.write)(&[b]) });
        unsafe { (self.interface.release)() }
    }
}

pub struct Event {
    pub(crate) id: u8,
    finalized: bool,
    source_crc: u32,
}

impl Event {
    pub fn new(interface: &'static TraceInterface, source_crc: u32) -> Self {
        Self {
            id: (interface.id)(),
            finalized: false,
            source_crc,
        }
    }

    pub fn send_frame(
        self,
        interface: &'static TraceInterface,
        f: impl FnOnce(&mut Frame),
    ) -> Self {
        f(&mut Frame::new(interface, self.source_crc, self.id));
        self
    }

    pub fn send_final_frame(self, interface: &'static TraceInterface, f: impl FnOnce(&mut Frame)) {
        let mut frame = Frame::new(interface, self.source_crc, self.id);
        f(&mut frame);
        frame.send_finalize();
        self.discard();
    }

    pub fn discard(mut self) {
        self.finalized = true;
    }
}

impl Drop for Event {
    fn drop(&mut self) {
        if !self.finalized {
            panic!("event was not finalized")
        }
    }
}
