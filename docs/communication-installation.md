# Install Bridget

[English](communication-installation.md) · [Français](communication-installation.fr.md) · [Documentation](README.md)

Build and try Bridget in a private namespace before replacing an existing
installation. A built binary does not update a running daemon, MCP server,
worker or T3 application.

## Install or update in one command

Latest published release (precompiled, no Rust needed):

```sh
curl -fsSL https://cartae.app/bridget/install.sh | sh
```

Latest source code on `main` (compiled locally):

```sh
curl -fsSL https://cartae.app/bridget/install.sh | sh -s -- --source
```

An exact published release:

```sh
curl -fsSL https://cartae.app/bridget/install.sh | sh -s -- --version v0.4.1
```

The same script is available directly from GitHub:

```sh
curl -fsSL https://raw.githubusercontent.com/guthubrx/bridget/main/scripts/install.sh | sh
```

For inspection before execution, download the script to a file and read it
before running `sh` on that file. HTTPS authenticates the download host;
the release's SHA-256 sidecar checks archive integrity, not an independent signature.

Run as your normal user, without sudo. Release installation needs curl, tar and
sha256sum or shasum. A platform without a published archive is refused; it never
silently falls back to a source build. Current v0.4.1 packages cover Linux x86-64
and macOS Apple Silicon. Other supported source targets require `--source`.

Source mode needs Git, Python 3.11+, a C compiler and its platform development
tools (Xcode Command Line Tools on macOS, a C build toolchain on Linux).
It installs Rust through the official rustup script if absent, uses the repository's
toolchain and Cargo.lock, and builds through the existing cache maintenance wrapper.
It keeps a dedicated source checkout in `.local/state/bridget-install/source`,
so cache ownership remains stable across updates. The cache follows the same
10 GiB / 7-day budget; successful installation purges its build intermediates.
An edited installer checkout is preserved and refused, never reset automatically.
An unavailable compiler or dependency is an explicit error, never a sudo prompt.

The executable is installed in the user's `.local/bin` directory, with a receipt
and a distinct previous executable in `.local/state/bridget-install`. Repeating
an identical installation preserves that previous executable. Profiles for the
user's Bash or Zsh shell are backed up before adding PATH setup. To use the
command immediately in the current terminal:

```sh
. "$HOME/.local/share/bridget/env"
bridget --version
```

Accounts, data and service definitions are preserved. No daemon, worker, tunnel
or T3 process is started or restarted. Installation is not federation: use
`bridget daemon` for a standalone instance, or follow the
[SSH federation guide](federation-services.md) to join an existing master.
Already running services keep their loaded version until deliberately restarted.
The retained executable is not a database rollback procedure.
An interrupted install can leave `.local/state/bridget-install/lock`. Inspect its
`pid` file and verify that no installer still runs before removing that lock
directory. Never remove the source checkout or user data to clear a lock.

## Prerequisites for an existing source checkout

Use macOS or Linux, Git and the Rust toolchain in `rust-toolchain.toml`
(currently 1.92.0). The build wrapper needs Python 3.11 or later. Cache
maintenance uses `lsof`. Remote project preparation also needs Python and Git
on the target machine. Install and authenticate each provider on the machine
where it will run; Bridget does not copy provider accounts.

## Build from source

```sh
git clone https://github.com/guthubrx/bridget.git
cd bridget
python3 scripts/build.py cargo build --locked --release -p bridget-daemon
bridget_bin="$(pwd -P)/target/release/bridget"
```

The release profile is the operational build. Debug builds are for development;
their performance does not qualify production timeout behavior.
See [build caches](build-cache.md) for the wrapper's maintenance policy.

For an independently packaged core, from a Git checkout:

```sh
package_parent=$(mktemp -d /tmp/bgpackage.XXXXXX)
package_parent=$(cd "$package_parent" && pwd -P)
bash scripts/package-089-core.sh "$package_parent/source"
cd "$package_parent/source"
shasum -a 256 -c SOURCE-MANIFEST.sha256
BRIDGET_BUILD_ID="$(cat BUILD_ID)" python3 scripts/build.py cargo build --locked --offline --release -p bridget-daemon
```

Offline mode requires the toolchain and locked dependencies to be cached.
The source manifest identifies the packaged bytes. `BUILD_ID` identifies the
source, including a `-dirty` marker when applicable. An archive without Git or
an explicit build identity may report `unknown`.

## Run a private instance

In the build checkout, keep `bridget_bin` set to the absolute candidate path:

```sh
umask 077
bridget_state=$(mktemp -d /tmp/bgcomm.XXXXXX)
export BRIDGET_HOME="$bridget_state"
export BRIDGET_SOCKET="$bridget_state/bridget.sock"
"$bridget_bin" daemon
```

Keep the resolved binary, state and socket paths. Set the **same** values in
another terminal before running:

```sh
"$bridget_bin" status
"$bridget_bin" agents --json --global
```

Do not run `mktemp` again for a client: that would select a different instance.
Keep the state directory private (0700), private files 0600 and the socket path
short enough for Unix sockets on both platforms. Do not substitute symlinks.

An inaccessible directory is an error, not an empty fleet. A standalone CLI
client with no project context may legitimately have an empty same-project
view. Supply its real absolute project path when needed:

```sh
"$bridget_bin" who --project-root /absolute/path/to/project
```

## Register a terminal agent

From the intended project directory, using the same namespace:

```sh
"$bridget_bin" codex
# Or:
"$bridget_bin" claude
```

A provider started as plain `codex` or `claude` is not automatically a Bridget
agent. A compatible host can mount Bridget with a separate attestation, but
an MCP tool declaration alone does not establish identity.

Codex must support the app-server and remote-resume interfaces required by the
installed Bridget driver. The historical interactive validation used Codex
0.153.4; check your actual driver and provider versions rather than treating
that test version as a current compatibility guarantee.

To resume an existing Codex conversation at launch:

```sh
bridget codex --name reviewer resume '<codex_thread_uuid>'
```

A resume refusal does not mean that a replacement conversation was created.
Read the provider error. Do not repeatedly resume a conversation that still
has an unresolved writer. Start a separate session only as an explicit choice.

The interactive session closes its TUI, private server and presence when it
exits. Managed persistent sessions use a different lifecycle. Preserve the
provider's normal home, account and permission policy. Use permissive provider
options only when the user explicitly wants them.

## Provider registry and accounts

The private registry for this namespace can live at
`$BRIDGET_HOME/agents.json`. The path is a shell template; resolve it locally.
Keep the file private and without symlinks.

Each definition supplies its executable, protocol, supported models and
capabilities. A declared type receives a corresponding
`project-discovery-<type>` read-only variant. Custom names are local registry
names, not portable provider names.

For a Claude-compatible service, use a dedicated launcher and profile. The
launcher should read credentials from an appropriate private mechanism, not
from a committed example. A separate Codex account can use a launcher setting
`CODEX_HOME`. A Claude profile uses `claude_config_dir` and must be private
(0700). Keep the model in the launch arguments consistent with the declared
model catalog.

Example shape for a dedicated Claude-compatible connection. The launcher and
profile paths must be real absolute paths on the selected machine. The
permissive managed flags shown here are appropriate only for an already
compatible, authorized parent policy; do not add them to bypass a refusal.

```json
{
  "agents": {
    "glm": {
      "command": "/absolute/path/to/glm-launcher",
      "args": ["--model", "glm-5.3", "--dangerously-skip-permissions", "--permission-mode", "bypassPermissions"],
      "protocol": "claude_stream_json",
      "forbidden_env": ["ANTHROPIC_API_KEY"],
      "claude_config_dir": "/absolute/path/to/private/glm-profile",
      "mcp": {"interactive": "claude", "acp_session": false},
      "capabilities": {"execution_paths": ["claude_stream_json"], "models": {"glm-5.3": {}}}
    }
  }
}
```

This is a declared example, not proof that this model/account is available.
The catalog does not test authentication. Verify a selected account through
an authorized real task. A slow managed launch can use its explicit
`--timeout` budget; do not infer success from a connected presence alone.

## T3 integration

T3 attests the calling conversation. Bridget still owns its native tasks.
Installing files does not reload an already running T3, bridge or MCP process.

For `t3_session_unavailable`, inspect the specific binding/transport reason and
the loaded versions. Compatible T3 resume handling remounts the current private
proof before a new turn; a failed remount must not borrow another thread's
identity. Do not loop through uninstall/reinstall operations.

For `identity_not_found`, first determine how the agent was started. Missing
wrapper registration and a lost T3 binding are different faults.

For `permission_attestation_unavailable`, an identity exists but the required
permission proof is unavailable. A successful catalog or session call does
not grant development rights.

For `Too many open files`, check connection lifecycle and loaded bridge code.
Raising the file-descriptor limit does not close abandoned links.

## SSH federation

Use [the federation guide](federation-services.md). The supplied SSH scripts
require explicit connection settings and a verified host-key file. They do not
copy private keys, enable agent forwarding or accept unknown host keys.

A federated client joins one master. It must not start a competing daemon
against that namespace. Federation alone does not make ordinary `spawn`
execute on a different host; remote tasks use the worker/delegation path.

## Stop, upgrade and roll back

For a private trial, stop only its agents, daemon and tunnel. Identify the exact
process and namespace before stopping anything. Keep its state for diagnosis
if needed. The production installation is unchanged.

For an operational upgrade, record the source commit, binary hash, build
identity and current service commands. Preserve a compatible rollback binary
and any required stopped-state backup. Verify the candidate, activate only
the intended component, then inspect its loaded identity.

Do not restore a database over an open database. Do not assume an older binary
can read a store after a newer migration. Preserve active work and use the
service's supported shutdown/reconnection path. This guide does not authorize
interrupting unrelated sessions.
