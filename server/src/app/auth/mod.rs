#[cfg(test)]
mod test;

use std::sync::Arc;
use std::collections::HashMap;

use crate::client::{Identity, SessionId};

pub type Credentials = Arc<HashMap<String, String>>;

#[derive(Debug)]
pub enum AuthOutcome {
    AuthOk,
    AuthErr(String),
}

pub async fn authenticate(identity: &Identity, session_id: SessionId, credentials: &Credentials, login: String, password: String) -> AuthOutcome {
    if is_valid_credentials(credentials, &login, &password) {
        identity.authorize_client(session_id, login).await;
        AuthOutcome::AuthOk
    } else {
        AuthOutcome::AuthErr("invalid credentials".to_string())
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
