# Roadmap

Plan for evolving the TCP chat after the technical audit (≈3 weeks in).

Goal: deepen Rust backend / async networking skills — not ship product features for their own sake.

Principle: **fix invariants and lifecycle before adding features.**

---

## Current baseline

Already in place:

- Tokio TCP server with per-connection tasks
- Length-prefixed framing + binary protocol (`shared`)
- Auth (in-memory credentials)
- Rooms (create / join / list / leave / send-to-room)
- Single-room membership policy (see § Membership rules)
- Unified `disconnect` path (leave rooms + remove session)
- Client runtime split: `Sessions` / `Identity` / `Outbound` (shared store) + `ConnectionDeps`
- Per-client write channel + slow-client eviction (outbound delivery policy)
- Application layer `app/` (auth / chat / rooms) returning outcomes; `transport/respond` maps to wire
- Use-case unit tests without TCP
- CLI client with slash commands
- Workspace: `server` / `client` / `shared`

Known gaps: Phase 2 remainder (2.3–2.5).

---

## Phase 1 — Foundation (do before new features)

Stabilize correctness of connections, rooms, and delivery.

| # | Item | Why | Done when |
|---|------|-----|-----------|
| 1.1 | ✅ Single disconnect path for all exit reasons (EOF, read error, frame too large, write failure, slow client) | Ghost sessions / room members | One function owns cleanup; write-task death triggers it |
| 1.2 | ✅ Room membership invariants (no duplicate joins; explicit leave; single-room + switch) | Broken fanout and leave | Documented rules + storage/protocol match them |
| 1.3 | ✅ Align unicast and broadcast delivery (`reply` / `send_message` vs `send_many` on Full/Closed) | Auth/room acks can be silently dropped | Same eviction/error policy on both paths |
| 1.4 | ✅ Sync project map: update README to real protocol/modules; remove legacy root `src/` (`src/bin/*`) | Mental model drift | README matches code; dead bins don't confuse |
| 1.5 | ✅ Unit tests for `RoomState` / auth decisions (no TCP) | Regressions on every change | create/join/recipients/leave/leave_all/duplicates/switch covered |

**Exit criteria:** no ghost members after disconnect; join rules are explicit; docs match reality; storage tests pass.

### Membership rules (1.2)

Contract for `MemoryRoomStorage` / room handlers. Storage keeps two indexes in sync: `room → members` and `session → current room`.

1. A session is in **0 or 1** room (not many at once).
2. **Join** another existing room → atomic **switch** (leave current, then join).
3. **Join** the same room again → `AlreadyMember`; membership unchanged.
4. **Create** → create room + **auto-join** (with switch if already elsewhere).
5. **Create** when the name exists → `AlreadyExist`; membership unchanged.
6. **Leave** (no room name) → not in any room. Domain outcome: `Left` | `WasNotMember` (not an error). Wire: both map to `RoomLeft` (idempotent for the client).
7. **Disconnect** → `leave_all` then remove session (defensive cleanup).
8. **Send** to a room only if the session is a member of that room.

### Delivery policy (1.3)

Outbound path for both unicast (`reply` → `send_message`) and broadcast (`send_many`):

1. Resolve `outbound_tx` under the clients lock, then **release** the lock before `try_send` (avoids deadlock with `remove_client`).
2. One helper (`deliver`) owns `try_send` + error handling.
3. `Full` or `Closed` → `remove_client` → `evict_tx` → unified `disconnect` (1.1). No silent drop of acks.
4. Payload shared via `Arc<Vec<u8>>` (clone the `Arc`, not the bytes).

---

## Phase 2 — Architecture & quality

Make the system easier to extend and reason about.

| # | Item | Why | Done when |
|---|------|-----|-----------|
| 2.1 | ✅ Split `ClientRegistry` responsibilities (session / identity / outbound) | God object blocks safe changes | Clear module boundaries; handlers don't need the whole world |
| 2.2 | ✅ Testable application layer (handlers return outcomes; transport maps to `ServerMessage`) | Business logic testable without sockets | Use-case unit tests exist |
| 2.3 | Harden protocol (type constants, decode errors instead of bare `Option`, optional version byte) | Fragile wire format | Decode failures are typed and visible |
| 2.4 | Structured logging with `tracing` + `session_id` | Concurrent debugging | Can follow one connection through lifecycle |
| 2.5 | Clarify `RoomManager` (real policies vs thin passthrough) | Fake layer worse than none | Manager owns policy **or** is removed/simplified honestly |

**Exit criteria:** adding a command doesn't require touching five unrelated concerns; core logic has tests; logs are useful under load.

### Client runtime split (2.1)

Replaced god-object `ClientRegistry` with one shared in-memory store and three narrow APIs:

1. **`Sessions`** — insert/remove connection; owns eviction signal.
2. **`Identity`** — login / auth checks for a `SessionId`.
3. **`Outbound`** — `reply` / `send_many` + delivery policy (Full/Closed → private remove helper).
4. **`ConnectionDeps`** — composition root for a TCP connection (`sessions` + `identity` + `outbound` + rooms + credentials); handlers receive only the slices they need.

### Testable application layer (2.2)

Use-cases return domain/application **outcomes**; transport maps them to wire + delivery. No sockets in use-case unit tests.

1. **`app/`** — application layer: `auth`, `chat`, `rooms` (each folder: `mod.rs` + `#[cfg(test)] mod test`).
2. **Outcomes** — `AuthOutcome`, `ChatOutcome`, `RoomOutcome` (decisions + routing intent, not `encode_frame`).
3. **`transport/respond`** — `apply_*_outcome` maps outcome → `ServerMessage` / `reply` / `send_many`.
4. **`connection`** — decode → use-case → apply; side effects like `authorize_client` stay in the use-case (auth), not in respond.
5. **Tests** — cover success / not-authenticated / error paths without TCP (in-memory `new_state` + `RoomManager`).

---

## Phase 3 — Features (networking-first)

Add capabilities that teach backend/networking — only after Phase 1 (ideally after 2.1–2.3).

Suggested order:

| # | Feature | Learning focus | Complexity |
|---|---------|----------------|------------|
| 3.1 | Client “current room” UX (leave already in 1.2) | State on client, less typing for `/room` | Low |
| 3.2 | Presence (join/leave notifications) | Fanout, event design | Medium |
| 3.3 | Idle timeout / max connections | Resource limits, `tokio::time`, DoS basics | Medium |
| 3.4 | Private messages | Routing, authorization | Medium |
| 3.5 | Graceful server shutdown | Cancellation, task tracking | Medium |
| 3.6 | Rate limiting | Backpressure beyond `try_send` | Medium |

Later (when in-memory truly limits learning):

- Persisted users/rooms (e.g. SQLite)
- Password hashing / less toy auth
- TLS
- Multi-instance fanout (Redis, etc.) — **intentionally late**

---

## Explicitly deferred

Do **not** prioritize yet:

- Rewrite from scratch
- Microservices / multi-process split
- Redis / Postgres “because production”
- HTTP/WebSocket gateway before core TCP lifecycle is solid
- Premature performance work (sharding locks, etc.) unless measuring a real bottleneck

---

## Near-term focus (next 5 steps)

1. ~~Single `disconnect` path (1.1).~~
2. ~~Membership rules + leave on the wire (1.2).~~
3. ~~Align delivery error handling (1.3: unicast = broadcast policy).~~
4. ~~README sync + remove legacy root `src/` (1.4).~~
5. ~~Unit tests for room storage / auth (1.5).~~
6. ~~Phase 2.1 — split `ClientRegistry` (Sessions / Identity / Outbound + ConnectionDeps).~~
7. ~~Phase 2.2 — testable application layer (outcomes + `respond` + use-case tests; `app/`).~~
8. Phase 2.3 — harden protocol (typed decode errors; optional version byte).

Phase 1 complete; 2.1–2.2 complete. Prefer 2.3 before Phase 3 features.

---

## How to use this file

- Check off Phase 1 before starting Phase 3 features.
- Prefer one vertical slice at a time (rule → code → test → README note).
- When choosing between a flashy feature and an invariant fix, choose the invariant.

Last updated: 2026-08-05 (2.2: app outcomes + transport/respond; use-case unit tests)
