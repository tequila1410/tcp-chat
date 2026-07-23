use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot, mpsc};

pub type SessionId = u64;

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
struct Client {
    login: Option<String>,
    sender: mpsc::Sender<Arc<Vec<u8>>>,
    shutdown: oneshot::Sender<()>,
}
type Clients = Arc<Mutex<HashMap<SessionId, Client>>>;

#[derive(Clone)]
pub struct ClientRegistry {
    clients: Clients,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub async fn insert_client(&self, sender: mpsc::Sender<Arc<Vec<u8>>>, shutdown: oneshot::Sender<()>) -> SessionId {
        let session_id: SessionId = NEXT_CLIENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        {
            let mut locked_clients = self.clients.lock().await;
            locked_clients.insert(session_id, Client { login: None, sender, shutdown });
        }
        session_id
    }

    pub async fn broadcast(&self, message_bytes: Vec<u8>, session_id: SessionId) -> Option<()> {
        let (senders, sender_login) = {
            let clients_lock = self.clients.lock().await;
    
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
                .collect::<Vec<(SessionId, mpsc::Sender<Arc<Vec<u8>>>)>>();
    
            let sender_login = clients_lock
                .get(&session_id)
                .and_then(|client| client.login.clone());
    
            (senders, sender_login)
        };
        
        if let Some(login) = sender_login {
            let mut message = format!("[{login}]: ").into_bytes();
            message.extend_from_slice(&message_bytes);
            message.push(b'\n');
            let message_arc = Arc::new(message);
            for (id, sender) in senders {
                match sender.try_send(message_arc.clone()) {
                    Ok(_) => println!("message sent"),
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        println!("Client {id} message full");
                        self.remove_client(id).await;
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        println!("Client {id} disconnected");
                        self.remove_client(id).await;
                    }
                };
            };
            return Some(());
        }
        None
    }

    pub async fn remove_client(&self, session_id: SessionId) {
        let mut clients_lock = self.clients.lock().await;
        if let Some(client) = clients_lock.remove(&session_id) {
            let _ = client.shutdown.send(());
        };
    }

    pub async fn send_message(&self, session_id: SessionId, message_bytes: Vec<u8>) {
        let mut client_lock = self.clients.lock().await;
        if let Some(client) = client_lock.get_mut(&session_id) {
            let _ = client.sender.try_send(Arc::new(message_bytes));
        }
    }

    pub async fn authorize_client(&self, session_id: SessionId, login: String) {
        let mut client_lock = self.clients.lock().await;
        if let Some(client) = client_lock.get_mut(&session_id) {
            client.login = Some(login);
        }
    }
}
