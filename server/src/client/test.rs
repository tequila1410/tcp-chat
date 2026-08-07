use tokio::sync::{mpsc, oneshot};

use super::*;

fn dummy_channels() -> (mpsc::Sender<Arc<Vec<u8>>>, oneshot::Sender<()>) {
    (mpsc::channel(32).0, oneshot::channel().0)
}

#[tokio::test]
async fn try_insert_rejects_when_at_max_clients() {
    let (sessions, _, _) = new_state();
    let max_clients = 2;

    let (tx, evict) = dummy_channels();
    sessions
        .try_insert_client(tx, evict, max_clients)
        .await
        .expect("first insert");

    let (tx, evict) = dummy_channels();
    sessions
        .try_insert_client(tx, evict, max_clients)
        .await
        .expect("second insert");

    let (tx, evict) = dummy_channels();
    let result = sessions.try_insert_client(tx, evict, max_clients).await;

    assert_eq!(result, Err(SessionsError::TooManyConnections));
}

#[tokio::test]
async fn try_insert_allows_new_client_after_remove() {
    let (sessions, _, _) = new_state();
    let max_clients = 1;

    let (tx, evict) = dummy_channels();
    let session_id = sessions
        .try_insert_client(tx, evict, max_clients)
        .await
        .expect("first insert");

    let (tx, evict) = dummy_channels();
    assert_eq!(
        sessions.try_insert_client(tx, evict, max_clients).await,
        Err(SessionsError::TooManyConnections)
    );

    sessions.remove_client(session_id).await;

    let (tx, evict) = dummy_channels();
    sessions
        .try_insert_client(tx, evict, max_clients)
        .await
        .expect("insert after remove");
}
