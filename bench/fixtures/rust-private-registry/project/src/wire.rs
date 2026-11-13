//! Builders for the byte layout of the parcel protocol, used by the traffic generator and the tests.

const DATA: u8 = 1;
const CONTROL: u8 = 2;
const HEARTBEAT: u8 = 3;

fn frame(kind: u8, channel: u16, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(7 + payload.len());
    out.push(kind);
    out.extend_from_slice(&channel.to_be_bytes());
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(payload);
    out
}

pub fn data(channel: u16, payload: &[u8]) -> Vec<u8> {
    frame(DATA, channel, payload)
}

pub fn control(channel: u16, code: u8) -> Vec<u8> {
    frame(CONTROL, channel, &[code])
}

pub fn heartbeat() -> Vec<u8> {
    frame(HEARTBEAT, 0, &[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_carry_a_seven_byte_header() {
        assert_eq!(data(2, b"ab").len(), 9);
        assert_eq!(heartbeat().len(), 7);
        assert_eq!(control(1, 5), vec![2, 0, 1, 0, 0, 0, 1, 5]);
    }
}
