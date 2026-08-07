#[cfg(test)]
mod test;

use std::collections::HashSet;

use crate::room_store::RoomStore;
use crate::client::{Identity, SessionId};

#[derive(PartialEq, Debug)]
pub enum ChatOutcome {
    NotAuthenticated,
    RoomError(String),
    Broadcast {
        recipients: HashSet<SessionId>,
        room: String,
        from: String,
        text: String,
    },
}

pub async fn send_to_room(
    identity: &Identity,
    room_store: &RoomStore,
    session_id: SessionId,
    room_name: String,
    text: String,
) -> ChatOutcome {
    if let Some(message_from) = identity.get_login(session_id).await {
        match room_store.recipients_for(&room_name, session_id).await {
            Ok(message_to) => ChatOutcome::Broadcast {
                recipients: message_to,
                room: room_name,
                from: message_from,
                text,
            },
            Err(error) => ChatOutcome::RoomError(error.to_string()),
        }
    } else {
        ChatOutcome::NotAuthenticated
    }
}
