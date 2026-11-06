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

    fn decode_object_identifier(bytes: &[u8]) -> Result<(ObjectIdentifier, usize), DecodeError> {
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
            let event_data = self
                .instance_event_data
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
