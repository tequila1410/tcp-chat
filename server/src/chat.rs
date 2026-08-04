use shared::framing::encode_frame;
use shared::protocol::ServerMessage;

use crate::room::{RoomManager, RoomStorage};
use crate::client::{Identity, Outbound, SessionId};


pub async fn send_to_room<S: RoomStorage>(identity: &Identity, outbound: &Outbound, room_manager: &RoomManager<S>, room_name: String, text: String, session_id: SessionId) {
    if let Some(message_from) = identity.get_login(session_id).await {
        match room_manager.recipients_for(&room_name, session_id).await {
            Ok(message_to) => {
                let payload = ServerMessage::Message{room: room_name, from: message_from, text}.serialize();
                let message = encode_frame(&payload);
                outbound.send_many(message, message_to).await;
            }
            Err(error) => {
                outbound.reply(session_id,  ServerMessage::RoomErr(error.to_string())).await;
            }
        }
    } else {
        outbound.reply(session_id,  ServerMessage::AuthErr("Not authenticated\n".to_string())).await;
    }
}
