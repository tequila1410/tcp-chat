pub mod memory;

use std::sync::Arc;

use crate::{client::SessionId, room::memory::MemoryRoomStorage};

pub trait RoomStorage {
    async fn create_room(&self, name: String) -> Result<(), RoomError>;
    async fn delete_room(&self, name: String) -> Result<(), RoomError>;
    async fn get_rooms(&self) -> Result<Vec<String>, RoomError>;
    async fn join_room(&self, name: String, session_id: SessionId) -> Result<(), RoomError>;
    async fn get_room_members(&self, name: String) -> Result<Vec<SessionId>, RoomError>;
    async fn recipients_for(&self, name: &str, session_id: SessionId) -> Result<Vec<SessionId>, RoomError>;
}

pub struct RoomManager<S: RoomStorage> {
    pub storage: Arc<S>,
}

impl<S: RoomStorage> Clone for RoomManager<S> {
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage)
        }
    }
}

impl<S: RoomStorage> RoomManager<S> {
    pub async fn create_room(&self, name: String) -> Result<(), RoomError> {
        self.storage.create_room(name).await
    }

    pub async fn join_room(&self, name: String, session_id: SessionId) -> Result<(), RoomError> {
        self.storage.join_room(name, session_id).await
    }

    pub async fn get_rooms(&self) -> Result<Vec<String>, RoomError> {
        self.storage.get_rooms().await
    }

    pub async fn get_room_members(&self, name: String) -> Result<Vec<SessionId>, RoomError> {
        self.storage.get_room_members(name).await
    }

    pub async fn recipients_for(&self, name: &str, session_id: SessionId) -> Result<Vec<SessionId>, RoomError> {
        self.storage.recipients_for(name, session_id).await
    }
}

impl RoomManager<MemoryRoomStorage> {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(MemoryRoomStorage::new())
        }
    }
}

pub struct Room {
    clients: Vec<SessionId>
}

#[derive(Debug, thiserror::Error)]
pub enum RoomError {
    #[error("room not found: {0}")]
    NotFound(String),
    #[error("room already exist: {0}")]
    AlreadyExist(String),
    #[error("storage error: {0}")]
    StorageError(String),
    #[error("user not exist: {0}")]
    NotMember(String),
}
