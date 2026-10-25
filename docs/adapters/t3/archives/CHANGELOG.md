# Archives techniques de l’adaptateur T3

Document d’origine : [CHANGELOG.md](../../../../CHANGELOG.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Keep ordinary mobile messages in Bridget while the agent is busy. Show a collapsible queue above the composer with edit, cancel and explicit steering actions.
- Preserve pending actions across uncertain responses. Reconcile native journal message identifiers and keep steering intent after delivery recovery.
- Support explicit Codex steering through the existing T3 V2 adapter while ordinary notifications continue to wait.

<a id="extrait-2"></a>

## Extrait 2

- No protocol, schema, dependency or T3 configuration change. Old expired
  request states remain historical.
- 46 targeted remote compatibility tests and three fix tests passed. The final
  selective SQL oracle was rerun alone and passed. No full suite was rerun.
- Packages, activation and the live recipe are separately recorded in the
  session 170 delivery receipt.

<a id="extrait-3"></a>

## Extrait 3

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

<a id="extrait-4"></a>

## Extrait 4

- The T3 bridge drains and closes finished subagent links instead of retaining
  thousands of descriptors. Concurrent resumes are bounded to 32.
- Compatible T3 (0.0.45-local.166+) remounts a resumed Codex thread's current private
  Bridget proof before its first turn. Stale proof remains refused; remount failure
  blocks the turn rather than borrowing another thread's identity.

<a id="extrait-5"></a>

## Extrait 5

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

<a id="extrait-6"></a>

## Extrait 6

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

<a id="extrait-7"></a>

## Extrait 7

- Quota computation includes target-account Codex credits.
- The T3 connector and its visual delivery are separate; this release does not
  prove that an already-open T3 application reloaded.
- Validation is targeted; no global workspace green result was claimed.

<a id="extrait-8"></a>

## Extrait 8

- Spawn honors `--timeout` plus margin instead of reporting an unknown outcome
  after ten seconds for a slow provider.
- Consecutive T3 observation notices share a turn and remain separate from messages.
- The historical busy-thread path delivered untracked Claude/Cursor messages
  immediately rather than waiting for turn end. Later guidance contracts refine
  provider/authority eligibility; tracked requests and notifications wait.

<a id="extrait-9"></a>

## Extrait 9

- Installation explains custom Claude-compatible profiles and separate Codex accounts.
- Tracked T3 replies identify their addressee so a relayed answer is not mistaken
  for an answer to the local user.
- Untracked message instructions distinguish “no acknowledgment” from “no action.”
- Send accepts an unambiguous UUID prefix of at least six characters (CLI/MCP)
  or exact display name (CLI).
- Unknown managed agent types report the known registry types rather than an
  unrelated missing-capability error.

<a id="extrait-10"></a>

## Extrait 10

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

<a id="extrait-11"></a>

## Extrait 11

- Large threads are not repeatedly copied into the observation journal.
- A source is not announced ready before journal validation.
- A process disappearing during descriptor inventory does not invalidate all threads.
- Attested T3 CLI sends no longer claim human authorship.
- Build identity is refreshed rather than frozen at the first compilation.
- Integration tests no longer rely on host load.
