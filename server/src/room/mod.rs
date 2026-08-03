//! Room membership and storage.
//!
//! Policy (Phase 1.2): a session is in 0 or 1 room; join/create switches;
//! leave is idempotent on the wire (`RoomLeft` for both outcomes).
//! Full contract: repo `roadmap.md` § Membership rules.

pub mod memory;

use std::{collections::HashSet, sync::Arc};

use crate::{client::SessionId, room::memory::MemoryRoomStorage};

/// Async storage port. Futures must be `Send` so callers can `tokio::spawn` work that uses rooms.
#[async_trait::async_trait]
pub trait RoomStorage: Send + Sync {
    async fn create_room(&self, session_id: SessionId, name: String) -> Result<(), RoomError>;
    async fn delete_room(&self, name: String) -> Result<(), RoomError>;
    async fn get_rooms(&self) -> Result<Vec<String>, RoomError>;
    async fn join_room(
        &self,
        name: String,
        session_id: SessionId,
    ) -> Result<(), RoomError>;
    async fn get_room_members(
        &self,
        name: String,
    ) -> Result<HashSet<SessionId>, RoomError>;
    async fn recipients_for(
        &self,
        name: &str,
        session_id: SessionId,
    ) -> Result<HashSet<SessionId>, RoomError>;
    async fn leave(&self, session_id: SessionId) -> LeaveOutcome;
    async fn leave_all(&self, session_id: SessionId);
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
    pub async fn create_room(&self, session_id: SessionId, name: String) -> Result<(), RoomError> {
        self.storage.create_room(session_id, name).await
    }

    pub async fn join_room(&self, name: String, session_id: SessionId) -> Result<(), RoomError> {
        self.storage.join_room(name, session_id).await
    }

    pub async fn get_rooms(&self) -> Result<Vec<String>, RoomError> {
        self.storage.get_rooms().await
    }

    pub async fn get_room_members(&self, name: String) -> Result<HashSet<SessionId>, RoomError> {
        self.storage.get_room_members(name).await
    }

    pub async fn recipients_for(&self, name: &str, session_id: SessionId) -> Result<HashSet<SessionId>, RoomError> {
        self.storage.recipients_for(name, session_id).await
    }

    pub async fn leave(&self, session_id: SessionId) -> LeaveOutcome {
        self.storage.leave(session_id).await
    }

    pub async fn leave_all(&self, session_id: SessionId) {
        self.storage.leave_all(session_id).await;
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
    clients: HashSet<SessionId>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaveOutcome {
    Left,
    WasNotMember,
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

    #[error("user already in room: {0}")]
    AlreadyMember(String),
}
