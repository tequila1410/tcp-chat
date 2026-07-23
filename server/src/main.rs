use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, oneshot};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

mod client;

use crate::client::{ClientRegistry, SessionId};

#[derive(Debug)]
enum Command {
    Auth(String, String),
    Message(String),
}

type UserDB = Arc<HashMap<String, String>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let user_db = set_user_db();

    let listener = TcpListener::bind("127.0.0.1:1313").await?;
    let client_registry = ClientRegistry::new();

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("{error:?}");
                continue;
            }
        };

        let user_db = Arc::clone(&user_db);
        let client_registry= client_registry.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_client(stream, client_registry, user_db).await {
                eprintln!("Client error: {error}");
            }
        });
    }
}

async fn handle_client(stream: TcpStream, client_registry: ClientRegistry, user_db: UserDB) -> io::Result<()> {
    let (sender, receiver) = mpsc::channel::<Arc<Vec<u8>>>(32);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    let session_id = client_registry.insert_client(sender, shutdown_tx).await;

    let (mut read_half, write_half) = stream.into_split();

    spawn_write_task(receiver, write_half, session_id);

    const MAX_FRAME_SIZE: usize = 2 * 1024;
    let mut buffer = [0u8; 1024];
    let mut pending = Vec::new();

    loop {
        tokio::select! {
            result = read_half.read(&mut buffer) => {
                let bytes_read = match result {
                    Ok(0) => {
                        println!("Client disconnected: {session_id}");
                        client_registry.remove_client(session_id).await;
                        return Ok(());
                    }
                    Ok(n) => n,
                    Err(error) => {
                        client_registry.remove_client(session_id).await;
                        return Err(error);
                    }
                };
                pending.extend_from_slice(&buffer[..bytes_read]);
                while let Some(position) = pending.iter().position(|&byte| byte == b'\n') {
                    let frame_len = position + 1;
                    if frame_len > MAX_FRAME_SIZE {
                        client_registry.remove_client(session_id).await;
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "message too large"));
                    }
                    let message_bytes = pending.drain(..=position).collect::<Vec<u8>>();

                    if let Ok(message_str) = std::str::from_utf8(&message_bytes) {
                        println!("{message_str}");
                        match parse_command(&message_str) {
                            Some(Command::Message(message)) => {
                                if client_registry.broadcast(message.into_bytes(), session_id).await.is_none() {
                                    client_registry.send_message(session_id, b"Not authenticated\n".to_vec()).await;
                                };
                            }
                            Some(Command::Auth(login, pass)) => {
                                if let Some(db_pass) = user_db.get(&login) && *db_pass == pass {
                                    client_registry.authorize_client(session_id, login).await;
                                } else {
                                    client_registry.send_message(session_id, b"Invalid data\n".to_vec()).await;
                                };
                            },
                            _ => {
                                client_registry.send_message(session_id, b"Can't pars command\n".to_vec()).await;
                                eprintln!("Cant pars command");
                            }
                        }
                        
                    }
                }
                if pending.len() > MAX_FRAME_SIZE {
                    client_registry.remove_client(session_id).await;
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "message too large"));
                }
            },
            _ = &mut shutdown_rx => {
                println!("Shutdown signal for {session_id}");
                return Ok(());
            }
        }
    }
}

fn spawn_write_task(mut receiver: Receiver<Arc<Vec<u8>>>, mut write_half: OwnedWriteHalf, session_id: SessionId) {
    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            if let Err(error) = write_half.write_all(&message).await {
                eprintln!("Failed to write to client {session_id}: {error}");
                break;
            }
        }
    });
}

fn set_user_db() -> UserDB {
    let mut db = HashMap::new();
    db.insert("Bandera".to_string(), "123123".to_string());
    db.insert("Vlados".to_string(), "123123".to_string());
    Arc::new(db)
}

fn parse_command(line: &str) -> Option<Command> {
    let line = line.trim();
    match line.split_once(' ') {
        Some(("AUTH", value)) => {
            let (login, pass) = value.split_once(':')?;
            Some(Command::Auth(login.to_string(), pass.to_string()))
        }
        Some(("MESSAGE", value)) => Some(Command::Message(value.to_string())),
        Some(_) => None,
        None => None
    }
}
