# Rust TCP Chat

Async TCP chat built with **Rust** and **Tokio**.

Learning project focused on backend networking: framing, a custom binary protocol, concurrency, and in-memory rooms.

---

## Features

- Concurrent clients (one Tokio task per connection)
- Length-prefixed TCP framing with a max frame size
- Custom binary protocol shared between server and client
- Login / password authentication
- Chat rooms: create, join, list, send messages to a room
- Slow-client protection (bounded write queue; full queue → disconnect)

---

## Requirements

- [Rust](https://rustup.rs/) (edition 2024 toolchain)

---

## Quick start

Clone the repo and create a `.env` in the workspace root (or copy from a package example):

```bash
cp server/.env.example .env
```

Default address:

```env
CONNECT_ADDR_LOCAL=127.0.0.1:1313
```

Start the server:

```bash
cargo run -p server
```

In another terminal, start a client:

```bash
cargo run -p client
```

Open more terminals with `cargo run -p client` to chat with several users.

### Demo accounts

Credentials are hardcoded for local experiments:

| Login   | Password |
|---------|----------|
| `Oliver`  | `123123` |
| `Emma` | `123123` |

---

## Client commands

Messages are sent as slash-commands:

```text
/login <login> <password>
/rooms
/create_room <name>
/join <name>
/room <name> <text>
```

Example session:

```text
/login Oliver 123123
/create_room rust
/join rust
/room rust hello from Oliver
```

---

## Project layout

```text
tcp-chat/
├── server/     # TCP chat server
├── client/     # CLI client
├── shared/     # Framing + protocol types
└── roadmap.md  # Development plan (learning path)
```

| Crate    | Role |
|----------|------|
| `server` | Accept connections, auth, rooms, fan-out |
| `client` | Interactive CLI over the same protocol |
| `shared` | Frame encode/decode and message types |

---

## Protocol (overview)

Wire format is **binary**, not line-based text:

1. **Frame:** `u32` big-endian length + payload (max payload ~8 KiB)
2. **Payload:** message type byte + length-prefixed fields

Client → server includes auth, room management, and `SendToRoom`.  
Server → client includes auth results, room events, and room messages.

See `shared/src/protocol.rs` and `shared/src/framing.rs` for the exact layout.

---

## Stack

- Rust / Tokio
- `mpsc` channels for per-client writes
- `Arc` + `Mutex` / `RwLock` for shared in-memory state
- Workspace crates: `server`, `client`, `shared`
