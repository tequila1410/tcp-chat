use std::sync::Arc;
use std::collections::HashMap;

use shared::framing::encode_frame;
use shared::protocol::ServerMessage;

use crate::client::{ClientRegistry, SessionId};

pub type Credentials = Arc<HashMap<String, String>>;

pub async fn authenticate(client_registry: &ClientRegistry, session_id: SessionId, credentials: &Credentials, login: String, password: String) {
    if let Some(db_pass) = credentials.get(&login) && *db_pass == password {
        client_registry.authorize_client(session_id, login).await;
        let payload = ServerMessage::AuthOk.serialize();
        let message = encode_frame(&payload);
        client_registry.send_message(session_id, message.to_vec()).await;
    } else {
        let payload = ServerMessage::Err("Not authenticated\n".to_string()).serialize();
        let message = encode_frame(&payload);
        client_registry.send_message(session_id, message.to_vec()).await;
    };
}

pub fn init_credentials() -> Credentials {
    let mut db = HashMap::new();
    db.insert("Bandera".to_string(), "123123".to_string());
    db.insert("Vlados".to_string(), "123123".to_string());
    Arc::new(db)
}
