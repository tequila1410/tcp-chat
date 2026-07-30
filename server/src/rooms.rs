use shared::framing::encode_frame;
use shared::protocol::ServerMessage;

use crate::client::{ClientRegistry, SessionId};
use crate::room::{RoomManager, RoomStorage};

pub async fn create_room<S: RoomStorage>(client_registry: &ClientRegistry, room_manager: &RoomManager<S>, session_id: SessionId, room_name: String) {
    if let Some(_) = client_registry.get_login(session_id).await {
        match room_manager.create_room(room_name).await {
            Ok(_) => {
                let payload = ServerMessage::RoomCreated("Room successfuly created".to_string()).serialize();
                let message = encode_frame(&payload);
                client_registry.send_message(session_id, message.to_vec()).await;
            }
            Err(_) => {
                let payload = ServerMessage::RoomErr("Room already exist\n".to_string()).serialize();
                let message = encode_frame(&payload);
                client_registry.send_message(session_id, message.to_vec()).await;
            }
        };
    } else {
        let payload = ServerMessage::AuthErr("Not authenticated\n".to_string()).serialize();
        let message = encode_frame(&payload);
        client_registry.send_message(session_id, message.to_vec()).await;
    }
}

pub async fn join_room<S: RoomStorage>(client_registry: &ClientRegistry, room_manager: &RoomManager<S>, session_id: SessionId, room_name: String) {
    if let Some(_) = client_registry.get_login(session_id).await {
        match room_manager.join_room(room_name, session_id).await {
            Ok(_) => {
                let payload = ServerMessage::RoomJoined("Room successfuly joined".to_string()).serialize();
                let message = encode_frame(&payload);
                client_registry.send_message(session_id, message.to_vec()).await;
            }
            Err(_) => {
                let payload = ServerMessage::RoomErr("No room with this name\n".to_string()).serialize();
                let message = encode_frame(&payload);
                client_registry.send_message(session_id, message.to_vec()).await;
            }
        };
    } else {
        let payload = ServerMessage::AuthErr("Not authenticated\n".to_string()).serialize();
        let message = encode_frame(&payload);
        client_registry.send_message(session_id, message.to_vec()).await;
    }
}

pub async fn get_rooms<S: RoomStorage>(client_registry: &ClientRegistry, room_manager: &RoomManager<S>, session_id: SessionId) {
    if let Some(_) = client_registry.get_login(session_id).await {
        match room_manager.get_rooms().await {
            Ok(rooms) => {
                let payload = ServerMessage::RoomsGet(rooms).serialize();
                let message = encode_frame(&payload);
                client_registry.send_message(session_id, message.to_vec()).await;
            }
            Err(err) => {
                println!("Creating room error: {err:?}");
            }
        }
    }
}