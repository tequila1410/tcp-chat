use shared::protocol::ServerMessage;

use crate::client::{ClientRegistry, SessionId};
use crate::room::{LeaveOutcome, RoomManager, RoomStorage};

pub async fn create_room<S: RoomStorage>(client_registry: &ClientRegistry, room_manager: &RoomManager<S>, session_id: SessionId, room_name: String) {
    if client_registry.is_client_authorized(session_id).await {
        match room_manager.create_room(session_id, room_name).await {
            Ok(_) => {
                client_registry.reply(session_id, ServerMessage::RoomCreated("Room successfuly created".to_string())).await;
            }
            Err(error) => {
                client_registry.reply(session_id, ServerMessage::RoomErr(error.to_string())).await;
            }
        };
    } else {
        client_registry.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
    }
}

pub async fn join_room<S: RoomStorage>(client_registry: &ClientRegistry, room_manager: &RoomManager<S>, session_id: SessionId, room_name: String) {
    if client_registry.is_client_authorized(session_id).await {
        match room_manager.join_room(room_name, session_id).await {
            Ok(_) => {
                client_registry.reply(session_id, ServerMessage::RoomJoined("Room successfuly joined".to_string())).await;
            }
            Err(error) => {
                client_registry.reply(session_id, ServerMessage::RoomErr(error.to_string())).await;
            }
        };
    } else {
        client_registry.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
    }
}

pub async fn get_rooms<S: RoomStorage>(client_registry: &ClientRegistry, room_manager: &RoomManager<S>, session_id: SessionId) {
    if client_registry.is_client_authorized(session_id).await {
        match room_manager.get_rooms().await {
            Ok(rooms) => {
                client_registry.reply(session_id, ServerMessage::RoomsGet(rooms)).await;
            }
            Err(error) => {
                client_registry.reply(session_id, ServerMessage::RoomErr(error.to_string())).await;
            }
        }
    } else {
        client_registry.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
    }
}

pub async fn leave_room<S: RoomStorage>(client_registry: &ClientRegistry, room_manager: &RoomManager<S>, session_id: SessionId) {
    if client_registry.is_client_authorized(session_id).await {
        match room_manager.leave(session_id).await {
            LeaveOutcome::Left => {
                client_registry.reply(session_id, ServerMessage::RoomLeft("Room successfuly left".to_string())).await;
            }
            LeaveOutcome::WasNotMember => {
                client_registry.reply(session_id, ServerMessage::RoomLeft("You are not in any room\n".to_string())).await;
            }
        }
    } else {
        client_registry.reply(session_id, ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
    }
}