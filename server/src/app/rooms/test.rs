use super::*;
use crate::client::{Sessions, new_state};
use tokio::sync::{mpsc, oneshot};

async fn authed_session(login: &str) -> (Sessions, Identity, SessionId, RoomStore) {
    let (sessions, identity, _) = new_state();
    let session_id = sessions.insert_client(mpsc::channel(32).0, oneshot::channel().0).await;
    identity.authorize_client(session_id, login.to_string()).await;
    let room_manager = RoomStore::new();
    (sessions, identity, session_id, room_manager)
}

async fn unauthed_session() -> (Identity, SessionId, RoomStore) {
    let (sessions, identity, _) = new_state();
    let session_id = sessions.insert_client(mpsc::channel(32).0, oneshot::channel().0).await;
    let room_manager = RoomStore::new();
    (identity, session_id, room_manager)
}

#[tokio::test]
async fn test_create_room_success() {
    let (_sessions, identity, session_id, room_manager) = authed_session("user").await;

    let outcome = create_room(&identity, &room_manager, session_id, "lobby".to_string()).await;

    assert_eq!(outcome, RoomOutcome::RoomCreated("lobby".to_string()));

    let rooms = room_manager.get_rooms().await.expect("rooms list");
    assert!(rooms.contains(&"lobby".to_string()));
}

#[tokio::test]
async fn test_create_room_not_authenticated() {
    let (identity, session_id, room_manager) = unauthed_session().await;

    let outcome = create_room(&identity, &room_manager, session_id, "lobby".to_string()).await;

    assert_eq!(outcome, RoomOutcome::NotAuthenticated);
    let rooms = room_manager.get_rooms().await.expect("rooms list");
    assert!(rooms.is_empty());
}

#[tokio::test]
async fn test_create_room_already_exists() {
    let (_sessions, identity, session_id, room_manager) = authed_session("user").await;
    room_manager
        .create_room(session_id, "lobby".to_string())
        .await
        .expect("first create");

    let outcome = create_room(&identity, &room_manager, session_id, "lobby".to_string()).await;

    assert!(matches!(outcome, RoomOutcome::RoomErr(_)));
}

#[tokio::test]
async fn test_join_room_success() {
    let (sessions, identity, owner_id, room_manager) = authed_session("owner").await;
    room_manager
        .create_room(owner_id, "lobby".to_string())
        .await
        .expect("create");

    let guest_id = sessions.insert_client(mpsc::channel(32).0, oneshot::channel().0).await;
    identity.authorize_client(guest_id, "guest".to_string()).await;

    let outcome = join_room(&identity, &room_manager, guest_id, "lobby".to_string()).await;

    assert_eq!(outcome, RoomOutcome::RoomJoined("lobby".to_string()));
}

#[tokio::test]
async fn test_join_room_not_authenticated() {
    let (identity, session_id, room_manager) = unauthed_session().await;

    let outcome = join_room(&identity, &room_manager, session_id, "lobby".to_string()).await;

    assert_eq!(outcome, RoomOutcome::NotAuthenticated);
}

#[tokio::test]
async fn test_join_room_not_found() {
    let (_sessions, identity, session_id, room_manager) = authed_session("user").await;

    let outcome = join_room(&identity, &room_manager, session_id, "missing".to_string()).await;

    assert!(matches!(outcome, RoomOutcome::RoomErr(_)));
}

#[tokio::test]
async fn test_join_room_already_member() {
    let (_sessions, identity, session_id, room_manager) = authed_session("user").await;
    room_manager
        .create_room(session_id, "lobby".to_string())
        .await
        .expect("create auto-joins");

    let outcome = join_room(&identity, &room_manager, session_id, "lobby".to_string()).await;

    assert!(matches!(outcome, RoomOutcome::RoomErr(_)));
}

#[tokio::test]
async fn test_get_rooms_success() {
    let (_sessions, identity, session_id, room_manager) = authed_session("user").await;
    room_manager
        .create_room(session_id, "lobby".to_string())
        .await
        .expect("create");
    room_manager
        .create_room(session_id, "lounge".to_string())
        .await
        .expect("create second");

    let outcome = get_rooms(&identity, &room_manager, session_id).await;

    match outcome {
        RoomOutcome::RoomsGet(rooms) => {
            assert_eq!(rooms.len(), 2);
            assert!(rooms.contains(&"lobby".to_string()));
            assert!(rooms.contains(&"lounge".to_string()));
        }
        other => panic!("expected RoomsGet, got {other:?}"),
    }
}

#[tokio::test]
async fn test_get_rooms_empty() {
    let (_sessions, identity, session_id, room_manager) = authed_session("user").await;

    let outcome = get_rooms(&identity, &room_manager, session_id).await;

    assert_eq!(outcome, RoomOutcome::RoomsGet(vec![]));
}

#[tokio::test]
async fn test_get_rooms_not_authenticated() {
    let (identity, session_id, room_manager) = unauthed_session().await;

    let outcome = get_rooms(&identity, &room_manager, session_id).await;

    assert_eq!(outcome, RoomOutcome::NotAuthenticated);
}

#[tokio::test]
async fn test_leave_room_success() {
    let room_name = "lobby".to_string();
    let (_sessions, identity, session_id, room_manager) = authed_session("user").await;
    room_manager
        .create_room(session_id, room_name.clone())
        .await
        .expect("create");

    let outcome = leave_room(&identity, &room_manager, session_id).await;

    assert_eq!(
        outcome,
        RoomOutcome::RoomLeft(room_name)
    );
}

#[tokio::test]
async fn test_leave_room_was_not_member() {
    let (_sessions, identity, session_id, room_manager) = authed_session("user").await;

    let outcome = leave_room(&identity, &room_manager, session_id).await;

    assert_eq!(
        outcome,
        RoomOutcome::RoomNotMember
    );
}

#[tokio::test]
async fn test_leave_room_not_authenticated() {
    let (identity, session_id, room_manager) = unauthed_session().await;

    let outcome = leave_room(&identity, &room_manager, session_id).await;

    assert_eq!(outcome, RoomOutcome::NotAuthenticated);
}
