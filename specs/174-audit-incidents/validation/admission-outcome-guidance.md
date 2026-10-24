# Refusal admission outcome — follow-up 174

2026-10-24. Delivery in progress.

A source mismatch can occur before receipt reservation. Existing guidance required the caller to confirm absence of a task without providing that fact. The maintainer had to read the same database repeatedly.

The V2 refusal handler now reads owner-scoped receipts and tasks under the existing daemon lock and exposes a bounded outcome plus recovery in `detail.admission`. Unknown storage or identity stays unknown. No schema, authority, admission, replay, or automatic execution changes. Existing generic error codes and MCP detail rendering remain unchanged.

T3 reference: `apps/server/src/vcs/GitVcsDriverCore.ts`, `resolveCommit` and `createWorktree`: resolve the exact Git revision and report an error before workspace creation. T3 has no equivalent to Bridget's remote durable admission receipts. Bridget reuses its existing receipt reader instead of adding a service or another status tool.

Four targeted tests cover an actual handler mismatch before reservation; pending/refused/expired records; an admitted task and missing identity; and an unreadable receipt table. All four passed on Cartae (0 failures, 0 ignored). The first compile found a missing test-helper limit argument; it was corrected before the successful run. Documentation check: 27 pages, 426 local links, 11 language pairs passed. Independent review and optimized delivery remain in progress.
