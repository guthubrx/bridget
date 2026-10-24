# Remaining permission candidates — audit 174

Date: 2026-10-24
Scope: PERM-DELIVERY, PERM-ROOT, PERM-ENV, PERM-UUID
Method: static reading of the code paths and existing tests. No Cargo run. No production code changed.

## Verdicts

| Candidate | Verdict | Code change |
|---|---|---|
| PERM-DELIVERY | Confirmed diagnostic defect. No authority bypass. | None. Fix proposed, not coded. |
| PERM-ROOT | Intended boundary. Not a defect. | None. |
| PERM-ENV | Intended behavior. Same OS account is inside the threat model. | None. |
| PERM-UUID | Not confirmed as exploitable. Fix would migrate persisted identities. | None. Options proposed. |

## PERM-DELIVERY

Claim: a remote delivery refused for a live reason is recorded with one generic reason.

Evidence:
- `crates/bridget-daemon/src/daemon/native_child_requests.rs:148` — `valid_delivery` calls `native_delegation::recheck_remote_dispatch(st, task, task).map_err(|_| "revoked")?`. Every error from the live recheck becomes `"revoked"`.
- `crates/bridget-daemon/src/daemon/native_child_requests.rs:858-865` — `tick` persists the `valid_delivery` reason through `invalidate`. The recorded reason is therefore `"revoked"` for any live-check failure.
- `crates/bridget-daemon/src/daemon/native_delegation.rs:7066` — the resume path maps `"revoked"` to `permission_not_inherited`. This file is owned by ROOT and was only read.
- `crates/bridget-daemon/src/daemon/native_child_requests.rs` `emit` — only `ChildRequestAnswer` frames run the remote recheck and `valid_delivery`. Cancel frames only restrict.

Discriminating proof: the existing test `remote162_remise_refuse_fait_natif_change_sans_exiger_parent_absent_ou_t3` in `crates/bridget-daemon/src/daemon/native_child_requests155_tests.rs:633-662`. It asserts that `recheck_remote_dispatch` returns `"permission_not_inherited"` (line 644) and `"permission_source_unavailable"` (line 657). The recheck layer already distinguishes causes. `valid_delivery` discards that distinction.

Authority: the live checks still run and still block delivery. No check is skipped and no permission is granted. The defect is in the recorded cause only.

Proposed fix, not coded:
1. Map each recheck error to its own code (for example quota exhausted, parent fact changed, source unavailable, revoked).
2. Add the new codes to the closed list in `crates/bridget-daemon/src/remote_worker.rs` (around lines 110-140). Otherwise remote peers classify them as `"unknown"`.
3. Update the `"revoked"` mapping in `native_delegation.rs:7066` so that resume keeps its current behavior. This is a ROOT-owned change and needs a hunk proposed to ROOT.
4. Decide how existing records that store `"revoked"` are read. This is a persisted-state question and is not decided here.

T3 reference: `McpSessionRegistry.ts` offers explicit revocation and a liveness TTL. It has no per-delivery reason taxonomy. Equivalent absent.

## PERM-ROOT

Claim: the project root stops at the first ancestor that contains `.git`. A linked worktree, where `.git` is a file, is therefore its own root and does not read the settings of the main checkout.

Evidence:
- `crates/bridget-daemon/src/native_permissions.rs:892-897` — `project_root` returns the first ancestor where `.git` exists.
- `crates/bridget-daemon/src/native_permissions.rs:1264` — the project loop reads `.claude/settings.json` and `.claude/settings.local.json` for each ancestor, then stops at `.git`.
- `crates/bridget-daemon/src/native_permissions149_tests.rs:393` — the existing comment states that `.git` bounds the ancestor walk.

Verdict: intended boundary. Extending the walk to the main repository would let a worktree inherit permission settings it did not declare. That relaxes authority and is out of scope for this audit.

Open point: whether the provider reads project settings for linked worktrees cannot be checked here. No T3 equivalent was found for project-root resolution. This is not a defect claim.

## PERM-ENV

Claim: the agent identity can be set through environment variables or a name file without checking the process marker.

Evidence:
- `crates/bridget-daemon/src/mcp_identity.rs:458-466` — when `expected_instance_id` and a readable, permitted name file are both present, the function returns the identity without reading the `AgentPidMarker`.
- `crates/bridget-daemon/src/mcp_identity.rs:470-490` — the marker check runs on the other path only.

Naming note: the environment variable is `BRIDGET_AGENT_INSTANCE_ID`, not `BRIDGET_INSTANCE_ID`.

Verdict: intended behavior. Whoever can set the environment of the MCP process or write the name file already runs under the same OS account. That account is inside the threat model. No cross-account escalation is shown.

T3 reference: no equivalent variable scheme was found in the read T3 files. Equivalent absent.

## PERM-UUID

Claim: `stable_uuid` uses one namespace for several domains with no separation. Raw identifiers and the prefixes `instance:`, `t3code-message:` and `t3code-reply:` share the same space. A crafted thread identifier could collide with a derived identifier from another domain.

Evidence:
- `crates/bridget-daemon/src/t3code.rs:350-357` — `stable_uuid` builds a v5 UUID on `NAMESPACE_098`, then sets the v4 presentation bits.
- Usages in `crates/bridget-daemon/src/t3code.rs` at lines 161-166, 1416-1417, 1485-1486, 2635, 3608 and the determinism test at 6975-6977.
- Same derivation in `crates/bridget-daemon/src/t3code_mcp.rs` at lines 73-74 and 650-651.

Verdict: not confirmed as exploitable. A collision needs a thread identifier that reaches `stable_uuid` without validation. The producer of that string was not verified in this audit. The threat model is also the same OS account.

Why not fixed: changing the derivation changes persisted identities (thread-state files, agent identifiers, instance identifiers). A silent change of identity is forbidden by the audit scope.

Options for the owner to decide:
1. Validate identifier format at the entry point, before `stable_uuid` is called. No persisted identity changes.
2. Use a separate domain prefix only for identifiers created after the change. Existing identifiers keep their current derivation.

T3 reference: the read T3 schema files use branded non-empty strings. They contain no UUID derivation. Equivalent absent.

## Tests

Not run:
- No Cargo command was run. The consolidated build belongs to ROOT.
- The existing tests cited above were not executed in this session.
- No new test was added. An unexecuted Rust test could break the consolidated build.

Discriminating tests proposed, not written:
- PERM-ROOT: create a temporary main repository with `.git` directory and `.claude/settings.json`, then a linked worktree with `.git` file below it. Assert that `project_root` returns the worktree path.
- PERM-DELIVERY: after the mapping in the fix above, assert that a live quota refusal produces its own code, not `"revoked"`.

No visible UI change was made, so no proof slideshow is required.
