# Communication reference

[English](reference-communication.md) · [Français](reference-communication.fr.md) · [Documentation](README.md)

The daemon is the authority for identities, messages, tracked requests and
delivery facts. CLI, MCP and journal views do not own separate message stores.
This guide describes public behavior. Dated validation records in the
[historical documentation](README.md#historical-and-contributor-records) do not
prove which code your current processes have loaded.

## Identity and discovery

Start terminal agents with `bridget codex` or `bridget claude`, or use a host
integration that attests the session. Knowing an agent UUID, instance or replay
scope does not grant authority. Private credentials are issued to the owning
wrapper and rotate on reconnection; old auxiliary connections lose authority.

```sh
bridget who
bridget agents --json
bridget who --global
bridget agents --json --global
```

Discovery defaults to the same project. Membership uses attested host/project
facts and the common Git root, not a mutable display domain. A repository and
its linked worktrees can share that identity; two repositories with the same
name do not.

`bridget_who` uses `scope:"same_project"` by default; `scope:"global"` requests
an intentional global view. `communication_project` and `project_relation`
distinguish `same`, `other` and `unknown`. Unknown projects do not become
local suggestions. A standalone CLI client can provide
`--project-root /absolute/path/to/project`; a rootless client may have an empty
local view and a warning. An unreachable inventory is an error, not an empty list.

`bridget rename "Reviewer"` changes the owning agent's display name only.
The UUID, instance, messages and retry identity remain. Names in received text
are display data, never authority.

[Adapter contract details](adapters/t3.md#note-8).

## Send and reply

From a registered agent:

```sh
bridget send --to '<agent_uuid>' --reply --timeout 120 -- 'Review this contract.'
bridget send --to '<sender_uuid>' --in-reply-to '<full_message_id>' -- 'Verified findings: ...'
bridget ledger --limit 20
```

Use full returned identifiers in stored workflows. `--reply` creates a tracked
request. `in_reply_to` correlates a response with that specific request.
`bridget reply` targets the last remembered sender; explicit target and
correlation are clearer when requests overlap.

Some managed wrappers forward a final answer automatically. Interactive terminal
sessions require an explicit linked reply. Follow the current wrapper's contract
and avoid sending the same answer through both routes.

Prefer the official MCP tools inside an agent host. A sandboxed shell failing
to open a socket does not prove MCP is unavailable. Do not change identity,
host or transport to bypass a refused operation.

## Cross-project communication

New messages and thread operations accessible to another known project need
`cross_project_reason`. It is a trimmed, nonempty string of 1–512 UTF-8 bytes,
without control characters. Null, blank or oversized values are refused.

```sh
bridget send --to '<agent_uuid>' \
  --cross-project-reason 'Compare the shared interface used by both projects' \
  -- 'Please check this interface discrepancy.'
```

For MCP, place the reason beside `to` or `action`, not inside the body.
The result can contain `project_warnings`; the sent body is unchanged.
The capability `communication_projects_v1` is required. A client does not
silently remove the reason for an older server.

All thread readers count as the audience. `notify:[]` means no notification,
not confidentiality. Unknown legacy projects can retain warning-based
compatibility, but do not erase a known mismatch between two other members.

A genuine reply to an accepted OPEN tracked request can reuse its durable
reason when the participants are reversed. A forged or unrelated
`in_reply_to` grants nothing. A retry preserves the frozen source context and
reason even if the surrounding project changes.

## Receipts and retries

| Outcome | Interpretation |
| --- | --- |
| `accepted` | Delivery acknowledged by the transport; not work completion. |
| `in_flight` | Delivery is in progress; keep the same request identity. |
| `outcome_unknown` | Neither success nor deterministic refusal is established. |
| `orphaned` | Reinjection needs an explicit decision. |
| `envelope_mismatch` | The key was reused with different arguments. |
| `idempotency_expired` | Retry protection expired; this does not authorize a fresh send. |

An idempotent retry preserves `id`, `issued_at`, instance and the full envelope:
target, body, reply flag, timeout, correlation, reason and source context.
Prepare the key/time pair before the first call if its first receipt may be lost.
Ordinary legacy CLI sends without a key do not acquire deduplication guarantees.

An idempotent ordinary CLI send requires `--id`, `--issued-at` and
`--issuer-scope` together. Linked CLI replies can derive scope from their
instance and reuse their original ID/time. A zero process exit alone is not
proof of a durable delivery acknowledgment.

A failed or partial write must not become a success acknowledgment. Delivery,
read acknowledgment, model completion and answer quality are different facts.
Neither retry protection nor durable records guarantee exactly-once external
business effects.

## Shared threads and handoffs

Shared threads preserve contributions and their authorship. Use `kind:history`
with `notify:[]` for evidence. Use short `action`, `blocker` or `decision`
entries when an agent must act. `supersedes_seq` explicitly replaces an
instruction by the same author for the same recipients without deleting history.

`read` can return references for historical/replaced entries. `history` reads
their exact bodies without changing the cursor. All authorized members can
read silent entries. Existing access rules still apply.

Handoff briefs and artifacts are inert data, not executable instructions or
automatic authority. Do not put secrets into a brief or rely on a cited path
as proof of access. The [delegation guide](delegation-native.md) documents
structured briefs and the child lifecycle.

## Queue, steer and interrupt

`bridget_send.intent` and CLI `--intent` distinguish:

| Intent | Behavior |
| --- | --- |
| Omitted or `queue` | Ordinary queued communication. |
| `steer` | Ask to guide the active turn; effect is not guaranteed. |
| `interrupt` | Ask for interruption while retaining the message in FIFO order. |

Unknown intents, including `auto`, are refused. `steer` with `reply:true`
is invalid. A receipt for guidance reports `effect:"unconfirmed"`.
Changing intent under the same key is an envelope mismatch.

Only an attested human Attach actor or an authorized parent can explicitly
steer/interrupt. A peer, bare client, self-targeting recipient or internal the desktop adapter
child cannot acquire that right by writing `from=human`.
An attested UI retains its Client role; UI provenance alone is not human
control authority.

[Adapter contract details](adapters/t3.md#note-9).

During pause, agent guidance is refused with `referent_paused`; ordinary
queueing remains available. Agent interruption of a child with an active
mission can return `mission_active`. Agent rate limits are one interruption
per pair in 30 seconds and three per target in 600 seconds. Refusals do not
consume that allowance. Human and internal pause controls have distinct rules.

[Adapter contract details](adapters/t3.md#note-10).

### Read a queue

```sh
bridget queue --target '<target_uuid>'
```

The view requires `QueueListV1` and returns at most 200 entries, with
`message_id`, `from`, `intent`, `reply`, `age_secs`, a 120-character
preview, `position` and state. Authorized parents, recipients, human Attach
and attested UI clients can see the full permitted queue; other authors see
their own messages. Internal the desktop adapter child proofs are refused.

[Adapter contract details](adapters/t3.md#note-11).

The view covers held/dispatching idempotent deliveries, not every internal
provider queue. Reading it neither starts a turn nor establishes provider order.

### Held queue and editing

`held_queue` is disabled by default per agent type. Its historical latency
measurement did not qualify enabling it globally. The catalog's setting is
the source for availability, not this document.

When enabled for an instance with `turn_attest`, ordinary idempotent client
messages can be held while busy or behind older held messages. Human/system
origins, guidance, thread alerts, observations, native missions and the desktop adapter threads
follow their own paths.

[Adapter contract details](adapters/t3.md#note-12).

`held` means durably stored before dispatch. `released` means dispatch began;
it does not prove injection. `queue_full` is a recorded admission refusal.
Closed or expired requests are removed through the existing notice paths.

```sh
bridget queue cancel --target '<target_uuid>' --message '<message_uuid>' --request '<cancel_key>'
bridget queue reorder --target '<target_uuid>' --message '<message_uuid>' --request '<order_key>' --before '<other_message_uuid>'
bridget queue promote --target '<target_uuid>' --message '<message_uuid>' --request '<promotion_key>'
```

Omitting CLI `--before` moves to the end. MCP reorder requires
`before_message_id`, with `null` meaning the end.

- Cancel: attested human, parent or author; removes a held message without interruption.
- Reorder: attested human or parent; changes held order transactionally.
- Promote: requires steering authority and an active turn; effect remains unconfirmed.

Mutations require `QueueEditV1`, target, message ID and a nonempty request key.
Receipts persist for seven days. Fresh identity/rights checks precede reading
them. Same parameters return the saved result; changed parameters produce
`envelope_mismatch`. A replay does not re-emit the effect.

Promotion requires `queue_fallback`. An unavailable steer can fall back to FIFO
after commit. A full queue or ambiguous provider response can leave delivery
uncertain, including possible duplicates. `applied` proves the recorded edit,
not model consumption. Body editing is outside this contract.

## Observe without inventing work

`who` reports presence/model facts; missing facts remain unknown.
`ledger` reads the master. A bounded ledger is not a total message count.
`status` does not open a substitute client-side database.

`attach` replays and follows a journal with sequences or an explicit gap.
A fresh connection does not prove active work. Type in attach to send a
message; Ctrl-C closes the view without stopping the agent. It is not a native
provider resume or an approval screen.

TTY output renders Markdown and sanitized commands, errors and permission facts.
Reasoning and routine successful turn endings can be hidden in that view
without changing the source journal. Non-TTY output retains plain diagnostics.
Resize reflows the current retained response, not all terminal scrollback.

Interactive editing supports Enter to send, distinct LF/Shift+Enter/Ctrl-J to
insert a newline, character/word movement, line deletion and bounded input
history. History is local to that attach opening (100 entries, 1 MiB).
`NO_COLOR` and `TERM=dumb` disable color; redirected output stays plain.

Managed Codex attach can use `/model MODEL EFFORT` for subsequent turns on
the same thread when the native catalog and `thread/settings/update` support
it. It does not alter the frozen launch definition. Interactive Codex uses
its native TUI's model selector. Unknown outcome is not confirmation.

### Share a journal or subscribe to facts

```sh
bridget journal '<source_uuid>' --tail 50 --to '<reviewer_uuid>' --reply
bridget events types
bridget events sub turn_ended --agent '<source_uuid>' --once --ttl 1800
bridget events sub file_collision --file '/absolute/project/src/*' --ttl 3600
bridget events list
bridget events unsub '<subscription_id>'
```

Journal excerpts default to 50 entries, at most 200 and 64 KiB, with explicit
gaps/truncation and pagination. Sharing uses the same communication guards.

Events include `turn_ended`, `permission_required`, `file_written` and
`file_collision`. They report facts observed after subscription, not mission
validation. They do not satisfy an outstanding reply request.

File events require supported structured provider writes. Arbitrary shell
writes, reads and unsupported adapters are not covered. Collision detection
compares different agents on the same host and normalized path within 30 seconds;
it neither locks files nor merges Git branches. Aliases and case differences
can limit detection.

Subscriptions default to one hour, at most seven days, 16 per agent and 128
overall. They survive daemon restart, with an outage warning. Facts never
observed during the outage cannot be reconstructed. `once` consumes its trigger
even if delivery fails. Counters expose throttling, suppressed events and gaps.
Observations do not require replies or create mandatory new work.

The missed-fact and wake interfaces are described in
[Native delegation](delegation-native.md#wakes-and-missed-facts).

## Own-state tools and service boundaries

Agents can rename themselves, set DND, change/reset a display domain and declare
runtime facts through their own-state tools. DND ranges from one second to seven
days, with a 60-minute default; only `off` disables it. Valid linked replies can
still arrive during DND. A declared runtime does not select a provider model.

A failed domain persistence write is not a durable success or a rollback.
Health projections keep unavailable values unknown, not zero. Control status
is a read, not permission to perform human-only mutations.

[Adapter contract details](adapters/t3.md#note-13).

Private credentials protect cooperative clients, not hostile processes with
control over the same operating-system account. Keep that account private.
Referenced HTML, paths and links remain inert in the communication core.

### Bounded MCP directory

bridget_who returns at most24KiB per page, with20agents by default (limit1to100).
Stopped presences are hidden unless include_stopped=true. Copy next_cursor into
cursor with the same scope, domain and include_stopped; limit may change.
Stop at has_more=false. The directory is live (live=true), without a snapshot
guarantee during presence changes. No presence or history is deleted.
Authenticated project scope is applied before pagination. An oversized single
entry is explicitly refused rather than silently truncated.

For the first bridget_send, omit both id and issued_at; Bridget returns them.
For replay, copy both values and the same content. An id alone is invalid even
on the first send; no message is sent in that case.
