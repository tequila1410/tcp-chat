# Rust TCP Chat

A simple TCP chat server written in Rust using the standard library.

This project is part of my Rust backend learning journey.
The goal is to understand networking, ownership, concurrency, and later migrate the project to Tokio.

---

## Features

- Multiple clients
- Thread per connection
- Message broadcasting
- TCP message framing
- Graceful client disconnect
- Shared client list using Arc<Mutex<_>>

---

## Project Structure

```
src/
 ├── main.rs
 ├── server.rs
 ├── client.rs
 └── ...
```

---

## Getting Started

Run the server

```bash
cargo run --bin server
```

Run a client

```bash
cargo run --bin client
```

Open multiple terminals to connect several clients.

---

## Technologies

- Rust
- std::net
- std::thread
- Arc
- Mutex

---

## Roadmap

- [x] TCP server
- [x] Multiple clients
- [x] Broadcast messages
- [x] Message framing
- [ ] Replace threads with Tokio
- [ ] User authentication
- [ ] Redis integration
- [ ] Private messages
- [ ] Rooms
- [ ] Logging

---

## Learning Goals

This project is built for educational purposes.

Current focus:

- Ownership
- Lifetimes
- TCP networking
- Concurrency
- Message framing
- Shared state