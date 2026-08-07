use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use shared::{framing::encode_frame, protocol::ServerMessage};
use tokio::sync::{Mutex, oneshot, mpsc};
use tracing::warn;

pub type SessionId = u64;

static NEXT_CLIENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
struct Client {
    login: Option<String>,
    outbound_tx: mpsc::Sender<Arc<Vec<u8>>>,
    evict_tx: oneshot::Sender<()>,
}

type Store = Arc<Mutex<HashMap<SessionId, Client>>>;

#[derive(Debug, PartialEq, Eq)]
pub enum SessionsError {
    TooManyConnections,
}

#[derive(Clone)]
pub struct Sessions { store: Store }

impl Sessions {
    pub async fn try_insert_client(
        &self,
        outbound_tx: mpsc::Sender<Arc<Vec<u8>>>,
        evict_tx: oneshot::Sender<()>,
        max_clients: usize,
    ) -> Result<SessionId, SessionsError> {
        let mut clients_lock = self.store.lock().await;
        if clients_lock.len() >= max_clients {
            return Err(SessionsError::TooManyConnections);
        }
        let session_id: SessionId = NEXT_CLIENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        clients_lock.insert(session_id, Client { login: None, outbound_tx, evict_tx });
        
        Ok(session_id)
    }

    pub async fn remove_client(&self, session_id: SessionId) {
        remove_client(&self.store, session_id).await;
    }
}

#[derive(Clone)]
pub struct Identity { store: Store }

impl Identity {
    pub async fn get_login(&self, session_id: SessionId) -> Option<String> {
        let clients_lock = self.store.lock().await;
        clients_lock.get(&session_id).and_then(|client| client.login.clone())
    }

    pub async fn authorize_client(&self, session_id: SessionId, login: String) {
        let mut client_lock = self.store.lock().await;
        if let Some(client) = client_lock.get_mut(&session_id) {
            client.login = Some(login);
        }
    }

    pub async fn is_client_authorized(&self, session_id: SessionId) -> bool {
        let client_lock = self.store.lock().await;
        if let Some(client) = client_lock.get(&session_id) {
            return client.login.is_some();
        }
        return false;
    }
}

#[derive(Clone)]
pub struct Outbound { store: Store }

impl Outbound {
    pub async fn send_many(&self, message_bytes: Vec<u8>, message_to: HashSet<SessionId>) {
        let outbound_txs = {
            let clients_lock = self.store.lock().await;

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
        for (session_id, outbound_tx) in outbound_txs {
            self.deliver(session_id, outbound_tx, message.clone()).await;
        };
    }

    async fn send_message(&self, session_id: SessionId, message_bytes: Vec<u8>) {
        let outbound_tx = {
            let clients_lock = self.store.lock().await;
            clients_lock.get(&session_id).map(|client| client.outbound_tx.clone())
        };
        if let Some(outbound_tx) = outbound_tx {
            let message = Arc::new(message_bytes);
            self.deliver(session_id, outbound_tx, message).await;
        }
    }

    async fn deliver(&self, session_id: SessionId, outbound_tx: mpsc::Sender<Arc<Vec<u8>>>, message_bytes: Arc<Vec<u8>>) {
        match outbound_tx.try_send(message_bytes) {
            Ok(_) => {},
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!(session_id, "outbound queue full, evicting");
                remove_client(&self.store, session_id).await;
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                warn!(session_id, "outbound channel closed, evicting");
                remove_client(&self.store, session_id).await;
            }
        }
    }    

    pub async fn reply(&self, session_id: SessionId, msg: ServerMessage) {
        let payload = msg.serialize();
        let message = encode_frame(&payload);
        self.send_message(session_id, message).await;
    }

}

async fn remove_client(store: &Store, session_id: SessionId) {
    let mut clients_lock = store.lock().await;
    if let Some(client) = clients_lock.remove(&session_id) {
        let _ = client.evict_tx.send(());
    };
}

pub fn new_state() -> (Sessions, Identity, Outbound) {
    let store = Arc::new(Mutex::new(HashMap::new()));
    let sessions = Sessions { store: store.clone() };
    let identity = Identity { store: store.clone() };
    let outbound = Outbound { store: store.clone() };
    (sessions, identity, outbound)
}

#[cfg(test)]
#[path = "test.rs"]
mod tests;
