use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use shared::{framing::encode_frame, protocol::ServerMessage};
use tokio::sync::{Mutex, oneshot, mpsc};

pub type SessionId = u64;

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
struct Client {
    login: Option<String>,
    outbound_tx: mpsc::Sender<Arc<Vec<u8>>>,
    evict_tx: oneshot::Sender<()>,
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

    pub async fn get_login(&self, session_id: SessionId) -> Option<String> {
        let clients_lock = self.clients.lock().await;
        clients_lock.get(&session_id).and_then(|client| client.login.clone())
    }

    pub async fn insert_client(
        &self,
        outbound_tx: mpsc::Sender<Arc<Vec<u8>>>,
        evict_tx: oneshot::Sender<()>,
    ) -> SessionId {
        let session_id: SessionId = NEXT_CLIENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        {
            let mut clients_lock = self.clients.lock().await;
            clients_lock.insert(session_id, Client { login: None, outbound_tx, evict_tx });
        }
        session_id
    }

    pub async fn send_many(&self, message_bytes: Vec<u8>, message_to: HashSet<SessionId>) {
        let outbound_txs = {
            let clients_lock = self.clients.lock().await;

            clients_lock
                .iter()
                .filter_map(|(id, client)| {
                    if message_to.contains(id) {
                        Some((*id, client.outbound_tx.clone()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<(SessionId, mpsc::Sender<Arc<Vec<u8>>>)>>()
        };
        let message = Arc::new(message_bytes);
        for (id, outbound_tx) in outbound_txs {
            match outbound_tx.try_send(message.clone()) {
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
    }

    pub async fn remove_client(&self, session_id: SessionId) {
        let mut clients_lock = self.clients.lock().await;
        if let Some(client) = clients_lock.remove(&session_id) {
            let _ = client.evict_tx.send(());
        };
    }

    pub async fn send_message(&self, session_id: SessionId, message_bytes: Vec<u8>) {
        let mut clients_lock = self.clients.lock().await;
        if let Some(client) = clients_lock.get_mut(&session_id) {
            let _ = client.outbound_tx.try_send(Arc::new(message_bytes));
        }
    }

    pub async fn reply(&self, session_id: SessionId, msg: ServerMessage) {
        let payload = msg.serialize();
        let message = encode_frame(&payload);
        self.send_message(session_id, message).await;
    }

    pub async fn authorize_client(&self, session_id: SessionId, login: String) {
        let mut client_lock = self.clients.lock().await;
        if let Some(client) = client_lock.get_mut(&session_id) {
            client.login = Some(login);
        }
    }

    pub async fn is_client_authorized(&self, session_id: SessionId) -> bool {
        let client_lock = self.clients.lock().await;
        if let Some(client) = client_lock.get(&session_id) {
            return client.login.is_some();
        }
        return false;
    }

}
