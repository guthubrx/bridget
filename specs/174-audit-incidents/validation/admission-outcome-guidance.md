# Refusal admission outcome — follow-up 174

2026-10-24. Delivered.

A source mismatch can occur before receipt reservation. Existing guidance required the caller to confirm absence of a task without providing that fact. The maintainer had to read the same database repeatedly.

The V2 refusal handler now reads owner-scoped receipts and tasks under the existing daemon lock and exposes a bounded outcome plus recovery in `detail.admission`. Unknown storage or identity stays unknown. No schema, authority, admission, replay, or automatic execution changes. Existing generic error codes and MCP detail rendering remain unchanged.

T3 reference: `apps/server/src/vcs/GitVcsDriverCore.ts`, `resolveCommit` and `createWorktree`: resolve the exact Git revision and report an error before workspace creation. T3 has no equivalent to Bridget's remote durable admission receipts. Bridget reuses its existing receipt reader instead of adding a service or another status tool.

Four targeted tests cover an actual handler mismatch before reservation; pending/refused/expired records; an admitted task and missing identity; and an unreadable receipt table. All four passed on Cartae (0 failures, 0 ignored). The first compile found a missing test-helper limit argument; it was corrected before the successful run. Documentation check: 27 pages, 426 local links, 11 language pairs passed. The expanded four-test run after review also passed (0 failures, 0 ignored).

Independent review 7af34a37 found two guidance gaps, without an authority or idempotency defect. The follow-up compares the original canonical envelope, reports envelope_mismatch explicitly, and limits absence wording to recorded facts. Tests now cover a changed body and a different owner. The review does not count as a second approval of the follow-up.

## Delivery

Source `6d0f01096d109b24096396cd0dce1d939809b1e9` merged into main and pushed. Optimized macOS and Linux builds installed and activated.

- macOS: ad-hoc signature verified; SHA-256 `a3818666049c5185b59b8b28a94cd256926cfe8691b7109ec8df057b0c1ca27c`; daemon build ID `6d0f01096d10`, PID 45405. T3 and its bridge retained their processes.
- Cartae: SHA-256 `68b15bb493f1a374ed58c7e6554d65cc21caee19d61aee9adb4477e48417c78f`; worker PID 4150007, executable path verified. The unrelated daemon retained its process. The previously observed business wrapper had already ended with exit code 0 and a clean result before the worker restart; no preservation of a still-running wrapper is claimed.
- Live MCP verification: an intentionally mismatched source SHA returned the existing error code plus `detail.admission.state=not_recorded`, `task_id=null`. The admission and task tables contained zero rows for this diagnostic key. No provider or business mission started.
- Politique was sent the delivery and field usage guidance; recipient acknowledgment is not claimed.

The earlier four-test run covered the initial patch; the final four tests also cover a different body under the same key and owner isolation. No full-suite or second independent approval is claimed. Existing clients use generic detail rendering, so this daemon-side guidance is visible without restarting T3.
