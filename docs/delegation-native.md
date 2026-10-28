# Native delegation

[English](delegation-native.md) · [Français](delegation-native.fr.md) · [Documentation](README.md)

Bridget owns its task engine. It creates and stores native tasks through its CLI
and MCP contracts. Manage each existing task through the engine that created it.

## A short workflow

From a registered parent agent:

```sh
bridget delegate --catalogue --json
bridget delegate 'Review the tests and report concrete risks.'
bridget task wait '<task_id>' --timeout 120 --json
bridget task status '<task_id>' --json
bridget task ack '<task_id>' --json
```

Use the official `bridget_auto_delegate` MCP tool for automatic fleet admission.
Let Bridget choose every unspecified machine, engine, connection, model and
effort. Preserve explicit user constraints. `bridget_delegate` is the explicit
native-provider interface; it is not a substitute for an unavailable automatic
admission path.

A shape example for automatic remote admission is below. Replace placeholders
with the real attested source path and full source commit before use:

```json
{
  "request_id": "contract-review-01",
  "task": "Read the contract and report defects with evidence. Do not modify files.",
  "cwd": "/absolute/path/to/project",
  "machine_id": "worker-machine-id",
  "posture": "discovery",
  "workspace": {"strategy": "shared"},
  "expected_base_commit": "<full_source_commit>"
}
```

Choose the source and destination separately. `cwd` is the absolute source
directory on the calling parent's machine. `machine_id` selects the destination;
it does not change what `cwd` means. For example, a parent on macOS targeting a
Linux worker must keep its macOS source path. Bridget maps that path to the worker.

`expected_base_commit` must contain the complete Git object ID: exactly 40
hexadecimal characters (SHA-1). Abbreviated IDs and branch names are refused before
admission. For Shared, resolve HEAD in the source directory. For Worktree, resolve
`workspace.base_ref` there (HEAD when absent). Preserve the recorded request and
its result before correcting a refused call; never change an admitted envelope.

The exact schema and available combinations come from the loaded MCP catalog.
A documented feature is not evidence that an old process has loaded it.

For direct messages, `project_unknown` in `project_warnings` is an advisory,
not a refusal. `in_flight` does not confirm that the child has processed the
message. After a native task has returned its result, use
`bridget_task_continue` for another round only if its status is continuable.

## Identity, permissions and catalog

The parent needs a live attested identity and project. It cannot choose its
identity in tool arguments. An agent launched as plain `codex` is not registered
just because Bridget tools appear in its configuration. Use the wrapper or the
host's supported attestation path.

`bridget_capabilities` describes the current parent and declared providers.
The CLI catalog can also be read by a human without creating an agent:

```sh
bridget delegate --catalogue
bridget delegate --catalogue --machine worker-machine-id --json
```

`--machine` filters catalog reading; `--machine-id` constrains a mission.

| Catalog field | Meaning |
| --- | --- |
| `can_run` | At least one posture passes the current verdict for this parent. |
| `constraints` | Reasons that limit availability; some quota information is advisory. |
| `checks.binary` | Binary check result. |
| `checks.auth:false` | Provider authentication was **not tested**. |
| `quota` | Observed windows, or unknown. |
| `verdict_context` | The attested context and scope used for this view. |
| `cwd_root`, `cwd_scope` | Authorized directory scope, not an arbitrary Git root. |
| `delegations_halted` | Human global admission control. |

The admission verdict remains authoritative. Catalog and admission are comparable
only with the same parent, posture, directory, scope and unchanged state.
A catalog cannot prove that a new remote account has credentials or credit.

Posture omitted means inherit the parent's proved policy. `discovery` restricts
the child to read-only work. `development` permits writing only if the parent
and target can preserve the policy. Omission is not a grant.

- Codex to Codex preserves approval, sandbox and network policy.
- Codex to Claude/compatible write execution requires a compatible full-access
  parent with approval policy `never`; `workspaceWrite` cannot be silently
  widened to full access.
- Claude/compatible inheritance requires matching observable launch/profile
  settings and the supported directory relationship. Changed settings are refused.
- Unsupported policy mappings fail closed with the relevant permission or
  confinement code.

A discovery child may have file-reading tools without a shell. Do not make
`git rev-parse` a mandatory read-only recipe step unless that child actually
has shell access.

`cwd_scope:project` uses the effective authorized root. `root` denotes an
explicit human grant. `same_project` can denote an externally attested project
without a grant; it does not invent a writable root. An unrelated repository
remains outside scope.

Legacy unproved discovery may require `bridget delegate-grant` from a human
terminal. Attested inherited delegation does not require an additional grant.
MCP never creates human grants, and revocation remains authoritative.

## Fleet selection and accounts

Five constraints are independent: engine, model, machine, connection and effort.
A connection is an account/environment on one machine, not a model. Model and
effort names must match the declared catalog. If a provider has no effort control,
do not invent a default such as medium.

`workload` is `general` by default or `compilation` explicitly. There is no
keyword-based workload inference. `project_id` selects an authorized remote
mapping; when omitted, unique mapping resolution and supported automatic
preparation apply.

The routing policy uses admissibility and bounded facts, then optional configured
decision-service/lightweight selection, then deterministic scoring. Decision
levels may only choose admissible combinations. They do not perform the mission
or gain tools, account access or permissions.

Private policy includes `enabled`, `automatic_axes`, `fixed_values`,
`connections`, `model_catalog` and `machine_policies`. Decision budgets are
bounded; stale telemetry stays unknown. Turning off an optional decision service
does not authorize an undeclared provider or silent V2-to-V1 downgrade.

Quota observations are scoped to the actual provider profile/instance.
Included quota at 100% does not necessarily mean Codex cannot run: usable
attested credits and an open spending control can permit it. Unknown credit
does not. Another account's balance cannot authorize this one.

The explicit native interface can accept an ordered `fallback` list of one to
three distinct `{agent_type,model,effort?}` alternatives. It applies only to
documented replaceable pre-admission reasons, not permission widening, workspace
changes or every failure. `substitution` records requested/selected targets,
reason and attempts. Exact replay keeps that choice. No fallback is inferred
after a task has been admitted.

## Request identity and task state

Keep the request key, complete arguments, task ID, child/mission references and
decision receipt. If the response is lost, inspect or replay the **same request**.
A new key can create another mission. Changing the source commit or any other
argument under a known key yields `envelope_mismatch`.

The CLI `delegate` freezes the expected source commit itself. It does not accept
`--expected-base-commit`. MCP automatic admission and the technical
`auto-delegate` path accept `expected_base_commit` explicitly.

Native states include `queued`, `starting`, `mission_pending`, `working`,
`waiting_for_children`, `cancelling`, `result_available`, `failed` and
`cancelled`. Read `work_state`, `error`, `error_message` and
`error_recovery` together with `status`. Historical preparation failures can
retain a queued status while the projected work state is failed.

A transport timeout is not proof that the provider never received a mission.
In particular, an uncertain remote Release can later reconcile to a real result.
Do not cancel, duplicate or replace it solely from the initial error text.
The message follows the evidence for that same task: unknown start, recorded
delivery attempt, available result, or pending/completed cleanup. The initial transport code
is retained; it does not override a later result.

A published result is immutable for its round. A turn ending alone does not
publish one. Active descendants can delay a parent's result. A result does not
mean commits were merged, deployment succeeded or the child process is stopped.

## Workspaces and exact Git revisions

Omitted `workspace` keeps the shared-directory behavior.
`{"strategy":"shared"}` requests it explicitly; these are different envelopes.

A separate worktree can use:

```json
{"strategy":"worktree","base_ref":"<full_source_commit>","branch":"review/contract-01"}
```

The base defaults to HEAD if omitted. A requested branch must be new; otherwise
the default is `bridget/<task-id-prefix>`. The child checkout is created under
`.worktrees/<task-id-prefix>`, preserving an authorized subdirectory offset.

For Shared Git, `expected_base_commit` is the source checkout HEAD.
For Worktree, it is the resolved `workspace.base_ref` commit, which can differ
from the parent's current branch. A same-named branch on the remote host cannot
replace the frozen SHA. Uncommitted files, ignored files and uninitialized
submodules are not copied.

Creation prerequisites are checked **on the target**. They include a supported
Git version (2.44+ for the documented worktree path), a non-bare repository,
safe paths and a repository-local ignore rule for `.worktrees/`.
A global ignore alone does not qualify. Unsupported executable filters,
configuration includes, partial-clone/LFS arrangements or target
`worktreeConfig` can be refused. Source-side `worktreeConfig` alone does
not establish that the remote target is unsupported.

The catalog's supported posture/provider/workspace combination is authoritative.
Discovery worktree support must not be inferred for every provider pairing.
A workspace-write child may modify files but lack permission to commit to the
common Git directory.

### Automatic remote preparation

On an installed SSH federation, the helper can prepare a missing repository
at the requested committed revision without requiring publication to GitHub.
It needs Python and Git on both machines. It reuses local SSH keys without
copying them.

Mappings are published after preparation, reread and frozen in admission.
The worker rechecks identity, permissions, Git state and availability.
A linked checkout uses the common repository for project identity and its real
checkout for revision/preparation. It must not advance the main checkout merely
because a child worktree uses another commit.

Transfers use a Git bundle. For diverged histories, a locally known common
ancestor is usable as an exclusion only after remote presence is confirmed.
The receiver verifies prerequisites again before import. If no usable common
base is established, a full-history bundle can still be necessary.

An existing checkout is not overwritten by transfer. A clean, unoccupied Shared
checkout can fast-forward during worker Prepare. Dirty, busy or divergent
checkouts are refused. A missing commit can be imported from an already
configured local mirror; unavailable objects remain a prerequisite failure.

Automatic V2 admission has a 600-second client budget for preparation. Helper
timeouts identify their operation, such as SSH, bundle export/verification/import
or worktree creation. A timed-out creation with an incomplete receipt must be
reconciled before retry: absence of a task ID is not proof that nothing happened.

### Workspace receipts and cleanup

The receipt records base, branch, path and state. Result inspection adds HEAD,
commit count, recent commits (up to 50), dirty state and whether HEAD is still
on the expected branch. A failed inspection leaves facts unavailable; it does
not invent a clean checkout.

`kept` means files and branch remain. It does not mean a task is active.
A new round refreshes current workspace facts; older published results remain
unchanged. `shared_writers` warns about another active writer without blocking it.

Bridget does not commit, push, merge divergent branches, rebase, create a PR or
delete the user's worktree. Integrate and preserve authorized work before
removing a clean, unused checkout and deleting a merged branch. Do not force
past a dirty/unmerged refusal.

| Refusal | Useful next step |
| --- | --- |
| `workspace_git_unsupported` | Check the target Git version. |
| `workspace_not_git` / `workspace_repo_unsupported` | Inspect the actual target repository and supported configuration. |
| `workspace_base_unavailable` | Establish that the exact commit is available. |
| `workspace_branch_exists` | Choose a new branch only for an authorized new request. |
| `workspace_dir_not_ignored` | Add a repository-local ignore for `.worktrees/`. |
| `workspace_dirty` / `workspace_busy` / `workspace_diverged` | Preserve existing work; resolve that concrete condition. |
| `workspace_limit` | Review retained worktrees; do not delete active work. |
| `project_mapping_unknown` / `ambiguous` / `mismatch` | Inspect the attested source and target mapping. |
| `project_preparation_*_timeout` | Inspect the named phase and durable preparation receipt. |

The historical worktree path reserves at most 32 distinct paths, including
in-progress reservations.

## Deliver work across machines

1. Choose the authorized repository, source commit and target. Supply the exact SHA.
2. Let supported preparation establish that revision and report its actual checkout.
3. Let the child commit/push only its assigned branch when the mandate permits it.
4. Have one integrator inspect results, fetch and integrate the child branch,
   then push and verify the published SHA.
5. Update other durable clean copies with a verified fast-forward. Preserve
   dirty or divergent copies instead of forcing them.
6. Clean only resources confirmed unused after work and evidence are retained.

[Adapter contract details](adapters/t3.md#note-38).

## Wait, read and acknowledge

```json
{"name":"bridget_task_wait","arguments":{"task_ids":["<task_id>"],"wait_timeout":600}}
```

A task wait accepts 1–16 distinct owned task IDs and an integer timeout of
1–3600 seconds (default 600). An unknown, purged or foreign task refuses the
whole call. The result separates `ready` and `pending` in input order and
reports `timed_out` and `waited_secs`.

Wait returns for a terminal result or an actionable parent question.
A timed-out wait **does not cancel the task**. Ready entries omit the full
result; read it through `bridget_task_status` or the result message.

A daemon shutdown can interrupt a read with `daemon_unreachable`. After an
unacknowledged mutation, the transport keeps `outcome_unknown`: the command may
have run. Wait for the connection to return, then read `bridget_task_status`
with the same `task_id`. Without a task ID, retain the admission `request_id`
and all arguments to recover its outcome. Do not create a replacement request
while the original outcome is unknown. Existing bounded admission recovery
keeps the same key.

With client196, a transport failure from `bridget_task_wait` triggers one new
read of the same task IDs, with a 1-second wait. Identity, ownership and permission
checks run again. Success includes `read_recovery` with
`cause: daemon_read_unavailable`, `initial_wait_secs` and `recheck_wait_secs: 1`.
This observation can be newer than the first read. Native denials and host
cancellation do not trigger recovery. A second transport failure returns
`task_wait_read_unavailable`: read `bridget_task_status` for the same tasks, then
report the code, time and task IDs to the maintainer if it persists. The exact
latency cause remains unknown until measured. No mission, round or cancellation
is replayed. Other tools are unchanged. The running client must have loaded this
code; installing a file does not reload an old process.

For `t3_session_unavailable` with `binding_transport_unavailable`, `primary_absent`
or `primary_not_live`, wait for the thread's Bridget link to reconnect and retry
the read. A `bridget_session` attestation alone does not prove this link is back.
If the refusal persists, report its code and retain the request. Do not change
permissions or recreate the mission. Credential and instance mismatches still
refuse without this transient recovery advice.

Status includes a bounded `summary` (2048 UTF-8 bytes), `summary_truncated`,
`result_bytes`, the detail-tool reference and completion `mailbox`.

| Mailbox state | Fact |
| --- | --- |
| `pending` | Result/failure notice prepared, not dispatched. |
| `claimed` | Dispatch engaged; delivery not proved. |
| `delivered` | Transport acknowledgment proved. |
| `acknowledged` | Parent explicitly declared reading it. |

```json
{"name":"bridget_task_ack","arguments":{"task_ids":["<task_id>"]}}
```

ACK accepts terminal results/failures, not cancellations or unfinished tasks.
It writes the read declaration once and returns `acknowledged` / `already`.
It does not change message delivery, cleanup or retention. Reading and waiting
alone write no acknowledgment. A result can appear both in a wait response and
as its normal message.

For a tracked message sent with `reply:true`:

```json
{"name":"bridget_request_wait","arguments":{"request_id":"<send_request_id>","wait_timeout":600}}
```

Only the sender can wait. `open` plus `timed_out_wait:true` means the wait
expired while the request stays open. `timed_out` means the request's response
deadline passed. `answered` can contain a 1024-byte summary; an older/unavailable
reply can instead report `reply_unavailable`. `cancelled` reflects explicit
request cancellation. Wait never resends the request.

There are at most four waits per MCP process and 64 in the daemon. The
`wait_v1` capability is required. Keep the chosen wait below the host tool
timeout. After a daemon/connection failure, repeat the read-only wait rather
than resubmitting work.

## Continue the same child

Request `keep_alive_secs` at admission (60–7200) to retain a child after its
result. `idle_deadline_at` begins when that result is sent. A read does not
extend it.

```json
{
  "name":"bridget_task_continue",
  "arguments":{
    "task_id":"<existing_task_id>",
    "request_id":"contract-correction-round-2",
    "task":"Correct the two reported defects and return verification evidence.",
    "brief":{
      "objective":"Correct the reviewed contract",
      "summary":"The first round identified two defects. Apply only those corrections."
    }
  }
}
```

Use a new request key for a **new round**, then keep it and its arguments
unchanged on retry. At most eight rounds are allowed. Child, instance,
directory, model and posture remain. Each round has a distinct mission/result.
The current result becomes empty when the next round opens; prior results
remain in their messages.

`continuable:true` is not a promise that a later call will pass fresh checks.
The guard, child presence, rights, pause and round limit are rechecked.
Compatible the desktop adapter provider-session renewal does not change task ownership; a
different permission policy still fails. A terminated execution cannot be
revived merely by changing its request key.

[Adapter contract details](adapters/t3.md#note-39).

`brief` requires objective and summary, with optional findings, decisions,
questions, next steps, references and limits. The rendered brief is bounded
to 16,384 bytes without truncation. Cited sources are not read or verified by
Bridget. Send the useful correction context rather than an entire transcript.

After the final result, cancel that task to close its guard. The result and
workspace remain. Guard expiry, parent cleanup or daemon restart can also end
continuation. A retained child occupies capacity until its stop is confirmed.

Common refusals: `keep_alive_not_enabled`, `keep_alive_ended`,
`round_in_progress`, `result_delivery_pending`, `round_limit`,
`remote_execution_ended`, `remote_execution_unconfirmed`,
`permission_not_inherited`. Use their specific recovery message; do not
interpret all of them as a request for broader rights.

### After the eight-round limit

`round_limit` prevents another round on that task. `continuable:true` describes
the retained guard; it does not override the limit.

Preserve the result, commits, untracked and ignored files, and useful packages.
The owning parent then closes the guard with `bridget_task_cancel`. On
`result_available`, the result and worktree remain. Check status after closure:
`continuable:false` alone does not prove that execution stopped.
`task_active` or `stop_unconfirmed` require stop confirmation;
`report_unavailable` requires inspection before proceeding. Do not remove a
directory to pass the limit.

After confirmed closure, a follow-up within the existing mandate uses a new
admission, request key and child. Its brief cites the previous task and result.
To start a new worktree from retained commits, first make the full commit
available in the parent's source repository. Use the same SHA for
`expected_base_commit` and `workspace.base_ref`. This neither moves the old
worktree nor copies its untracked or ignored files.

Do not reset the round counter or reuse the child's identity. A new key cannot
reopen the refused ninth round. If the scope changes or stop/preservation is
unconfirmed, obtain the owner's or user's decision before a new mission.

## Child questions and approvals

`child_requests` defaults to `deny`. `forward` supports only declared
Codex request forms and is frozen in the task envelope. It does not enlarge
permissions or make an approval-policy `never` provider ask for approval.

A structured, non-secret user-input request goes to the direct parent.
Supported command approvals go to an attested human within the frozen policy.
The parent cannot approve actions. Unsupported file changes, permission grants,
stdin requests or third-party MCP forms are refused.

```json
{
  "name":"bridget_task_respond",
  "arguments":{
    "task_id":"<task_id>",
    "request_id":"<child_question_id>",
    "decision":"accept",
    "answer":{"<question_id>":["<answer>"]}
  }
}
```

Answers map question IDs to string arrays and are limited to 8 KiB.
`decline` and `cancel` omit `answer`. The provider envelope is generated by
the transport; do not submit it in place of Bridget's answer object.

Parent questions have up to 600 seconds; human decisions up to 3600 seconds.
Neither may exceed the mission deadline minus 30 seconds. Too little remaining
time returns `deadline_too_close`. Status exposes pending questions and recent
outcomes; a decision recorded does not prove it was delivered.

During pause, responding is refused. An invalidated approval is not replayed
after resume. A quoted `cwd` is unverified display data, not proof of confinement.

## Deadlines, pause, cancellation and retention

`reply_timeout` is an integer from 60 to 86,400 seconds, default 3600.
Omission and an explicit default are different envelopes. A child's explicit
deadline cannot exceed its parent's effective remaining time.

The mission deadline begins with admitted mission delivery, not provider startup.
The transport receives this native mission deadline, including when it exceeds the provider's default turn timeout. Re-reading a delivery does not grant a fresh time budget. Ordinary messages keep their provider timeout.

Business time excludes qualifying pause/quota intervals; a caller's wait budget
continues. A raw `mission_deadline_at` alone is therefore not always the
effective remaining duration.

During pause, new admissions are refused but exact replays, catalog, status and
waits remain available. Tasks can report `suspended:true` without changing
native state. A previously submitted startup can still reach its startup
deadline. Results already acquired and cancellation remain possible.

Ordinary crash recovery retains task/mission identity and permits at most two
recorded resume decisions. It does not make an ended guard continuable or
erase uncertain delivery.

After a deadline failure, status includes `error_message` and `error_recovery`.
A `failed` task is not continuable, even if keep-alive was requested. An exact
admission replay reads that task; it does not restart it. Wait for stop confirmation
and the workspace report, retain commits and partial outputs, and check for detached
business processes. A separately authorized recovery mission needs a new request ID
and an explicit brief to reuse those outputs and finish only the remaining work.
Bridget does not roll back effects or restart production automatically.

Cancellation checks that all descendants are managed and stoppable before
stopping them, descendants first. An unverifiable precheck stops none.
Confirmed stops remain durable if a later stop fails. A task can stay
`cancelling` with `native_cancel_unverifiable`; that is not confirmed cleanup.

For a cancelled remote launch that has no command receipt, a compatible master
and worker can close the exact command with `FenceUnstarted` after `Cancel`
returns `UnknownExecution`. This requires a cancelling task with no mission
issued and a known command, instance, generation, placement and deadline. Only
`StartFenced` confirms durable closure against a late Start; no provider is
started. An absent receipt, lost reply or old worker leaves cleanup unconfirmed.
Read `error_cause` and `error_recovery`; retain the task, request key and outputs.
If `worker_start_recovery_v1` is unavailable, ask the maintainer to qualify and
load compatible binaries through admitted maintenance. Do not force a restart,
edit storage or create a replacement mission.


Capacity limits are 128 active tasks and 16 active children per instance.
Retention is at most 4096 tasks, purged by complete root trees. Age maintenance
targets trees older than seven days, up to 128 per hourly pass; capacity pressure
can remove eligible older trees sooner.

Active tasks, incomplete cleanup, guards, missing result-delivery acknowledgments
and live fleet instances prevent purge. A read ACK is not a delivery ACK.
After purge, status returns `task_unavailable`; the request key returns
`request_expired` for the retained 30-day tombstone period.

Some definitive admission refusals also persist for 30 days:
`cwd_outside_parent_project`, `cwd_unavailable`, `model_unavailable`,
`effort_unavailable`, `agent_type_unknown`. Exact replay returns the
recorded refusal. Pause and transient permission failures have distinct rules.
Never infer no admission solely from a missing first response.

## Optional usage-limit recovery

`resume_on_limit:true` opts in; absent and false remain off and are different
envelopes. A qualifying observed exhausted window must reset in the future,
within six hours. At most two quota rounds are retained. A global Codex limit
flag without supporting window facts is insufficient.

Status reports `quota_wait` with `waiting` or `relaunching`.
The same task, mission, provider, model, effort, definition and inherited
permissions are retained. Fallback is not rerun. A dispatch reservation is
not an ACK, and an ACK is not business success.

The quota wait overlaps pause intervals without double-counting. It stops
freezing child business time at reset plus 30 seconds; parent time is not
extended. Cancellation prevents recovery. Final errors include `usage_limit`,
`native_relaunch_failed` and `native_refusal_uncorrelated`.
Temporal quota correlation is not proof that quota caused the failure;
repeating work can repeat external effects.

## Human controls

```sh
bridget control delegations
bridget control delegations --state all --limit 100 --json
bridget control delegations cancel '<task_id>'
bridget control halt-delegations --reason 'Human review'
bridget control halt-delegations --reason 'Stop new and active work' --cancel-active
bridget control resume-delegations
bridget control status
```

These use attested human authority; mutations require an interactive terminal.
An agent cannot gain it by declaring a human name. Halting admissions is
durable and distinct from pause. Without `--cancel-active`, existing tasks
remain. Resuming admissions does not lift pause.

A cancellation count records targeted trees, not stopped processes.
Command replay returns its prior receipt without cancelling later tasks.
Catalog/status/cancellation stay available while admission is halted.

Discovery children can use only their allowed read/own-state MCP surface.
Sending, publishing, thread writes and further delegation remain refused.
A shell or another identity is not a workaround.

## Wakes and missed facts

Check the loaded capabilities before use. `bridget_wake` supports
`schedule`, `list` and `cancel`. Targets are self, an agent UUID or a thread
UUID. Intervals start at 60 seconds; calendar schedules use daemon-local
`HH:MM` and distinct weekdays 1–7. Labels are at most 2048 UTF-8 bytes.

Schedule creation is not idempotent: if its response is lost, list before
creating another. Limits are 16 schedules per owner and 128 total, including
retained cancelled entries. Scheduling self can be own-state work for a
discovery child; targeting an agent/thread remains a send.

CLI one-shot wakes use an explicit stable ID on retry. They do not launch an
offline model. Prepared/deferred states do not prove delivery. Unknown delivery
is not automatically resent. Thread receipt proves a contribution, not reading.

Pause delays future scheduling; an interval catches up only its latest due
occurrence. A fixed time more than 600 seconds late is skipped. Nonexistent
local times are skipped; repeated times use the first matching UTC instant
once per calendar date. Delegation halt does not cancel wake schedules.

`bridget_events` with `action:"missed"` reads retained facts using optional
`after_seq` and `limit`. It neither deletes facts nor advances acknowledgment.
Retention is bounded (8192 global facts, 64 per subscription, seven days);
oversized facts and evictions produce explicit gaps rather than invented text.

Return summaries aggregate counts/types/time windows and gaps, not full private
event bodies. Only a correlated delivery ACK advances a prepared summary's
threshold. Facts never observed cannot be reconstructed.

## Diagnose a refusal without duplicating work

Keep the complete receipt outside public Git. Distinguish:

- **Before admission:** no task ID, but a lost response can still leave an
  unresolved durable request.
- **Preparation:** missing Git objects, unsafe/dirty checkout, mapping or storage
  failure. `StoreUnavailable` alone does not prove disk full.
- **Startup/reservation:** an admitted task may exist without a delivered mission.
- **Release:** a lost acknowledgment can coexist with provider execution.
- **Continuation:** expired guard, ended execution and changed permissions are
  separate conditions.

Use status and the recorded recovery instruction. Preserve worktrees and unknown
outcomes. Do not manufacture another mission, change provider or widen rights
to make a refusal disappear.

### Worktree count limit

A candidate refusal with `code: workspace_limit` means Bridget reached its limit
of 128 registered Git worktrees under the target checkout's `.worktrees` directory.
Retained and nested worktrees count too. This is not a disk-space diagnosis or
a claim that the application repository has uncommitted changes.

The refusal includes `message` and `recovery` in `detail.reasons[]`. Inspect completed missions and
preserve their results before removing unused worktrees. Do not delete active
work or change the request key, base commit or provider to bypass the check.
After verified cleanup, replay the same envelope; admission is re-evaluated.
Bridget does not clean or replay automatically. Older peers retain the generic
`workspace_unavailable` response.

#### Cleanup routine

Plan cleanup before the project reaches 96 worktrees. The task owner first closes
the completed task, then integrates or preserves its commits. Archive generated
outputs and evidence, including ignored files: a clean Git status alone is not
enough. Recheck task activity and Git state immediately before each removal.
Use `git worktree remove` without `--force` and retain the branch. Keep active,
modified, locked or unmerged worktrees, and parents containing other worktrees.
This is an operator routine; Bridget installs no periodic deletion job.

### Source Git preflight failure

`workspace_base_preflight_failed` reports a failed source check in shared mode
or when no workspace mode is supplied. Its `detail.message` and `detail.recovery`
explain the source/destination distinction without exposing raw system or Git
errors. On the parent's machine, check that `cwd` exists and is readable, then
run `git rev-parse --show-toplevel` and `git rev-parse HEAD` there. A successful
SSH check on the destination does not validate this source. Correct the path or
Git access if this fails. In shared mode, the source HEAD must equal
`expected_base_commit`. Keep any explicitly required shared mode; do not move
an active checkout or substitute another SHA to pass the check. An authorized
worktree review can instead pin the exact SHA in both base fields.

This technical refusal does not close an ongoing admission or prove that no
task exists. Follow any known task. An identical retry keeps the same key and
all parameters; `admission_pending` means the request is still pending. Before
using a new key with corrected parameters, confirm that neither a task nor an
ongoing admission remains for the previous request. Do not create a new request
while the outcome is unknown.

For `workspace_preflight_technical`, `detail.message` and `detail.recovery` explain the next checks. The `cwd` argument identifies the source directory on the calling parent’s machine, not the destination directory. Check the directory, Git access and the requested reference. Reconcile the existing request before retrying; a technical error does not prove that no task exists. Raw Git errors are not exposed in this response.
Known branch-exists and worktree-limit causes receive their own recovery advice, while the outer error code stays unchanged.


### Missing or mismatched source commit

`workspace_base_unavailable` means Bridget could not resolve the requested base.
Check Git and the exact commit in `cwd` on the **parent's machine**. Availability
on the destination alone is insufficient. If the source lacks the commit,
identify the configured remote and branch containing it, then fetch that branch
into the source repository. Verify the exact SHA again. Do not reset, pull a new
base into the working branch, or change `expected_base_commit` to bypass refusal.

`expected_base_commit_mismatch` means the resolved base differs from the expected
commit. For a frozen worktree review, set `workspace.base_ref` and
`expected_base_commit` to the same full SHA. A moving `main` then cannot change
the requested base. Shared mode instead checks the source HEAD.

Follow any existing task with `bridget_task_status`. If none is known, after
fixing the source perform at most one identical retry with the same key and all
parameters. This retrieves the existing outcome or retries admission; it does
not reopen a recorded refusal. Once a durable refusal and absence of a task are
confirmed, use a new key after correction. Changed parameters also require a
new key after confirming the previous request created no task. Repeated refusal
alone does not prove either durability or absence of a task. Stop retrying if the
outcome remains unknown. These are source checks, not worktree capacity
failures. Bridget does not fetch from the network or choose another base for you.

### Worktree lifecycle and capacity

A completed round does not close its task or remove its worktree. If you need
another round, continue the existing task. When the work is finished, read and
acknowledge its result, close that same task, then inspect the retained work.

Task status adds `workspace.retention` with `code`, `message` and `recovery`.
These fields describe the task and its recorded Git report; they are not a fresh
filesystem scan or permission to remove a directory.

| Code | Meaning |
|---|---|
| `task_active` | The task is running or can still continue. |
| `stop_unconfirmed` | Process cleanup has not been confirmed. |
| `report_unavailable` | The recorded Git facts are missing or incomplete. |
| `changes_present` | The report contains uncommitted changes. |
| `branch_changed` | The reported HEAD is not on the task's expected branch. |
| `commits_to_review` | Commits exist beyond the base; integration must be checked. |
| `review_required` | Check current files, ignored evidence, integration and other users. |

A clean report, an acknowledgement or a zero commit count is never a deletion
approval. Files ignored by Git, other threads and later edits may still matter.

From the Bridget source checkout, inspect a chosen repository without deleting
anything:

```sh
python3 scripts/build.py worktrees --repo /absolute/path/to/project --base-ref main
python3 scripts/build.py worktrees --repo /absolute/path/to/project --base-ref main --json
```

The report separates the full Git register from worktrees under the chosen
directory's `.worktrees` folder. Only the latter is the capacity scope shown here.
The warning threshold is 96 and the limit is 128. This observation counts Git
registrations, not in-flight creation reservations or unregistered retained
directories; admission may therefore observe a larger total. Run it on the
destination machine for destination capacity. An unavailable observation is not
a count of zero.

A `candidat_a_examiner` still needs task/thread ownership checks and a fresh inspection
before removal. Preserve active, modified, ignored, locked, nested and unmerged
work. An incomplete process inventory keeps candidates protected. Use Git's
normal removal without `--force`, after archiving evidence; retain the branch.
The inspection command does not create a cleanup schedule or change admission.

Choose `{"strategy":"shared"}` explicitly when the job can safely read the
selected checkout at the required base without isolation. Choose
`{"strategy":"worktree"}` when edits must be isolated or the job needs a separate
checkout. A failed admission does not authorize switching strategies or changing
the source SHA. Existing paths remain unchanged, including nested worktrees.

#### Bounded process observation (176)

Each inventory shares a 30-second command budget. Captures are limited to 8 MiB for process observation and 1 MiB per Git command. A register above 4,096 entries is rejected as unavailable; it is never reported as a smaller count. Entries not examined before the budget expires remain protected as `lecture_inconnue`. The JSON `observation` object reports limits, elapsed time, Git calls and incomplete checks. These are command and capture limits, not an operating-system memory quota. Filesystem operations and final output can add time beyond the command budget.

On Linux, observation uses the existing non-interactive authorization for `sudo -n -- /usr/bin/lsof`. It changes no permissions and asks for no password. Without that access, observation remains unknown. On macOS it uses existing local `lsof` access. Both paths reject incomplete access and unknown record formats. The calling process is included.

A Linux mount warning requires a second bounded observation. Every file on an exempt mount, unknown record or exemption annotation still blocks removal. Overlapping bind aliases, nested mounts, ambiguous scope mounts and a changed mount table also block removal. A different device alone is never proof that a mount cannot expose a project file.

Git base and commit facts are reused only within one pass. Working-tree status is read separately, including ignored and untracked files. No ownership, task closure or exclusive filesystem lock is inferred. After a timeout, the script requests TERM for its own command group and reports an unconfirmed stop when applicable. It never sends KILL.

The cache cleaner excludes only its own `.cargo-lock` descriptor while holding that lock. Other files and other processes remain visible. A scope containing non-ASCII or non-printable characters, a backslash or a caret is reported as unknown because lsof can escape its name. Process observations remain snapshots: a process can open a file after the check.

### Child answer verification unavailable

A child-answer refusal with `permission_source_unavailable` or `delivery_check_unavailable` means that Bridget could not verify delivery authority. It does not establish a revocation. No positive answer was delivered. Read the existing task with `bridget_task_status`, restore the relevant permission source or configuration, and answer only a still-open request. Do not create a replacement task or widen permissions. `provider_quota_exhausted` remains a separate quota refusal.

A decision recorded as `answered_by_parent` is not proof of delivery. Read `recent_requests[].delivery_state` separately. If a later check invalidates the answer, `delivery_state` is `void` and `delivery_error` gives a closed diagnostic code. The recorded decision stays unchanged. A generic `delivery_invalidated` hides any private provider detail. Do not repeat an answered request or create a replacement task.

### Admission state included in a refusal

Automatic V2 admission refusals include `detail.admission` when the request reaches the admission handler. It contains `request_id`, `state`, `task_id` (or null), `observed_at` (Unix seconds), and `recovery`. This is an owner-scoped observation of the existing records, not a new reservation. Read this state together with the cause of refusal.

- `not_recorded`: no task, in-progress receipt, stored refusal, or tombstone was found for that key. Bridget supplies the confirmation; no maintainer lookup is needed. Correct the cause and replay the same key and parameters. Before changing parameters and using a new key, wait for every previously submitted call with this key to finish: a concurrent call can still record an admission. Do not change `expected_base_commit` without authorization.
- `pending`: the existing admission can still create a task. Keep the same key and parameters; do not create another request.
- `admitted`: follow the supplied task ID.
- `refused`: a terminal refusal without a task is stored. Identical replay does not reopen it; use a new key only after the authorized correction.
- `envelope_mismatch`: this key belongs to a different request body. The new body was not admitted; read the outcome using the original envelope.
- `expired`: the previous record has expired. This does not prove that no work happened.
- `unknown`: identity, storage, or the legacy record could not establish the outcome. Preserve the key and parameters; do not infer absence.

The observation does not cover a concurrent call submitted later. Transport failures before a daemon response and early attestation failures may have no `admission` field; absence of this field is not proof of absence of work. No automatic retry, cancellation, or permission change is introduced.

An explicit V2 replay can resume a `chosen` admission interrupted after its decision, within the same coordinator epoch. It preserves the recorded envelope and choice, then rechecks the Git source, permissions and target. An ongoing preparation for the same owner and key returns `admission_pending`. Replaying `pending` or `decision_started` does not start a new decision. An admitted task and a durable refusal remain unchanged. Maintenance remains blocked until the admission is settled.

The validated choice is stored as `chosen` before its revalidation probe. A temporary probe refusal neither reruns the decision maker nor loses the selected effort. After the cause is corrected, an identical replay resumes that choice only; source, permissions and target must still pass before task creation. A terminal refusal is stored as `refused` and identical replay does not reopen it. Reading a legacy receipt already stuck in `decision_started` does not repair it: ask the maintainer to check the receipt and loaded binary, without editing the store or forcing a restart.

## Réparer le dossier déplacé d'une tâche — contrat185

Ce parcours exige la capacité `task_workspace_repair_v1` du maître,
`workspace_repair_v1` du worker et un wrapper compatible. Il ne constitue pas
une preuve d'activation sur une installation existante.

Le parent propriétaire utilise `bridget_task_repair_workspace` avec `task_id`,
`request_id`, `target_cwd` et `expected_head` (SHA complet du HEAD conservé).
`target_cwd` désigne la racine réelle du worktree sur la machine d'exécution.
Le worker vérifie le dépôt, sa branche et le décalage du dossier de travail.
Le déplacement doit déjà être terminé ; Bridget ne déplace ni ne supprime les fichiers métier.

La tâche doit être gardée et au repos après remise du résultat. La réparation
ne crée aucun tour métier. Elle ferme les remises, vérifie les droits, arrête le
seul fournisseur détenu sous borne et reprend sa conversation native attestée.
Un arrêt incertain, un autre fil ou une politique différente interdit la suite.
Les canons de naissance et l'historique restent conservés.

Commande équivalente :
```sh
bridget task repair-workspace TASK_ID /srv/projets/exemple/dossiers/tache --expected-head SHA_COMPLET --request-id CLE_STABLE --json
```

Lire `workspace_repair.phase` et `workspace_repair.code` au statut. Un résultat
`published` permet une continuation distincte. Une phase en cours exige de
conserver la même clé et tous les paramètres pour réconcilier l'opération.
Un refus enregistré se relit sans nouvel effet. Après un refus confirmé sans
effet, une décision corrigée peut utiliser une nouvelle clé sur la même tâche.
Il n'y a aucun remplacement de mission.

Le contrôle ordinaire de continuation ne suit pas un lien symbolique pour
contourner un déplacement. Un ancien wrapper peut refuser avec
`workspace_repair_child_upgrade_required` : installer un binaire ne met pas à
jour son code chargé. Une migration ciblée admise reste alors nécessaire ;
aucun redémarrage de service ni changement direct de base n'est implicite.

La première version accepte une migration publiée par génération et huit
tentatives au plus. Les adaptations non qualifiées refusent sans démarrer une
conversation neuve. Sans inventaire avant le déplacement, l'exhaustivité de
cette copie antérieure reste inconnue ; la réparation conserve les fichiers qu'elle vérifie.


## Local worker diagnostic (188)

`bridget worker diagnose` reuses the worker permission recheck reads. A verified candidate can run this command without restarting any service. It creates no worker, provider, slot or task and dispatches no mission. Master state remains separate.

Supply the installed worker’s seven options (`--machine-id`, `--worker-id`, `--provisioning-revision`, `--credential-file`, `--state-root`, `--registry-file`, `--master-socket`) and its complete admitted tuple (`--command-id`, `--task-id`, `--instance-id`, `--generation`). Run as the private files’ owner, using the live worker’s environment. Do not print its variables or credential. Delegated CLI restrictions still apply.

The output has `scope: local_read_only_recheck`, `authoritative: false`, `check`, `stage`, `reason`, a closed `cause`, `recovery` and `human_action`. A local pass is neither a live worker attestation nor proof of mission dispatch. Different environments may change the verdict. A file changed during the read yields `diagnostic_observation_changed`: wait for the observed write to finish and read the same tuple again. A missing receipt remains unknown and does not prove process termination. Preserve the task, original request key and all files. Use native repair or request an owner decision if restoring the refused source exceeds the authorized scope.

An unbound local reader only observes files under their owning UID: no delegated lineage marker or explicit identity selector may be present. Invalid namespace configuration or ambiguous identity refuses before worker files are read. Registry input is limited to 16 MiB.

Diagnostic188 requires Linux and a live worker. It binds the supplied options to the kernel owner of the store lock and its process birth, then observes its source environment in memory. It never changes the reader environment or adopts an identity. The result remains a non-authoritative local read, distinct from mission delivery. Ambiguous registry context, delegated lineage or unstable observations close the command.

The reader PATH must also equal the verified worker PATH: the existing Git controller uses the reader process tools. A mismatch returns diagnostic_tool_environment_mismatch before the business check. Only this tool setting may be selected explicitly; never copy identity variables or secrets.

A `resource_saturated` refusal exposes the measurements and limits actually evaluated in `detail.capacity`, when retained. This evidence describes the refusal time, never current capacity. The durable code is unchanged; evidence is added atomically without replacing the initial snapshot. Legacy refusals explicitly report missing evidence. Confirm a durable refusal without a task and fresh capacity on the same machine before a new key; do not stop other owners’ tasks. An MCP preflight attestation refusal reports `stage: caller_scope`: that call has not dispatched the business operation. Unverified checks remain `unknown`, without task context before authorization.

A `kind:candidate/code:resource_limit` reason means that the worker inbound request queue was full when it rejected the probe. `detail.probe_capacity` distinguishes this historical fact from unknown current state. It proves neither low memory/disk nor an authentication failure. `kind:worker/code:resource_limit` remains distinct and does not establish this cause. For `not_recorded`, confirm that no concurrent call remains, then replay only the same key and envelope after native operations drain. No implicit restart, cancellation of other owners’ tasks or change of target.


## Last master dispatch refusal (190)

For a remote `mission_pending` task whose mission has not yet been issued, an authorized owner status may contain `detail.dispatch_refusal`. This field retains the last refusal actually observed by the existing dispatch checks. It contains the stage, a closed code, observation time and recovery guidance. When the worker answered, attached facts describe its phase, native readiness and the presence or match of expected proofs. Policy values, secrets and business messages are not exposed.

`scope: last_observed_master_refusal` and `authorizes_dispatch: false` are explicit: this historical fact authorizes no dispatch and may no longer describe current state. The volatile cache holds at most128 tasks, bound to the owner, child instance, mission, round and placement. After restart, eviction or an identity change, `state: unobserved_since_start` reports missing correlated evidence. Absence proves neither success nor failure. Reading status adds no network check.

The initial startup error remains under its existing contract. This diagnostic changes no task, receipt, slot, guard, permission or engine choice. A closed technical event `bridget_dispatch_refusal190` is emitted when the observed refusal changes, even if warning logs are disabled. It does not replace owner status or authorize repair.

Preserve the task, request key and files. Restore only the refused source within the authorized scope. If this is impossible, ask the owner or user for the specific decision on permissions, capacity or maintenance. Do not create a replacement mission, cancel other work or force a guard to obtain diagnostics.


## Preparation and storage (191)

`project_preparation_disk_full` records an ENOSPC errno actually observed during preparation I/O; `project_preparation_disk_quota` records EDQUOT. These are storage facts at that time, not provider account or current capacity facts. `project_preparation_failed` keeps the precise cause unknown. Refusals retain the code and guide checks of source and target volumes, inodes and storage quotas. No free-form stderr or private path is copied into the refusal.

Identify file owners and task state before archiving or removal. Never move a live working directory. Replay a `not_recorded` request strictly only after correcting the cause and excluding concurrent calls. A known or unknown admission stays bound to its key. Ask the owner or user for a specific decision when files or storage exceed the agent mandate.

The error `stdout Codex fermé pendant le tour` is preserved. Guidance requests stderr correlated to the command, instance and generation, then journal and temporary storage checks. Without evidence, the cause remains unknown. Failure does not prove process termination: wait for native cleanup and the workspace report, preserve outputs, then use only an authorized follow-up task. Neither `bridget_task_continue` nor replay of the initial key resumes a `failed` task.

Full owner status adds `detail.provider_failure`: task, admission key, child, machine, instance and, only when the canonical command and native link agree, command and generation. `command_identity_confirmed:false` leaves the last two fields unknown; an old slot never replaces them. Give this reference to the responsible operator to locate the native receipt and exact stderr. Summary views and other owners do not receive this reference. No stderr content, secret or private path is published. Generic preparation cause is `unobserved`, distinct from admission state `not_recorded`.

The remote dispatch proof waits at most30seconds for its `Reconcile` (192). A worker may perform several Git reads, each bounded to10seconds. The former5second master timeout could expire while an allowed read was still running, allowing another proof request to enter the queue. The single asynchronous slot and task rotation are unchanged. This timeout changes no queue capacity, permit, permission, identity, epoch, dispatch criterion or mission counter. A foreign identity remains rejected and a different digest still refuses dispatch. Beyond30seconds the outcome remains unconfirmed; this change guarantees neither success nor freedom from all saturation. It never retries a refused business request.

A silent worker may hold this slot for up to30seconds and delay other proofs sharing it; the tick stays nonblocking and rotation order is preserved. The next proof waits for the slot to become available.

A remote check computes one fresh Git workspace report for both validation and its response (193). The preparation receipt and Git anchor remain checked. A permission refusal never becomes dispatch authorization; an available technical report is still returned. No report is reused across calls. This removes redundant reads without guaranteeing freedom from saturation or changing the eight-entry queue or admission criteria.

## Start refused before mission delivery

`remote_start_failed:remote_start:ResourceLimit` does not prove low memory, a full disk, or a provider quota. Versions that attest `worker_start_recovery_v1` distinguish the technical queue, command store and metadata when the cause was observed. A historical refusal without that evidence keeps an unknown cause.

Keep the task, request key and original arguments. Follow the recovery guidance returned by Bridget. When supported and authorized, strict admission replay resumes the same task after checking current permissions and durably closing the previous command before any birth. It creates no new mission or round and preserves the workspace. An existing command receipt or an uncertain execution blocks this recovery.

An older master or worker can require an upgrade first. Do not replace the mission or change the engine, target or permissions to bypass the refusal. If evidence or activation is missing, ask the maintainer for the exact native operation and its human prerequisite. Corrected sources do not prove that the running binary has loaded them.

### Refusal after target facts change (199)

`routing_connection_changed` keeps its public code. `detail.stage` identifies the check and `detail.changes` contains only observed categories: definition, permissions, worker connection epoch, mapping, project or execution contract. `detail.checks` supplies the corresponding checks. No configuration content, private path or digest is exposed. The closed, bounded diagnostic is stored with the durable refusal; replay returns the first recorded cause without another probe or task.

An older refusal without this detail returns `stage=unobserved` and an empty list. Its facts must be compared by the Bridget maintainer. The code alone establishes no network outage, revoked account or changed permission. Reattesting the parent does not reopen a durable refusal.

If only the worker epoch changed, verify native availability on the same machine. For changed definitions, permissions or mappings, confirm the expected configuration with its owner; do not broaden permissions to bypass the refusal. After a durable refusal without a task is confirmed and the observed change is resolved or explicitly validated, a new key requests a new admission. An unknown outcome retains its key and prohibits duplication. This refusal prescribes no halt or restart.

### Automatic admission parameters (200)

`bridget_auto_delegate` does not accept `agent_type`. Omit it; `engine` selects the runtime (for example `codex`), `machine_id` selects the destination, and `connection_id` selects a specific catalog connection only when required. Unspecified axes remain free. `agent_type` belongs to the separate explicit path.

The client rejects this field before connecting, and the coordinator keeps the public `auto_admission_invalid` code with `detail.field=agent_type`. Parameters may be corrected under the same key only before any receipt, after checking that no task or concurrent call exists. If a receipt exists, preserve its canonical contract and follow its state; never describe a changed envelope as a strict replay. Keep the requested exact SHA and workspace.

Recovery of a capacity-refused Start preserves the chosen effort, including
a choice different from the profile default. The expected digest is the
admitted execution definition, revalidated by the worker before Start/Release.
A `routing_connection_changed` refusal with `detail.stage=start_recovery`
reports only closed change categories. The admitted task stays preserved:
after correction or approval of the change, replay its exact key and envelope;
do not create another mission or edit receipts.
