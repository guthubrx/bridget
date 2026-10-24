# Bridget

<p align="center"><img src="assets/branding/bridget-logo.png" alt="Bridget" width="180"></p>

**A local communication layer for your coding agents.**

[English](README.md) · [Français](README.fr.md) · [Documentation](docs/README.md) · [Releases](https://github.com/guthubrx/bridget/releases)

Bridget connects agent sessions across providers and machines. Agents can find
each other, exchange messages, ask for a reply, share a thread and delegate work
without copying conversations by hand.

A Rust daemon owns identities, delivery receipts and task state. CLI and MCP
clients use that same authority. T3 Code can connect to it, but Bridget's native
delegation engine also works without T3.

## What you can do

- **Communicate:** address an agent, request a reply and inspect the recorded outcome.
- **Share context:** use shared threads, a handoff brief or a bounded journal excerpt.
- **Delegate:** give a child a mission, inspect its result and continue it when its guard permits.
- **Work remotely:** connect machines over SSH and run supported tasks in separate Git worktrees.
- **Keep control:** inspect queues, cancel owned tasks and use human controls to halt new delegations.

Codex and Claude Code use native drivers. Cursor and Gemini use supported
integration paths. A provider listed in the catalog still needs an installed
program, a configured account and permissions on the selected machine.

## Start here

Install the latest published release on macOS or Linux:

```sh
curl -fsSL https://cartae.app/bridget/install.sh | sh
```

For the latest code on `main`, compiled on your machine:

```sh
curl -fsSL https://cartae.app/bridget/install.sh | sh -s -- --source
```

Run the same command to update. The installer preserves accounts and data.
It installs the CLI, without starting or restarting a daemon or joining a master.
Read the [installation guide](docs/communication-installation.md) for prerequisites,
an exact release version, and standalone versus federated setup.

To build an existing checkout:

```sh
git clone https://github.com/guthubrx/bridget.git
cd bridget
python3 scripts/build.py cargo build --locked --release -p bridget-daemon
```

Use the toolchain declared in `rust-toolchain.toml`. This builds a candidate;
it does not replace or restart your daemon.

Once Bridget is installed and its daemon is reachable, start an agent through
the wrapper from your project directory:

```sh
bridget codex
# Or:
bridget claude
```

**Starting plain `codex` or `claude` does not by itself register a Bridget
identity.** A compatible host integration may register it separately. An agent
must have an attested identity before using agent-only communication or delegation.

Inside a registered agent session:

```sh
bridget who
bridget agents --json
bridget send --to '<agent_uuid>' --reply --timeout 120 -- 'Review the contract and report concrete defects.'
bridget ledger --limit 20
```

Use an identifier returned by the directory. A display name is not an authority.
The directory defaults to the same project. An intentional cross-project
exchange needs `--cross-project-reason`.

## Delegate and follow the result

```sh
bridget delegate --catalogue --json
bridget delegate 'Review the tests and report the main risks.'
bridget task wait '<task_id>'
bridget task status '<task_id>'
bridget task ack '<task_id>'
```

The catalog is a capability view, not an authentication test. Preserve the task
ID, request key and exact arguments. If an admission response is lost, inspect
or replay that same request; a new key can create a second mission.

Read the [delegation guide](docs/delegation-native.md) for permissions, remote
preparation, worktrees, continuation and cancellation.

## What a receipt proves

| Fact | What it means |
| --- | --- |
| Accepted request | Bridget recorded the request. |
| Delivery acknowledged | The transport acknowledged delivery. |
| Read acknowledged | The parent explicitly declared that it read the result. |
| Result available | A task result is recorded; review its contents and evidence. |
| Unknown outcome | Delivery or execution is unresolved; do not assume failure or resend under a new key. |

A turn ending is not proof that a mission succeeded. A successful build is not
proof that the active service loaded it. A result does not mean its commits
were merged or deployed.

## Privacy and boundaries

Bridget stores local state and uses Unix sockets. Federation uses your SSH
configuration supplied to Bridget. It does not require a public HTTP service.
Provider traffic still follows the selected provider and account.

Credentials, live databases and raw operational logs belong outside Git.
Messages, thread history and briefs can be read by their authorized audience;
`notify: []` suppresses notifications, not access.

See [security and privacy](SECURITY.md), the
[repository audit](docs/security-audit.md), and
[communication guarantees](docs/reference-communication.md).
No secret scanner can prove that a repository contains no sensitive information.

## Documentation

| Topic | English | Français |
| --- | --- | --- |
| Installation and terminal identity | [Guide](docs/communication-installation.md) | [Guide](docs/communication-installation.fr.md) |
| Communication, receipts and queues | [Reference](docs/reference-communication.md) | [Référence](docs/reference-communication.fr.md) |
| Native delegation and remote work | [Guide](docs/delegation-native.md) | [Guide](docs/delegation-native.fr.md) |
| SSH federation | [Guide](docs/federation-services.md) | [Guide](docs/federation-services.fr.md) |
| Build cache maintenance | [Guide](docs/build-cache.md) | [Guide](docs/build-cache.fr.md) |
| Deprecations | [Register](docs/DEPRECATIONS.md) | [Registre](docs/DEPRECATIONS.fr.md) |
| Release history | [Changelog](CHANGELOG.md) | [Journal](CHANGELOG.fr.md) |

English is the default for public guides. French versions remain available.
Design decisions, specifications and dated validation records retain their
original language and are listed separately in the [documentation index](docs/README.md).
Their historical test counts are not a claim about the current checkout.

## Architecture

Three crates form the communication core:

- `bridget-core`: message types, canonical forms and shared guards.
- `bridget-transport`: wire protocol and provider transports.
- `bridget-daemon`: daemon, CLI, MCP, storage and task lifecycle.

Frame sizes, pages, queues and operation budgets are bounded. Explicit refusal
codes distinguish missing prerequisites from successful admission.

## License

MIT. See [LICENSE](LICENSE).
