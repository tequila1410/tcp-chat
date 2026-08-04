use shared::protocol::ServerMessage;

use crate::client::{Identity, Outbound, SessionId};
use crate::room::{LeaveOutcome, RoomManager, RoomStorage};

pub async fn create_room<S: RoomStorage>(identity: &Identity, outbound: &Outbound, room_manager: &RoomManager<S>, session_id: SessionId, room_name: String) {
    if identity.is_client_authorized(session_id).await {
        match room_manager.create_room(session_id, room_name).await {
            Ok(_) => {
                outbound.reply(session_id, ServerMessage::RoomCreated("Room successfuly created".to_string())).await;
            }
            Err(error) => {
                outbound.reply(session_id, ServerMessage::RoomErr(error.to_string())).await;
            }
        };
    } else {
        outbound.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
    }
}

pub async fn join_room<S: RoomStorage>(identity: &Identity, outbound: &Outbound, room_manager: &RoomManager<S>, session_id: SessionId, room_name: String) {
    if identity.is_client_authorized(session_id).await {
        match room_manager.join_room(session_id, room_name).await {
            Ok(_) => {
                outbound.reply(session_id, ServerMessage::RoomJoined("Room successfuly joined".to_string())).await;
            }
            Err(error) => {
                outbound.reply(session_id, ServerMessage::RoomErr(error.to_string())).await;
            }
        };
    } else {
        outbound.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
    }
}

pub async fn get_rooms<S: RoomStorage>(identity: &Identity, outbound: &Outbound, room_manager: &RoomManager<S>, session_id: SessionId) {
    if identity.is_client_authorized(session_id).await {
        match room_manager.get_rooms().await {
            Ok(rooms) => {
                outbound.reply(session_id, ServerMessage::RoomsGet(rooms)).await;
            }
            Err(error) => {
                outbound.reply(session_id, ServerMessage::RoomErr(error.to_string())).await;
            }
        }
    } else {
        outbound.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
    }
}

pub async fn leave_room<S: RoomStorage>(identity: &Identity, outbound: &Outbound, room_manager: &RoomManager<S>, session_id: SessionId) {
    if identity.is_client_authorized(session_id).await {
        match room_manager.leave(session_id).await {
            LeaveOutcome::Left => {
                outbound.reply(session_id, ServerMessage::RoomLeft("Room successfuly left".to_string())).await;
            }
            LeaveOutcome::WasNotMember => {
                outbound.reply(session_id, ServerMessage::RoomLeft("You are not in any room\n".to_string())).await;
            }
        }
    } else {
        outbound.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
    }
}
