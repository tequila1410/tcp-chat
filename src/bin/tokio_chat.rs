use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::sync::atomic::AtomicU64;

#[derive(Debug)]
struct ClientHandle {
    login: Option<String>,
    sender: mpsc::Sender<Vec<u8>>,
    shutdown: oneshot::Sender<()>,
}

#[derive(Debug)]
enum Command {
    Auth(String, String),
    Message(String),
}

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);
type SessionId = u64;
type UserDB = Arc<HashMap<String, String>>;
type Clients = Arc<Mutex<HashMap<SessionId, ClientHandle>>>;

#[tokio::main]
async fn main() -> io::Result<()> {
    let user_db = set_user_db();

    let listener = TcpListener::bind("127.0.0.1:1313").await?;
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(connection) => connection,
            Err(error) => {
                eprintln!("{error:?}");
                continue;
            }
        };

        let clients = Arc::clone(&clients);
        let user_db = Arc::clone(&user_db);
        tokio::spawn(async move {
            if let Err(error) = handle_client(stream, clients, user_db).await {
                eprintln!("Client error: {error}");
            }
        });
    }
}

async fn handle_client(stream: TcpStream, clients: Clients, user_db: UserDB) -> io::Result<()> {
    let (sender, receiver) = mpsc::channel::<Vec<u8>>(32);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
    let session_id: SessionId = NEXT_CLIENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    {
        let mut locked_clients = clients.lock().await;
        locked_clients.insert(session_id, ClientHandle { login: None, sender, shutdown: shutdown_tx });
    }
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
                        remove_client(&clients, session_id).await;
                        return Ok(());
                    }
                    Ok(n) => n,
                    Err(error) => {
                        remove_client(&clients, session_id).await;
                        return Err(error);
                    }
                };
                pending.extend_from_slice(&buffer[..bytes_read]);
                while let Some(position) = pending.iter().position(|&byte| byte == b'\n') {
                    let frame_len = position + 1;
                    if frame_len > MAX_FRAME_SIZE {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "message too large"));
                    }
                    let message_bytes = pending.drain(..=position).collect::<Vec<u8>>();

                    if let Ok(message_str) = std::str::from_utf8(&message_bytes) {
                        println!("{message_str}");
                        match parse_command(&message_str) {
                            Some(Command::Message(message)) => {
                                let client_login = {
                                    let client_lock = clients.lock().await;
                                    client_lock.get(&session_id).and_then(|c| c.login.clone())
                                };
                                if client_login.is_some() {
                                    broadcast(message.into_bytes(), &clients, session_id).await;
                                }
                            },
                            Some(Command::Auth(login, pass)) => {
                                let mut client_lock = clients.lock().await;
                                if let Some(db_pass) = user_db.get(&login) && *db_pass == pass {
                                    if let Some(client) = client_lock.get_mut(&session_id) {
                                        client.login = Some(login);
                                    }
                                } else {
                                    if let Some(client) = client_lock.get_mut(&session_id) {
                                        let _ = client.sender.try_send(b"Invalid data\n".to_vec());
                                    }
                                };
                            },
                            _ => {
                                let mut client_lock = clients.lock().await;
                                if let Some(client) = client_lock.get_mut(&session_id) {
                                    let _ = client.sender.try_send(b"Cant pars command\n".to_vec());
                                }
                                eprintln!("Cant pars command");
                            }
                        }
                        
                    }
                }
                if pending.len() > MAX_FRAME_SIZE {
                    remove_client(&clients, session_id).await;
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

async fn remove_client(clients: &Clients, session_id: SessionId) {
    let mut clients_lock = clients.lock().await;
    if let Some(client) = clients_lock.remove(&session_id) {
        let _ = client.shutdown.send(());
    };
}

fn spawn_write_task(mut receiver: Receiver<Vec<u8>>, mut write_half: OwnedWriteHalf, session_id: SessionId) {
    tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            if let Err(error) = write_half.write_all(&message).await {
                eprintln!("Failed to write to client {session_id}: {error}");
                break;
            }
        }
    });
}

async fn broadcast(message_bytes: Vec<u8>, clients: &Clients, session_id: SessionId) {
    let (senders, sender_login) = {
        let clients_lock: tokio::sync::MutexGuard<'_, HashMap<u64, ClientHandle>> = clients.lock().await;

        let senders = clients_lock
            .iter()
            .filter_map(|(id, client_handle)| {
                if client_handle.login.is_some() && *id != session_id {
                    Some((
                        *id,
                        client_handle.sender.clone()
                    ))
                } else {
                    None
                }
            })
            .collect::<Vec<(SessionId, mpsc::Sender<Vec<u8>>)>>();

        let sender_login = clients_lock
            .get(&session_id)
            .and_then(|client| client.login.clone())
            .unwrap_or("Unknown".to_string());

        (senders, sender_login)
    };
    
    let mut message = format!("[{sender_login}]: ").into_bytes();
    message.extend_from_slice(&message_bytes);
    message.push(b'\n');
    for (id, sender) in senders {
        match sender.try_send(message.clone()) {
            Ok(_) => {println!("message sent")}
            Err(TrySendError::Full(_)) => {
                println!("Client {id} message full");
                remove_client(&clients, id).await;
            }
            Err(TrySendError::Closed(_)) => {
                println!("Client {id} disconnected");
                remove_client(&clients, id).await;
            }
        };
    }
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
