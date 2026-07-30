use std::sync::Arc;
use std::collections::HashMap;

use shared::protocol::ServerMessage;

use crate::client::{ClientRegistry, SessionId};

pub type Credentials = Arc<HashMap<String, String>>;

pub async fn authenticate(client_registry: &ClientRegistry, session_id: SessionId, credentials: &Credentials, login: String, password: String) {
    if let Some(db_pass) = credentials.get(&login) && *db_pass == password {
        client_registry.authorize_client(session_id, login).await;
        client_registry.reply(session_id,  ServerMessage::AuthOk).await;
    } else {
        client_registry.reply(session_id,  ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
    };
}

pub fn init_credentials() -> Credentials {
    let mut db = HashMap::new();
    db.insert("Bandera".to_string(), "123123".to_string());
    db.insert("Vlados".to_string(), "123123".to_string());
    Arc::new(db)
}
