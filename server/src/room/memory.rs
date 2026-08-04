use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

use crate::room::{LeaveOutcome, Room, RoomError, RoomStorage};
use crate::client::SessionId;

struct RoomState {
    rooms: HashMap<String, Room>,
    user_rooms: HashMap<SessionId, String>,
}

impl RoomState {
    fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            user_rooms: HashMap::new(),
        }
    }

    fn create_room(&mut self, session_id: SessionId, room_name: String) -> Result<(), RoomError> {
        if self.rooms.contains_key(&room_name) {
            let error_message = format!("Room name {room_name} exist");
            return Err(RoomError::AlreadyExist(error_message));
        }

        self.rooms.insert(room_name.clone(), Room { clients: HashSet::new() });
        self.join_room(session_id, room_name)?;
        Ok(())
    }

    fn join_room(&mut self, session_id: SessionId, room_name: String) -> Result<(), RoomError> {
        {
            let room = self
                .rooms
                .get(&room_name)
                .ok_or_else(|| RoomError::NotFound(format!("Can't find room {room_name}")))?;
            if room.clients.contains(&session_id) {
                return Err(RoomError::AlreadyMember(format!(
                    "User {session_id} already in room {room_name}"
                )));
            }
        }

        self.leave_all(session_id);

        let room = self
            .rooms
            .get_mut(&room_name)
            .ok_or_else(|| RoomError::NotFound(format!("Can't find room {room_name}")))?;
        room.clients.insert(session_id);
        self.user_rooms.insert(session_id, room_name);
        Ok(())
    }

    fn leave(&mut self, session_id: SessionId) -> LeaveOutcome {
        match self.user_rooms.remove(&session_id) {
            Some(room_name) => {
                if let Some(room) = self.rooms.get_mut(&room_name) {
                    room.clients.remove(&session_id);
                }
                LeaveOutcome::Left
            }
            None => LeaveOutcome::WasNotMember,
        }
    }

    fn leave_all(&mut self, session_id: SessionId) {
        let _ = self.leave(session_id);
    }

    fn recipients_for(
        &self,
        room_name: &str,
        session_id: SessionId,
    ) -> Result<HashSet<SessionId>, RoomError> {
        let room = self
            .rooms
            .get(room_name)
            .ok_or_else(|| RoomError::NotFound(room_name.to_string()))?;

        if !room.clients.contains(&session_id) {
            return Err(RoomError::NotMember(session_id.to_string()));
        }

        Ok(room
            .clients
            .iter()
            .copied()
            .filter(|client| *client != session_id)
            .collect())
    }
}

pub struct MemoryRoomStorage {
    state: RwLock<RoomState>
}

impl MemoryRoomStorage {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(RoomState::new())
        }
    }
}

#[async_trait::async_trait]
impl RoomStorage for MemoryRoomStorage {
    async fn create_room(&self, session_id: SessionId, room_name: String) -> Result<(), super::RoomError> {
        let mut state = self.state.write().await;
        state.create_room(session_id, room_name)
    }

    async fn join_room(&self, session_id: SessionId, room_name: String) -> Result<(), RoomError> {
        let mut state = self.state.write().await;
        state.join_room(session_id, room_name)
    }

    async fn get_room_members(&self, room_name: String) -> Result<HashSet<SessionId>, RoomError> {
        let state = self.state.read().await;
        match state.rooms.get(&room_name) {
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

    async fn recipients_for(&self, room_name: &str, session_id: SessionId) -> Result<HashSet<SessionId>, RoomError> {
        let state = self.state.read().await;
        state.recipients_for(room_name, session_id)
    }

    async fn delete_room(&self, room_name: String) -> Result<(), RoomError> {
        let mut state = self.state.write().await;
        match state.rooms.remove(&room_name) {
            Some(_) => Ok(()),
            None => {
                let error_message = format!("Can't find room {room_name}");
                Err(RoomError::NotFound(error_message))
            }
        }
    }

    async fn get_rooms(&self) -> Result<Vec<String>, RoomError> {
        let state = self.state.read().await;
        let room_names = state.rooms.keys().cloned().collect();
        Ok(room_names)
    }

    async fn leave(&self, session_id: SessionId) -> LeaveOutcome {
        let mut state = self.state.write().await;
        state.leave(session_id)
    }

    async fn leave_all(&self, session_id: SessionId) {
        let mut state = self.state.write().await;
        state.leave_all(session_id);
    }
}

#[cfg(test)]
#[path = "memory_tests.rs"]
mod tests;