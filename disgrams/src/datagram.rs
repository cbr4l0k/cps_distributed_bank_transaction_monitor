use crate::errors::{DisgramsError, Result};
use std::time::{SystemTime, UNIX_EPOCH};

pub const HEADER_LEN: usize = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub node_id: u16,
    pub seq: u32,
    pub timestamp: Option<u64>,
}

impl Header {
    pub fn new(node_id: u16, seq: u32) -> Self {
        Header {
            node_id,
            seq,
            timestamp: None,
        }
    }

    pub fn to_byte_stream(&mut self) -> [u8; 14] {
        match self.timestamp {
            Some(timestamp) => self.to_byte_stream_with_timestamp(timestamp),
            None => self.to_byte_stream_update(),
        }
    }

    pub fn to_byte_stream_update(&mut self) -> [u8; 14] {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        self.timestamp = Some(timestamp);
        self.to_byte_stream_with_timestamp(timestamp)
    }

    fn to_byte_stream_with_timestamp(&self, timestamp: u64) -> [u8; 14] {
        let mut out = [0u8; 14];
        out[0..2].copy_from_slice(&self.node_id.to_be_bytes());
        out[2..6].copy_from_slice(&self.seq.to_be_bytes());
        out[6..14].copy_from_slice(&timestamp.to_be_bytes());
        out
    }

    pub fn from_byte_stream(bytes: [u8; 14]) -> Self {
        let node_id = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
        let seq = u32::from_be_bytes(bytes[2..6].try_into().unwrap());
        let timestamp = u64::from_be_bytes(bytes[6..14].try_into().unwrap());

        Header {
            node_id,
            seq,
            timestamp: Some(timestamp),
        }
    }
}

pub fn extract_node_id(packet: &[u8]) -> Result<u16> {
    if packet.len() < 2 {
        return Err(DisgramsError::InvalidPacketLength(packet.len(), 2));
    }
    let node_id = u16::from_be_bytes(packet[0..2].try_into().unwrap());
    Ok(node_id)
}

#[cfg(test)]
mod tests {
    use super::Header;

    #[test]
    fn header_round_trips_node_id_and_seq() {
        let mut header = Header::new(42, 1337);

        let decoded = Header::from_byte_stream(header.to_byte_stream());

        assert_eq!(decoded.node_id, header.node_id);
        assert_eq!(decoded.seq, header.seq);
        assert!(decoded.timestamp.is_some());
    }

    #[test]
    fn to_byte_stream_uses_existing_timestamp() {
        let mut header = Header {
            node_id: 42,
            seq: 1337,
            timestamp: Some(123_456),
        };

        let decoded = Header::from_byte_stream(header.to_byte_stream());

        assert_eq!(decoded.timestamp, Some(123_456));
        assert_eq!(header.timestamp, Some(123_456));
    }

    #[test]
    fn to_byte_stream_updates_missing_timestamp() {
        let mut header = Header::new(42, 1337);

        let decoded = Header::from_byte_stream(header.to_byte_stream());

        assert!(header.timestamp.is_some());
        assert_eq!(decoded.timestamp, header.timestamp);
    }

    #[test]
    fn to_byte_stream_update_replaces_existing_timestamp() {
        let mut header = Header {
            node_id: 42,
            seq: 1337,
            timestamp: Some(1),
        };

        let decoded = Header::from_byte_stream(header.to_byte_stream_update());

        assert_ne!(header.timestamp, Some(1));
        assert_eq!(decoded.timestamp, header.timestamp);
    }
}
