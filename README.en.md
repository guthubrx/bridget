# Bridget

[🇫🇷 Français](README.md) · **🇬🇧 English**

A peer-to-peer, transport-agnostic communication protocol for CLI agents.

## Overview

Bridget lets CLI agents (Codex, Claude, Gemini) talk to each other in real time
through a central daemon. The protocol is independent of its transport: tmux
today, a network socket tomorrow.

## Quick start

```bash
# Build
cargo build --release

# Start the daemon
./target/release/bridget daemon &

# Start an agent inside tmux
bridget codex

# From another terminal, send a message
bridget send --to codex-1 "Review this file" --reply

# List connected agents
bridget who
```

## Scope and trust model

Bridget aims at two things: a **reliable communication channel** between CLI
agents, and a **lightweight protocol**. It is not a security product and does not
claim to be one.

**Operating assumptions.** Agents run on the same trusted machine, under the same
user account, and talk over a local Unix socket. Every connected agent is assumed
to be cooperative.

**What Bridget guarantees:**

- a message is either delivered, or refused with an actionable reason — never silently dropped;
- no loops between agents, no duplicates, no re-delivery;
- a dropped connection is recovered automatically, under the same identity;
- a pending request has a deadline, reminders, and can be cancelled.

**What Bridget does not do:**

- it does not authenticate the sender of a control message: any local process able
  to write to the socket can rename an agent, or change its domain or
  availability;
- it neither encrypts nor signs anything;
- it does not isolate agents that would distrust one another;
- it does not withstand a hostile local process.

Consequently, do not expose the socket to a network or an account you do not
trust. The SSH federation described below tunnels the socket: trust rests
entirely on SSH, not on Bridget.

These limits are scope decisions, not oversights. Lifting them would require a
full authorisation model — worth building the day agents with different trust
levels have to coexist, pointless today, and expensive in complexity for a
protocol meant to stay readable end to end.

## Architecture

```
Agent A (CLI)          Agent B (CLI)
    │                      │
    ▼                      ▼
Wrapper A              Wrapper B
    │                      │
    └──────► daemon ◄──────┘
              │
         SQLite (ledger)
```

- **Daemon** (`bridget daemon`) — local Unix socket, routes messages, keeps the
  ledger and the guardrails.
- **Wrappers** (`bridget codex`, `bridget claude`) — start the CLI agent
  (fork+exec), connect to the daemon, receive pushed messages.
- **CLI client** (`bridget send`) — sends a message from any shell.

## Crates

| Crate | Role |
|-------|------|
| `bridget-core` | Pure logic: routing, circuit breaker, deduplication, envelopes |
| `bridget-transport` | JSON protocol + Transport trait + tmux implementation |
| `bridget-daemon` | Daemon + CLI (the `bridget` binary) |

## Reliability guardrails

These mechanisms protect a conversation from itself — loops, duplicates, endless
waiting. They do not protect against a hostile third party: see the trust model
above.

- **Circuit breaker** — at most 8 exchanges per conversation within 180 s (configurable)
- **Content deduplication** — blocks duplicate sends
- **Per-ID quarantine** — blocks re-delivery (misroutes)
- **Hops** — anti-loop hop budget (default: 4)
- **Reply yes/no** — tells questions apart from statements
- **No self-messaging** — an agent cannot talk to itself
- **Progressive escalation** — automatic reminders at T/3 and 2T/3, then a failure notice at T
- **Configurable timeout** — `--timeout <seconds>` (default 60 s)
- **Cancellable requests** — a `--reply` request carries an identifier; its sender can stop it with `bridget cancel <id>`, which removes the reminders and releases the recipient from any obligation to answer.

Cancellation is cooperative: it interrupts neither a tool nor a model already at
work, but it ends the Bridget request, its reminders and the duty to reply.
`bridget requests` lets a sender review their requests and their state.

## Commands

### Starting an agent

| Command | Effect |
|---|---|
| `bridget codex [ARGS…]` | starts Codex and connects it to the daemon |
| `bridget claude [ARGS…]` | starts Claude Code and connects it to the daemon |
| `bridget gemini [ARGS…]` | starts Gemini and connects it to the daemon |
| `bridget gclaude [ARGS…]` | `gclaude` variant, agent type `claude` |
| `bridget -- <CMD> [ARGS…]` | custom agent; the binary must be in the allow-list |
| `--name <name>` | explicit initial name, instead of the auto-incremented one |

Trailing arguments are passed through to the agent untouched. For Codex and
Claude Code, the wrapper adds the permission flags required to open the socket,
and injects an initial prompt telling the agent how to reply — unless a prompt is
already supplied.

### Communicating

| Command | Effect |
|---|---|
| `bridget send --to <name> <msg>` | sends a message |
| `… --reply` | expects an answer: the request is tracked, with a deadline and reminders |
| `… --timeout <s>` | request deadline (default: 60 s) |
| `… --hops <n>` | remaining hop budget (default: 4) |
| `… --from <name>` | declared sender, for relaying |
| `bridget reply <msg>` | answers the last sender without retyping their name |
| `bridget cancel <id> [--reason <text>]` | cancels a request that became pointless: no more reminders, recipient released |
| `bridget requests` | lists my tracked requests and their state |

### Observing

| Command | Effect |
|---|---|
| `bridget who [--domain <d>]` | human-readable directory: name, type, host, OS, transport, domain, model, effort, state |
| `bridget agents [--json] [--domain <d>]` | same directory, machine-readable |
| `bridget discover` | alias for `who` |
| `bridget status` | daemon health, paths, agent and message counts |
| `bridget ledger` | last twenty recorded messages |
| `bridget version` | binary version |
| `bridget help` | inline help, summary of every command |

### Describing yourself

| Command | Effect |
|---|---|
| `bridget rename <name>` | renames the current agent; the name survives reconnections |
| `bridget domain <name>` \| `--reset` | overrides the derived domain, or reverts to it |
| `bridget runtime --model <m> [--effort <e>]` | declares the current model, for an agent without automatic detection |
| `bridget dnd [off] [--duration 30m]` | refuses or accepts interruptions again |
| `bridget install-hooks [--remove]` | installs automatic model detection for Claude Code |

The five commands in this section only work **from inside a Bridget agent**: they
rely on the identity provided by the wrapper and fail with an explicit message in
an ordinary shell.

### Internal use

`bridget hook claude-runtime` is called by the Claude Code hook; it reads the
payload from standard input and stays silent. It is not meant to be run by hand,
except for diagnosis.

## Settings and environment

| Variable | Effect |
|---|---|
| `BRIDGET_TRANSPORT` | transport name advertised in the directory (default: `unix`, or the value read from `~/.config/bridget/federation.env`) |
| `BRIDGET_AGENT_NAME` | agent name, exported by the wrapper to the agent process |
| `BRIDGET_AGENT_NAME_FILE` | file holding the current name; this is what prevails after a `rename` |
| `HOSTNAME` | advertised host, otherwise the output of `hostname` |
| `RUST_LOG=debug` | verbose logging, including the source of every model observation |

Files, all under `~/.cache/bridget/`:

| Path | Contents |
|---|---|
| `bridget.sock` | the daemon's Unix socket |
| `bridget.db` | SQLite ledger and tracked requests |
| `agent-names/` | persistent names, per session or per active agent |
| `agent-domains/` | overridden domains |
| `last-sender-<agent>` | last sender, for `bridget reply` |

Behavioural values, hard-coded in this version: circuit breaker 8 exchanges per
180 s, deduplication 180 s, quarantine 3600 s, ledger pruned after 7 days,
unreachable presence retained 300 s, heartbeat 3 s, reconnection at 1-2-4-8-16
then 30 s at most, Codex model probe every 20 s, do-not-disturb 60 min by
default, messages up to 10,000 characters and agent names up to 100.

## Agent model and effort level

`bridget who` shows the current model and effort level of every agent, kept up to
date when a human changes them mid-session. The agent type (`claude`, `codex`)
says nothing about actual capability: the model is what decides who gets which
task.

```text
  NAME     TYPE    HOST         OS     TRANSPORT  MODEL          EFFORT  STATE
  agent-1  claude  local-host   macOS  unix       claude-opus-5  high    connected
  agent-2  codex   local-host   macOS  unix       gpt-5.6-terra  xhigh   connected
  remote   claude  server       Linux  ssh        —              —       unreachable
```

An em dash marks a value that has never been observed — Bridget never invents a
model from a configuration default. Some models expose no effort level at all: the
column then stays empty rather than inheriting the previous model's value.

Detection differs per agent, because the two do not behave alike:

| Agent | Mechanism | Setup |
|-------|-----------|-------|
| Codex | the wrapper locates the session file the process keeps open and reads the latest turn context from it | none, on by default |
| Claude Code | a `Stop` hook reports the model at the end of every turn | `bridget install-hooks`, once |
| Others | explicit declaration | `bridget runtime --model <M> --effort <E>` |

### Installing detection for Claude Code

```bash
bridget install-hooks            # adds a Stop hook to ~/.claude/settings.json
bridget install-hooks --remove   # removes it
```

The command **modifies a file outside the repository**: it first writes a
timestamped backup (`settings.json.bak-YYYYMMDD-HHMMSS`) and prints its path.
Insertion is additive — existing hooks are preserved — and idempotent. The hook is
inert outside Bridget: an ordinary Claude session is unaffected. It only applies
to sessions started after installation.

The Codex probe reads the session file only every 20 seconds, and only reports to
the daemon when the value changed: an idle agent generates no traffic at all.

## Work domains

Every agent carries a **domain**, derived with no configuration from the
repository it was started in: the git root gives the name, or the current
directory when there is no repository. Two agents started anywhere inside the
same project share a domain.

```text
  NAME     TYPE    DOMAIN     MODEL          STATE
  agent-1  claude  bridget    claude-opus-5  connected
  agent-2  codex   project-b  gpt-5.6-terra  connected
  agent-3  claude  project-b  claude-opus-5  dnd
```

A domain **tidies, it does not partition**: every agent stays visible and
reachable across domains. It is a cue for picking a recipient, not a security
mechanism — cross-project communication is a common use, notably to have code
reviewed by an agent from another repository.

```bash
bridget who                     # every agent, with its domain
bridget who --domain bridget    # only that domain
bridget domain cross-review     # override, preserved across reconnections
bridget domain --reset          # back to the domain derived from the repository
```

The name is taken raw, exactly as the directory is named: a repository filed
under `12.my-project` yields the domain `12.my-project`, sorting prefix included.
No implicit prettifying rule, which would be impossible to guess; the override is
there for the cases where the repository name does not fit.

## Do not disturb

An agent in the middle of a task can refuse interruptions:

```bash
bridget dnd                    # 60 minutes by default
bridget dnd --duration 15m     # or 90s, 2h
bridget dnd off                # lift immediately
```

Its state becomes `dnd` in the directory, and any message addressed to it is
**refused with a reason**: "agent-1 does not want to be disturbed (12 min left)".
The sender learns it at once and decides — wait, retry, or ask someone else.
Nothing is queued: a message resurfacing an hour later, out of context, is worth
less than a straight refusal.

Escalation reminders for pending requests are suspended too. The failure notice
sent to the requester, however, is still delivered: it does not disturb the
recipient.

The default duration is a safety net: an agent left on do-not-disturb becomes
reachable again on its own, without anyone having to remember.

## Tests

```bash
cargo test          # 86 tests (unit + integration)
```

## Remote deployment

```bash
./scripts/deploy-remote.sh <user@host> [port] [daemon|client-only]
```

This procedure targets Linux: it installs Rust, builds, deploys the binary and
sets up a user systemd service. The `daemon` mode (default) creates a standalone
Bridget daemon. The `client-only` mode installs the client alone; it is required
for a federated host, so as not to create a second daemon and a competing socket.

## SSH federation

To enrol any SSH machine into the single local daemon, with no public port:

```bash
./scripts/federate-ssh.sh install project-a --host example.tld --user user --port 2222
./scripts/federate-ssh.sh status project-a
./scripts/federate-ssh.sh remove project-a
```

The reverse tunnel publishes the master daemon's Unix socket on the remote
machine. Agents started there use the very same Bridget directory. Host, user,
port, SSH key and remote socket path are all configurable. Without
`--remote-socket`, the script asks the Linux host for its real `$HOME` and uses
`$HOME/.cache/bridget/bridget.sock` there: no `/home/...` path is assumed. To
install the client binary on that host without a remote daemon:

```bash
./scripts/deploy-remote.sh user@example.tld 2222 client-only
```

When a tunnel breaks temporarily, the remote AI process keeps working. Its
Bridget wrapper then removes the agent from the directory and retries with
exponential backoff and jitter (roughly 1, 2, 4, 8, 16, then 30 seconds at most).
After 60 seconds of stable connection, that delay is reset to its minimum. The
agent is re-registered automatically under the same name as soon as the SSH
socket reappears — including if it was renamed in the meantime with
`bridget rename`.

`bridget who` also shows the execution host, the OS, the transport and the
presence state in aligned columns. The OS is detected by the wrapper (`macOS`,
`Linux`, and so on) so a request can be routed to tools that actually exist
there. After a disconnection, a remote instance stays visible as `unreachable`
for five minutes, which tells a network loss apart from an agent deliberately
shut down.

### SSH prerequisites on the Linux side

The SSH server must allow reverse forwarding for the enrolled account. On a
hardened server, the administrator can add a file such as
`/etc/ssh/sshd_config.d/60-bridget.conf`:

```text
Match User user
    AllowTcpForwarding remote
    AllowStreamLocalForwarding yes
```

Then validate and reload the service (`sshd -t`, then `systemctl reload ssh`).
That permission stays limited to the account concerned; Bridget requires neither a
public TCP port nor a Bridget daemon on the federated Linux host.

## Licence

MIT — see [LICENSE](LICENSE).
