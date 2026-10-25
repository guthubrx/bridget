# Refusal admission outcome — follow-up 174

2026-10-24. Delivered.

A source mismatch can occur before receipt reservation. Existing guidance required the caller to confirm absence of a task without providing that fact. The maintainer had to read the same database repeatedly.

The V2 refusal handler now reads owner-scoped receipts and tasks under the existing daemon lock and exposes a bounded outcome plus recovery in `detail.admission`. Unknown storage or identity stays unknown. No schema, authority, admission, replay, or automatic execution changes. Existing generic error codes and MCP detail rendering remain unchanged.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/admission-outcome-guidance.md#extrait-1).

Four targeted tests cover an actual handler mismatch before reservation; pending/refused/expired records; an admitted task and missing identity; and an unreadable receipt table. All four passed on Cartae (0 failures, 0 ignored). The first compile found a missing test-helper limit argument; it was corrected before the successful run. Documentation check: 27 pages, 426 local links, 11 language pairs passed. The expanded four-test run after review also passed (0 failures, 0 ignored).

Independent review 7af34a37 found two guidance gaps, without an authority or idempotency defect. The follow-up compares the original canonical envelope, reports envelope_mismatch explicitly, and limits absence wording to recorded facts. Tests now cover a changed body and a different owner. The review does not count as a second approval of the follow-up.

## Delivery

Source `6d0f01096d109b24096396cd0dce1d939809b1e9` merged into main and pushed. Optimized macOS and Linux builds installed and activated.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/admission-outcome-guidance.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/admission-outcome-guidance.md#extrait-3).
