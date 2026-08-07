use std::{io, sync::Arc};

use shared::framing::{decode_frame, FrameResult};
use shared::protocol::{ClientMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, info_span, warn, Instrument};

use crate::app::auth::authenticate;
use crate::app::auth::Credentials;
use crate::app::chat::send_to_room;
use crate::app::rooms::{create_room, get_rooms, join_room, leave_room};
use crate::client::{Identity, Outbound, SessionId, Sessions};
use crate::room_store::RoomStore;
use crate::transport::respond::{apply_auth_outcome, apply_chat_outcome, apply_room_outcome};

#[derive(Debug)]
enum DisconnectReason {
    Eof,
    ReadError,
    FrameTooLarge,
    Evicted,
    WriterDone,
} 

pub struct ConnectionDeps {
    sessions: Sessions,
    identity: Identity,
    outbound: Outbound,
    room_store: RoomStore,
    credentials: Credentials,
}

impl ConnectionDeps {
    pub fn new(sessions: Sessions, identity: Identity, outbound: Outbound, room_store: RoomStore, credentials: Credentials) -> Self {
        Self { sessions, identity, outbound, room_store, credentials }
    }
}

impl Clone for ConnectionDeps {
    fn clone(&self) -> Self {
        Self { sessions: self.sessions.clone(), identity: self.identity.clone(), outbound: self.outbound.clone(), room_store: self.room_store.clone(), credentials: self.credentials.clone() }
    }
}

pub async fn handle_connection(
    stream: TcpStream,
    deps: ConnectionDeps,
) -> io::Result<()> {
    let (outbound_tx, outbound_rx) = mpsc::channel::<Arc<Vec<u8>>>(32);
    let (evict_tx, mut evict_rx) = oneshot::channel::<()>();
    let (writer_done_tx, mut writer_done_rx) = oneshot::channel::<()>();
    
    let session_id = deps.sessions.insert_client(outbound_tx, evict_tx).await;

    let span = info_span!("connection", session_id);

    async move {
        let (mut read_half, write_half) = stream.into_split();

        spawn_write_task(outbound_rx, write_half, writer_done_tx);

        info!("session started");

        let mut buffer = [0u8; 1024];
        let mut pending = Vec::new();

        loop {
            tokio::select! {
                result = read_half.read(&mut buffer) => {
                    let bytes_read = match result {
                        Ok(0) => {
                            disconnect(&deps.sessions, &deps.room_store, session_id, DisconnectReason::Eof).await;
                            return Ok(());
                        }
                        Ok(n) => n,
                        Err(error) => {
                            disconnect(&deps.sessions, &deps.room_store, session_id, DisconnectReason::ReadError).await;
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
                                                let outcome = send_to_room(&deps.identity, &deps.room_store, session_id, room, text).await;
                                                apply_chat_outcome(&deps.outbound, session_id, outcome).await;
                                            }
                                            ClientMessage::Auth{login, password} => {
                                                let outcome = authenticate(&deps.identity, session_id, &deps.credentials, login, password).await;
                                                apply_auth_outcome(&deps.outbound, session_id, outcome).await;
                                            }
                                            ClientMessage::CreateRoom(room_name) => {
                                                let outcome = create_room(&deps.identity, &deps.room_store, session_id, room_name).await;
                                                apply_room_outcome(&deps.outbound, session_id, outcome).await;
                                            }
                                            ClientMessage::JoinRoom(room_name) => {
                                                let outcome = join_room(&deps.identity, &deps.room_store, session_id, room_name).await;
                                                apply_room_outcome(&deps.outbound, session_id, outcome).await;
                                            }
                                            ClientMessage::GetRooms => {
                                                let outcome = get_rooms(&deps.identity, &deps.room_store, session_id).await;
                                                apply_room_outcome(&deps.outbound, session_id, outcome).await;
                                            }
                                            ClientMessage::LeaveRoom => {
                                                let outcome = leave_room(&deps.identity, &deps.room_store, session_id).await;
                                                apply_room_outcome(&deps.outbound, session_id, outcome).await;
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        warn!(?error, "can't deserialize frame");
                                    }
                                }
                            }
                            FrameResult::Incomplete => {
                                break;
                            }
                            FrameResult::TooLarge => {
                                disconnect(&deps.sessions, &deps.room_store, session_id, DisconnectReason::FrameTooLarge).await;
                                return Err(io::Error::new(io::ErrorKind::InvalidData, "Message too large"));
                            }
                        }
                    }
                },
                _ = &mut evict_rx => {
                    disconnect(&deps.sessions, &deps.room_store, session_id, DisconnectReason::Evicted).await;
                    return Ok(());
                },
                _ = &mut writer_done_rx => {
                    disconnect(&deps.sessions, &deps.room_store, session_id, DisconnectReason::WriterDone).await;
                    return Ok(());
                }
            }
        }
    }
    .instrument(span)
    .await
}

fn spawn_write_task(
    mut outbound_rx: mpsc::Receiver<Arc<Vec<u8>>>,
    mut write_half: OwnedWriteHalf,
    writer_done_tx: oneshot::Sender<()>,
) {
    tokio::spawn(async move {
        while let Some(message) = outbound_rx.recv().await {
            if let Err(_) = write_half.write_all(&message).await {
                break;
            }
        }
        let _ = writer_done_tx.send(());
    });
}

async fn disconnect(sessions: &Sessions, room_store: &RoomStore, session_id: SessionId, reason: DisconnectReason) {
    room_store.leave_all(session_id).await;
    sessions.remove_client(session_id).await;
    info!(?reason, "session disconnected");
}
