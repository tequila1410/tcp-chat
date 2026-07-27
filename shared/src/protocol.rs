#[derive(Debug, PartialEq)]
pub enum ClientMessage {
    Auth {
        login: String,
        password: String,
    },
    Message(String),
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
        }
        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 5 {
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
    Err(String)
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
                let (auth_error_bytes, _) = read_bytes(&bytes, 1)?;
                let error = String::from_utf8(
                    auth_error_bytes.to_vec()
                ).ok()?;
                Some(Self::AuthErr(error))
            },
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
            },
            _ => None
        }
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
