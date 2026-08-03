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
- Rooms (create / join / list / send-to-room)
- `ClientRegistry` + per-client write channel + slow-client eviction
- CLI client with slash commands
- Workspace: `server` / `client` / `shared`

Known gaps (see audit): session lifecycle holes, membership invariants, god-object registry, weak tests/docs.

---

## Phase 1 — Foundation (do before new features)

Stabilize correctness of connections, rooms, and delivery.

| # | Item | Why | Done when |
|---|------|-----|-----------|
| 1.1 | ✅ Single disconnect path for all exit reasons (EOF, read error, frame too large, write failure, slow client) | Ghost sessions / room members | One function owns cleanup; write-task death triggers it |
| 1.2 | Room membership invariants (no duplicate joins; explicit leave semantics; decide multi-room policy) | Broken fanout and leave | Documented rules + storage behavior matches them |
| 1.3 | Align unicast and broadcast delivery (`reply` / `send_message` vs `send_many` on Full/Closed) | Auth/room acks can be silently dropped | Same eviction/error policy on both paths |
| 1.4 | Sync project map: update README to real protocol/modules; archive or remove legacy `src/bin/*` | Mental model drift | README matches code; dead bins don't confuse |
| 1.5 | Unit tests for `MemoryRoomStorage` + auth decisions (no TCP) | Regressions on every change | create/join/recipients/leave_all/duplicates covered |

**Exit criteria:** no ghost members after disconnect; join rules are explicit; docs match reality; storage tests pass.

---

## Phase 2 — Architecture & quality

Make the system easier to extend and reason about.

| # | Item | Why | Done when |
|---|------|-----|-----------|
| 2.1 | Split `ClientRegistry` responsibilities (session / identity / outbound) | God object blocks safe changes | Clear module boundaries; handlers don't need the whole world |
| 2.2 | Testable application layer (handlers return outcomes; transport maps to `ServerMessage`) | Business logic testable without sockets | Use-case unit tests exist |
| 2.3 | Harden protocol (type constants, decode errors instead of bare `Option`, optional version byte) | Fragile wire format | Decode failures are typed and visible |
| 2.4 | Structured logging with `tracing` + `session_id` | Concurrent debugging | Can follow one connection through lifecycle |
| 2.5 | Clarify `RoomManager` (real policies vs thin passthrough) | Fake layer worse than none | Manager owns policy **or** is removed/simplified honestly |

**Exit criteria:** adding a command doesn't require touching five unrelated concerns; core logic has tests; logs are useful under load.

---

## Phase 3 — Features (networking-first)

Add capabilities that teach backend/networking — only after Phase 1 (ideally after 2.1–2.3).

Suggested order:

| # | Feature | Learning focus | Complexity |
|---|---------|----------------|------------|
| 3.1 | Leave room + client “current room” UX | State transitions, protocol evolution | Low |
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

1. Draw session lifecycle (all enter/exit paths) → implement one `disconnect`.
2. Fix room membership rules + tests.
3. Align delivery error handling (unicast = broadcast policy).
4. Update README; clean legacy bins from the active mental model.
5. Ship one small feature on top (leave room or presence).

---

## How to use this file

- Check off Phase 1 before starting Phase 3 features.
- Prefer one vertical slice at a time (rule → code → test → README note).
- When choosing between a flashy feature and an invariant fix, choose the invariant.

Last updated: 2026-08-03
