# Changelog

[English](CHANGELOG.md) · [Français](CHANGELOG.fr.md)

Published versions are added to the repository history under `vX.Y.Z` tags.
Versions follow semantic versioning and dates use YYYY-MM-DD. Validation
figures below describe the named release's evidence, not the current checkout.

## 0.8.4 — 2026-11-06

- Admit origin-side commit-view preparation when the target is the same provisioned machine. Keep other workspace strategies and all authority checks unchanged.
- Report the closed validation field and cause for origin-relay refusals, with the size limit when known. A generic refusal no longer implies an older client image.

## 0.8.3 — 2026-11-05

- Read only the selected launch definitions when processing worker observations, instead of decoding the full retained history for every command.
- Preserve a worker connection when a late refusal refers to the exact conserved view of a closed task. Identity, connection epoch and live-request correlation remain checked.
- Report closed worker refusal diagnostics without command bodies or credentials.

## 0.8.2 — 2026-11-05

- Give explicitly selected Git inventories60seconds and remove the pause between successful reads. Timeout diagnostics retain a closed command, timings, volumes and stop state.
- Fix64MiB view-reservation readers and add a narrowly scoped cold quarantine recovery that preserves receipts and starts no process.
- Allow256MiB public view maps, stream closed-reference manifest reads and include the map in free-space checks. Compatible reader floors reject older rollback images.
- Keep cancelled, unpublished view preparations scheduled until native conservation is confirmed.

## 0.8.1 — 2026-11-04

- Raise selected commit-view payloads to32GiB, inventories/manifests to64MiB and helper deadlines to30minutes. Preserve disk, object integrity and mission checks.
- Keep an admitted task visible across interrupted origin attestations and preserve the request key and unknown outcomes.

## 0.8.0 — 2026-11-03

- Native sleeper agents: history, explicit wake-up and refusal of new direct, group and scheduled requests. Previously admitted work and its results are kept.
- Classification actions sync both ways by default for linked clients, with a Bridget-only setting, durable receipts and the same command key while its outcome remains unknown.
- Active/Sleeper views in the app, `agents sleep|wake` commands, CLI filters and caches respecting server classification. Retained state requires a reader that understands dormancy.

## 0.7.8 — 2026-11-03

- Atomically relay verified context for prepared projects, including linked checkouts and previously authorized additional roots.

## 0.7.7 — 2026-11-03

- Declare the specialists input schema as an object and expose composed schemas through complete local references, so strict MCP clients can discover all37tools at startup. Accepted actions, permissions and guards are unchanged.

## 0.7.6 — 2026-11-02

- Explain unconfirmed remote starts with task-scoped checks and preserved request references. Historical start errors no longer hide later recorded work or results. No change to retries, admission or permissions.

## 0.7.5 — 2026-11-02

- Keep live bridge connections during bounded read timeouts. Suspend observations until a fresh check succeeds. Report closed RPC failure causes without replaying uncertain deliveries.

## 0.7.4 — 2026-11-01

- Add explicit global --table output for finite CLI inventories, details and receipts, before or after the command. Family columns, hostnames, Unicode wrapping and colour-free pipes share one native renderer. JSON, scopes, confirmations and exit codes stay unchanged.

## 0.7.3 — 2026-11-01

- Update status includes every registered machine, including missing and stale reports.
- Local updater setup supports machine-specific worker units on SYS, KS and macOS.
- Stopped services stay stopped. Missing observation dates stay unknown.
- Linux package built on Debian12/glibc2.36; Android7003 uses the existing signing certificate.

## 0.7.2 — 2026-11-01

- Specialist lists, searches and details also return the current human display name from the agent profile, separately from roles and stable IDs. Renames appear on the next read, including while offline.

## 0.7.1 — 2026-11-01

- Admission recovery waits for the parent's live identity and permission evidence instead of treating missing evidence as changed permissions. Explicit revocation still refuses admission.
- Bounded client recovery keeps reading the same receipt through transport interruptions and returns observed terminal refusals. An unreadable outcome stays unknown. Historical requests17/18 are not replayed.
- The Android0.7.0 application is unchanged.

## 0.7.0 — 2026-11-01

Add declared roles and human-designated responsibilities, versioned specialist profiles, durable mandates, on-demand sessions, sourced cross-project memory, definition import/export and optional team compositions. MCP, CLI and the Android catalogue share native authority and receipts. No extra work budget is enabled by default. A compatible specialist-request reader is required before rollback.

## 0.6.5 — 2026-10-30

- Task status explains a provider model rejection and the decision required from the user.
- The catalogue reads the current configuration and excludes disabled connections.
- No automatic model substitution, account change or task replay.

## [0.6.4] - 2026-10-29

- Correct the authenticated native permission ceiling across engines without exporting the origin application's authority.
- Offer missing provider programs during installation with explicit selection and existing accounts preserved.
- Distinguish installed programs, compatibility, personal sign-in and actual model access.
- Publish native fixes delivered since0.6.3: replayable admissions, bounded diagnostics, compact preparation and an explicit fleet catalogue.
- Rebuild the Android package as0.6.4 to align the release version, with the same interface, certificate and signing lineage.

## [0.6.3] - 2026-10-25

- Add a left conversation drawer with favorites, search and explicit attention states.
- Keep drafts and reading positions per agent; add loaded-history search and editable system dictation.
- Prepare files and photos locally before explicit sending; show code languages and exact copy controls.
- Add isolated interactive HTML previews alongside static and source modes, with no privileged bridge or network access.
- Project correlated native context-compaction activity without inferred progress.

## Unreleased

### Android message queue

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-1).

### Android and permanent federation access

- Add the native Android app: QR pairing, scoped directory, messages, files,
  task actions, questions, durable retries and protected local storage.
- Add the HTTPS mobile gateway with revocable device credentials and bounded work.
- Add optional minimal notifications, local quiet hours and signed APK updates.
- Add origin attestations and project preparation over the authenticated worker
  channel, plus a private workstation connection to a permanent server master.


### Documentation and repository hygiene

- English is the default README and public-guide language. French editions remain available.
- Public navigation separates current guides from original-language historical records.
- Deployment addresses and a personal address in tracked documents use example values.
- Absolute workstation dependency symlinks are removed.
- Secret scanning checks immutable historical findings separately from new changes.
  See [the audit and its limits](docs/security-audit.md).

## [0.4.1] - 2026-10-16

### Fixed

- An accepted remote response closes its tracked request even without the parent's
  transport acknowledgment, avoiding a false no-response alert.
- Maintenance retries that closure after storage failure, even if task cleanup
  already finished. It retains the result without restarting work.
- Correlation checks mission, round, child and parent. A genuinely absent reply
  still follows the configured deadline.

### Compatibility and validation

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-2).

## [0.4.0] - 2026-10-15

### Added

- Fleet catalog entries expose declared machine, engine, connection, model and
  effort support. `bridget delegate --catalogue --json` works through the native Bridget interface.
- Tasks can constrain each selection axis independently. Omitted effort keeps
  the provider's native behavior.
- Routing considers fresh load facts, bounded Git continuity, declared build
  capacity and scoped account quota/credit observations.
- A configurable decision chain applies deterministic filters, an optional
  compatible decision service, an optional lightweight selector and deterministic
  fallback scoring. Selectors choose execution combinations, not task results.
- Decision services are declared per machine. Disabled/unavailable services do
  not prevent authorized fallback levels. Every choice remains within admissible
  combinations.
- Durable state and Lineage v4 retain the selected option, provenance and bounded
  facts. Exact replay returns the saved receipt without another decision.

### Git and remote execution

- Remote startup checks the expected source commit before preparing work.
- Separate tasks can use separate worktrees. Routing does not itself push or
  merge delivered work; delivery remains explicit coordination.
- A declared model does not prove account access or unknown quota/credit.
  Each machine retains its own authorized connections.

### Compatibility and operation

- Lineage v1–v3 keep their existing shapes. V4 adds decision metadata.
- V2 admission and probe capabilities are negotiated explicitly, without silent
  conversion to an older admission.
- Existing receipts survive store migration. After a V2 admission receipt is
  written, do not return to a 0.3.0 binary solely because routing was disabled.
  Use a compatible binary or a validated coherent restoration that loses no work.
- Bridget provides terminal access to its native engine.

### Validation and known limits

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-3).

## [0.3.0] - 2026-10-15

### Added

- `bridget delegate <MISSION>` provides a short native child workflow.
- `bridget task` exposes list, status, wait, continue, respond, cancel and read ACK.
- `--json` supports scripts. A terminal without agent identity is refused before
  agent operations and receives registration guidance.

### Limits

- Coordination needs a live Bridget agent identity; a human terminal does not
  automatically become an agent.
- `task list` is a human global projection. Other operations remain scoped to
  their owning coordinator.

## [0.2.1] - 2026-10-15

### Fixed

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-4).

### Operation and validation

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-5).

## [0.2.0] - 2026-10-14

### Added

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-6).

### Fixed and limits

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-7).

## [0.1.3] - 2026-10-02

### Fixed

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-8).

### Documentation and changes

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-9).

## [0.1.2] - 2026-10-02

### Documentation

- Read-only managed launches are documented separately from human-terminal
  authorization for development. An undeclared type is not a read-only policy failure.

### Fixed

- A turn immediately followed by another between bridge reads is no longer
  reported as an observation gap when its origin was observed.

## [0.1.1] - 2026-09-25

### Changed

- `bridget --help` reports the actual package version instead of a fixed 0.1.0.

### Documentation

- Updated README and changelog.
- Publication adds versioned history instead of replacing it.

## [0.1.0] - 2026-09-25

First versioned public release. Earlier publications replaced history.

### Added

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-10).

### Fixed

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-11).
