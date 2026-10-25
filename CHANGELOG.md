# Changelog

[English](CHANGELOG.md) · [Français](CHANGELOG.fr.md)

Published versions are added to the repository history under `vX.Y.Z` tags.
Versions follow semantic versioning and dates use YYYY-MM-DD. Validation
figures below describe the named release's evidence, not the current checkout.

## Unreleased

### Android message queue

- Keep ordinary mobile messages in Bridget while the agent is busy. Show a collapsible queue above the composer with edit, cancel and explicit steering actions.
- Preserve pending actions across uncertain responses. Reconcile native journal message identifiers and keep steering intent after delivery recovery.
- Support explicit Codex steering through the existing T3 V2 adapter while ordinary notifications continue to wait.

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

- No protocol, schema, dependency or T3 configuration change. Old expired
  request states remain historical.
- 46 targeted remote compatibility tests and three fix tests passed. The final
  selective SQL oracle was rerun alone and passed. No full suite was rerun.
- Packages, activation and the live recipe are separately recorded in the
  session 170 delivery receipt.

## [0.4.0] - 2026-10-15

### Added

- Fleet catalog entries expose declared machine, engine, connection, model and
  effort support. `bridget delegate --catalogue --json` works without T3.
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
- Bridget remains usable from a terminal independently of T3.

### Validation and known limits

- The scoped V15 r2 audit reported PASS and grade A; its penalty score was 98.
  Unrated axes remained unrated. No critical/high/medium finding was open.
- The release retained low findings RUSTSEC-2026-0098 and RUSTSEC-2026-0099 for
  rustls-webpki 0.101.7, and the RUSTSEC-2024-0449 maintenance notice for
  unsafe-libyaml 0.2.11. No dependency remediation was claimed.
- One full run recorded 2820 passed, 32 failed and 71 ignored tests. Later targeted
  results passed for 29 failures; three session-159 cancellation cases were not
  validatable on Linux. The original T3 failure cause remained unknown.
  No globally passing workspace was claimed.
- Git, package, installation and runtime evidence were recorded separately.

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

- The T3 bridge drains and closes finished subagent links instead of retaining
  thousands of descriptors. Concurrent resumes are bounded to 32.
- Compatible T3 (0.0.45-local.166+) remounts a resumed Codex thread's current private
  Bridget proof before its first turn. Stale proof remains refused; remount failure
  blocks the turn rather than borrowing another thread's identity.

### Operation and validation

- The normal 4096 soft file limit sufficed in the recorded recipe. Raising it
  is not a replacement for fixing link lifecycle.
- Resume no longer requires repeated bridge uninstall/reinstall cycles.
  Check the versions actually loaded first.
- Targeted results: 13 lifecycle, 102 T3 module and 22 contract tests passed;
  one contract test remained ignored.
- The live recipe reduced links from 1130 to 32 and descriptors from 4539 to 167.
  No new EMFILE/Too many open files was observed after activation.
- The crate's full suite retained 91 inherited failures outside affected files;
  no global green result was claimed.

## [0.2.0] - 2026-10-14

### Added

- Attested internal T3 subagents can discover/send through MCP with child
  provenance and replies routed to the parent, without durable child identity
  or new administration rights.
- Messages/batches display human names alongside full UUIDs.
- The external Agent Loop tracks acceptance, verifiable progress, grouped
  reminders and next decisions; an empty queue or turn end is not task closure.
- Shared threads support silent history and explicit instruction replacement
  through `supersedes_seq`, without deleting/replaying history.
- Same-project discovery is the default. Global discovery is explicit;
  cross-project sends carry a reason. Unknown projects remain unknown.
- Native delegation adds controlled admission, durable tasks and rounds under
  one authority, without implicit engine/machine switching.
- Held messages, wakes, missed facts, controlled queue edits, capability
  catalogs and target-account Codex credits are supported.

### Fixed and limits

- Quota computation includes target-account Codex credits.
- The T3 connector and its visual delivery are separate; this release does not
  prove that an already-open T3 application reloaded.
- Validation is targeted; no global workspace green result was claimed.

## [0.1.3] - 2026-10-02

### Fixed

- Spawn honors `--timeout` plus margin instead of reporting an unknown outcome
  after ten seconds for a slow provider.
- Consecutive T3 observation notices share a turn and remain separate from messages.
- The historical busy-thread path delivered untracked Claude/Cursor messages
  immediately rather than waiting for turn end. Later guidance contracts refine
  provider/authority eligibility; tracked requests and notifications wait.

### Documentation and changes

- Installation explains custom Claude-compatible profiles and separate Codex accounts.
- Tracked T3 replies identify their addressee so a relayed answer is not mistaken
  for an answer to the local user.
- Untracked message instructions distinguish “no acknowledgment” from “no action.”
- Send accepts an unambiguous UUID prefix of at least six characters (CLI/MCP)
  or exact display name (CLI).
- Unknown managed agent types report the known registry types rather than an
  unrelated missing-capability error.

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

- Durable observation subscriptions resume after daemon restart and report lost
  outage facts rather than requiring resubscription.
- Source-state notifications wait for 30 seconds of stability. An unstable source
  produces at most one notice every five minutes.
- T3 background/subagent-triggered turns remain observable.
- Uncertain T3 reply pairing is labeled rather than silently retained.
- A second T3 application sharing data is identified; state it removes on exit
  is restored.
- Reply pairing uses proved turn origin/timestamps rather than counts.
- Hourly maintenance purges expired deliveries and departed-process identity
  state and rotates journals; build maintenance handles eligible unused resources.
- Untracked messages for the same thread can share a turn.

### Fixed

- Large threads are not repeatedly copied into the observation journal.
- A source is not announced ready before journal validation.
- A process disappearing during descriptor inventory does not invalidate all threads.
- Attested T3 CLI sends no longer claim human authorship.
- Build identity is refreshed rather than frozen at the first compilation.
- Integration tests no longer rely on host load.
