#![cfg_attr(not(any(test, feature = "std")), no_std)]

use core::num::NonZero;

mod rzcobs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MessageType {
    End = 0,
    Timestamp = 1,
    Operation = 2,
    Bytes = 3,
    Instance = 4,
    Finalize = 5,
}

impl MessageType {
    const MASK: u8 = 0b0000_0111;

    fn from_byte(byte: u8) -> Option<Self> {
        match byte & Self::MASK {
            0 => Some(MessageType::End),
            1 => Some(MessageType::Timestamp),
            2 => Some(MessageType::Operation),
            3 => Some(MessageType::Bytes),
            4 => Some(MessageType::Instance),
            5 => Some(MessageType::Finalize),
            _ => None,
        }
    }
}

struct Timestamp {
    timestamp: u64,
}

struct Instance {
    val: u8,
}

struct Bytes {
    len: NonZero<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    object_id: u32,
    index: Option<i32>,
    operation: RWOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RWOp {
    None = 0b00,
    Read = 0b01,
    Write = 0b10,
    Modify = 0b11,
}

impl RWOp {
    const MASK: u8 = 0b0000_0011;

    fn from_byte(byte: u8) -> Self {
        match byte & Self::MASK {
            0b00 => RWOp::None,
            0b01 => RWOp::Read,
            0b10 => RWOp::Write,
            0b11 => RWOp::Modify,
            _ => unreachable!(),
        }
    }
}

pub mod encoder {
    use core::{
        num::NonZero,
        sync::atomic::{AtomicU16, Ordering},
    };

    use crate::{Bytes, Instance, MessageType, Operation, RWOp, Timestamp, rzcobs::Encoder};

    unsafe extern "Rust" {
        safe fn _device_driver_trace_acquire();
        unsafe fn _device_driver_trace_write(byte: u8);
        unsafe fn _device_driver_trace_release();
        safe fn _device_driver_trace_timestamp() -> Option<u64>;
    }

    impl Timestamp {
        const MAX_LEN: usize = 9;

        fn encode<'b>(&self, buffer: &'b mut [u8]) -> &'b [u8] {
            debug_assert!(buffer.len() >= Self::MAX_LEN);

            buffer[0] = MessageType::Timestamp as u8;
            buffer[1..][..8].copy_from_slice(&self.timestamp.to_le_bytes());

            &buffer[..9]
        }
    }

    impl Instance {
        fn encode(&self) -> [u8; 2] {
            [MessageType::Instance as u8, self.val]
        }
    }

    impl Bytes {
        fn encode(&self) -> [u8; 1] {
            debug_assert!(self.len.get() <= 32);

            // | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
            // |       len-1       |   mtype   |
            [((self.len.get() - 1) as u8) << 3 | MessageType::Bytes as u8]
        }
    }

    impl Operation {
        pub fn new(object_id: u32, index: Option<i32>, operation: RWOp) -> Self {
            Self {
                object_id,
                index,
                operation,
            }
        }

        const MAX_LEN: usize = 9;

        fn encode<'b>(&self, buffer: &'b mut [u8]) -> &'b [u8] {
            debug_assert!(buffer.len() >= Self::MAX_LEN);

            // | 7 | 6 | 5 | 4 |   3   | 2 | 1 | 0 |
            // |   0   |  op   | index |   mtype   |
            buffer[0] = (self.operation as u8) << 4
                | (self.index.is_some() as u8) << 3
                | MessageType::Operation as u8;

            buffer[1..][..4].copy_from_slice(&self.object_id.to_le_bytes());

            let mut buffer_pos = 5;

            if let Some(index) = self.index {
                buffer[buffer_pos..][..4].copy_from_slice(&index.to_le_bytes());
                buffer_pos += 4;
            }

            &buffer[..buffer_pos]
        }
    }

    pub struct Event {
        pub(crate) id: u16,
    }

    static NEXT_ID: AtomicU16 = AtomicU16::new(0);

    impl Event {
        pub fn init_single_with(
            instance: u8,
            operation: Operation,
            bytes: &[u8],
            finalize: bool,
        ) -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::AcqRel);
            let mut encoder = Encoder::new();

            unsafe {
                _device_driver_trace_acquire();
                encoder.start_frame(|b| _device_driver_trace_write(b));

                encoder.write(&id.to_le_bytes(), |b| _device_driver_trace_write(b));
                encoder.write(&Instance { val: instance }.encode(), |b| {
                    _device_driver_trace_write(b)
                });

                if let Some(timestamp) = _device_driver_trace_timestamp() {
                    let mut buf = [0; Timestamp::MAX_LEN];
                    encoder.write(Timestamp { timestamp }.encode(&mut buf), |b| {
                        _device_driver_trace_write(b)
                    });
                }

                let mut buf = [0; Operation::MAX_LEN];
                encoder.write(operation.encode(&mut buf), |b| {
                    _device_driver_trace_write(b)
                });

                for byte_chunk in bytes.chunks(32) {
                    encoder.write(
                        &Bytes {
                            len: NonZero::new(byte_chunk.len()).unwrap(),
                        }
                        .encode(),
                        |b| _device_driver_trace_write(b),
                    );
                    encoder.write(byte_chunk, |b| _device_driver_trace_write(b));
                }

                if finalize {
                    encoder.write(&[MessageType::Finalize as u8], |b| {
                        _device_driver_trace_write(b)
                    });
                }

                encoder.end_frame(|b| _device_driver_trace_write(b));
                _device_driver_trace_release();
            }

            Self { id }
        }

        pub fn init(instance: u8) -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::AcqRel);

            let mut encoder = Encoder::new();

            unsafe {
                _device_driver_trace_acquire();
                encoder.start_frame(|b| _device_driver_trace_write(b));
                encoder.write(&id.to_le_bytes(), |b| _device_driver_trace_write(b));
                encoder.write(&Instance { val: instance }.encode(), |b| {
                    _device_driver_trace_write(b)
                });
                if let Some(timestamp) = _device_driver_trace_timestamp() {
                    let mut buf = [0; Timestamp::MAX_LEN];
                    encoder.write(Timestamp { timestamp }.encode(&mut buf), |b| {
                        _device_driver_trace_write(b)
                    });
                }
                encoder.end_frame(|b| _device_driver_trace_write(b));
                _device_driver_trace_release();
            }

            Self { id }
        }

        pub fn send_operation(&self, operation: Operation) {
            let mut buf = [0; Operation::MAX_LEN];
            let operation_buf = operation.encode(&mut buf);

            let mut encoder = Encoder::new();

            unsafe {
                _device_driver_trace_acquire();
                encoder.start_frame(|b| _device_driver_trace_write(b));
                encoder.write(&self.id.to_le_bytes(), |b| _device_driver_trace_write(b));
                encoder.write(operation_buf, |b| _device_driver_trace_write(b));
                encoder.end_frame(|b| _device_driver_trace_write(b));
                _device_driver_trace_release();
            }
        }

        pub fn send_bytes(&self, bytes: &[u8], finalize: bool) {
            let mut encoder = Encoder::new();
            unsafe {
                _device_driver_trace_acquire();
                encoder.start_frame(|b| _device_driver_trace_write(b));
                encoder.write(&self.id.to_le_bytes(), |b| _device_driver_trace_write(b));
                for byte_chunk in bytes.chunks(32) {
                    encoder.write(
                        &Bytes {
                            len: NonZero::new(byte_chunk.len()).unwrap(),
                        }
                        .encode(),
                        |b| _device_driver_trace_write(b),
                    );
                    encoder.write(byte_chunk, |b| _device_driver_trace_write(b));
                }

                if finalize {
                    encoder.write(&[MessageType::Finalize as u8], |b| {
                        _device_driver_trace_write(b)
                    });
                }

                encoder.end_frame(|b| _device_driver_trace_write(b));
                _device_driver_trace_release();
            }
        }
    }
}

#[cfg(any(test, feature = "std"))]
pub mod decoder {
    use std::collections::{HashMap, vec_deque::VecDeque};

    use crate::{Bytes, Instance, MessageType, Operation, RWOp, Timestamp, rzcobs::rzcobs_decode};

    impl Timestamp {
        fn decode(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
            if bytes.len() < 9 {
                return Err(DecodeError::Incomplete);
            }

            if MessageType::from_byte(bytes[0]) != Some(MessageType::Timestamp) {
                return Err(DecodeError::Malformed);
            }

            let timestamp = u64::from_le_bytes(*bytes[1..][..8].as_array().unwrap());
            Ok((Self { timestamp }, 9))
        }
    }

    impl Operation {
        fn decode(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
            if bytes.len() < 6 {
                return Err(DecodeError::Incomplete);
            }

            if MessageType::from_byte(bytes[0]) != Some(MessageType::Operation) {
                return Err(DecodeError::Malformed);
            }

            let mut operation = Operation {
                object_id: 0,
                index: None,
                operation: RWOp::None,
            };

            operation.operation = RWOp::from_byte((bytes[0] & 0b0011_0000) >> 4);
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

    impl Bytes {
        fn decode(bytes: &[u8]) -> Result<(Vec<u8>, usize), DecodeError> {
            if bytes.is_empty() {
                return Err(DecodeError::Incomplete);
            }

            if MessageType::from_byte(bytes[0]) != Some(MessageType::Bytes) {
                return Err(DecodeError::Malformed);
            }

            let len = ((bytes[0] & 0b1111_1000) >> 3) + 1;
            let bytes = &bytes[1..][..len as usize];

            Ok((bytes.to_vec(), bytes.len() + 1))
        }
    }

    impl Instance {
        fn decode(bytes: &[u8]) -> Result<(Self, usize), DecodeError> {
            if bytes.len() < 2 {
                return Err(DecodeError::Incomplete);
            }

            if MessageType::from_byte(bytes[0]) != Some(MessageType::Instance) {
                return Err(DecodeError::Malformed);
            }

            Ok((Instance { val: bytes[1] }, 2))
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Event {
        pub instance: u8,
        pub timestamp: Option<u64>,
        pub operations: Vec<(Vec<Operation>, Vec<Vec<u8>>)>,
    }

    impl Event {
        // bytes must already be decoded and not contain the id bytes anywhere. Full event (possibly consisting of multiple frames) must be present
        pub fn decode(frames: &[Vec<u8>]) -> Result<Self, DecodeError> {
            let mut event = Event {
                instance: 0,
                timestamp: None,
                operations: Vec::new(),
            };
            let mut instance_seen = false;

            let mut push_new_op = true;
            for bytes in frames {
                let mut bytes = &bytes[..];
                while !bytes.is_empty() {
                    let message_type =
                        MessageType::from_byte(*bytes.first().ok_or(DecodeError::Incomplete)?)
                            .ok_or(DecodeError::Malformed)?;

                    let bytes_used = match message_type {
                        MessageType::End => {
                            break;
                        }
                        MessageType::Timestamp => {
                            if event.timestamp.is_some() {
                                return Err(DecodeError::Malformed);
                            }
                            let (ts, len) = Timestamp::decode(bytes)?;
                            event.timestamp = Some(ts.timestamp);
                            len
                        }
                        MessageType::Operation => {
                            let (op, len) = Operation::decode(bytes)?;
                            if push_new_op {
                                push_new_op = false;
                                event.operations.push((vec![op], Vec::new()));
                            } else {
                                event.operations.last_mut().unwrap().0.push(op);
                            }
                            len
                        }
                        MessageType::Bytes => {
                            let (bytes_vec, len) = Bytes::decode(bytes)?;
                            event.operations.last_mut().unwrap().1.push(bytes_vec);
                            push_new_op = true;
                            len
                        }
                        MessageType::Instance => {
                            instance_seen = true;
                            let (instance, len) = Instance::decode(bytes)?;
                            event.instance = instance.val;
                            len
                        }
                        MessageType::Finalize => {
                            if !instance_seen {
                                return Err(DecodeError::Malformed);
                            } else {
                                return Ok(event);
                            }
                        }
                    };

                    bytes = &bytes[bytes_used..];
                }
            }

            Err(DecodeError::Incomplete)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DecodeError {
        Incomplete,
        Malformed,
    }

    pub struct StreamingDecoder {
        event_frames: HashMap<u16, Vec<Vec<u8>>>,
        buffer: VecDeque<u8>,
    }

    impl StreamingDecoder {
        pub fn new() -> Self {
            Self {
                event_frames: Default::default(),
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

                let Ok(mut frame) = rzcobs_decode(&frame) else {
                    return results;
                };

                if frame.len() < 2 {
                    continue;
                }

                let id = u16::from_le_bytes(*frame[0..2].as_array().unwrap());
                let frames = self.event_frames.entry(id).or_default();

                frames.push(frame.split_off(2));

                let decoded = Event::decode(frames);

                if matches!(decoded, Err(DecodeError::Incomplete)) {
                    continue;
                } else {
                    self.event_frames.remove(&id);
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
    use core::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    use crate::{
        decoder::{DecodeError, Event as DecodeEvent, StreamingDecoder},
        encoder::Event as EncodeEvent,
    };

    use super::*;

    static TAKEN: AtomicBool = AtomicBool::new(false);
    static mut RESTORE_STATE: critical_section::RestoreState =
        critical_section::RestoreState::invalid();
    static mut DECODER: Option<StreamingDecoder> = None;
    static EVENTS: Mutex<Vec<Result<DecodeEvent, DecodeError>>> = Mutex::new(Vec::new());

    #[unsafe(no_mangle)]
    extern "Rust" fn _device_driver_trace_acquire() {
        unsafe {
            let state = critical_section::acquire();
            if TAKEN.swap(true, Ordering::AcqRel) {
                panic!("device-driver trace acquire taken reentrantly");
            }
            RESTORE_STATE = state;
        }
    }
    #[unsafe(no_mangle)]
    extern "Rust" fn _device_driver_trace_write(byte: u8) {
        #[allow(static_mut_refs)]
        unsafe {
            EVENTS
                .lock()
                .unwrap()
                .extend_from_slice(&DECODER.get_or_insert_default().push_bytes(&[byte]));
        }
    }
    #[unsafe(no_mangle)]
    extern "Rust" fn _device_driver_trace_release() {
        unsafe {
            TAKEN.store(false, Ordering::Release);
            critical_section::release(RESTORE_STATE);
        }
    }
    #[unsafe(no_mangle)]
    extern "Rust" fn _device_driver_trace_timestamp() -> Option<u64> {
        Some(5)
    }

    #[test]
    fn roundtrip_single_events() {
        let _ = EncodeEvent::init_single_with(
            55,
            Operation::new(5, None, crate::RWOp::None),
            &[],
            true,
        );
        let modify_event = EncodeEvent::init_single_with(
            56,
            Operation::new(5, None, crate::RWOp::Modify),
            &[1, 2, 3],
            false,
        );
        let _ = EncodeEvent::init_single_with(
            57,
            Operation::new(5, None, crate::RWOp::Read),
            &[1, 2, 3],
            true,
        );
        modify_event.send_bytes(&[1, 2, 5], true);

        let events = EVENTS.lock().unwrap();
        let none_event = events
            .iter()
            .find(|e| e.as_ref().is_ok_and(|e| e.instance == 55))
            .unwrap();
        let modify_event = events
            .iter()
            .find(|e| e.as_ref().is_ok_and(|e| e.instance == 56))
            .unwrap();
        let read_event = events
            .iter()
            .find(|e| e.as_ref().is_ok_and(|e| e.instance == 57))
            .unwrap();

        assert_eq!(
            *none_event,
            Ok(DecodeEvent {
                instance: 55,
                timestamp: Some(5),
                operations: vec![(vec![Operation::new(5, None, crate::RWOp::None)], Vec::new())]
            })
        );
        assert_eq!(
            *modify_event,
            Ok(DecodeEvent {
                instance: 56,
                timestamp: Some(5),
                operations: vec![(
                    vec![Operation::new(5, None, crate::RWOp::Modify)],
                    vec![vec![1, 2, 3], vec![1, 2, 5]]
                )]
            })
        );
        assert_eq!(
            *read_event,
            Ok(DecodeEvent {
                instance: 57,
                timestamp: Some(5),
                operations: vec![(
                    vec![Operation::new(5, None, crate::RWOp::Read)],
                    vec![vec![1, 2, 3]]
                )]
            })
        );
    }

    #[test]
    fn roundtrip_bulk_events() {
        let bulk_event = EncodeEvent::init(123);
        bulk_event.send_operation(Operation::new(654, Some(-90), crate::RWOp::Modify));
        bulk_event.send_operation(Operation::new(655, Some(1), crate::RWOp::Modify));
        bulk_event.send_bytes(&[1, 2, 3, 4], false);
        bulk_event.send_bytes(&[0, 2, 3, 4], true);

        let events = EVENTS.lock().unwrap();
        let bulk_event = events
            .iter()
            .find(|e| e.as_ref().is_ok_and(|e| e.instance == 123))
            .unwrap();

        assert_eq!(
            *bulk_event,
            Ok(DecodeEvent {
                instance: 123,
                timestamp: Some(5),
                operations: vec![(
                    vec![
                        Operation::new(654, Some(-90), crate::RWOp::Modify),
                        Operation::new(655, Some(1), crate::RWOp::Modify)
                    ],
                    vec![vec![1, 2, 3, 4], vec![0, 2, 3, 4]]
                )]
            })
        );
    }
}
