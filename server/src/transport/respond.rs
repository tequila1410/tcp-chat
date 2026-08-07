use shared::{framing::encode_frame, protocol::ServerMessage};
use tracing::{info, warn};

use crate::app::{auth::AuthOutcome, chat::ChatOutcome, rooms::RoomOutcome};
use crate::client::{Outbound, SessionId};

pub async fn apply_chat_outcome(outbound: &Outbound, session_id: SessionId, outcome: ChatOutcome) {
    match outcome {
        ChatOutcome::NotAuthenticated => {
            warn!("send_to_room rejected: not authenticated");
            outbound.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
        }
        ChatOutcome::RoomError(error) => {
            warn!(?error, "send_to_room rejected");
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
            info!("authenticated");
            outbound.reply(session_id, ServerMessage::AuthOk).await;
        }
        AuthOutcome::AuthErr(error) => {
            warn!(?error, "authentication failed");
            outbound.reply(session_id, ServerMessage::AuthErr(error)).await;
        }
    }
}

pub async fn apply_room_outcome(outbound: &Outbound, session_id: SessionId, outcome: RoomOutcome) {
    match outcome {
        RoomOutcome::RoomCreated(message) => {
            info!(room = message, "room created");
            outbound.reply(session_id, ServerMessage::RoomCreated(message)).await;
        }
        RoomOutcome::RoomErr(error) => {
            warn!(?error, "room error");
            outbound.reply(session_id, ServerMessage::RoomErr(error)).await;
        }
        RoomOutcome::NotAuthenticated => {
            warn!("room command rejected: not authenticated");
            outbound.reply(session_id, ServerMessage::AuthErr("Not authenticated".to_string())).await;
        }
        RoomOutcome::RoomJoined(message) => {
            info!(room = message, "room joined");
            outbound.reply(session_id, ServerMessage::RoomJoined(message)).await;
        }
        RoomOutcome::RoomLeft(message) => {
            info!(room = message, "room left");
            outbound.reply(session_id, ServerMessage::RoomLeft(message)).await;
        }
        RoomOutcome::RoomNotMember => {
            info!("leave: was not member");
            outbound.reply(session_id, ServerMessage::RoomLeft("not member of any room".to_string())).await;
        }
        RoomOutcome::RoomsGet(rooms) => {
            outbound.reply(session_id, ServerMessage::RoomsGet(rooms)).await;
        }
    }
}