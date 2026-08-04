use std::sync::Arc;
use std::collections::HashMap;

use shared::protocol::ServerMessage;

use crate::client::{ClientRegistry, SessionId};

pub type Credentials = Arc<HashMap<String, String>>;

pub async fn authenticate(client_registry: &ClientRegistry, session_id: SessionId, credentials: &Credentials, login: String, password: String) {
    if is_valid_credentials(credentials, &login, &password) {
        client_registry.authorize_client(session_id, login).await;
        client_registry.reply(session_id, ServerMessage::AuthOk).await;
    } else {
        client_registry.reply(session_id,  ServerMessage::AuthErr("Invalid credentials".to_string())).await;
    }
}

fn is_valid_credentials(credentials: &Credentials, login: &str, password: &str) -> bool {
    credentials.get(login).is_some_and(|db_pass| db_pass == password)
}

pub fn init_credentials() -> Credentials {
    let mut db = HashMap::new();
    db.insert("Oliver".to_string(), "123123".to_string());
    db.insert("Emma".to_string(), "123123".to_string());
    Arc::new(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_credentials() -> Credentials {
        let mut db = HashMap::new();
        db.insert("Alice".to_string(), "secret".to_string());
        Arc::new(db)
    }

    #[test]
    fn test_is_valid_credentials() {
        let credentials = test_credentials();
        assert!(is_valid_credentials(&credentials, "Alice", "secret"));
    }

    #[test]
    fn test_is_not_valid_credentials() {
        let credentials = test_credentials();
        assert!(!is_valid_credentials(&credentials, "Alice", "secret1"));
        assert!(!is_valid_credentials(&credentials, "Ali", "secret"));
    }
}
