use std::io;
use std::env;

mod client;
mod room;
mod chat;
mod auth;
mod rooms;
mod transport;

use crate::client::new_state;
use crate::room::memory::MemoryRoomStorage;
use crate::room::RoomManager;
use crate::auth::init_credentials;
use crate::transport::connection::ConnectionDeps;
use crate::transport::tcp::run;

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    let addr = env::var("CONNECT_ADDR_LOCAL").expect("Connection address must be set");
    let credentials = init_credentials();
    let room_manager: RoomManager<MemoryRoomStorage> = RoomManager::new();

    let (sessions, identity, outbound) = new_state();

    let deps = ConnectionDeps::new(sessions, identity, outbound, room_manager, credentials);

    run(&addr, deps).await
}
