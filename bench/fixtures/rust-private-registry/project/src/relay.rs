use std::collections::HashMap;

use parcel_codec::Decoder;

use crate::frames::{drain, Message};

/// Routes decoded messages to per-channel counters.
pub struct Relay {
    decoder: Decoder,
    delivered: HashMap<u16, u64>,
    controls: u64,
    heartbeats: u64,
}

impl Relay {
    pub fn new(max_frame: usize) -> Relay {
        Relay { decoder: Decoder::new(max_frame), delivered: HashMap::new(), controls: 0, heartbeats: 0 }
    }

    /// Feeds bytes from a connection and routes whatever frames they complete.
    pub fn feed(&mut self, bytes: &[u8]) -> Result<usize, String> {
        self.decoder.push(bytes).map_err(|e| e.to_string())?;
        let messages = drain(&mut self.decoder);
        let count = messages.len();
        for message in messages {
            match message {
                Message::Data { channel, payload } => {
                    *self.delivered.entry(channel).or_insert(0) += payload.len() as u64;
                }
                Message::Control { .. } => self.controls += 1,
                Message::Heartbeat => self.heartbeats += 1,
            }
        }
        Ok(count)
    }

    pub fn delivered(&self, channel: u16) -> u64 {
        self.delivered.get(&channel).copied().unwrap_or(0)
    }

    pub fn controls(&self) -> u64 {
        self.controls
    }

    pub fn heartbeats(&self) -> u64 {
        self.heartbeats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire;

    #[test]
    fn data_frames_are_counted_per_channel() {
        let mut relay = Relay::new(4096);
        let mut bytes = wire::data(3, b"hello");
        bytes.extend(wire::data(3, b"world!"));
        bytes.extend(wire::data(9, b"x"));
        assert_eq!(relay.feed(&bytes), Ok(3));
        assert_eq!(relay.delivered(3), 11);
        assert_eq!(relay.delivered(9), 1);
    }

    #[test]
    fn control_and_heartbeat_frames_are_tallied() {
        let mut relay = Relay::new(4096);
        let mut bytes = wire::control(1, 7);
        bytes.extend(wire::heartbeat());
        bytes.extend(wire::heartbeat());
        assert_eq!(relay.feed(&bytes), Ok(3));
        assert_eq!((relay.controls(), relay.heartbeats()), (1, 2));
    }

    #[test]
    fn an_oversized_frame_is_refused() {
        let mut relay = Relay::new(16);
        let bytes = wire::data(1, &[0u8; 64]);
        assert!(relay.feed(&bytes).is_err());
    }
}
