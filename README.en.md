# Bridget — inter-agent communication

Bridget connects agents from different providers. One daemon owns identities,
messages, tracked requests and delivery facts. CLI, MCP and journal observation
are views of that same authority.

**This extraction focuses on communication.** No GUI, HTTP server, Docker/project
runtime or Maicie implementation is required. Maicie may remain an external
consumer of the public protocol. Native Codex/Claude drivers and ACP provide the
primary session path; tmux is not required.

## Branch status

Session 089 has a validated standalone package: 1,199 automated tests passed,
50 real crash cycles were replayed, fmt/clippy passed and independent review
findings were fixed and checked. A real native Codex exchange and journal attachment
have passed. The real Claude gate remains open on local authentication; GLM has
not been validated. Cross-server SSH, reconnect and the isolated 600-event,
60-second load gates have passed. Copy-only migrations and targeted security
gates have been exercised. Unverified provider accounts remain outside this verdict.
Nothing has been deployed to the existing fleet.

See the [evidence log](specs/089-communication-core/implementation.md),
[task list](specs/089-communication-core/tasks.md) and
[contract](specs/089-communication-core/contracts/communication.md).

## Send, reply, inspect

From an agent already registered in the current Bridget namespace:

These examples assume the **extracted binary**, not the `bridget` installed by
the old product. Follow the independent installation guide first; no global
alias, service or PATH is replaced automatically.

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

An address is a UUID. `bridget rename "Team B"` or `bridget_rename` changes only
the owning session's display name; retries, instance and history stay bound to
the same identity. Neither a provider name nor `--from` grants another identity.

### Communication guarantees and upgrade 099

A CLI or MCP client acting for an agent must present the private credential
issued to its wrapper. Knowing an agent UUID, instance or replay scope is not
authorization. The credential rotates when the owner reconnects and old auxiliary
connections lose their authority. It stays in the wrapper's private state,
including when the daemon is reached through SSH, never in the public directory.

Upgrade the daemon, wrappers and clients together, then reconnect the wrappers
in the **verified namespace**. Older auxiliaries without a credential receive
an explicit refusal; there is no fallback to declarative registration. This does
not isolate hostile processes controlling the same operating-system account and
able to read its private files. Do not share that account with an untrusted party.

Classic delivery waits at most one second for its output lock and write, without
holding the daemon's global state lock. A failed write does not receive a success
acknowledgment. Partial delivery may remain indeterminate: inspect request state
before retrying. A delivery acknowledgment proves neither provider completion
nor answer quality.

The 094 contract documents a closed list of twelve Bridget tools:
`bridget_who`, `bridget_send`, `bridget_cancel`, `bridget_ledger`, both artifact
tools and the six additions `bridget_rename`, `bridget_dnd`, `bridget_domain`,
`bridget_runtime`, `bridget_status`, `bridget_control_status`. They use the same
authority; this list is neither global MCP approval nor automatic permission for
Maicie tools. Reply with `to`, `body` and `in_reply_to`. The packaged
[skill](skills/bridget/SKILL.md) provides examples and retry guidance, while its
[094 command inventory](skills/bridget/references/commandes.md) classifies every
CLI root. These files do not prove which binary is installed or which catalogue
an already-running MCP session holds, and global profiles are never modified
automatically.

## Administer an SSH federation

The 096 binary embeds the federation manager, so ordinary human use no longer
depends on a repository or script path.

```sh
bridget federate ssh://cartae.app -p 2222
bridget federate status
bridget federate remove ssh://cartae.app -p 2222 --label cartae-core
```

The first command reuses an attested installation without mutation when the DNS
name resolves to its recorded IP. DNS never replaces the stored SSH target or
the explicit `known_hosts` file. `status` is a local inventory and performs no
SSH operation. Removing a link found only through a DNS alias requires `--label`
outside a double TTY; a double TTY may instead confirm the displayed label,
recorded host and port. Missing values for a new destination are prompted only
when both stdin and stdout are terminals; otherwise the command fails with the
required flags. See [the federation service guide](docs/federation-services.md).

## Reaching t3code threads (session 098)

The 098 binary embeds a bridge to [t3code](https://github.com/pingdotgg/t3code)
that never modifies t3code: it reads the local server over loopback with a
session issued by the official `t3` CLI and presents every open thread as a
Bridget agent (`TYPE` = the thread provider, `TRANSPORT` t3code, `MODE` cli,
display name = thread title).

```sh
bridget t3 install        # dedicated t3 session (t3 auth session issue) + bridge service
bridget t3 status         # t3code server, session, service, exposed threads
bridget send --to Alpha --reply -- 'Mission…'   # starts a turn in thread “Alpha”
bridget t3 uninstall      # revokes the session, removes the service, wipes bridge state
```

Requirements: t3code running (the app or `t3 --mode web --no-browser`) and the
`t3` CLI installed (`npm i -g t3`). A delivered message waits for the thread to
be idle (two-minute bound), starts a turn with the message text, and the reply
of that turn goes back to the sender as a linked reply: pairing follows the FIFO
order of turns after the last turn closed at delivery time. The held session is
administrative (t3code 0.0.40 issues no other kind) and lives only in a 0600
file inside Bridget state; a 401 triggers one renewal, a second one is an
explicit failure shown by `status`. The thread journal (`bridget attach`) never
replays history older than the installation. An archived thread leaves the
directory, never t3code. See [ADR 034](docs/decisions/034-adaptateur-t3code.md).

The bridge processes cancellation while waiting for an idle thread: a request
cancelled before dispatch does not start a later turn. Expiration and automatic
reminders do not open an extra provider turn. Cancellation after t3code
accepts a turn does not guarantee provider interruption. A prepared response is
retained until confirmation, including recipient disconnects and bridge restarts;
recovery does not rerun the provider task. The journal retains long text within
its bounds or explicitly reports a gap, never silently cutting at 4,096 characters.

## Native interactive Claude Code, without tmux (session 097)

From a real terminal (iTerm, Terminal, remote shell), in the intended directory:

```sh
bridget claude
bridget claude --resume
bridget claude --name reviewer --model claude-opus-5
```

The wrapper owns a pseudo-terminal: Claude Code starts inside it with its
native interface (colours, resizing, permissions), keystrokes and output are
relayed verbatim, and Bridget messages are pasted into the conversation like
human input (bracketed paste, then Enter). `who` shows `claude | claude_pty |
cli`, never `tmux` without tmux. Without a terminal the launch is refused and
`bridget spawn claude` is suggested. No permission bypass is added; an explicit
bypass passed by the user is forwarded unchanged. The session journal relays
human and assistant turns read from the Claude transcript, so `bridget attach
<UUID>` works as for interactive Codex. Other interactive aliases (`gemini`,
custom agents) still rely on tmux and are refused at launch without a pane.

Managed Claude (`bridget spawn claude`) inherits the human's account session:
the official CLI needs the real HOME and `USER`; no API key is read. The real
recipe is recorded in
[specs/097-claude-sans-tmux/implementation.md](specs/097-claude-sans-tmux/implementation.md).

## Native interactive Codex, without tmux (session 090)

From a real terminal, in the intended working directory:

```sh
bridget codex
bridget codex -m gpt-5.6-terra
bridget codex --name coderBridget --yolo resume <Codex-thread-UUID>
bridget codex --name horizon-original --yolo resume horizon-original
bridget codex --name horizon-original --yolo resume
```

This branch opens the **official Codex TUI** against a private local app-server.
Human input and Bridget messages share one thread. The UUID printed at startup
is the session address. Codex replies using `bridget_send` with `in_reply_to`;
its final on-screen answer is **not automatically forwarded** to the sender.

Ordinary messages queue behind an active human turn. Native permissions stay
under human control; managed-session permissive defaults are never inherited.
Explicit model/profile/configuration, sandbox and approval options are forwarded.

Verified contract: Codex **0.153.4**, experimental Unix remote connection and
`legacy` history. One Bridget identity is bound to the initial thread. Internal
subagents belong to Codex: creating/resuming them neither closes the session nor
registers another Bridget identity. Other-thread notifications do not change the
message destination. The integration does not implicitly follow `/new` or
`/resume` navigation: relaunch Bridget to address another conversation. Other Bridget agents
remain independent and reachable. Quit and relaunch to change directory.
This first version rejects `--cd`, images and local providers;
unsupported options are rejected rather than silently ignored.

`resume <UUID>` or `resume <Codex-name>` selects the initial thread, preserving
its history and title. Exact names must be unique: missing or ambiguous names
are rejected without opening a session. Bare `resume` opens a Bridget selection
menu using Codex's `thread/list`: enter a number, `n`/`p` for pages, `q` or Ctrl-C
to cancel. This is not Codex's native picker: the official TUI starts after
selection, once Bridget registration and journaling are ready. No temporary
thread is created. The menu lists non-archived interactive threads across
directories, most recently updated first. Listing is bounded to 1,000 threads
and 10 seconds; incomplete results are rejected, never used for a guess.
The name after `resume` is the **Codex conversation name**, not the Bridget name.
`--name` resolves the Bridget identity bearing that name (80 characters maximum),
or creates one for a new name. No Bridget UUID to remember: inactive names can
be reused, while active agents are protected against a second launch.
With `resume <Codex-UUID>`, the existing conversation is resumed; without it,
a new conversation starts under the same identity. The thread→identity binding
is stored locally, so resuming a thread previously launched by this version
also works without `--name`. For older unbound threads, supply the name once.
Conflicting name/thread identities are rejected without mutation. `--agent-id`
remains an advanced option, not a prerequisite for human resume.
The agent can later use `bridget rename <name>` without changing its UUID.
At exit, copy the final `Reprendre : bridget codex …` command: Codex's generic
`--remote …` hint above it refers to a temporary socket that is now closed.
The initial prompt is never replayed.
`--yolo` aliases `--dangerously-bypass-approvals-and-sandbox`: it explicitly
disables Codex sandbox and approvals and is never added by default.

| Command | Purpose and lifetime |
|---|---|
| `bridget codex` | Foreground native interaction; quitting closes this session. |
| `bridget spawn codex --persistent …` | Supervised agent independent of the terminal. |
| `bridget attach <uuid>` | Journal replay/follow and message input, not a native TUI resume. |

To assign development to a new Codex, the human runs in their terminal:
`bridget spawn codex --persistent --cwd "$PWD" --posture development`.
This profile permits writes in that directory, not shell-command network access
or automatic permission escalation. Declared MCP tools remain a separate access
path. It applies only to this order and requires TTY input/output.
Without `--posture`, the global policy applies; `--posture discovery` selects
read-only access. `relaunch` preserves the old agent's frozen permissions: it
cannot turn a discovery agent into a developer. No global policy change is needed.

A Bridget daemon reconnect preserves the thread and identity. The configured
Bridget socket may be SSH-forwarded; the Codex app-server remains local and
private. No new HTTP server or browser access. Building this branch does not
replace or install the running binary automatically.

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

`status`, `who` and `agents` reject an unavailable inventory instead of printing
an empty list. Their probe is bounded; `status` never opens a client-side database.
The protocol does not publish an exhaustive message count, so this total remains
explicitly unavailable. A bounded ledger view must not be presented as that total.

`attach` replays then follows the journal with continuous sequences or an explicit
`Gap`. An unavailable source, end of stream and completed catch-up are distinct.
Connection or freshness does not prove task activity or completion.

**Talk from attach:** type a message and press Enter to send it; Ctrl-C closes
the view without stopping the agent. This is neither a provider TUI resume nor
a permission approval screen. The terminal status line shows client/type, model,
effort and state from the same inventory as `who`. Without refreshed data it becomes
unavailable. The actual provider is never inferred from “Claude” or “Codex”.
Native Codex commands and approval requests are rendered as sanitized facts.

The terminal view renders response Markdown: headings, lists, emphasis, quotes
and visually distinct code below a separate compact header. The display name is
cosmetic: communication addresses remain UUIDs. Reasoning and ordinary successful
turn endings are hidden in this view only; commands, permissions, refusals,
errors and gaps remain visible. Non-TTY output retains technical diagnostics,
and the source journal is unchanged. Code blocks are styled, without
language-specific lexical highlighting or active terminal hyperlinks.

Resizing reflows the current response or the last response still managed on
screen, plus the input and status, without another keystroke. Text already
committed to terminal scrollback belongs to the terminal: Bridget does not
rewrite the complete history. The next turn releases the retained response.

The input has a full-width grey background that follows its height, with the
colored status underneath. With interactive TTY input and output, `Enter` (CR)
sends while the distinct LF produced by `Shift+Enter` or Ctrl-J inserts a newline
at the cursor. `Option+Enter` and `Escape`, then `Enter` are ignored. Left/Right
moves by one complete UTF-8 character and Option+Left/Right by a word (a
non-whitespace run). Ctrl-A/E moves to the logical line start/end; Ctrl-U/K
deletes up to those boundaries without removing the LF. Ctrl-W or Option+Backspace
deletes the previous word, Option-D the next one, and Ctrl-Y reinserts the latest
fragment deleted by those commands. Ordinary Backspace removes only the previous
character and does not replace that register. Outside double TTY, CR and LF keep
the legacy send behavior and these new editing controls are inert. Up/Down recalls
successfully emitted inputs with the cursor at the end and restores both the
current draft and its cursor position after the newest entry. Enhanced keyboard
mode is temporary; disabling it under `TERM=dumb` does not disable double-TTY
CR/LF distinction. This history is
volatile and limited to the current attach opening, 100 entries and 1 MiB;
reopening starts empty. Resizing preserves the complete input. `NO_COLOR` or
`TERM=dumb` disables colors; `TERM=dumb` also keeps legacy keyboard input without
enhanced-mode activation, and redirected output remains plain.

For **managed Codex**, `/model gpt-5.6-terra medium` in attach selects the model
and effort for subsequent turns of the **same thread**. No restart, hidden prompt
or lost history. Both values must be advertised by the native model catalogue;
rejections leave settings unchanged, and an unknown outcome is not confirmation.
This requires updated daemon/wrapper and Codex `thread/settings/update` (tested:
0.153.4). It does not modify the frozen launch definition, which is reloaded on
a Bridget relaunch. For **interactive** `bridget codex`, use the native TUI's
`/model` instead. Other drivers explicitly refuse this attach control.

### Skill, MCP or CLI?

The skill is the operating guide; MCP executes structured communication tools.
Agents prefer MCP for messages, linked replies, inventory and request cancellation.
Discover deferred tools before falling back to the shell. A sandboxed shell socket
refusal is not evidence of an MCP failure. Managed wrappers may relay final answers
automatically; human interactive sessions require an explicit linked reply. Do not
send the same answer through both paths.

In 094, an agent may change **its own state** through MCP, without a target field:
rename its display (never its UUID), enable DND for 1 second to 7 days (`60m` by
default) or disable it, change/reset its domain, and declare its runtime. Only
`off` disables DND; an already-expired deadline is rejected without mutation
or an invented extension. A valid linked reply remains deliverable during DND.
The domain follows the same technical ASCII canon as the CLI; reset restores the
actually derived domain, including after a restart. Domain persistence is confirmed
separately from its in-memory application: `domain_persistence_failed` rules out
claiming durability or rollback. `bridget_runtime` declares model/effort with
source `Declared`; unlike `/model` in attach or the provider TUI, it selects
nothing at the provider.

`bridget_status` exposes sanitized health; unavailable inventory yields an
unknown count, not zero. `bridget_control_status` reads control state, the open
decision count and, when requested, 0 to 50 events. A store failure remains an
unavailability. A client's technical scope is not by itself daemon authentication.

Spawn, stop and relaunch remain explicitly authorized CLI operations, not MCP
supervision tools. Check the receipt and effective profile: connected does not mean
writable. Remaining Maicie tools target an external service, not a communication
dependency. Artifacts stay inert and do not open a graphical interface.

Fleet operations, `control`/`inbox` decisions, hooks, migration, `reprise`,
`reaper`, daemon and terminal remain human-only or internal, with concrete reasons
in the [complete inventory](skills/bridget/references/commandes.md). A live older
MCP server retains its binary and catalogue: use a native client reload mechanism
only when known, otherwise have the human reopen the session. Domain guarantees
also require the cooperating client and wrapper versions to be actually loaded.
Never invent a reload command or interrupt a conversation merely to refresh tools.

Referenced content retains bytes, provenance and access controls. HTML is inert
data: no rendering, JavaScript or browser is part of the core.

## Build independently

Implementation directory:
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/089-communication-core`.
Rust is pinned in rust-toolchain.toml.

```sh
cd /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/089-communication-core
PATH=/Users/moi/.cargo/bin:$PATH cargo build --locked -p bridget-daemon
bridget_state=$(mktemp -d /tmp/bgcore.XXXXXX)
export BRIDGET_HOME="$bridget_state"
export BRIDGET_SOCKET="$BRIDGET_HOME/bridget.sock"
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/089-communication-core/target/debug/bridget daemon
```

Use a NEW private, short, absolute state directory. The daemon stays in the
foreground. Other terminals must use the same variables and extracted binary.
Provider HOME is preserved during normal operation for subscription access;
there is no fallback to a paid API. No existing daemon, launchd service, tunnel
or provider profile is replaced. The
[independent installation and rollback guide](docs/communication-installation.md)
documents the allowlisted source package, checksums and account requirements.
This does not authorize a fleet migration.

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
