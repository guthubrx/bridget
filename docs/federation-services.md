# SSH federation

[English](federation-services.md) · [Français](federation-services.fr.md) · [Documentation](README.md)

A Bridget federation has **one master daemon**. Reverse SSH forwarding exposes
a client socket on another machine. Clients share the master's directory and
ledger. This is neither database replication nor automatic failover.

The hostnames, IP addresses and paths below are examples. Supply your actual
authorized settings locally; do not publish them or your private key.

## Install from the binary

`bridget federate` embeds the canonical SSH script. It materializes a private
invocation copy and removes that copy afterward. Installation places a stable
runner in the user's installation directory. The resulting service does not
depend on the source checkout or the binary used to install it.

```sh
bridget federate ssh://worker.example.net -p 2222
bridget federate status
```

Destinations use `ssh://[user@]host[:port]`; the default port is 22. A URL port
and `-p` must agree. An existing installation is matched by its exact name
before DNS lookup. A DNS alias can find a recorded address for non-mutating
reuse; it does not rewrite the SSH target or bypass `known_hosts`.

Lookup is bounded to five seconds and 64 addresses. The inventory is bounded
to 128 installations. Ambiguous or oversized results are refused; `--label`
can disambiguate an installation.

New destinations can require `--label`, `--user`, `--identity`,
`--known-hosts`, `--root`, `--socket`, `--remote-root` and
`--remote-socket`. Missing values are prompted only when both input and output
are terminals. Automation gets an explicit list of missing options; piped
input is not consumed as answers.

## Connection and service behavior

The script's `run` and `install` commands share the connection options:
`--label`, `--host`, `--user`, `--port`, `--identity`,
`--known-hosts`, `--root`, `--socket`, `--remote-root`,
`--remote-socket`.

Paths must be absolute and validated. The identity key and host-key file must
already exist with private permissions. Unknown host keys are never accepted
automatically.

- macOS uses a user LaunchAgent; Linux uses a `systemd --user` unit.
- The service supervises foreground SSH and reconnects at a bounded cadence.
- `status --label NAME` reports service state, not message delivery.
- `remove --label NAME` removes only a recognized installation.
- A stale socket is reclaimed only after type, owner, permission and connection
  checks. A live socket or ambiguous failure remains a refusal.
- Removal retains SSH keys and Bridget data. If shutdown or endpoint cleanup
  cannot be proved, files remain for diagnosis.

On Linux, check `loginctl show-user USER -p Linger` if the user manager must
survive logout. The script does not change this policy or request sudo.
A macOS LaunchAgent starts at login, not before it. The master must remain on.

## Client and master are different roles

The default communication namespace is `.cache/bridget-core` under the
provider's home, with socket `bridget.sock`. A private `federation.env`
containing `channel=ssh-unix` records the channel.

A federated client must not start another daemon in that namespace.
A `spawn` request handled by the master launches on the master's host.
Use [native remote delegation](delegation-native.md) to select a worker host.
A tunnel does not turn ordinary spawn into a remote scheduler.

During migration, preserve old units, aliases, databases and a rollback record.
Stop only the authorities authorized for replacement. Do not delete a socket
before binding it, and do not fall back to an old database after a tunnel failure.

## Explicit installation example

From a source checkout on the master, with all paths replaced by real absolute
paths:

```sh
bash /absolute/path/to/bridget/scripts/federate-ssh.sh install \
  --label worker-core --host worker.example.net --user user --port 2222 \
  --identity /absolute/path/to/private/key --known-hosts /absolute/path/to/known_hosts \
  --root /absolute/path/to/master/state --socket /absolute/path/to/master/state/bridget.sock \
  --remote-root /absolute/path/to/client/state --remote-socket /absolute/path/to/client/state/bridget.sock
```

The script intentionally avoids implicit SSH configuration. Verify host keys
before running it. Start with the script's documented dry-run mode when preparing
a new deployment.

```sh
bridget federate status --label worker-core
ssh -p 2222 user@worker.example.net /absolute/path/to/bridget who
```

The second command checks the communication endpoint from the client.
An active service alone does not establish that endpoint's health.

## Remove a connection

Removal intentionally interrupts remote communication:

```sh
bridget federate remove ssh://worker.example.net -p 2222 --label worker-core
```

An exact textual host match can be removed directly. If the match exists only
through DNS, non-interactive use requires `--label`. Interactive use displays
the recorded target and asks for the literal confirmation `oui`. A conflicting
destination and label are refused without mutation.

## Update a remote client

The versioned `scripts/deploy-remote.sh` prepares a client-only Linux candidate.
It does not restart host applications, the master, its bridge or the existing tunnel, and does
not make ordinary spawn remote.

Use a clean isolated Git source and `--expect-commit` with its full commit SHA.
Record the runner revision, documentation revision and compiled source
separately. Without an expected commit, legacy dirty-source operation cannot
prove an exact release source.

Use a new installation prefix and an explicitly installed Cargo toolchain.
The runner builds with `--locked --offline --jobs 2`, a target directory inside
the prefix and `nice -n 10`. It does not download missing tools. Run only one
delivery build at a time. Keep a failed prefix's evidence before choosing a
separate retry prefix.

The transfer and SHA-256 manifest use the same filtered Git file list.
Symlinks, non-regular entries, Git internals, build output, databases and sockets
are excluded with a record. ASCII filenames with spaces are supported;
newlines and backslashes in names are refused. The target verifies the
manifest, exact file set, bytes and absence of links before compilation.

Before activation, verify architecture, permissions, hash and build identity.
Run the candidate's `status` and `who` by absolute path. Keep the current
binary available for rollback. Switch only the explicitly owned alias, checking
its expected target first; do not overwrite a reference changed by another
operator.

After activation, resolve the alias again and repeat the status/directory
checks. A two-client transport recipe verifies registration and bidirectional
messages, not the behavior of a real model.

## Validation and rollback

The historical service tests cover launchd and systemd boundaries using doubles.
The network recipe `scripts/tests/federation_095_roundtrip.py` uses two test
clients, not two models. The original deployment receipts are historical and
do not establish the state of your installation.

To roll back, first reconcile active sessions. Remove only the new tunnel and
restore the preserved alias, service definitions and profiles intentionally.
Never copy a SQLite backup over an open database. Restoring a separate old
daemon restores a separate directory, not the federation.

Keep the active binary, compatible rollback, source manifest and receipts.
Remove only build/worktree resources confirmed unused by the operation.
