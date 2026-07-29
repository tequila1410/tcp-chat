use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, oneshot};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use shared::framing::{decode_frame, encode_frame, FrameResult};
use shared::protocol::{ClientMessage, ServerMessage};

mod client;
mod room;

use crate::client::{ClientRegistry, SessionId};
use crate::room::memory::MemoryRoomStorage;
use crate::room::{RoomManager, RoomStorage};

type Credentials = Arc<HashMap<String, String>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let credentials = init_credentials();

    let listener = TcpListener::bind("127.0.0.1:1313").await?;
    let client_registry = ClientRegistry::new();
    let room_manager: RoomManager<MemoryRoomStorage> = RoomManager::new();

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("{error:?}");
                continue;
            }
        };

        let credentials = Arc::clone(&credentials);
        let client_registry= client_registry.clone();
        let room_manager = room_manager.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_client(stream, client_registry, room_manager, credentials).await {
                eprintln!("Client error: {error}");
            }
        });
    }
}

async fn handle_client<S: RoomStorage>(stream: TcpStream, client_registry: ClientRegistry, room_manager: RoomManager<S>, credentials: Credentials) -> io::Result<()> {
    let (sender, receiver) = mpsc::channel::<Arc<Vec<u8>>>(32);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    let session_id = client_registry.insert_client(sender, shutdown_tx).await;

    let (mut read_half, write_half) = stream.into_split();

    spawn_write_task(receiver, write_half, session_id);

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
                loop {
                    match decode_frame(&mut pending) {
                        FrameResult::Complete(frame) => {
                            if let Some(client_message) = ClientMessage::deserialize(&frame) {
                                match client_message {
                                    ClientMessage::Message(message) => {
                                        println!("Client message: {message}");
                                        if let Some(message_from) = client_registry.get_login(session_id).await {
                                            let payload = ServerMessage::Message{from: message_from, text: message}.serialize();
                                            let message = encode_frame(&payload);
                                            client_registry.broadcast(message, session_id).await;
                                        } else {
                                            let payload = ServerMessage::AuthErr("Not authenticated\n".to_string()).serialize();
                                            let message = encode_frame(&payload);
                                            client_registry.send_message(session_id, message.to_vec()).await;
                                        }
                                    }
                                    ClientMessage::Auth{login, password} => {
                                        if let Some(db_pass) = credentials.get(&login) && *db_pass == password {
                                            client_registry.authorize_client(session_id, login).await;
                                            let payload = ServerMessage::AuthOk.serialize();
                                            let message = encode_frame(&payload);
                                            client_registry.send_message(session_id, message.to_vec()).await;
                                        } else {
                                            let payload = ServerMessage::Err("Not authenticated\n".to_string()).serialize();
                                            let message = encode_frame(&payload);
                                            client_registry.send_message(session_id, message.to_vec()).await;
                                        };
                                    }
                                    ClientMessage::CreateRoom(room_name) => {
                                        if let Some(_) = client_registry.get_login(session_id).await {
                                            match room_manager.create_room(room_name).await {
                                                Ok(_) => {
                                                    let payload = ServerMessage::RoomCreated("Room successfuly created".to_string()).serialize();
                                                    let message = encode_frame(&payload);
                                                    client_registry.send_message(session_id, message.to_vec()).await;
                                                }
                                                Err(_) => {
                                                    let payload = ServerMessage::RoomErr("Room already exist\n".to_string()).serialize();
                                                    let message = encode_frame(&payload);
                                                    client_registry.send_message(session_id, message.to_vec()).await;
                                                }
                                            };
                                        } else {
                                            let payload = ServerMessage::AuthErr("Not authenticated\n".to_string()).serialize();
                                            let message = encode_frame(&payload);
                                            client_registry.send_message(session_id, message.to_vec()).await;
                                        }
                                    }
                                    ClientMessage::JoinRoom(room_name) => {
                                        if let Some(_) = client_registry.get_login(session_id).await {
                                            match room_manager.join_room(room_name, session_id).await {
                                                Ok(_) => {
                                                    let payload = ServerMessage::RoomJoined("Room successfuly joined".to_string()).serialize();
                                                    let message = encode_frame(&payload);
                                                    client_registry.send_message(session_id, message.to_vec()).await;
                                                }
                                                Err(_) => {
                                                    let payload = ServerMessage::RoomErr("No room with this name\n".to_string()).serialize();
                                                    let message = encode_frame(&payload);
                                                    client_registry.send_message(session_id, message.to_vec()).await;
                                                }
                                            };
                                        } else {
                                            let payload = ServerMessage::AuthErr("Not authenticated\n".to_string()).serialize();
                                            let message = encode_frame(&payload);
                                            client_registry.send_message(session_id, message.to_vec()).await;
                                        }
                                    }
                                    ClientMessage::GetRooms => {
                                        if let Some(_) = client_registry.get_login(session_id).await {
                                            match room_manager.get_rooms().await {
                                                Ok(rooms) => {
                                                    let payload = ServerMessage::RoomsGet(rooms).serialize();
                                                    let message = encode_frame(&payload);
                                                    client_registry.send_message(session_id, message.to_vec()).await;
                                                }
                                                Err(err) => {
                                                    println!("Creating room error: {err:?}");
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                println!("Can't deserialize frame");
                            }
                        }
                        FrameResult::Incomplete => {
                            break;
                        }
                        FrameResult::TooLarge => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, "Message too large"));
                        }
                    }
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

fn init_credentials() -> Credentials {
    let mut db = HashMap::new();
    db.insert("Bandera".to_string(), "123123".to_string());
    db.insert("Vlados".to_string(), "123123".to_string());
    Arc::new(db)
}
