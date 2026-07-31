use std::io;
use std::env;

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
    dotenvy::dotenv().ok();
    let addr = env::var("CONNECT_ADDR_LOCAL").expect("Connection address must be set");
    let credentials = init_credentials();
    let client_registry = ClientRegistry::new();
    let room_manager: RoomManager<MemoryRoomStorage> = RoomManager::new();

    run(&addr, &client_registry, &room_manager, &credentials).await
}
