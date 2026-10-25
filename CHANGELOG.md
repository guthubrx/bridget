# Changelog

[English](CHANGELOG.md) · [Français](CHANGELOG.fr.md)

Published versions are added to the repository history under `vX.Y.Z` tags.
Versions follow semantic versioning and dates use YYYY-MM-DD. Validation
figures below describe the named release's evidence, not the current checkout.

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
