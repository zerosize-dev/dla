use parcel_codec::{Decoder, Frame, Kind};

/// What the relay keeps of a decoded frame once the codec's pointer has been dealt with.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Data { channel: u16, payload: Vec<u8> },
    Control { channel: u16, code: u8 },
    Heartbeat,
}

/// Reads the frames the decoder has ready and turns each into a `Message`.
pub fn drain(decoder: &mut Decoder) -> Vec<Message> {
    let mut out = Vec::new();
    while let Some(ptr) = decoder.next_frame() {
        let frame: &Frame = unsafe { &*ptr };
        match frame.kind() {
            Kind::Data => {
                out.push(Message::Data { channel: frame.channel(), payload: frame.payload().to_vec() });
                unsafe { parcel_codec::release(ptr) };
            }
            Kind::Control => {
                let code = frame.payload().first().copied().unwrap_or(0);
                out.push(Message::Control { channel: frame.channel(), code });
            }
            Kind::Heartbeat => out.push(Message::Heartbeat),
        }
    }
    out
}
