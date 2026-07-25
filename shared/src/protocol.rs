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
    AuthResponse(String),
    Message {
        form: String,
        text: String,
    },
    Error(String),
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
