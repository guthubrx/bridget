# Automatic updates

Bridget checks the latest stable release of the official guthubrx/bridget GitHub repository. The default is on. The Android Settings switch controls the phone and the registered Mac/Linux hosts. A disabled choice survives restarts. A disconnected phone keeps a dated, pending host choice; it does not claim the server received it.

## Release contents

A release uses one strictly increasing version, for example 0.6.0, and these assets:
- bridget-0.6.0-darwin-arm64.tar.gz, containing bridget-0.6.0-darwin-arm64/bridget;
- bridget-0.6.0-linux-x86_64.tar.gz, containing bridget-0.6.0-linux-x86_64/bridget;
- bridget-android.apk, signed with the installed application's signing key or an Android-verified successor.

The release API supplies each asset's SHA-256 digest and exact size. Missing digests or platform assets stop installation. Drafts, prereleases, arbitrary URLs, build identifiers and older versions are not update candidates. Publishing main alone does not publish an update. Publish all verified assets before making a release latest. A GitHub account able to replace releases remains part of the distribution trust boundary.

## Android

JobScheduler checks approximately every six hours, subject to Android battery/network scheduling. Opening the app also checks when due. The automatic path downloads into private storage, verifies size/digest/package/version/signing lineage and uses PackageInstaller. It waits while the app is visible or an operation is running. Android may require permission to install from Bridget and a system confirmation; the app exposes both. It never grants itself these permissions.

Switching off stops future automatic work and attempts to abandon an uncommitted automatic installation. An installation already committed to Android may finish. A failed version is not automatically committed repeatedly. Manual check and installation remain available. Association, conversations, drafts and receipts are not cleared. Android does not perform an automatic downgrade.

## Mac and Linux

The stable executable link is `~/.local/bin/bridget`. The embedded updater runs as a user service with Python3.9 or later: `bridget updates watch`. Its private `BRIDGET_HOME/update-host.json` contains `host` and `services`. Linux roles are `core`, `mobile` and `worker`; the worker unit is `bridget-worker-cartae.service`. Supervised services execute the stable link. Configuration contains no downloaded command.

[Adapter contract details](adapters/t3.md#note-50).

A per-host file lock permits one updater. Metadata is checked every six hours and an existing prepared candidate is reconsidered every minute. Downloading does not stop missions. Native activation obtains a common, durable 180-second barrier. Active or uncertain missions, admissions and commands defer activation. New deliveries receive update_in_progress while the barrier is held. A second host cannot activate concurrently.

The updater atomically switches the executable link, restarts only configured Bridget services, and verifies their loaded executable and the core version. A failed activation restores the prior link and checks the services again. A lost reply after the final release of the barrier never triggers a late rollback. Interrupted activation is reconciled before a further attempt. Recovery also requires the barrier; it never restarts services underneath new work.

Services and data use the existing OS account. The phone needs the explicit update-management grant and federation-wide access. Agent connections cannot use the mobile administrative role. The installation administrator grants this permission with bridget updates grant DEVICE in a terminal. The ordinary device scope is preserved.

Commands:
- bridget updates status: policy and dated host states;
- bridget updates check: request a new check;
- bridget updates auto on or off: persist the policy;
- bridget updates install 0.6.0: explicitly request that exact advertised version, including when automatic mode is off;
- bridget updates once: one host pass;
- bridget updates watch: persistent host watcher.

An exact manual target cannot silently select a different release. Turning automatic mode off also clears a queued manual target.

## Loaded clients and cleanup

[Adapter contract details](adapters/t3.md#note-51).

Only update-owned version directories are removed automatically, after a complete open-file inventory. The active executable, the rollback executable, other live versions, unrelated installations, evidence and unknown inventories are preserved. Build intermediates remain subject to scripts/build.py. Keep one verified rollback; do not erase a worktree or pending task to make an update possible.

The first rollout must install the update-capable binary and configure the watchers through the normal controlled deployment. The updater does not grant itself an exception to bootstrap an old daemon. See the session 179 delivery receipt for actual installed/active versions and any deferred component.

The update-management grant is stored in the private update policy, separately from legacy mobile device scopes. Revocation and expiry are still checked on every request. An older binary can therefore read the unchanged device scope after rollback.

A downloaded file is not an active version. An existing MCP process keeps its loaded code until a supported remount. Preserve session credentials. A different build ID alone does not prove that the daemon is older.

## Check provider clients before activation (209)

The candidate exposes bridget clients check, with --json for a structured report. It inspects local configured programs only, without authentication, model access checks or missions. registry_file identifies the namespace registry, or the default native worker registry when the former is absent. A custom installation can select its private registry through BRIDGET_CLIENTS_REGISTRY=/absolute/path/agents.json. This selection changes no file.

This release supports Claude Code2.1.280+ and Codex0.162.1+. The Claude floor follows the attested model rejection in208; the Codex floor follows the child-request reference schema in155. These are Bridget support floors, not guarantees for all future models.

Before stopping services, replacing links or activating, the installer and updater query **the candidate**. An old installed client yields client_update_required; an unavailable check yields client_check_unavailable. The report identifies the host, program, observed and required versions, and the required decision. Update status preserves it under client_compatibility. A missing client remains unavailable; it does not prevent installing Bridget alone.

Consent to Bridget updates does not authorize provider-client updates. Request separate consent, use the existing official installation mechanism, preserve accounts and models, rerun the check and qualify an authorized real launch. This check installs or upgrades no provider program. Opaque launchers and unreadable versions remain unverified. Existing tasks continue; only candidate activation on the affected host is suspended. A legacy candidate without this command remains unverified.
