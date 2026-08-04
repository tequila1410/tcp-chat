use std::io;
use tokio::net::TcpListener;

use crate::room::{RoomStorage};
use crate::transport::connection::{ConnectionDeps, handle_connection};

pub async fn run<S: RoomStorage + 'static>(
    addr: &str,
    deps: ConnectionDeps<S>,
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

        let deps = deps.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, deps).await {
                eprintln!("Client error: {error}");
            }
        });
    }
}