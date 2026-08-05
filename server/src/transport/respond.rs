use shared::{framing::encode_frame, protocol::ServerMessage};

use crate::app::{auth::AuthOutcome, chat::ChatOutcome, rooms::RoomOutcome};
use crate::client::{Outbound, SessionId};

pub async fn apply_chat_outcome(outbound: &Outbound, session_id: SessionId, outcome: ChatOutcome) {
    match outcome {
        ChatOutcome::NotAuthenticated => {
            outbound.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
        }
        ChatOutcome::RoomError(error) => {
            outbound.reply(session_id, ServerMessage::RoomErr(error)).await;
        }
        ChatOutcome::Broadcast { recipients, room, from, text } => {
            let payload = ServerMessage::Message { room, from, text }.serialize();
            let message = encode_frame(&payload);
            outbound.send_many(message, recipients).await;
        }
    }
}

pub async fn apply_auth_outcome(outbound: &Outbound, session_id: SessionId, outcome: AuthOutcome) {
    match outcome {
        AuthOutcome::AuthOk => {
            outbound.reply(session_id, ServerMessage::AuthOk).await;
        }
        AuthOutcome::AuthErr(error) => {
            outbound.reply(session_id, ServerMessage::AuthErr(error)).await;
        }
    }
}

pub async fn apply_room_outcome(outbound: &Outbound, session_id: SessionId, outcome: RoomOutcome) {
    match outcome {
        RoomOutcome::RoomCreated(message) => {
            outbound.reply(session_id, ServerMessage::RoomCreated(message)).await;
        }
        RoomOutcome::RoomErr(error) => {
            outbound.reply(session_id, ServerMessage::RoomErr(error)).await;
        }
        RoomOutcome::NotAuthenticated => {
            outbound.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
        }
        RoomOutcome::RoomJoined(message) => {
            outbound.reply(session_id, ServerMessage::RoomJoined(message)).await;
        }
        RoomOutcome::RoomLeft(message) => {
            outbound.reply(session_id, ServerMessage::RoomLeft(message)).await;
        }
        RoomOutcome::RoomsGet(rooms) => {
            outbound.reply(session_id, ServerMessage::RoomsGet(rooms)).await;
        }
    }
}