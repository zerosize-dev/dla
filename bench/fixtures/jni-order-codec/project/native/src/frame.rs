//! Frame storage for the order codec: the bytes handed over by the JVM plus
//! the values derived from them.

/// Size of the fixed header at the start of every frame.
pub const HEADER_LEN: usize = 16;

/// Magic bytes at offset zero of a well-formed frame.
pub const MAGIC: [u8; 4] = *b"ORD1";

pub struct Frame {
    pub bytes: Vec<u8>,
    pub checksum: u32,
}

impl Frame {
    pub fn new(bytes: Vec<u8>) -> Frame {
        let checksum = checksum(&bytes);
        Frame { bytes, checksum }
    }

    /// Number of order lines announced by the header, if the header is intact.
    pub fn line_count(&self) -> Option<u32> {
        if self.bytes.len() < HEADER_LEN || self.bytes[..4] != MAGIC {
            return None;
        }
        let raw: [u8; 4] = self.bytes[8..12].try_into().ok()?;
        Some(u32::from_be_bytes(raw))
    }
}

/// Rolling checksum over the frame, multiplier 31 with wrapping arithmetic.
pub fn checksum(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(7u32, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_of_empty_input_is_the_seed() {
        assert_eq!(checksum(&[]), 7);
    }

    #[test]
    fn checksum_matches_hand_computed_value() {
        assert_eq!(checksum(&[1, 2]), (7 * 31 + 1) * 31 + 2);
    }

    #[test]
    fn header_line_count_is_read() {
        let mut bytes = vec![0u8; 32];
        bytes[..4].copy_from_slice(&MAGIC);
        bytes[8..12].copy_from_slice(&5u32.to_be_bytes());
        assert_eq!(Frame::new(bytes).line_count(), Some(5));
    }

    #[test]
    fn short_frame_has_no_line_count() {
        assert_eq!(Frame::new(vec![0u8; 4]).line_count(), None);
    }
}
