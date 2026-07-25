use std::io::{self, Read, Write, stdin};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::env;

enum Command {
    Auth(String),
    Message(String),
    AuthResponse(String),
    UnParsed(String),
}

fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();

    let is_loged_in = Arc::new(Mutex::new(false));

    let addr = env::var("CONNECT_ADDR_LOCAL").expect("Connection address must be set");
    let client = TcpStream::connect(addr)?;
    let mut read_client = client.try_clone()?;

    let is_loged_in_c = Arc::clone(&is_loged_in);
    thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        let mut pending = Vec::new();
        loop {
            let bytes_read = match read_client.read(&mut buffer) {
                Ok(0) => {
                    println!("Server closed connection");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            };

            pending.extend_from_slice(&buffer[..bytes_read]);
            while let Some(pos) = pending.iter().position(|&byte| byte == b'\n') {
                let message = pending.drain(..=pos).collect::<Vec<u8>>();
                match message_parse(&message) {
                    Some(message) => {
                        match message {
                            Command::AuthResponse(message) => {
                                let mut is_loged_in_c = is_loged_in_c.lock().unwrap();
                                *is_loged_in_c = true;
                                println!("{message}");
                            },
                            Command::UnParsed(message) => {
                                println!("{message}");
                            }
                            _ => {}
                        }
                    }
                    None => {
                        println!("Can't parse command");
                    }
                };
            }
        }
    });

    let mut user_message = String::new();
    loop {
        println!("Type text");
        user_message.clear();
        stdin().read_line(&mut user_message)?;
        send_message(&client, &user_message, is_loged_in.clone())?;
    }
}

fn send_message(mut client: &TcpStream, user_message: &str, is_loged_in: Arc<Mutex<bool>>) -> io::Result<()> {
    // let message: Vec<u8>;
    // let is_loged_in = is_loged_in.lock().unwrap();
    // if *is_loged_in {
    //     message = format!("MESSAGE {user_message}").into_bytes();
    // } else {
    //     message = format!("AUTH {user_message}").into_bytes();
    // }
    let message = user_message.as_bytes();
    client.write_all(&message)?;
    Ok(())
}

fn message_parse(message: &Vec<u8>) -> Option<Command> {
    if let Ok(message_str) = std::str::from_utf8(message) {
        return match message_str.split_once(' ') {
            Some(("AUTH_RESP", value)) => Some(Command::AuthResponse(value.to_string())),
            Some(("MESSAGE", value)) => Some(Command::Message(value.to_string())),
            _ => Some(Command::UnParsed(message_str.to_string())),
        };
    }
    None
}
