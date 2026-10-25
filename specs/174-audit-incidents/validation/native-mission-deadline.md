# Native mission transport deadline — follow-up 174

2026-10-24. Delivered.

## Incident and cause

A remote Claude mission requested 14,400 seconds. Its durable start was
14:31:01 UTC and its mission deadline 18:31:01 UTC. It failed at 15:16:01 UTC,
exactly 2,700 seconds after delivery. The persisted envelope carried
reply_timeout=14400 and no deadline_at. The delivery projection unconditionally
used the provider default, 2700, while native tracking used the requested budget.

The correction projects the native deadline from the existing task and admitted
request before the generic provider fallback. It uses the existing pause/quota
overlap calculation. A missing checkpoint derives its origin from durable
admission, not the current time. Missing tracking or mismatched endpoints fail
closed. Persisted message bytes, request identity, permissions and replay rules
do not change. Ordinary messages retain the generic provider timeout.

Failed deadline tasks now expose error_message and error_recovery. They cannot
be continued or restarted by admission replay. Recovery requires stop confirmation,
retention of existing commits and outputs, a check of detached business processes,
and a separately authorized new mission that reuses the partial work.

## Reference

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/native-mission-deadline.md#extrait-1).

## Verification

- RED: the real native handler/socket fixture received the provider deadline
  instead of the requested native deadline.
- GREEN: three daemon tests cover short, long and default budgets; delayed start;
  durable origin before checkpoint; no renewed budget at re-reading; endpoint
  mismatch; missing tracked request; ordinary message fallback; and status guidance
  without task mutation.
- One transport test uses a real shell fixture that responds after two seconds,
  with a one-second provider default and a five-second explicit deadline: completed.
- Nine native deadline regression tests, one ordinary delivery test and one human
  message/steer test passed.
- Documentation: 27 pages, 426 local links, 11 language pairs passed.

The first guidance test incorrectly expected result_available from an existing
cancelled seed. Only that assertion was corrected to compare its actual saved
state. No production failure is inferred from this fixture error.

No business task was replayed, cancelled or modified. No complete suite or long
live model run is claimed. The existing handling of pauses after an input has
already reached the provider is not changed by this delivery-time projection.

## Review and delivery

Independent review 344e31f9: APPROVE with limits. It read the current source and
did not execute Git or tests. Errors while reading the native store remain closed,
including for ordinary messages whose native membership cannot be established.
The review does not constitute a full audit.

Source 5ac9e79599efee622b26e4142e7e41f9a8503d8d merged and pushed to main.
The final targeted total is 16 passing tests, no failures or ignored cases.
The existing Claude background cancellation/failure/expiry test also passed.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/native-mission-deadline.md#extrait-2).

The fix applies to new delivery projections. An already-injected provider turn
retains its original absolute deadline. No four-hour live model run was performed.
The affected terminated task remains failed; a separately authorized recovery
must reuse its preserved outputs and must check detached business processes.
