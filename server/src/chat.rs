use shared::framing::encode_frame;
use shared::protocol::ServerMessage;

use crate::room::{RoomManager, RoomStorage};
use crate::client::{ClientRegistry, SessionId};


pub async fn send_to_room<S: RoomStorage>(client_registry: &ClientRegistry, room_manager: &RoomManager<S>, room: String, text: String, session_id: SessionId) {
    if let Some(message_from) = client_registry.get_login(session_id).await {
        match room_manager.recipients_for(&room, session_id).await {
            Ok(message_to) => {
                let payload = ServerMessage::Message{room: room, from: message_from, text}.serialize();
                let message = encode_frame(&payload);
                client_registry.send_many(message, message_to).await;
            }
            Err(error) => {
                let payload = ServerMessage::RoomErr(error.to_string()).serialize();
                let message = encode_frame(&payload);
                client_registry.send_message(session_id, message.to_vec()).await;
            }
        }
    } else {
        let payload = ServerMessage::AuthErr("Not authenticated\n".to_string()).serialize();
        let message = encode_frame(&payload);
        client_registry.send_message(session_id, message.to_vec()).await;
    }
}