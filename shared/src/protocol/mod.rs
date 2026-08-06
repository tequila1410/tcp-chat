mod client_message;
mod server_message;

pub use client_message::ClientMessage;
pub use server_message::ServerMessage;

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    InvalidMessageType(u8),
    InvalidMessageLength(usize),
    Truncated,
    InvalidUTF8,
}

fn read_u32(bytes: &[u8], pos: usize) -> Result<(u32, usize), DecodeError> {
    if bytes.len() < pos + 4 {
        return Err(DecodeError::Truncated);
    }
    let value = u32::from_be_bytes(
        bytes[pos..pos + 4]
            .try_into()
            .map_err(|_| DecodeError::Truncated)?,
    );
    Ok((value, pos + 4))
}

fn read_bytes(bytes: &[u8], pos: usize) -> Result<(&[u8], usize), DecodeError> {
    let (len, payload_start) = read_u32(bytes, pos)?;
    let len = len as usize;
    let payload_end = payload_start + len;
    if bytes.len() < payload_end {
        return Err(DecodeError::Truncated);
    }
    Ok((&bytes[payload_start..payload_end], payload_end))
}

fn read_string(bytes: &[u8], pos: usize) -> Result<(String, usize), DecodeError> {
    let (bytes_slice, pos) = read_bytes(bytes, pos)?;
    let text = String::from_utf8(bytes_slice.to_vec()).map_err(|_| DecodeError::InvalidUTF8)?;
    Ok((text, pos))
}
