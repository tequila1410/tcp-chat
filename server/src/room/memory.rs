use std::collections::HashMap;
use tokio::sync::RwLock;

use crate::room::{Room, RoomError, RoomStorage};
use crate::client::SessionId;


pub struct MemoryRoomStorage {
    rooms: RwLock<HashMap<String, Room>>
}

impl MemoryRoomStorage {
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new())
        }
    }
}

impl RoomStorage for MemoryRoomStorage {
    async fn create_room(&self, room_name: String) -> Result<(), super::RoomError> {
        let mut rooms = self.rooms.write().await;

        if rooms.contains_key(&room_name) {
            let error_message = format!("Room name {room_name} exist");
            return Err(RoomError::AlreadyExist(error_message));
        }

        rooms.insert(room_name, Room { clients: vec![] });

        Ok(())
    }

    async fn join_room(&self, room_name: String, session_id: SessionId) -> Result<(), RoomError> {
        let mut rooms = self.rooms.write().await;
        match rooms.get_mut(&room_name) {
            Some(room) => {
                room.clients.push(session_id);
                Ok(())
            }
            None => {
                let error_message = format!("Can't find room {room_name}");
                Err(RoomError::NotFound(error_message))
            }
        }
    }

    async fn get_room_members(&self, room_name: String) -> Result<Vec<SessionId>, RoomError> {
        let rooms = self.rooms.read().await;
        match rooms.get(&room_name) {
            Some(room) => {
                // need optimization if clients in room will be more
                let clients = room.clients.clone();
                Ok(clients)
            }
            None => {
                let error_message = format!("Can't find room {room_name}");
                Err(RoomError::NotFound(error_message))
            }
        }
    }

    async fn recipients_for(&self, room_name: &str, session_id: SessionId) -> Result<Vec<SessionId>, RoomError> {
        let rooms = self.rooms.read().await;
        if let Some(room) = rooms.get(room_name) {
            if room.clients.contains(&session_id) {
                let clients = room.clients.clone().into_iter().filter(|client| *client != session_id).collect::<Vec<SessionId>>();
                return Ok(clients);
            } else {
                return Err(RoomError::NotMember(session_id.to_string()));
            }
        } else {
            return Err(RoomError::NotFound(room_name.to_string()));
        }
    }

    async fn delete_room(&self, room_name: String) -> Result<(), RoomError> {
        let mut rooms = self.rooms.write().await;
        match rooms.remove(&room_name) {
            Some(_) => Ok(()),
            None => {
                let error_message = format!("Can't find room {room_name}");
                Err(RoomError::NotFound(error_message))
            }
        }
    }

    async fn get_rooms(&self) -> Result<Vec<String>, RoomError> {
        let rooms = self.rooms.read().await;
        let room_names = rooms.keys().cloned().collect();
        Ok(room_names)
    }

    async fn leave_all(&self, session_id: SessionId) {
        let mut rooms = self.rooms.write().await;
        rooms.iter_mut().for_each(|(_, room)| {
            room.clients.retain(|id| *id != session_id);
        });
    }

}