# Rust TCP Chat

Async TCP chat built with **Rust** and **Tokio**.

Learning project focused on backend networking: framing, a custom binary protocol, concurrency, and in-memory rooms.

---

## Features

- Concurrent clients (one Tokio task per connection)
- Length-prefixed TCP framing with a max frame size
- Custom binary protocol shared between server and client
- Login / password authentication
- Chat rooms: create, join, leave, list, send messages to a room
- Membership: a session is in at most one room; join/create switches room
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
/leave
/room <name> <text>
```

You can be in **at most one room**. `/join` or `/create_room` switches you into that room (leaves the previous one). `/leave` exits the current room; repeating `/leave` is OK. `/create_room` also joins you as a member.

Example session:

```text
/login Oliver 123123
/create_room rust
/room rust hello from Oliver
/leave
/join general
```

---

## Project layout

Cargo **workspace** (root `Cargo.toml` has members only — no root package):

```text
tcp-chat/
├── server/     # TCP chat server (`cargo run -p server`)
├── client/     # CLI client (`cargo run -p client`)
├── shared/     # Framing + protocol types (library)
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

Client → server: `Auth`, `CreateRoom`, `JoinRoom`, `LeaveRoom`, `GetRooms`, `SendToRoom`.  
Server → client: `AuthOk` / `AuthErr`, room acks (`RoomCreated` / `RoomJoined` / `RoomLeft` / `RoomErr` / `RoomsGet`), `Message`, `Err`.

Membership rules: `roadmap.md` § Membership rules (1.2).  
Delivery (full/closed write queue → disconnect): `roadmap.md` § Delivery policy (1.3).  
Wire layout: `shared/src/protocol.rs` and `shared/src/framing.rs`.

---

## Stack

- Rust / Tokio
- `mpsc` channels for per-client writes
- `Arc` + `Mutex` / `RwLock` for shared in-memory state
- Workspace crates: `server`, `client`, `shared`
