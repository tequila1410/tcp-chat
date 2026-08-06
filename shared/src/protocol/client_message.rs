use crate::protocol::{DecodeError, read_string};

#[repr(u8)]
enum ClientMessageType {
    Auth = 1,
    LeaveRoom = 2,
    CreateRoom = 3,
    JoinRoom = 4,
    GetRooms = 5,
    SendToRoom = 6,
}

impl TryFrom<u8> for ClientMessageType {
    type Error = DecodeError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ClientMessageType::Auth),
            2 => Ok(ClientMessageType::LeaveRoom),
            3 => Ok(ClientMessageType::CreateRoom),
            4 => Ok(ClientMessageType::JoinRoom),
            5 => Ok(ClientMessageType::GetRooms),
            6 => Ok(ClientMessageType::SendToRoom),
            n => Err(DecodeError::InvalidMessageType(n)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ClientMessage {
    Auth {
        login: String,
        password: String,
    },

    CreateRoom(String),
    JoinRoom(String),
    LeaveRoom,
    GetRooms,
    SendToRoom {
        room: String,
        text: String,
    },
}

impl ClientMessage {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        match self {
            ClientMessage::Auth { login, password } => {
                bytes.push(ClientMessageType::Auth as u8);

                let login_length = login.len() as u32;
                bytes.extend_from_slice(&login_length.to_be_bytes());
                bytes.extend_from_slice(login.as_bytes());

                let pass_length = password.len() as u32;
                bytes.extend_from_slice(&pass_length.to_be_bytes());
                bytes.extend_from_slice(password.as_bytes());
            }
            ClientMessage::LeaveRoom => {
                bytes.push(ClientMessageType::LeaveRoom as u8);
            }
            ClientMessage::CreateRoom(room_name) => {
                bytes.push(ClientMessageType::CreateRoom as u8);

                let room_name_length = room_name.len() as u32;
                bytes.extend_from_slice(&room_name_length.to_be_bytes());
                bytes.extend_from_slice(room_name.as_bytes());
            }
            ClientMessage::JoinRoom(room_name) => {
                bytes.push(ClientMessageType::JoinRoom as u8);

                let room_name_length = room_name.len() as u32;
                bytes.extend_from_slice(&room_name_length.to_be_bytes());
                bytes.extend_from_slice(room_name.as_bytes());
            }
            ClientMessage::GetRooms => {
                bytes.push(ClientMessageType::GetRooms as u8);
            }
            ClientMessage::SendToRoom { room, text } => {
                bytes.push(ClientMessageType::SendToRoom as u8);

                let room_length = room.len() as u32;
                bytes.extend_from_slice(&room_length.to_be_bytes());
                bytes.extend_from_slice(room.as_bytes());

                let text_length = text.len() as u32;
                bytes.extend_from_slice(&text_length.to_be_bytes());
                bytes.extend_from_slice(text.as_bytes());
            }
        }
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < 1 {
            return Err(DecodeError::Truncated);
        }

        let message_type = ClientMessageType::try_from(bytes[0])?;

        match message_type {
            ClientMessageType::Auth => {
                let (login, pos) = read_string(&bytes, 1)?;
                let (password, _) = read_string(&bytes, pos)?;
                Ok(Self::Auth{ login, password })
            },
            ClientMessageType::LeaveRoom => {
                Ok(Self::LeaveRoom)
            }
            ClientMessageType::CreateRoom => {
                let (message, _) = read_string(&bytes, 1)?;
                Ok(Self::CreateRoom(message))
            }
            ClientMessageType::JoinRoom => {
                let (message, _) = read_string(&bytes, 1)?;
                Ok(Self::JoinRoom(message))
            }
            ClientMessageType::GetRooms => {
                Ok(Self::GetRooms)
            }
            ClientMessageType::SendToRoom => {
                let (room, pos) = read_string(&bytes, 1)?;
                let (text, _) = read_string(&bytes, pos)?;
                Ok(Self::SendToRoom { room, text })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::DecodeError;

    #[test]
    fn auth_round_trip() {
        let message = ClientMessage::Auth {
            login: "vlad".to_string(),
            password: "secret".to_string(),
        };
        let bytes = message.serialize();
        assert_eq!(ClientMessage::deserialize(&bytes).unwrap(), message);
    }

    #[test]
    fn send_to_room_round_trip() {
        let message = ClientMessage::SendToRoom {
            room: "rust".to_string(),
            text: "hello".to_string(),
        };
        let bytes = message.serialize();
        assert_eq!(ClientMessage::deserialize(&bytes).unwrap(), message);
    }

    #[test]
    fn leave_room_round_trip() {
        let message = ClientMessage::LeaveRoom;
        let bytes = message.serialize();
        assert_eq!(ClientMessage::deserialize(&bytes).unwrap(), message);
    }

    #[test]
    fn empty_payload_is_truncated() {
        assert_eq!(
            ClientMessage::deserialize(&[]).unwrap_err(),
            DecodeError::Truncated
        );
    }

    #[test]
    fn unknown_message_type() {
        assert_eq!(
            ClientMessage::deserialize(&[99]).unwrap_err(),
            DecodeError::InvalidMessageType(99)
        );
    }

    #[test]
    fn truncated_string_field() {
        // CreateRoom + length=5, but only 1 payload byte
        let mut bytes = vec![ClientMessageType::CreateRoom as u8];
        bytes.extend_from_slice(&5u32.to_be_bytes());
        bytes.push(b'x');
        assert_eq!(
            ClientMessage::deserialize(&bytes).unwrap_err(),
            DecodeError::Truncated
        );
    }

    #[test]
    fn invalid_utf8_in_string_field() {
        let mut bytes = vec![ClientMessageType::CreateRoom as u8];
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.push(0xff);
        assert_eq!(
            ClientMessage::deserialize(&bytes).unwrap_err(),
            DecodeError::InvalidUTF8
        );
    }
}
