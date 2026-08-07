use super::*;

#[test]
fn create_room_creates_and_auto_joins() {
    let mut state = RoomState::new();
    let session_id = 1;
    let room_name = "test_room".to_string();

    assert!(state.create_room(session_id, room_name.clone()).is_ok());

    let room = state.rooms.get(&room_name).expect("room should exist");
    assert!(room.clients.contains(&session_id));
    assert_eq!(state.user_rooms.get(&session_id), Some(&room_name));
}

#[test]
fn create_room_rejects_duplicate_name() {
    let mut state = RoomState::new();
    let session_id = 1;
    let room_name = "test_room".to_string();

    assert!(state.create_room(session_id, room_name.clone()).is_ok());

    let result = state.create_room(session_id, room_name.clone());
    assert!(matches!(result, Err(RoomError::AlreadyExist(_))));

    let room = state.rooms.get(&room_name).expect("room should still exist");
    assert!(room.clients.contains(&session_id));
    assert_eq!(state.user_rooms.get(&session_id), Some(&room_name));
}

#[test]
fn create_room_switches_from_current_room() {
    let mut state = RoomState::new();
    let session_id = 1;
    let first_room_name = "test_room".to_string();
    let second_room_name = "test_room2".to_string();

    assert!(state.create_room(session_id, first_room_name.clone()).is_ok());

    assert!(state.create_room(session_id, second_room_name.clone()).is_ok());
    let first_room = state.rooms.get(&first_room_name).expect("room should still exist");
    let second_room = state.rooms.get(&second_room_name).expect("room should still exist");
    assert!(!first_room.clients.contains(&session_id));
    assert!(second_room.clients.contains(&session_id));
    assert_eq!(state.user_rooms.get(&session_id), Some(&second_room_name));
}

#[test]
fn join_room_joins_user_to_room() {
    let mut state = RoomState::new();
    let session_id = 1;
    let guest_session_id = 2;
    let room_name = "test_room".to_string();

    assert!(state.create_room(session_id, room_name.clone()).is_ok());
    assert!(state.join_room(guest_session_id, room_name.clone()).is_ok());

    let room = state.rooms.get(&room_name).expect("room should still exist");
    assert!(room.clients.contains(&session_id));
    assert!(room.clients.contains(&guest_session_id));
    assert_eq!(state.user_rooms.get(&session_id), Some(&room_name));
    assert_eq!(state.user_rooms.get(&guest_session_id), Some(&room_name));
}

#[test]
fn join_room_rejects_duplicate_user() {
    let mut state = RoomState::new();
    let session_id = 1;
    let room_name = "test_room".to_string();

    assert!(state.create_room(session_id, room_name.clone()).is_ok());

    let result = state.join_room(session_id, room_name.clone());
    assert!(matches!(result, Err(RoomError::AlreadyMember(_))));

    let room = state.rooms.get(&room_name).expect("room should still exist");
    assert!(room.clients.contains(&session_id));
    assert_eq!(state.user_rooms.get(&session_id), Some(&room_name));
}

#[test]
fn join_room_rejects_unknown_room() {
    let mut state = RoomState::new();
    let session_id = 1;

    let result = state.join_room(session_id, "missing".to_string());
    assert!(matches!(result, Err(RoomError::NotFound(_))));
    assert!(state.user_rooms.get(&session_id).is_none());
}

#[test]
fn join_room_switches_from_current_room() {
    let mut state = RoomState::new();
    let session_id = 1;
    let other_session_id = 2;
    let first_room_name = "room_a".to_string();
    let second_room_name = "room_b".to_string();

    assert!(state.create_room(session_id, first_room_name.clone()).is_ok());
    assert!(state.create_room(other_session_id, second_room_name.clone()).is_ok());
    assert!(state.join_room(session_id, second_room_name.clone()).is_ok());

    let first_room = state.rooms.get(&first_room_name).expect("first room should exist");
    let second_room = state.rooms.get(&second_room_name).expect("second room should exist");
    assert!(!first_room.clients.contains(&session_id));
    assert!(second_room.clients.contains(&session_id));
    assert!(second_room.clients.contains(&other_session_id));
    assert_eq!(state.user_rooms.get(&session_id), Some(&second_room_name));
    assert_eq!(state.user_rooms.get(&other_session_id), Some(&second_room_name));
}

#[test]
fn create_room_rejects_duplicate_keeps_other_membership() {
    let mut state = RoomState::new();
    let session_id = 1;
    let other_session_id = 2;
    let first_room_name = "room_a".to_string();
    let second_room_name = "room_b".to_string();

    assert!(state.create_room(session_id, first_room_name.clone()).is_ok());
    assert!(state.create_room(other_session_id, second_room_name.clone()).is_ok());

    let result = state.create_room(session_id, second_room_name.clone());
    assert!(matches!(result, Err(RoomError::AlreadyExist(_))));

    let first_room = state.rooms.get(&first_room_name).expect("first room should exist");
    let second_room = state.rooms.get(&second_room_name).expect("second room should exist");
    assert!(first_room.clients.contains(&session_id));
    assert!(!second_room.clients.contains(&session_id));
    assert_eq!(state.user_rooms.get(&session_id), Some(&first_room_name));
}

#[test]
fn leave_removes_membership() {
    let mut state = RoomState::new();
    let session_id = 1;
    let room_name = "test_room".to_string();

    assert!(state.create_room(session_id, room_name.clone()).is_ok());
    assert!(matches!(state.leave(session_id), LeaveOutcome::Left(_)));

    let room = state.rooms.get(&room_name).expect("room should still exist");
    assert!(!room.clients.contains(&session_id));
    assert!(state.user_rooms.get(&session_id).is_none());
}

#[test]
fn leave_when_not_member() {
    let mut state = RoomState::new();
    assert_eq!(state.leave(1), LeaveOutcome::WasNotMember);
}

#[test]
fn leave_all_clears_membership() {
    let mut state = RoomState::new();
    let session_id = 1;
    let room_name = "test_room".to_string();

    assert!(state.create_room(session_id, room_name.clone()).is_ok());
    state.leave_all(session_id);

    let room = state.rooms.get(&room_name).expect("room should still exist");
    assert!(!room.clients.contains(&session_id));
    assert!(state.user_rooms.get(&session_id).is_none());
}

#[test]
fn recipients_for_excludes_sender() {
    let mut state = RoomState::new();
    let session_id = 1;
    let guest_session_id = 2;
    let room_name = "test_room".to_string();

    assert!(state.create_room(session_id, room_name.clone()).is_ok());
    assert!(state.join_room(guest_session_id, room_name.clone()).is_ok());

    let recipients = state
        .recipients_for(&room_name, session_id)
        .expect("sender is a member");
    assert_eq!(recipients, HashSet::from([guest_session_id]));
}

#[test]
fn recipients_for_alone_is_empty() {
    let mut state = RoomState::new();
    let session_id = 1;
    let room_name = "test_room".to_string();

    assert!(state.create_room(session_id, room_name.clone()).is_ok());

    let recipients = state
        .recipients_for(&room_name, session_id)
        .expect("sender is a member");
    assert!(recipients.is_empty());
}

#[test]
fn recipients_for_rejects_non_member() {
    let mut state = RoomState::new();
    let session_id = 1;
    let outsider_id = 2;
    let room_name = "test_room".to_string();

    assert!(state.create_room(session_id, room_name.clone()).is_ok());

    let result = state.recipients_for(&room_name, outsider_id);
    assert!(matches!(result, Err(RoomError::NotMember(_))));
}

#[test]
fn recipients_for_rejects_unknown_room() {
    let state = RoomState::new();
    let result = state.recipients_for("missing", 1);
    assert!(matches!(result, Err(RoomError::NotFound(_))));
}
