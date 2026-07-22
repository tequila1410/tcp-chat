use std::io;
use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1313").await?;

    loop {
        let (stream, peer_addr) = match listener.accept().await {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("Failed to accept connection: {error}");
                continue;
            }
        };

        tokio::spawn(async move {
            println!("Client connected: {peer_addr}");

            if let Err(error) = handle_client(stream, peer_addr).await {
                eprintln!("Client error {peer_addr}: {error}");
            }
        });
    }
}

async fn handle_client(mut stream: TcpStream, peer_addr: SocketAddr) -> io::Result<()> {
    const MAX_FRAME_SIZE: usize = 8 * 1024;
    let mut buffer = [0u8; 1024];
    let mut pending = Vec::new();

    loop {
        let bytes_read = match stream.read(&mut buffer).await {
            Ok(0) => {
                println!("Client disconnected: {peer_addr}");
                return Ok(());
            }
            Ok(n) => n,
            Err(error) if is_client_disconnect_error(&error) => {
                println!("Client reset connection: {peer_addr}");
                return Ok(());
            }
            Err(error) => return Err(error),
        };

        pending.extend_from_slice(&buffer[..bytes_read]);
        println!("{pending:?}");
        while let Some(position) = pending.iter().position(|&byte| byte == b'\n') {
            if position + 1 > MAX_FRAME_SIZE {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "message too large"));
            }
            let message = pending.drain(..=position).collect::<Vec<u8>>();
            stream.write_all(&message).await?;
        }
        if pending.len() > MAX_FRAME_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "message too large"));
        }

    }
}

fn is_client_disconnect_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe
    )
}
