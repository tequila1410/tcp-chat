#[cfg(test)]
mod test;

use crate::client::{Identity, SessionId};
use crate::room_store::LeaveOutcome;
use crate::room_store::RoomStore;

#[derive(Debug, PartialEq)]
pub enum RoomOutcome {
    RoomCreated(String),
    RoomErr(String),
    RoomJoined(String),
    RoomLeft(String),
    RoomNotMember,
    RoomsGet(Vec<String>),
    NotAuthenticated,
}

pub async fn create_room(identity: &Identity, room_store: &RoomStore, session_id: SessionId, room_name: String) -> RoomOutcome {
    if identity.is_client_authorized(session_id).await {
        match room_store.create_room(session_id, room_name.clone()).await {
            Ok(_) => {
                RoomOutcome::RoomCreated(room_name)
            }
            Err(error) => {
                RoomOutcome::RoomErr(error.to_string())
            }
        }
    } else {
        RoomOutcome::NotAuthenticated
    }
}

pub async fn join_room(identity: &Identity, room_store: &RoomStore, session_id: SessionId, room_name: String) -> RoomOutcome {
    if identity.is_client_authorized(session_id).await {
        match room_store.join_room(session_id, room_name.clone()).await {
            Ok(_) => {
                RoomOutcome::RoomJoined(room_name)
            }
            Err(error) => {
                RoomOutcome::RoomErr(error.to_string())
            }
        }
    } else {
        RoomOutcome::NotAuthenticated
    }
}

pub async fn get_rooms(identity: &Identity, room_store: &RoomStore, session_id: SessionId) -> RoomOutcome {
    if identity.is_client_authorized(session_id).await {
        match room_store.get_rooms().await {
            Ok(rooms) => {
                RoomOutcome::RoomsGet(rooms)
            }
            Err(error) => {
                RoomOutcome::RoomErr(error.to_string())
            }
        }
    } else {
        RoomOutcome::NotAuthenticated
    }
}

pub async fn leave_room(identity: &Identity, room_store: &RoomStore, session_id: SessionId) -> RoomOutcome {
    if identity.is_client_authorized(session_id).await {
        match room_store.leave(session_id).await {
            LeaveOutcome::Left(room_name) => {
                RoomOutcome::RoomLeft(room_name)
            }
            LeaveOutcome::WasNotMember => {
                RoomOutcome::RoomNotMember
            }
        }
    } else {
        RoomOutcome::NotAuthenticated
    }
}
