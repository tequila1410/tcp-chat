use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

type Clients = Arc<Mutex<Vec<(SocketAddr, TcpStream)>>>;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("Failed to accept connection: {error}");
                continue;
            }
        };
        let clients_clone = Arc::clone(&clients);
        let peer_addr = match stream.peer_addr() {
            Ok(addr) => addr,
            Err(error) => {
                eprintln!("Failed to get peer address: {error}");
                continue;
            }
        };
        {
            let mut clients_lock = clients_clone.lock().unwrap();
            match stream.try_clone() {
                Ok(val) => clients_lock.push((peer_addr, val)),
                Err(error) => {
                    eprintln!("Failed to clone stream for client {peer_addr}: {error}");
                    continue;
                }
            }
        }
        thread::spawn(move || {
            println!("Client connected: {peer_addr}");
            if let Err(error) = handle_client(stream, peer_addr, clients_clone) {
                eprintln!("Client error {peer_addr}: {error}");
            }
        });
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, peer_addr: SocketAddr, clients: Clients) -> io::Result<()> {
    let mut buffer = [0u8; 1024];
    let mut pending = Vec::new();

    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected: {peer_addr}");
                remove_client(&clients, peer_addr);
                return Ok(());
            }
            Ok(n) => n,
            Err(error) if is_client_disconnect_error(&error) => {
                println!("Client reset connection: {peer_addr}");
                remove_client(&clients, peer_addr);
                return Ok(());
            }
            Err(error) => return Err(error),
        };

        pending.extend_from_slice(&buffer[..bytes_read]);
        while let Some(pos) = pending.iter().position(|&byte| byte == b'\n') {
            let message = pending.drain(..=pos).collect::<Vec<u8>>();
            broadcast_message(&clients, peer_addr, message.as_slice())?;
        }
    }
}

fn is_client_disconnect_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe
    )
}

fn remove_client(clients: &Clients, peer_addr: SocketAddr) {
    let mut clients_lock = clients.lock().unwrap();
    clients_lock.retain(|(addr, _)| *addr != peer_addr);
}

fn broadcast_message(clients: &Clients, sender_addr: SocketAddr, message: &[u8]) -> io::Result<()> {
    let mut disconnected_clients = Vec::new();
    {
        let mut clients = clients.lock().unwrap();
        
        for (client_addr, client_stream) in clients.iter_mut() {
            if *client_addr == sender_addr {
                continue;
            }
            match client_stream.write_all(message) {
                Ok(()) => {},
                Err(error) if is_client_disconnect_error(&error) => {
                    disconnected_clients.push(*client_addr);
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
    }

    for client_addr in disconnected_clients {
        remove_client(&clients, client_addr);
    }

    Ok(())
}