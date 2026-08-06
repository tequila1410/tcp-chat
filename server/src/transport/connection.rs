use std::{io, sync::Arc};

use shared::framing::{FrameResult, decode_frame};
use shared::protocol::ClientMessage;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::sync::{mpsc, oneshot};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;

use crate::app::auth::authenticate;
use crate::app::chat::{send_to_room};
use crate::client::{Identity, Outbound, Sessions};
use crate::app::rooms::{create_room, get_rooms, join_room, leave_room};
use crate::transport::respond::{apply_auth_outcome, apply_chat_outcome, apply_room_outcome};
use crate::app::auth::Credentials;
use crate::room::{RoomManager, RoomStorage};
use crate::client::SessionId;

pub struct ConnectionDeps<S: RoomStorage> {
    sessions: Sessions,
    identity: Identity,
    outbound: Outbound,
    rooms: RoomManager<S>,
    credentials: Credentials,
}

impl<S: RoomStorage> ConnectionDeps<S> {
    pub fn new(sessions: Sessions, identity: Identity, outbound: Outbound, rooms: RoomManager<S>, credentials: Credentials) -> Self {
        Self { sessions, identity, outbound, rooms, credentials }
    }
}

impl<S: RoomStorage> Clone for ConnectionDeps<S> {
    fn clone(&self) -> Self {
        Self { sessions: self.sessions.clone(), identity: self.identity.clone(), outbound: self.outbound.clone(), rooms: self.rooms.clone(), credentials: self.credentials.clone() }
    }
}

pub async fn handle_connection<S: RoomStorage + 'static>(
    stream: TcpStream,
    deps: ConnectionDeps<S>,
) -> io::Result<()> {
    let (outbound_tx, outbound_rx) = mpsc::channel::<Arc<Vec<u8>>>(32);
    let (evict_tx, mut evict_rx) = oneshot::channel::<()>();
    let (writer_done_tx, mut writer_done_rx) = oneshot::channel::<()>();

    let session_id = deps.sessions.insert_client(outbound_tx, evict_tx).await;

    let (mut read_half, write_half) = stream.into_split();

    spawn_write_task(outbound_rx, write_half, session_id, writer_done_tx);

    let mut buffer = [0u8; 1024];
    let mut pending = Vec::new();

    loop {
        tokio::select! {
            result = read_half.read(&mut buffer) => {
                let bytes_read = match result {
                    Ok(0) => {
                        println!("Client disconnected: {session_id}");
                        disconnect(&deps.sessions, &deps.rooms, session_id).await;
                        return Ok(());
                    }
                    Ok(n) => n,
                    Err(error) => {
                        disconnect(&deps.sessions, &deps.rooms, session_id).await;
                        return Err(error);
                    }
                };
                pending.extend_from_slice(&buffer[..bytes_read]);
                loop {
                    match decode_frame(&mut pending) {
                        FrameResult::Complete(frame) => {
                            match ClientMessage::deserialize(&frame) {
                                Ok(client_message) => {
                                    match client_message {
                                        ClientMessage::SendToRoom { room, text } => {
                                            let outcome = send_to_room(&deps.identity, &deps.rooms, session_id, room, text).await;
                                            apply_chat_outcome(&deps.outbound, session_id, outcome).await;
                                        }
                                        ClientMessage::Auth{login, password} => {
                                            let outcome = authenticate(&deps.identity, session_id, &deps.credentials, login, password).await;
                                            apply_auth_outcome(&deps.outbound, session_id, outcome).await;
                                        }
                                        ClientMessage::CreateRoom(room_name) => {
                                            let outcome = create_room(&deps.identity, &deps.rooms, session_id, room_name).await;
                                            apply_room_outcome(&deps.outbound, session_id, outcome).await;
                                        }
                                        ClientMessage::JoinRoom(room_name) => {
                                            let outcome = join_room(&deps.identity, &deps.rooms, session_id, room_name).await;
                                            apply_room_outcome(&deps.outbound, session_id, outcome).await;
                                        }
                                        ClientMessage::GetRooms => {
                                            let outcome = get_rooms(&deps.identity, &deps.rooms, session_id).await;
                                            apply_room_outcome(&deps.outbound, session_id, outcome).await;
                                        }
                                        ClientMessage::LeaveRoom => {
                                            let outcome = leave_room(&deps.identity, &deps.rooms, session_id).await;
                                            apply_room_outcome(&deps.outbound, session_id, outcome).await;
                                        }
                                    }
                                }
                                Err(error) => {
                                    println!("Can't deserialize frame: {error:?}");
                                }
                            }
                        }
                        FrameResult::Incomplete => {
                            break;
                        }
                        FrameResult::TooLarge => {
                            disconnect(&deps.sessions, &deps.rooms, session_id).await;
                            return Err(io::Error::new(io::ErrorKind::InvalidData, "Message too large"));
                        }
                    }
                }
            },
            _ = &mut evict_rx => {
                println!("Evict signal for {session_id}");
                disconnect(&deps.sessions, &deps.rooms, session_id).await;
                return Ok(());
            },
            _ = &mut writer_done_rx => {
                println!("Writer done for {session_id}");
                disconnect(&deps.sessions, &deps.rooms, session_id).await;
                return Ok(());
            }
        }
    }
}

fn spawn_write_task(
    mut outbound_rx: mpsc::Receiver<Arc<Vec<u8>>>,
    mut write_half: OwnedWriteHalf,
    session_id: SessionId,
    writer_done_tx: oneshot::Sender<()>,
) {
    tokio::spawn(async move {
        while let Some(message) = outbound_rx.recv().await {
            if let Err(error) = write_half.write_all(&message).await {
                eprintln!("Failed to write to client {session_id}: {error}");
                break;
            }
        }
        let _ = writer_done_tx.send(());
    });
}

async fn disconnect<S: RoomStorage>(sessions: &Sessions, room_manager: &RoomManager<S>, session_id: SessionId) {
    room_manager.leave_all(session_id).await;
    sessions.remove_client(session_id).await;
}
