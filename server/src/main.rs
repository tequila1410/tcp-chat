use std::io;

mod client;
mod room;
mod chat;
mod auth;
mod rooms;
mod transport;

use crate::client::ClientRegistry;
use crate::room::memory::MemoryRoomStorage;
use crate::room::RoomManager;
use crate::auth::init_credentials;
use crate::transport::tcp::run;

#[tokio::main]
async fn main() -> io::Result<()> {
    let credentials = init_credentials();
    let client_registry = ClientRegistry::new();
    let room_manager: RoomManager<MemoryRoomStorage> = RoomManager::new();

    run("127.0.0.1:1313", &client_registry, &room_manager, &credentials).await
}
