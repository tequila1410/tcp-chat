use tokio::sync::{mpsc, oneshot};

use crate::client::new_state;

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

#[tokio::test]
async fn test_authenticate_success() {
    let credentials = test_credentials();
    let (sessions, identity, _) = new_state();
    let session_id = sessions.insert_client(mpsc::channel(32).0, oneshot::channel().0).await;
    let outcome = authenticate(&identity, session_id, &credentials, "Alice".to_string(), "secret".to_string()).await;
    assert_eq!(identity.get_login(session_id).await.as_deref(), Some("Alice"));
    assert!(matches!(outcome, AuthOutcome::AuthOk));
}

#[tokio::test]
async fn test_authenticate_failure() {
    let credentials = test_credentials();
    let (sessions, identity, _) = new_state();
    let session_id = sessions.insert_client(mpsc::channel(32).0, oneshot::channel().0).await;
    let outcome = authenticate(&identity, session_id, &credentials, "Alice".to_string(), "secret1".to_string()).await;
    assert_eq!(identity.get_login(session_id).await.as_deref(), None);
    assert!(matches!(outcome, AuthOutcome::AuthErr(_)));
}
