use crate::protocol::{DecodeError, read_string, read_u32};

#[repr(u8)]
enum ServerMessageType {
    AuthOk = 1,
    AuthErr = 2,
    Message = 3,
    Err = 4,
    RoomCreated = 5,
    RoomJoined = 6,
    RoomErr = 7,
    RoomsGet = 8,
    RoomLeft = 9,
}

impl TryFrom<u8> for ServerMessageType {
    type Error = DecodeError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ServerMessageType::AuthOk),
            2 => Ok(ServerMessageType::AuthErr),
            3 => Ok(ServerMessageType::Message),
            4 => Ok(ServerMessageType::Err),
            5 => Ok(ServerMessageType::RoomCreated),
            6 => Ok(ServerMessageType::RoomJoined),
            7 => Ok(ServerMessageType::RoomErr),
            8 => Ok(ServerMessageType::RoomsGet),
            9 => Ok(ServerMessageType::RoomLeft),
            n => Err(DecodeError::InvalidMessageType(n)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ServerMessage {
    AuthOk,
    AuthErr(String),
    Message {
        room: String,
        from: String,
        text: String,
    },
    Err(String),
    RoomLeft(String),
    RoomCreated(String),
    RoomJoined(String),
    RoomErr(String),
    RoomsGet(Vec<String>)
}

impl ServerMessage {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        match self {
            Self::AuthErr(error) => {
                bytes.push(ServerMessageType::AuthErr as u8);

                let error_length = error.len() as u32;
                bytes.extend_from_slice(&error_length.to_be_bytes());
                bytes.extend_from_slice(error.as_bytes());
            }
            Self::AuthOk => {
                bytes.push(ServerMessageType::AuthOk as u8);
            }
            Self::Message{room, from, text} => {
                bytes.push(ServerMessageType::Message as u8);

                let room_length = room.len() as u32;
                bytes.extend_from_slice(&room_length.to_be_bytes());
                bytes.extend_from_slice(room.as_bytes());

                let from_length = from.len() as u32;
                bytes.extend_from_slice(&from_length.to_be_bytes());
                bytes.extend_from_slice(from.as_bytes());

                let text_length = text.len() as u32;
                bytes.extend_from_slice(&text_length.to_be_bytes());
                bytes.extend_from_slice(text.as_bytes());
            }
            Self::Err(error) => {
                bytes.push(ServerMessageType::Err as u8);

                let error_length = error.len() as u32;
                bytes.extend_from_slice(&error_length.to_be_bytes());
                bytes.extend_from_slice(error.as_bytes());
            }
            Self::RoomCreated(message) => {
                bytes.push(ServerMessageType::RoomCreated as u8);

                let message_length = message.len() as u32;
                bytes.extend_from_slice(&message_length.to_be_bytes());
                bytes.extend_from_slice(message.as_bytes());
            }
            Self::RoomJoined(message) => {
                bytes.push(ServerMessageType::RoomJoined as u8);

                let message_length = message.len() as u32;
                bytes.extend_from_slice(&message_length.to_be_bytes());
                bytes.extend_from_slice(message.as_bytes());
            }
            Self::RoomErr(error) => {
                bytes.push(ServerMessageType::RoomErr as u8);

                let error_length = error.len() as u32;
                bytes.extend_from_slice(&error_length.to_be_bytes());
                bytes.extend_from_slice(error.as_bytes());
            }
            Self::RoomsGet(rooms) => {
                bytes.push(ServerMessageType::RoomsGet as u8);

                let rooms = Self::serialize_strings(&rooms);
                bytes.extend_from_slice(&rooms);
            }
            Self::RoomLeft(message) => {
                bytes.push(ServerMessageType::RoomLeft as u8);

                let message_length = message.len() as u32;
                bytes.extend_from_slice(&message_length.to_be_bytes());
                bytes.extend_from_slice(message.as_bytes());
            }
        }
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 1 {
            return Err(DecodeError::InvalidMessageLength(bytes.len()));
        }

        let message_type = ServerMessageType::try_from(bytes[0])?;

        match message_type {
            ServerMessageType::AuthErr => {
                let (auth_error, _) = read_string(&bytes, 1)?;
                Ok(Self::AuthErr(auth_error))
            }
            ServerMessageType::AuthOk => {
                Ok(Self::AuthOk)
            }
            ServerMessageType::Message => {
                let (room, pos) = read_string(&bytes, 1)?;
                let (from, pos) = read_string(&bytes, pos)?;
                let (text, _) = read_string(&bytes, pos)?;
                Ok(Self::Message{room, from, text})
            }
            ServerMessageType::Err => {
                let (error, _) = read_string(&bytes, 1)?;
                Ok(Self::Err(error))
            }
            ServerMessageType::RoomCreated => {
                let (message, _) = read_string(&bytes, 1)?;
                Ok(Self::RoomCreated(message))
            }
            ServerMessageType::RoomJoined => {
                let (message, _) = read_string(&bytes, 1)?;
                Ok(Self::RoomJoined(message))
            }
            ServerMessageType::RoomErr => {
                let (error, _) = read_string(&bytes, 1)?;
                Ok(Self::RoomErr(error))
            }
            ServerMessageType::RoomsGet => {
                let rooms = Self::deserialize_strings(&bytes[1..])?;
                Ok(Self::RoomsGet(rooms))
            }
            ServerMessageType::RoomLeft => {
                let (message, _) = read_string(&bytes, 1)?;
                Ok(Self::RoomLeft(message))
            }
        }
    }

    fn serialize_strings(strings: &[String]) -> Vec<u8> {
        let mut bytes = Vec::new();

        let count = strings.len() as u32;
        bytes.extend_from_slice(&count.to_be_bytes());

        for s in strings {
            let len = s.len() as u32;
            bytes.extend_from_slice(&len.to_be_bytes());
            bytes.extend_from_slice(s.as_bytes());
        }

        bytes
    }

    fn deserialize_strings(bytes: &[u8]) -> Result<Vec<String>, DecodeError> {
        let (count, mut pos) = read_u32(bytes, 0)?;
        let mut result = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let (text, next) = read_string(bytes, pos)?;
            result.push(text);
            pos = next;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::DecodeError;

    #[test]
    fn auth_ok_round_trip() {
        let message = ServerMessage::AuthOk;
        let bytes = message.serialize();
        assert_eq!(ServerMessage::deserialize(&bytes).unwrap(), message);
    }

    #[test]
    fn message_round_trip() {
        let message = ServerMessage::Message {
            room: "rust".to_string(),
            from: "vlad".to_string(),
            text: "hello".to_string(),
        };
        let bytes = message.serialize();
        assert_eq!(ServerMessage::deserialize(&bytes).unwrap(), message);
    }

    #[test]
    fn rooms_get_round_trip() {
        let message = ServerMessage::RoomsGet(vec![
            "general".to_string(),
            "rust".to_string(),
        ]);
        let bytes = message.serialize();
        assert_eq!(ServerMessage::deserialize(&bytes).unwrap(), message);
    }

    #[test]
    fn rooms_get_empty_list_round_trip() {
        let message = ServerMessage::RoomsGet(vec![]);
        let bytes = message.serialize();
        assert_eq!(ServerMessage::deserialize(&bytes).unwrap(), message);
    }

    #[test]
    fn empty_payload_is_invalid_length() {
        assert_eq!(
            ServerMessage::deserialize(&[]).unwrap_err(),
            DecodeError::InvalidMessageLength(0)
        );
    }

    #[test]
    fn unknown_message_type() {
        assert_eq!(
            ServerMessage::deserialize(&[99]).unwrap_err(),
            DecodeError::InvalidMessageType(99)
        );
    }

    #[test]
    fn truncated_string_field() {
        let mut bytes = vec![ServerMessageType::AuthErr as u8];
        bytes.extend_from_slice(&4u32.to_be_bytes());
        bytes.extend_from_slice(b"ab");
        assert_eq!(
            ServerMessage::deserialize(&bytes).unwrap_err(),
            DecodeError::Truncated
        );
    }

    #[test]
    fn invalid_utf8_in_string_field() {
        let mut bytes = vec![ServerMessageType::RoomErr as u8];
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.push(0xff);
        assert_eq!(
            ServerMessage::deserialize(&bytes).unwrap_err(),
            DecodeError::InvalidUTF8
        );
    }

    #[test]
    fn rooms_get_truncated_list() {
        // RoomsGet + count=2, but no string payloads
        let mut bytes = vec![ServerMessageType::RoomsGet as u8];
        bytes.extend_from_slice(&2u32.to_be_bytes());
        assert_eq!(
            ServerMessage::deserialize(&bytes).unwrap_err(),
            DecodeError::Truncated
        );
    }
}
