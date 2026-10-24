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

T3 `packages/contracts/src/baseSchemas.ts` accepts branded non-empty identifiers.
Imported thread identifiers are not restricted to UUIDs. With thread A=`example`
and thread B=`instance:example`, B's agent ID equals A's instance ID. This is an
equality across roles, not an equality of two agent IDs or two instance IDs.

Bridget `t3code_mcp::resolve_binding` supplies the pair derived from one thread.
`mcp_identity::load_credential` validates both fields in the credential file.
`daemon::register_auxiliary` first resolves the agent's route, then checks that
route's credential and instance, and finally the live pair. A value from another
role cannot pass these checks by itself. Revocations also use separate tables:
`native_delegation_revocations.agent_id` and
`native_delegation_grants.owner_instance`. Message IDs are stored in message
records, not looked up as agent authority. Reply IDs additionally carry `t3-`.
Existing tests `spec099_preuve_auxiliaire_valide_bornee_et_revoquee` and
`spec172_binding_refusal_preserves_guards_and_explains_failed_check` exercise
credential, instance and live-route rejection. The opaque identifier integration
test protects imported identifiers. No identifier migration is justified here.

## T3 reference and limits

T3 `apps/server/src/mcp/McpSessionRegistry.ts` binds a random credential to a scope
containing distinct thread and provider-instance fields. Its revocation API is
separate from session liveness. Bridget retains the same separation of identity
selection, credential proof and live authority. T3 has no remote child-answer
wire protocol or equivalent delivery-error mapping to copy.

This is a focused code-path qualification, not a claim that all project policy
combinations or same-account hostile processes are covered. Consolidated test
results and independent final review are recorded in the delivery evidence.

The final review also identified SQL lookup failures and an absent parent route being labeled revoked. They now report delivery_check_unavailable; a proven revocation remains revoked. The late-invalidation fixture records an answer, removes its authority source, and checks the real tick, cancellation frame and status projection.
