use std::io::{self, Read, Write, stdin};
use std::net::TcpStream;
use std::thread;
use std::env;

fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();

    let addr = env::var("CONNECT_ADDR").expect("Connection address must be set");
    let mut client = TcpStream::connect(addr)?;
    let mut read_client = client.try_clone()?;

    thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            match read_client.read(&mut buffer) {
                Ok(0) => {
                    println!("Server closed connection");
                    break;
                }
                Ok(n) => {
                    let msg = String::from_utf8_lossy(&buffer[..n]);
                    println!("\n[MSG] {}", msg);
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }
    });

    let mut user_message = String::new();
    loop {
        println!("Type text");
        user_message.clear();
        stdin().read_line(&mut user_message)?;
        client.write_all(user_message.as_bytes())?;
    }
}
