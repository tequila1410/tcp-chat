use std::collections::HashSet;

use crate::{client::SessionId};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;


pub struct Room {
    clients: HashSet<SessionId>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaveOutcome {
    Left(String),
    WasNotMember,
}

#[derive(Debug, thiserror::Error)]
pub enum RoomError {
    #[error("room not found: {0}")]
    NotFound(String),

    #[error("room already exist: {0}")]
    AlreadyExist(String),

    #[error("user not exist: {0}")]
    NotMember(String),

    #[error("user already in room: {0}")]
    AlreadyMember(String),
}

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
            return Err(RoomError::AlreadyExist(room_name.clone()));
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
                .ok_or_else(|| RoomError::NotFound(room_name.clone()))?;
            if room.clients.contains(&session_id) {
                return Err(RoomError::AlreadyMember(room_name));
            }
        }

        self.leave_all(session_id);

        let room = self
            .rooms
            .get_mut(&room_name)
            .ok_or_else(|| RoomError::NotFound(room_name.clone()))?;
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
                LeaveOutcome::Left(room_name)
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

#[derive(Clone)]
pub struct RoomStore {
    state: Arc<RwLock<RoomState>>
}

impl RoomStore {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(RoomState::new()))
        }
    }

    pub async fn create_room(&self, session_id: SessionId, room_name: String) -> Result<(), RoomError> {
        let mut state = self.state.write().await;
        state.create_room(session_id, room_name)
    }

    pub async fn join_room(&self, session_id: SessionId, room_name: String) -> Result<(), RoomError> {
        let mut state = self.state.write().await;
        state.join_room(session_id, room_name)
    }

    pub async fn recipients_for(&self, room_name: &str, session_id: SessionId) -> Result<HashSet<SessionId>, RoomError> {
        let state = self.state.read().await;
        state.recipients_for(room_name, session_id)
    }

    pub async fn get_rooms(&self) -> Result<Vec<String>, RoomError> {
        let state = self.state.read().await;
        let room_names = state.rooms.keys().cloned().collect();
        Ok(room_names)
    }

    pub async fn leave(&self, session_id: SessionId) -> LeaveOutcome {
        let mut state = self.state.write().await;
        state.leave(session_id)
    }

    pub async fn leave_all(&self, session_id: SessionId) {
        let mut state = self.state.write().await;
        state.leave_all(session_id);
    }
}

#[cfg(test)]
#[path = "room_store_test.rs"]
mod tests;