#![cfg_attr(not(any(test, feature = "std")), no_std)]

mod rzcobs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MessageType {
    #[cfg_attr(not(any(test, feature = "std")), expect(unused))]
    End = 0,
    Timestamp = 1,
    ObjectIdentifier = 2,
    OperationData = 3,
    OperationAddress = 4,
    Finalize = 5,
}

impl MessageType {
    #[cfg_attr(not(any(test, feature = "std")), expect(unused))]
    const MASK: u8 = 0b0000_0111;

    #[cfg_attr(not(any(test, feature = "std")), expect(unused))]
    fn from_byte(byte: u8) -> Option<Self> {
        match byte & Self::MASK {
            0 => Some(MessageType::End),
            1 => Some(MessageType::Timestamp),
            2 => Some(MessageType::ObjectIdentifier),
            3 => Some(MessageType::OperationData),
            4 => Some(MessageType::OperationAddress),
            5 => Some(MessageType::Finalize),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectIdentifier {
    object_id: u32,
    index: Option<i32>,
}

impl ObjectIdentifier {
    pub fn new(object_id: u32, index: Option<i32>) -> Self {
        Self { object_id, index }
    }
}

pub struct TraceInterface {
    /// Acquire global mutable access to the interface.
    /// This must not be reentrant.
    pub acquire: fn(),
    /// Write data into the sink
    pub write: unsafe fn(buf: &[u8]),
    /// Get the current time in microseconds.
    /// If a timestamp is not desired, return None.
    pub timestamp: unsafe fn() -> Option<u64>,
    /// Release the global mutable access to the interface.
    pub release: unsafe fn(),
    /// Get a new id for trace events.
    /// Should increment by 1 every time when called and wrap around to 0 at the end.
    pub id: fn() -> u8,
}

pub mod encoder {
    use crate::{MessageType, ObjectIdentifier, TraceInterface, rzcobs::Encoder};

    pub struct Frame {
        encoder: Encoder,
        interface: &'static TraceInterface,
    }

    impl Frame {
        pub fn new(interface: &'static TraceInterface, instance: u8, id: u8) -> Self {
            (interface.acquire)();
            let mut encoder = Encoder::new();
            encoder.start_frame(|b| unsafe { (interface.write)(&[b]) });

            encoder.write(&[instance, id], |b| unsafe { (interface.write)(&[b]) });

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
            buffer[0] =
                (operation.index.is_some() as u8) << 3 | MessageType::ObjectIdentifier as u8;

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
    }

    impl Event {
        pub fn new(interface: &'static TraceInterface) -> Self {
            Self {
                id: (interface.id)(),
                finalized: false,
            }
        }

        pub fn send_frame(
            self,
            interface: &'static TraceInterface,
            f: impl FnOnce(&mut Frame),
        ) -> Self {
            f(&mut Frame::new(interface, 55, self.id));
            self
        }

        pub fn send_final_frame(
            self,
            interface: &'static TraceInterface,
            f: impl FnOnce(&mut Frame),
        ) {
            let mut frame = Frame::new(interface, 55, self.id);
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
}

#[cfg(any(test, feature = "std"))]
pub mod decoder {
    use std::collections::{HashMap, vec_deque::VecDeque};

    use crate::{MessageType, ObjectIdentifier, rzcobs::rzcobs_decode};

    struct OperationDataChunk {
        write: bool,
        bytes: Vec<u8>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub struct Operation {
        pub objects: Vec<ObjectIdentifier>,
        pub address: Vec<u8>,
        pub read_data: Vec<u8>,
        pub write_data: Vec<u8>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Event {
        pub instance: u8,
        pub id: u8,
        pub timestamp: Option<u64>,
        pub operations: Vec<Operation>,
    }

    impl Event {
        // bytes must already be decoded and not contain the id bytes anywhere. Full event (possibly consisting of multiple frames) must be present
        pub fn decode(frames: &[Vec<u8>]) -> Result<Self, DecodeError> {
            if frames.is_empty() {
                return Err(DecodeError::Incomplete);
            }

            if frames[0].len() < 2 {
                return Err(DecodeError::Incomplete);
            }

            let mut event = Event {
                instance: frames[0][0],
                id: frames[0][1],
                timestamp: None,
                operations: Vec::new(),
            };

            // When true, a new object identifier will start a new operation
            let mut push_new_op = true;

            for frame in frames {
                if frame.len() < 3 {
                    return Err(DecodeError::Incomplete);
                }

                let instance = frame[0];
                let id = frame[1];

                if instance != event.instance || id != event.id {
                    return Err(DecodeError::Malformed);
                }

                let mut frame = &frame[2..];
                while !frame.is_empty() {
                    let message_type =
                        MessageType::from_byte(frame[0]).ok_or(DecodeError::Malformed)?;

                    let bytes_used = match message_type {
                        MessageType::End => {
                            break;
                        }
                        MessageType::Timestamp => {
                            if event.timestamp.is_some() {
                                return Err(DecodeError::Malformed);
                            }
                            let (ts, len) = Self::decode_timestamp(frame)?;
                            event.timestamp = Some(ts);
                            len
                        }
                        MessageType::ObjectIdentifier => {
                            let (object, len) = Self::decode_object_identifier(frame)?;
                            let active_operation = if push_new_op {
                                push_new_op = false;
                                event.operations.push_mut(Operation::default())
                            } else {
                                event.operations.last_mut().unwrap()
                            };
                            active_operation.objects.push(object);
                            len
                        }
                        MessageType::OperationData => {
                            let (OperationDataChunk { write, bytes }, len) =
                                Self::decode_operation_data(frame)?;
                            let active_operation = event.operations.last_mut().unwrap();

                            if write {
                                active_operation.write_data.extend_from_slice(&bytes);
                            } else {
                                active_operation.read_data.extend_from_slice(&bytes);
                            }

                            push_new_op = true;
                            len
                        }
                        MessageType::OperationAddress => {
                            let (address, len) = Self::decode_operation_address(frame)?;
                            let active_operation = event.operations.last_mut().unwrap();

                            if !active_operation.address.is_empty() {
                                // Can't set the address twice
                                return Err(DecodeError::Malformed);
                            }

                            active_operation.address = address;

                            push_new_op = true;
                            len
                        }
                        MessageType::Finalize => {
                            return Ok(event);
                        }
                    };

                    frame = &frame[bytes_used..];
                }
            }

            Err(DecodeError::Incomplete)
        }

        fn decode_operation_data(bytes: &[u8]) -> Result<(OperationDataChunk, usize), DecodeError> {
            if bytes.is_empty() {
                return Err(DecodeError::Incomplete);
            }

            if MessageType::from_byte(bytes[0]) != Some(MessageType::OperationData) {
                return Err(DecodeError::Malformed);
            }

            let len = ((bytes[0] & 0b1111_0000) >> 4) + 1;
            let write = (bytes[0] & 0b0000_1000) > 0;

            Ok((
                OperationDataChunk {
                    write,
                    bytes: bytes[1..][..len as usize].to_vec(),
                },
                1 + len as usize,
            ))
        }

        fn decode_operation_address(bytes: &[u8]) -> Result<(Vec<u8>, usize), DecodeError> {
            if bytes.is_empty() {
                return Err(DecodeError::Incomplete);
            }

            if MessageType::from_byte(bytes[0]) != Some(MessageType::OperationAddress) {
                return Err(DecodeError::Malformed);
            }

            let len = ((bytes[0] & 0b1111_0000) >> 4) + 1;

            Ok((bytes[1..][..len as usize].to_vec(), 1 + len as usize))
        }

        fn decode_timestamp(bytes: &[u8]) -> Result<(u64, usize), DecodeError> {
            if bytes.len() < 9 {
                return Err(DecodeError::Incomplete);
            }

            if MessageType::from_byte(bytes[0]) != Some(MessageType::Timestamp) {
                return Err(DecodeError::Malformed);
            }

            let timestamp = u64::from_le_bytes(*bytes[1..][..8].as_array().unwrap());
            Ok((timestamp, 9))
        }

        fn decode_object_identifier(
            bytes: &[u8],
        ) -> Result<(ObjectIdentifier, usize), DecodeError> {
            if bytes.len() < 6 {
                return Err(DecodeError::Incomplete);
            }

            if MessageType::from_byte(bytes[0]) != Some(MessageType::ObjectIdentifier) {
                return Err(DecodeError::Malformed);
            }

            let mut operation = ObjectIdentifier {
                object_id: 0,
                index: None,
            };

            let index = (bytes[0] & 0b0000_1000) > 0;

            operation.object_id = u32::from_le_bytes(*bytes[1..5].as_array().unwrap());

            if index {
                if bytes.len() < 9 {
                    return Err(DecodeError::Incomplete);
                }
                operation.index = Some(i32::from_le_bytes(*bytes[5..9].as_array().unwrap()));
            }

            Ok((operation, if index { 9 } else { 5 }))
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DecodeError {
        Incomplete,
        Malformed,
    }

    struct EventData {
        id: u8,
        frames: Vec<Vec<u8>>,
    }

    pub struct StreamingDecoder {
        instance_event_data: HashMap<u8, EventData>,
        buffer: VecDeque<u8>,
    }

    impl StreamingDecoder {
        pub fn new() -> Self {
            Self {
                instance_event_data: Default::default(),
                buffer: Default::default(),
            }
        }

        pub fn push_bytes(&mut self, bytes: &[u8]) -> Vec<Result<Event, DecodeError>> {
            let mut results = Vec::new();

            self.buffer.extend(bytes);

            while self.buffer.contains(&0) {
                // Remove any zeroes at the start
                while self.buffer.pop_front_if(|val| *val == 0).is_some() {}

                if !self.buffer.contains(&0) {
                    return results;
                }

                let mut frame = Vec::new();

                while let Some(byte) = self.buffer.pop_front_if(|val| *val != 0) {
                    frame.push(byte);
                }

                self.buffer.pop_front(); // Pop the extra 0

                let Ok(frame) = rzcobs_decode(&frame) else {
                    continue;
                };

                if frame.len() < 2 {
                    continue;
                }

                let frame_instance = frame[0];
                let frame_id = frame[1];
                let event_data =
                    self.instance_event_data
                        .entry(frame_instance)
                        .or_insert(EventData {
                            id: frame_id,
                            frames: Vec::new(),
                        });

                if event_data.id != frame_id {
                    // We're getting newer(?) data. We require everything to arrive in order.
                    // So to recover, throw away the old data.
                    *event_data = EventData {
                        id: frame_id,
                        frames: Vec::new(),
                    };
                }

                event_data.frames.push(frame);

                let decoded = Event::decode(&event_data.frames);

                if matches!(decoded, Err(DecodeError::Incomplete)) {
                    continue;
                } else {
                    self.instance_event_data.remove(&frame_instance);
                    results.push(decoded);
                }
            }

            results
        }
    }

    impl Default for StreamingDecoder {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicU8, Ordering};

    use critical_section::RestoreState;

    use crate::{
        decoder::{Event as DecodeEvent, Operation, StreamingDecoder},
        encoder::Event as EncodeEvent,
    };

    use super::*;

    #[test]
    fn roundtrip_single_events() {
        static mut WRITTEN_BYTES: Vec<u8> = Vec::new();
        static mut TAKEN: bool = false;
        static mut RESTORE_STATE: RestoreState = RestoreState::invalid();

        static INTERFACE: TraceInterface = TraceInterface {
            acquire: || {
                unsafe { RESTORE_STATE = critical_section::acquire() };
                if unsafe { TAKEN } {
                    panic!("trace interface taken reentrantly");
                }
            },
            write: |buf| unsafe {
                #[allow(static_mut_refs)]
                WRITTEN_BYTES.extend(buf);
            },
            timestamp: || Some(5),
            release: || unsafe {
                TAKEN = false;
                critical_section::release(RESTORE_STATE);
            },
            id: || {
                static ID: AtomicU8 = AtomicU8::new(0);
                ID.fetch_add(1, Ordering::Relaxed)
            },
        };

        let none_event = EncodeEvent::new(&INTERFACE);
        let none_event_id = none_event.id;
        none_event.send_final_frame(&INTERFACE, |f| {
            f.send_timestamp();
            f.send_object_identifier(&ObjectIdentifier::new(5, None));
        });

        let modify_event = EncodeEvent::new(&INTERFACE);
        let modify_event_id = modify_event.id;
        let modify_event = modify_event.send_frame(&INTERFACE, |f| {
            f.send_timestamp();
            f.send_object_identifier(&ObjectIdentifier::new(5, None));
            f.send_address(&[1]);
            f.send_bytes(false, &[1, 2, 3]);
        });
        modify_event.send_final_frame(&INTERFACE, |f| {
            f.send_bytes(true, &[1, 2, 5]);
        });

        let read_event = EncodeEvent::new(&INTERFACE);
        let read_event_id = read_event.id;
        read_event.send_final_frame(&INTERFACE, |f| {
            f.send_timestamp();
            f.send_object_identifier(&ObjectIdentifier::new(5, None));
            f.send_address(&[12, 34, 56]);
            f.send_bytes(false, &[1, 2, 3]);
        });

        (INTERFACE.acquire)();
        #[allow(static_mut_refs)]
        let written_bytes = unsafe { WRITTEN_BYTES.clone() };
        unsafe {
            (INTERFACE.release)();
        }

        println!("Cobs encoded written bytes: {:X?}", &written_bytes);

        let mut decoder = StreamingDecoder::new();
        let decoded_events = decoder
            .push_bytes(&written_bytes)
            .into_iter()
            .map(|e| e.unwrap())
            .collect::<Vec<_>>();

        let decoded_none_event = decoded_events
            .iter()
            .find(|e| e.id == none_event_id)
            .unwrap();
        let decoded_modify_event = decoded_events
            .iter()
            .find(|e| e.id == modify_event_id)
            .unwrap();
        let decoded_read_event = decoded_events
            .iter()
            .find(|e| e.id == read_event_id)
            .unwrap();

        assert_eq!(
            decoded_none_event,
            &DecodeEvent {
                instance: 55,
                id: none_event_id,
                timestamp: Some(5),
                operations: vec![Operation {
                    objects: vec![ObjectIdentifier::new(5, None)],
                    address: Vec::new(),
                    read_data: Vec::new(),
                    write_data: Vec::new()
                }]
            }
        );
        assert_eq!(
            decoded_modify_event,
            &DecodeEvent {
                instance: 55,
                id: modify_event_id,
                timestamp: Some(5),
                operations: vec![Operation {
                    objects: vec![ObjectIdentifier::new(5, None)],
                    address: vec![1],
                    read_data: vec![1, 2, 3],
                    write_data: vec![1, 2, 5],
                }]
            }
        );
        assert_eq!(
            decoded_read_event,
            &DecodeEvent {
                instance: 55,
                id: read_event_id,
                timestamp: Some(5),
                operations: vec![Operation {
                    objects: vec![ObjectIdentifier::new(5, None)],
                    address: vec![12, 34, 56],
                    read_data: vec![1, 2, 3],
                    write_data: Vec::new(),
                }]
            }
        );
    }

    #[test]
    fn roundtrip_bulk_events() {
        static mut WRITTEN_BYTES: Vec<u8> = Vec::new();
        static mut TAKEN: bool = false;
        static mut RESTORE_STATE: RestoreState = RestoreState::invalid();

        static INTERFACE: TraceInterface = TraceInterface {
            acquire: || {
                unsafe { RESTORE_STATE = critical_section::acquire() };
                if unsafe { TAKEN } {
                    panic!("trace interface taken reentrantly");
                }
            },
            write: |buf| unsafe {
                #[allow(static_mut_refs)]
                WRITTEN_BYTES.extend(buf);
            },
            timestamp: || Some(5),
            release: || unsafe {
                TAKEN = false;
                critical_section::release(RESTORE_STATE);
            },
            id: || {
                static ID: AtomicU8 = AtomicU8::new(0);
                ID.fetch_add(1, Ordering::Relaxed)
            },
        };

        let bulk_event = EncodeEvent::new(&INTERFACE);
        let bulk_event_id = bulk_event.id;
        let bulk_event = bulk_event.send_frame(&INTERFACE, |f| {
            f.send_object_identifier(&ObjectIdentifier::new(654, Some(-90)));
            f.send_object_identifier(&ObjectIdentifier::new(655, Some(1)));
            f.send_address(&[0xFF, 0xFF]);
            f.send_bytes(false, &[0xDD; 60]);
        });
        bulk_event.send_final_frame(&INTERFACE, |f| {
            f.send_bytes(true, &[0, 2, 3, 4]);
        });

        (INTERFACE.acquire)();
        #[allow(static_mut_refs)]
        let written_bytes = unsafe { WRITTEN_BYTES.clone() };
        unsafe {
            (INTERFACE.release)();
        }

        let mut decoder = StreamingDecoder::new();
        let decoded_events = decoder
            .push_bytes(&written_bytes)
            .into_iter()
            .map(|e| e.unwrap())
            .collect::<Vec<_>>();

        let bulk_event = decoded_events
            .iter()
            .find(|e| e.id == bulk_event_id)
            .unwrap();

        assert_eq!(
            bulk_event,
            &DecodeEvent {
                instance: 55,
                id: bulk_event_id,
                timestamp: None,
                operations: vec![Operation {
                    objects: vec![
                        ObjectIdentifier::new(654, Some(-90)),
                        ObjectIdentifier::new(655, Some(1))
                    ],
                    address: vec![0xFF, 0xFF],
                    read_data: vec![0xDD; 60],
                    write_data: vec![0, 2, 3, 4],
                }]
            }
        );
    }
}
