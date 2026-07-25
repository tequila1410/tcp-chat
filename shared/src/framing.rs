pub enum FrameResult {
    Complete(Vec<u8>),
    Incomplete,
    TooLarge,
}

pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let payload_len = payload.len() as u32;
    let mut encoded_frame = payload_len.to_be_bytes().to_vec();
    encoded_frame.extend_from_slice(payload);
    encoded_frame
}

pub fn decode_frame(bytes: &mut Vec<u8>) -> FrameResult {
    let header = match bytes.get(0..4) {
        Some(value) => value,
        None => return FrameResult::Incomplete,
    };

    let frame_len = match header.try_into() {
        Ok(arr) => u32::from_be_bytes(arr) as usize,
        Err(_) => return FrameResult::Incomplete,
    };
    if frame_len > 8 * 1024 {
        return FrameResult::TooLarge;
    }
    let frame_bytes = &bytes[4..];

    if frame_bytes.len() >= frame_len as usize {
        bytes.drain(..4);
        let res = bytes.drain(..frame_len as usize).collect::<Vec<u8>>();
        return FrameResult::Complete(res);
    }
    return FrameResult::Incomplete;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_encode_decode_test() {
        let payload = b"hello world";

        let encoded = encode_frame(payload);

        let mut buffer = encoded.clone();

        match decode_frame(&mut buffer) {
            FrameResult::Complete(decoded) => {
                assert_eq!(decoded, payload);
                assert!(buffer.is_empty());
            }
            _ => panic!("Expected complete frame")
        };

        
    } 

    #[test]
    fn decode_frame_returns_none_if_not_enough_data() {
        let payload = b"hello";

        let mut encoded = encode_frame(payload);

        encoded.pop();

        match decode_frame(&mut encoded) {
           FrameResult::Incomplete => {}
           _ =>  panic!("Expected complete frame")
        }
    }

    #[test]
    fn decode_multiple_frames() {
        let frame1 = encode_frame(b"hello");
        let frame2 = encode_frame(b"world");

        let mut buffer = Vec::new();

        buffer.extend_from_slice(&frame1);
        buffer.extend_from_slice(&frame2);

        match decode_frame(&mut buffer) {
            FrameResult::Complete(vec) => assert_eq!(vec, b"hello"),
            _ => panic!("Expected complete frame")
        }
        match decode_frame(&mut buffer) {
            FrameResult::Complete(vec) => assert_eq!(vec, b"world"),
            _ => panic!("Expected complete frame")
        }

        assert!(buffer.is_empty());
    }
}
