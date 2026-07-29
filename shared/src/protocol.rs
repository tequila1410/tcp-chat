#[derive(Debug, PartialEq)]
pub enum ClientMessage {
    Auth {
        login: String,
        password: String,
    },
    Message(String),
    CreateRoom(String),
    JoinRoom(String),
    GetRooms
}

impl ClientMessage {
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        match self {
            ClientMessage::Auth { login, password } => {
                bytes.push(1);

                let login_length = login.len() as u32;
                bytes.extend_from_slice(&login_length.to_be_bytes());
                bytes.extend_from_slice(login.as_bytes());

                let pass_length = password.len() as u32;
                bytes.extend_from_slice(&pass_length.to_be_bytes());
                bytes.extend_from_slice(password.as_bytes());
            }
            ClientMessage::Message(message) => {
                bytes.push(2);

                let message_length = message.len() as u32;
                bytes.extend_from_slice(&message_length.to_be_bytes());
                bytes.extend_from_slice(message.as_bytes());
            }
            ClientMessage::CreateRoom(room_name) => {
                bytes.push(3);

                let room_name_length = room_name.len() as u32;
                bytes.extend_from_slice(&room_name_length.to_be_bytes());
                bytes.extend_from_slice(room_name.as_bytes());
            }
            ClientMessage::JoinRoom(room_name) => {
                bytes.push(4);

                let room_name_length = room_name.len() as u32;
                bytes.extend_from_slice(&room_name_length.to_be_bytes());
                bytes.extend_from_slice(room_name.as_bytes());
            }
            ClientMessage::GetRooms => {
                bytes.push(5);
            }
        }
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 1 {
            return None;
        }

        let message_type = bytes[0];

        match message_type {
            1 => {
                let (login_bytes, pos) = read_bytes(&bytes, 1)?;
                let login = String::from_utf8(
                    login_bytes.to_vec()
                ).ok()?;

                let (password_bytes, _) = read_bytes(&bytes, pos)?;
                let password = String::from_utf8(
                    password_bytes.to_vec()
                ).ok()?;
                Some(Self::Auth{ login, password })
            },
            2 => {
                let (message_bytes, _) = read_bytes(&bytes, 1)?;
                let message = String::from_utf8(
                    message_bytes.to_vec()
                ).ok()?;
                Some(Self::Message(message))
            }
            3 => {
                let (message_bytes, _) = read_bytes(&bytes, 1)?;
                let message = String::from_utf8(
                    message_bytes.to_vec()
                ).ok()?;
                Some(Self::CreateRoom(message))
            }
            4 => {
                let (message_bytes, _) = read_bytes(&bytes, 1)?;
                let message = String::from_utf8(
                    message_bytes.to_vec()
                ).ok()?;
                Some(Self::JoinRoom(message))
            }
            5 => {
                Some(Self::GetRooms)
            }
            _ => None
        }
    }
}

fn read_bytes(bytes: &[u8], pos: usize) -> Option<(&[u8], usize)> {
    if bytes.len() < pos + 4 {
        return None;
    }
    let text_len = u32::from_be_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
    if bytes.len() < pos + 4 + text_len {
        return None;
    }
    let bytes_slice = &bytes[pos + 4..pos + 4 + text_len];
    Some((bytes_slice, pos + 4 + text_len))
}

pub enum ServerMessage {
    AuthOk,
    AuthErr(String),
    Message {
        from: String,
        text: String,
    },
    Err(String),

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
                bytes.push(1);

                let error_length = error.len() as u32;
                bytes.extend_from_slice(&error_length.to_be_bytes());
                bytes.extend_from_slice(error.as_bytes());
            }
            Self::AuthOk => {
                bytes.push(2);
            }
            Self::Message{from, text} => {
                bytes.push(3);

                let from_length = from.len() as u32;
                bytes.extend_from_slice(&from_length.to_be_bytes());
                bytes.extend_from_slice(from.as_bytes());

                let text_length = text.len() as u32;
                bytes.extend_from_slice(&text_length.to_be_bytes());
                bytes.extend_from_slice(text.as_bytes());
            }
            Self::Err(error) => {
                bytes.push(4);

                let error_length = error.len() as u32;
                bytes.extend_from_slice(&error_length.to_be_bytes());
                bytes.extend_from_slice(error.as_bytes());
            }
            Self::RoomCreated(message) => {
                bytes.push(5);

                let message_length = message.len() as u32;
                bytes.extend_from_slice(&message_length.to_be_bytes());
                bytes.extend_from_slice(message.as_bytes());
            }
            Self::RoomJoined(message) => {
                bytes.push(6);

                let message_length = message.len() as u32;
                bytes.extend_from_slice(&message_length.to_be_bytes());
                bytes.extend_from_slice(message.as_bytes());
            }
            Self::RoomErr(error) => {
                bytes.push(7);

                let error_length = error.len() as u32;
                bytes.extend_from_slice(&error_length.to_be_bytes());
                bytes.extend_from_slice(error.as_bytes());
            }
            Self::RoomsGet(rooms) => {
                bytes.push(8);

                let rooms = Self::serialize_strings(&rooms);
                bytes.extend_from_slice(&rooms);
            }
        }
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 1 {
            return None;
        }

        let message_type = bytes[0];
        println!("message_type {message_type}");

        match message_type {
            1 => {
                let (auth_error_bytes, _) = read_bytes(&bytes, 1)?;
                let error = String::from_utf8(
                    auth_error_bytes.to_vec()
                ).ok()?;
                Some(Self::AuthErr(error))
            }
            2 => {
                Some(Self::AuthOk)
            }
            3 => {
                let (from_bytes, pos) = read_bytes(&bytes, 1)?;
                let from = String::from_utf8(
                    from_bytes.to_vec()
                ).ok()?;
                
                let (text_bytes, _) = read_bytes(&bytes, pos)?;
                let text = String::from_utf8(
                    text_bytes.to_vec()
                ).ok()?;

                Some(Self::Message{from, text})
            }
            4 => {
                let (error_bytes, _) = read_bytes(&bytes, 1)?;
                let error = String::from_utf8(
                    error_bytes.to_vec()
                ).ok()?;
                Some(Self::Err(error))
            }
            5 => {
                let (message_bytes, _) = read_bytes(&bytes, 1)?;
                let message = String::from_utf8(
                    message_bytes.to_vec()
                ).ok()?;
                Some(Self::RoomCreated(message))
            }
            6 => {
                let (message_bytes, _) = read_bytes(&bytes, 1)?;
                let message = String::from_utf8(
                    message_bytes.to_vec()
                ).ok()?;
                Some(Self::RoomJoined(message))
            }
            7 => {
                let (error_bytes, _) = read_bytes(&bytes, 1)?;
                let error = String::from_utf8(
                    error_bytes.to_vec()
                ).ok()?;
                Some(Self::RoomErr(error))
            }
            8 => {
                let rooms = Self::deserialize_strings(&bytes[1..])?;
                Some(Self::RoomsGet(rooms))
            }
            _ => None
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

    fn deserialize_strings(bytes: &[u8]) -> Option<Vec<String>> {
        let mut cursor = 0;
    
        let count = u32::from_be_bytes(
            bytes.get(cursor..cursor+4)?
                .try_into()
                .ok()?
        );
    
        cursor += 4;
    
        let mut result = Vec::new();
    
        for _ in 0..count {
            let len = u32::from_be_bytes(
                bytes.get(cursor..cursor+4)?
                    .try_into()
                    .ok()?
            ) as usize;
    
            cursor += 4;
    
            let text = String::from_utf8(
                bytes.get(cursor..cursor+len)?.to_vec()
            ).ok()?;
    
            cursor += len;
    
            result.push(text);
        }
    
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_round_trip() {
        let message = ClientMessage::Auth {
            login: "vlad".to_string(),
            password: "123456".to_string(),
        };

        let bytes = message.serialize();
        let deserialized = ClientMessage::deserialize(&bytes).unwrap();

        assert_eq!(message, deserialized);
    }
}
