use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:1313")?;

    let mut data = Vec::new();

    // frame + хвост без \n
    data.extend_from_slice(b"hello\n");

    println!("Sending {} bytes", data.len());

    stream.write_all(&data)?;

    let mut buffer = [0u8; 1024];

    loop {
        let n = stream.read(&mut buffer)?;

        if n == 0 {
            println!("Server closed connection");
            break;
        }

        println!(
            "Received {} bytes: {:?}",
            n,
            &buffer[..n]
        );
    }

    Ok(())
}