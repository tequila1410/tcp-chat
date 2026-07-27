use std::io;
use std::env;

use tokio::io::{AsyncWriteExt, AsyncBufReadExt, AsyncReadExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream};

use shared::framing::{FrameResult, decode_frame, encode_frame};
use shared::protocol::{ClientMessage, ServerMessage};

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    let addr = env::var("CONNECT_ADDR_LOCAL").expect("Connection address must be set");
    let stream = TcpStream::connect(addr).await?;
    let (read_half, mut write_half) = stream.into_split();

    spawn_read_task(read_half);

    let stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await? {
        handle_message(&mut write_half, &line).await;
    }
    Ok(())
}

fn spawn_read_task(mut read_half: OwnedReadHalf) {
    tokio::spawn( async move {
        let mut buffer = [0u8; 1024];
        let mut pending = Vec::new();

        loop {
            let bytes_read = match read_half.read(&mut buffer).await {
                Ok(0) => {
                    println!("Server closed connection");
                    return Ok(());
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    return Err(e);
                }
            };
            pending.extend_from_slice(&buffer[..bytes_read]);

            loop {
                match decode_frame(&mut pending) {
                    FrameResult::Complete(frame) => {
                        if let Some(message) = ServerMessage::deserialize(&frame) {
                            match message {
                                ServerMessage::Message { from, text } => {
                                    println!("[{from}]: {text}");
                                }
                                ServerMessage::AuthErr(error) => {
                                    println!("Auth error: {error}");
                                }
                                ServerMessage::AuthOk => {
                                    println!("Authenticated success!");
                                }
                                ServerMessage::Err(error) => {
                                    println!("{error}");
                                }
                            }
                        }
                    }
                    FrameResult::TooLarge => {
                        println!("TooLarge");
                        break;
                    }
                    FrameResult::Incomplete => {
                        break;
                    }
                }
            }
        }
    });
}

async fn handle_message(write_half: &mut OwnedWriteHalf, user_message: &str) {
    if user_message.starts_with('/') {
        handle_command(write_half, user_message).await;
    } else {
        let message = ClientMessage::Message(String::from(user_message));
        let message_bytes = encode_frame(&message.serialize());
        send_message(write_half, &message_bytes).await;
    }
}

async fn handle_command(client: &mut OwnedWriteHalf, user_message: &str) {
    let (command, args) = match user_message.split_once(' ') {
        Some((cmd, args)) => (cmd, args),
        None => (user_message, "")
    };
    match command {
        "/login" => {
            if let Some((login, password)) = args.split_once(' ') {
                let message = ClientMessage::Auth{login: login.to_string(), password: password.to_string()}.serialize();
                let message_bytes = encode_frame(&message);
                send_message(client, &message_bytes).await;
            }
        }
        _ => {}
    }
}

async fn send_message(client: &mut OwnedWriteHalf, user_message: &[u8]) {
    if let Err(err) = client.write_all(user_message).await {
        eprintln!("Can't send message to server: {err}");
    };
}
