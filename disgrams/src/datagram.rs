pub const HEADER_LEN: usize = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub node_id: u16,
    pub seq: u32,
    pub timestamp: u64,
}

impl Header {
    pub fn new(node_id: u16, seq: u32, timestamp: u64) -> Self {
        Header {
            node_id,
            seq,
            timestamp,
        }
    }

    pub fn to_byte_stream(self) -> [u8; 14] {
        let mut out = [0u8; 14];
        out[0..2].copy_from_slice(&self.node_id.to_be_bytes());
        out[2..6].copy_from_slice(&self.seq.to_be_bytes());
        out[6..14].copy_from_slice(&self.timestamp.to_be_bytes());
        out
    }

    pub fn from_byte_stream(bytes: [u8; 14]) -> Self {
        let node_id = u16::from_be_bytes(bytes[0..2].try_into().unwrap());
        let seq = u32::from_be_bytes(bytes[2..6].try_into().unwrap());
        let timestamp = u64::from_be_bytes(bytes[6..14].try_into().unwrap());

        Header {
            node_id,
            seq,
            timestamp,
        }
    }
}
