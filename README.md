# Rust TCP Chat

A TCP chat server built with Rust and Tokio async runtime.

This project is part of my Rust backend learning journey.
The goal is to understand async networking, ownership, concurrency, and protocol design.

---

## Features

- Multiple concurrent clients via Tokio async tasks
- Line-based text protocol (`AUTH`, `MESSAGE`)
- Message broadcasting to all authenticated clients
- TCP message framing with max frame size protection
- Graceful client disconnect and forced eviction (slow clients)
- Shared client registry using `Arc<Mutex<_>>`
- Workspace structure: `server`, `client`, `shared`

---

## Project Structure

```
test-tcp/
 ├── Cargo.toml          # workspace root
 ├── server/
 │    ├── Cargo.toml
 │    └── src/
 │         ├── main.rs   # connection handling, protocol parsing
 │         └── client.rs # ClientRegistry, Client
 ├── client/
 │    ├── Cargo.toml
 │    └── src/
 │         └── main.rs
 └── shared/
      ├── Cargo.toml
      └── src/
           └── lib.rs    # shared protocol types (WIP)
```

---

## Protocol

**Client → Server:**
```
AUTH login:password\n
MESSAGE text\n
```

**Server → Client:**
```
MESSAGE [login]: text\n
ERROR reason\n
```

---

## Getting Started

Run the server:

```bash
cargo run -p server
```

Run a client:

```bash
cargo run -p client
```

Open multiple terminals to connect several clients.

---

## Technologies

- Rust
- Tokio (async runtime)
- `Arc<Mutex<_>>` for shared state
- `mpsc` channels for per-client write tasks
- `oneshot` channels for shutdown signaling

---

## Roadmap

- [x] TCP server
- [x] Multiple clients
- [x] Broadcast messages
- [x] Message framing
- [x] Tokio async runtime
- [x] User authentication
- [x] Workspace structure
- [x] ClientRegistry abstraction
- [ ] Shared protocol crate
- [ ] Client binary
- [ ] Structured logging (tracing)
- [ ] Private messages
- [ ] Rooms

---

## Learning Goals

- Ownership and borrowing in async context
- Tokio tasks, channels, select!
- TCP networking and message framing
- Shared mutable state across async tasks
- Workspace and crate organization
- Protocol design
