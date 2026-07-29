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
    async fn create_room(&self, name: String) -> Result<(), super::RoomError> {
        let mut rooms = self.rooms.write().await;

        if rooms.contains_key(&name) {
            let error_message = format!("Room name {name} exist");
            return Err(RoomError::AlreadyExist(error_message));
        }

        rooms.insert(name, Room { clients: vec![] });

        Ok(())
    }

    async fn join_room(&self, name: String, session_id: SessionId) -> Result<(), RoomError> {
        let mut rooms = self.rooms.write().await;
        match rooms.get_mut(&name) {
            Some(room) => {
                room.clients.push(session_id);
                Ok(())
            }
            None => {
                let error_message = format!("Can't find room {name}");
                Err(RoomError::NotFound(error_message))
            }
        }
    }

    async fn get_room_members(&self, name: String) -> Result<Vec<SessionId>, RoomError> {
        let rooms = self.rooms.read().await;
        match rooms.get(&name) {
            Some(room) => {
                // need optimization if clients in room will be more
                let clients = room.clients.clone();
                Ok(clients)
            }
            None => {
                let error_message = format!("Can't find room {name}");
                Err(RoomError::NotFound(error_message))
            }
        }
    }

    async fn delete_room(&self, name: String) -> Result<(), RoomError> {
        let mut rooms = self.rooms.write().await;
        match rooms.remove(&name) {
            Some(_) => Ok(()),
            None => {
                let error_message = format!("Can't find room {name}");
                Err(RoomError::NotFound(error_message))
            }
        }
    }

    async fn get_rooms(&self) -> Result<Vec<String>, RoomError> {
        let rooms = self.rooms.read().await;
        let room_names = rooms.keys().cloned().collect();
        Ok(room_names)
    }
}