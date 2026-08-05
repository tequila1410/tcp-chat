use super::*;
use crate::client::new_state;
use tokio::sync::{mpsc, oneshot};

#[tokio::test]
async fn test_send_to_room() {
    let (sessions, identity, _) = new_state();
    let first_session_id = sessions.insert_client(mpsc::channel(32).0, oneshot::channel().0).await;
    let second_session_id = sessions.insert_client(mpsc::channel(32).0, oneshot::channel().0).await;
    identity.authorize_client(first_session_id, "user".to_string()).await;
    identity.authorize_client(second_session_id, "user2".to_string()).await;
    let room_manager = RoomManager::new();
    room_manager.create_room(first_session_id, "test room".to_string()).await.unwrap();
    room_manager.join_room(second_session_id, "test room".to_string()).await.unwrap();
    let outcome = send_to_room(&identity, &room_manager, first_session_id, "test room".to_string(), "Hello, world!".to_string()).await;
    assert_eq!(outcome, ChatOutcome::Broadcast {
        recipients: HashSet::from([second_session_id]),
        room: "test room".to_string(),
        from: "user".to_string(),
        text: "Hello, world!".to_string(),
    });
}

#[tokio::test]
async fn test_send_to_room_not_authenticated() {
    let (sessions, identity, _) = new_state();
    let session_id = sessions.insert_client(mpsc::channel(32).0, oneshot::channel().0).await;
    let room_manager = RoomManager::new();
    let outcome = send_to_room(&identity, &room_manager, session_id, "test room".to_string(), "Hello, world!".to_string()).await;
    assert_eq!(outcome, ChatOutcome::NotAuthenticated);
}

#[tokio::test]
async fn test_send_to_room_room_error() {
    let (sessions, identity, _) = new_state();
    let session_id = sessions.insert_client(mpsc::channel(32).0, oneshot::channel().0).await;
    identity.authorize_client(session_id, "user".to_string()).await;
    let room_manager = RoomManager::new();
    let outcome = send_to_room(&identity, &room_manager, session_id, "test room".to_string(), "Hello, world!".to_string()).await;
    assert!(matches!(outcome, ChatOutcome::RoomError(_)));
}
