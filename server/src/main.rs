use std::io;
use std::env;

mod client;
mod room;
mod transport;
mod app;

use crate::client::new_state;
use crate::room::memory::MemoryRoomStorage;
use crate::room::RoomManager;
use crate::app::auth::init_credentials;
use crate::transport::connection::ConnectionDeps;
use crate::transport::tcp::run;

use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .with_target(false)
        .init();

    dotenvy::dotenv().ok();
    let addr = env::var("CONNECT_ADDR_LOCAL").expect("Connection address must be set");
    let credentials = init_credentials();
    let room_manager: RoomManager<MemoryRoomStorage> = RoomManager::new();

    let (sessions, identity, outbound) = new_state();

    let deps = ConnectionDeps::new(sessions, identity, outbound, room_manager, credentials);

    info!(%addr, "server starting");
    run(&addr, deps).await
}
