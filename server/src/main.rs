use std::io;
use std::env;

mod client;
mod room_store;
mod transport;
mod app;

use crate::client::new_state;
use crate::room_store::RoomStore;
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
    let max_clients = env::var("MAX_CLIENTS").expect("MAX_CLIENTS must be set").parse().unwrap();
    let idle_timeout_secs = env::var("IDLE_TIMEOUT_SECS").expect("IDLE_TIMEOUT_SECS must be set").parse().unwrap();
    let credentials = init_credentials();
    let room_store= RoomStore::new();

    let (sessions, identity, outbound) = new_state();

    let deps = ConnectionDeps::new(sessions, identity, outbound, room_store, credentials, max_clients, idle_timeout_secs);

    info!(%addr, "server starting");
    run(&addr, deps).await
}
