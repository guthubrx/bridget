# Bridget — inter-agent communication

Bridget connects agents from different providers. One daemon owns identities,
messages, tracked requests and delivery facts. CLI, MCP and journal observation
are views of that same authority.

**This extraction focuses on communication.** No GUI, HTTP server, Docker/project
runtime or Maicie implementation is required. Maicie may remain an external
consumer of the public protocol. Native Codex/Claude drivers and ACP provide the
primary session path; tmux is not required.

## Branch status

Session 089 is being validated. Physical extraction and local guarantees are
implemented progressively. A real native Codex exchange and journal attachment
have passed. The real Claude gate remains open on local authentication; GLM has
not been validated. Cross-server SSH, reconnect and the isolated 600-event,
60-second load gates have passed. Migration, security and full
regression gates remain mandatory. Nothing has been deployed to the existing fleet.

See the [evidence log](specs/089-communication-core/implementation.md),
[task list](specs/089-communication-core/tasks.md) and
[contract](specs/089-communication-core/contracts/communication.md).

## Send, reply, inspect

From an agent already registered in the current Bridget namespace:

```sh
bridget who
bridget agents --json
bridget send --to '<agent_id_uuid>' --reply --timeout 120 -- 'Check this point and reply with your findings.'
bridget send --to '<sender_uuid>' --in-reply-to '<full_message_id>' -- 'Verified result: …'
bridget ledger --limit 20
bridget attach '<agent_id_uuid>'
```

`agents --json` (or MCP who) exposes addressable UUIDs; CLI `who` shows human names.
Replace placeholders with identifiers from the directory and received message.
`--reply` opens a tracked request; `in_reply_to` binds a response to THAT request.
Never shorten its ID. `bridget reply` targets the last remembered sender; explicit
target and correlation are safer when several requests coexist.

An address is a UUID. `bridget rename "Team B"` changes only its display name from
the owning session. Neither a provider name nor `--from` grants another identity.

MCP tools `bridget_who`, `bridget_send`, `bridget_ledger` use the same socket.
Reply with `to`, `body` and `in_reply_to`. The packaged
[skill](skills/bridget/SKILL.md) provides examples and retry guidance; global
profiles are never modified automatically.

## Interpret outcomes literally

| Status | Meaning |
|---|---|
| `accepted` | Transport delivery acknowledged, not intellectual work completed. |
| `in_flight` | Delivery is attested in progress; do not create a new send key. |
| `outcome_unknown` | Outcome unknown; neither success nor deterministic refusal. |
| `orphaned` | Orphaned delivery; reinjection requires an explicit decision. |
| `envelope_mismatch` | Reused key with different arguments; original record unchanged. |
| `idempotency_expired` | Retry protection expired, not permission to resend silently. |

A retry keeps the same `id`, `issued_at`, instance and arguments, including body,
target, timeout, `reply` and `in_reply_to`. Prepare the key/time pair before the
first call if the workflow must survive losing its first receipt. A new key
cannot promise deduplication after an ambiguous result.

Ordinary CLI sends without a key retain legacy behavior. An idempotent ordinary
CLI send requires `--id`, `--issued-at`, `--issuer-scope` together. Linked CLI
replies can derive scope from their instance and replay their original ID/time.
Exit code zero alone is not proof of a durable acknowledgement.

## Observe facts

`who` shows reported presence/model facts; missing signals remain unknown.
`ledger` reads the master; connection loss never creates an empty client database.
MCP request scope defaults to `mine` (incoming and outgoing); `all` reads the
authorized global scope.

`attach` replays then follows the journal with continuous sequences or an explicit
`Gap`. An unavailable source, end of stream and completed catch-up are distinct.
Connection or freshness does not prove task activity or completion.

Referenced content retains bytes, provenance and access controls. HTML is inert
data: no rendering, JavaScript or browser is part of the core.

## Build independently

Implementation directory:
`/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core`.
Rust is pinned in rust-toolchain.toml.

```sh
cd /Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core
PATH=/Users/moi/.cargo/bin:$PATH cargo build --locked -p bridget-daemon
bridget_state=$(mktemp -d /tmp/bgcore.XXXXXX)
export BRIDGET_HOME="$bridget_state"
export BRIDGET_SOCKET="$BRIDGET_HOME/bridget.sock"
/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/target/debug/bridget daemon
```

Use a NEW private, short, absolute state directory. The daemon stays in the
foreground. Other terminals must use the same variables and extracted binary.
Provider HOME is preserved during normal operation for subscription access;
there is no fallback to a paid API. No existing daemon, launchd service, tunnel
or provider profile is replaced. Full adoption instructions remain a final gate.

## Cross-server communication

One master daemon, Unix socket forwarding through SSH, the same public protocol,
UUIDs, canonical bytes and ledger. No public HTTP channel or second message store.
Transfer scripts require explicit paths, SSH identity, known host verification
and a free target. Tunnel loss is not a reason to restart a provider or invent a
delivery acknowledgement. The real remote recipe and measurements remain required.

## Quality and scope

Three crates: bridget-core (domain), bridget-transport (protocol/drivers/journal),
bridget-daemon (authority, transactions, CLI/MCP). Shared canonicalization and
request-closing helpers preserve atomic delivery-related facts.

Tests use private HOME/BRIDGET_HOME/TMPDIR, watchdogs and owned-child cleanup.
Do not run unaudited historical harnesses against real HOME. Ignored provider or
SSH tests do not count as passed gates. The
[test map](specs/089-communication-core/test-map.md) records their dispositions.

Bridget does not prove work correctness or completion. Task orchestration, T3
integration, A2A servers and catalogue bookkeeping remain outside this extraction.
