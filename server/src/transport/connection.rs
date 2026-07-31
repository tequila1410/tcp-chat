use std::{io, sync::Arc};

use shared::framing::{FrameResult, decode_frame};
use shared::protocol::ClientMessage;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::sync::{mpsc, oneshot};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;

use crate::auth::authenticate;
use crate::chat::send_to_room;
use crate::rooms::{create_room, get_rooms, join_room};
use crate::{auth::Credentials, client::{ClientRegistry, SessionId}, room::{RoomManager, RoomStorage}};



pub async fn handle_connection<S: RoomStorage + 'static>(
    stream: TcpStream,
    client_registry: ClientRegistry,
    room_manager: RoomManager<S>,
    credentials: Credentials,
) -> io::Result<()> {
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
                        disconnect(&client_registry, &room_manager, session_id).await;
                        return Ok(());
                    }
                    Ok(n) => n,
                    Err(error) => {
                        disconnect(&client_registry, &room_manager, session_id).await;
                        return Err(error);
                    }
                };
                pending.extend_from_slice(&buffer[..bytes_read]);
                loop {
                    match decode_frame(&mut pending) {
                        FrameResult::Complete(frame) => {
                            if let Some(client_message) = ClientMessage::deserialize(&frame) {
                                match client_message {
                                    ClientMessage::SendToRoom { room, text } => {
                                        send_to_room(&client_registry, &room_manager, room, text, session_id).await;
                                    }
                                    ClientMessage::Auth{login, password} => {
                                        authenticate(&client_registry, session_id, &credentials, login, password).await;
                                    }
                                    ClientMessage::CreateRoom(room_name) => {
                                        create_room(&client_registry, &room_manager, session_id, room_name).await;
                                    }
                                    ClientMessage::JoinRoom(room_name) => {
                                        join_room(&client_registry, &room_manager, session_id, room_name).await;
                                    }
                                    ClientMessage::GetRooms => {
                                        get_rooms(&client_registry, &room_manager, session_id).await;
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
                            disconnect(&client_registry, &room_manager, session_id).await;
                            return Err(io::Error::new(io::ErrorKind::InvalidData, "Message too large"));
                        }
                    }
                }
            },
            _ = &mut shutdown_rx => {
                println!("Shutdown signal for {session_id}");
                room_manager.leave_all(session_id).await;
                return Ok(());
            }
        }
    }
}

fn spawn_write_task(mut receiver: mpsc::Receiver<Arc<Vec<u8>>>, mut write_half: OwnedWriteHalf, session_id: SessionId) {
    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            if let Err(error) = write_half.write_all(&message).await {
                eprintln!("Failed to write to client {session_id}: {error}");
                break;
            }
        }
    });
}

async fn disconnect<S: RoomStorage>(client_registry: &ClientRegistry, room_manager: &RoomManager<S>, session_id: SessionId) {
    room_manager.leave_all(session_id).await;
    client_registry.remove_client(session_id).await;
}
