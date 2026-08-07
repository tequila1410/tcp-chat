use std::io;
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::transport::connection::{handle_connection, ConnectionDeps};

pub async fn run(
    addr: &str,
    deps: ConnectionDeps,
) -> io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!(%addr, "listening");
    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(connection) => connection,
            Err(error) => {
                error!(?error, "accept failed");
                continue;
            }
        };

        info!(%peer, "accepted connection");

        let deps = deps.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, deps).await {
                error!(%peer, ?error, "connection task failed");
            }
        });
    }
}