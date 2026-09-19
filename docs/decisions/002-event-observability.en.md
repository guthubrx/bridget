# ADR 002 — Event observability inspired by a third-party system

[🇫🇷 Français](002-persistence-inspiree-systeme-tiers.md) · **🇬🇧 English**

## Context

After reviewing a third-party system, we identified observability patterns (JSONL
events) that could improve Bridget's debuggability and traceability.

## Patterns observed in that system

### 1. JSONL events for traceability

```python
.runs/
└── [run_id]/
    ├── run.json           # Run state
    ├── events.jsonl       # Event log
    └── result.json        # Final result
```

Every event (creation, answer, cancellation, escalation) is appended to
`events.jsonl` for complete post-mortem traceability.

### 2. Why that system uses the filesystem

- it has NO persistent database
- the JSONL file is ITS source of truth for history
- atomic writes compensated for the absence of transactions

## Bridget's current architecture

Bridget uses **SQLite** as its source of truth:

```
Daemon + SQLite (implemented in session 003)
├── messages (routing history)
├── tracked_requests (tracked requests)
└── agents (presence)
```

**SQLite already provides:**
- ✅ Atomic writes (ACID transactions + fsync)
- ✅ Persistence across restarts
- ✅ No possible corruption
- ✅ Full state replay

## Critical analysis

**Atomic JSON writing is REDUNDANT with SQLite:**
- SQLite already guarantees atomicity and durability
- Duplicating state (JSON + SQLite) means needless complexity
- Keeping two sources in sync invites bugs

**What is genuinely useful:**
- ➕ Observability (events) for debugging
- 📝 Traceability of state transitions
- 🔍 Post-mortem search

## Proposal adapted to Bridget's architecture

### Option: an events table in SQLite

```sql
CREATE TABLE request_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_id TEXT NOT NULL,
    event_type TEXT NOT NULL,  -- created, answered, cancelled, timed_out, escalated
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    details TEXT,               -- optional JSON
    FOREIGN KEY (request_id) REFERENCES tracked_requests(id)
);
```

**Benefits:**
- ✅ A single source of truth (SQLite)
- ✅ Transactions guarantee consistency
- ✅ SQL queries for analysis
- ✅ JSONL export still possible if needed
- ✅ Simple, coherent architecture

### Alternative option: JSONL export on demand

```rust
fn export_request_events(conn: &Connection, request_id: &str) -> Result<Vec<Event>> {
    // Read from SQLite
    // Produce JSONL for external tooling
}
```

## Decision

**REJECT** the JSON + SQLite duplication (architecturally inconsistent).

**ACCEPT** observability through a `request_events` table in SQLite, because it:
- ➕ guarantees consistency (a single source of truth)
- ➕ benefits from SQLite transactions
- ➕ still allows JSONL export if required
- 🎯 stays simple and maintainable

**Priority**: P2 (debugging improvement, not functionally critical)

---

**Status**: REJECTED (original version) — ACCEPTED (revised version)
**Date**: 2026-08-15
**Impact**: one events table added to SQLite, no major architectural change
