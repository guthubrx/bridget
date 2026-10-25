# Qualification of the four permission candidates

Date: 2026-10-24. Scope: the four unresolved permission hypotheses from audit 174.
The initial independent reading is preserved in the private audit evidence.

| Original candidate | Qualification | Action |
|---|---|---|
| PERM-DELIVERY, same-task proof | No authority bypass: `emit` obtains the remote proof before a positive answer and rechecks the worker epoch and current task. Cancel frames only restrict. | Refute the bypass hypothesis. Fix the separate diagnostic defect below. |
| PERM-ROOT, worktree policy | `.git` bounds project policy lookup. A linked checkout is its own project directory. Reading untracked settings from another checkout would change that authority. | Keep the boundary. Provider behavior outside these read paths is not certified. |
| PERM-ENV, environment instance | Environment resolution selects an identity; it does not authenticate it. `load_credential` checks the private file and both identifiers. `register_auxiliary` checks the live primary connection and its credential. | Keep the supported marker-free path. Same-account hostile processes remain outside this boundary. |
| PERM-UUID, input domain | Cross-role equal strings are possible, but agent, instance and message identifiers are not interchangeable authorities. | Refute the proposed collision-based impersonation; preserve persisted IDs. |

## Delivery diagnostic

`valid_delivery` previously mapped every failure from the live local or remote
check to `revoked`. This included quota exhaustion and missing permission sources.
The new closed mapping preserves `provider_quota_exhausted` and
`permission_source_unavailable`; other verification failures become
`delivery_check_unavailable`. A proven `permission_not_inherited` remains denied.
No check is removed. No response is sent on failure. Historical rows are not
rewritten. `void_reason` is an existing text field; no schema change is needed.
`ChildRequestCancel` contains the request tuple, not this reason. Contrary to the
initial review suggestion, no worker wire enum needs extending.

The synchronous Respond check preserves the distinct codes. A later invalidation leaves the historical decision intact; task status exposes its separate delivery_state and a closed delivery_error. Unavailable checks receive a
constant MCP recovery message: inspect the existing task and pending requests,
restore the relevant source, and do not create a replacement or widen rights.
The fixture `spec174_child_delivery_unavailable_is_not_revoked` exercises a live
remote child, a missing source, explicit denial, an opaque error and a quota.
It also checks that a private error canary does not reach the public diagnostic.

## Identity domains: concrete counterexample to the original assumption

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/remaining-permissions.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/remaining-permissions.md#extrait-2).

## adaptateur de bureau reference and limits

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/remaining-permissions.md#extrait-3).

This is a focused code-path qualification, not a claim that all project policy
combinations or same-account hostile processes are covered. Consolidated test
results and independent final review are recorded in the delivery evidence.

The final review also identified SQL lookup failures and an absent parent route being labeled revoked. They now report delivery_check_unavailable; a proven revocation remains revoked. The late-invalidation fixture records an answer, removes its authority source, and checks the real tick, cancellation frame and status projection.
