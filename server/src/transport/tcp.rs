use std::io;
use tokio::net::TcpListener;

use crate::auth::Credentials;
use crate::client::ClientRegistry;
use crate::room::{RoomManager, RoomStorage};
use crate::transport::connection::handle_connection;

pub async fn run<S: RoomStorage + 'static>(
    addr: &str,
    client_registry: &ClientRegistry,
    room_manager: &RoomManager<S>,
    credentials: &Credentials,
) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("{error:?}");
                continue;
            }
        };

        let client_registry= client_registry.clone();
        let room_manager = room_manager.clone();
        let credentials = credentials.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, client_registry, room_manager, credentials).await {
                eprintln!("Client error: {error}");
            }
        });
    }
}