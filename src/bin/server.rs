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
            clients_lock.push((peer_addr, stream.try_clone().unwrap()));
        }
        thread::spawn(move || {
            if let Err(error) = handle_client(stream, peer_addr, clients_clone) {
                eprintln!("Client error {peer_addr}: {error}");
            }
        });
    }
    Ok(())
}

fn handle_client(mut stream: TcpStream, peer_addr: SocketAddr, clients: Clients) -> io::Result<()> {
    let mut buffer = [0u8; 1024];

    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected: {peer_addr}");
                {
                    let mut clients = clients.lock().unwrap();
                    clients.retain(|(addr, _)| *addr != peer_addr);
                }
                return Ok(());
            }
            Ok(n) => n,
            Err(error) if is_client_disconnect_error(&error) => {
                println!("Client reset connection: {peer_addr}");
                {
                    let mut clients = clients.lock().unwrap();
                    clients.retain(|(addr, _)| *addr != peer_addr);
                }
                return Ok(());
            }
            Err(error) => return Err(error),
        };

        let mut clients = clients.lock().unwrap();
        
        println!("{:#?}", clients);
        for (client_addr, client_stream) in clients.iter_mut() {
            if *client_addr == peer_addr {
                continue;
            }
            match client_stream.write_all(&buffer[..bytes_read]) {
                Ok(()) => {},
                Err(error) if is_client_disconnect_error(&error) => {
                    println!("Client reset connection: {peer_addr}");
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        
    }
}

fn is_client_disconnect_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionReset | io::ErrorKind::BrokenPipe
    )
}

fn remove_client() {
    
}