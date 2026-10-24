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

T3 ClaudeAdapter.ts, sendTurn (4881–5035), records turnState and queues the input.
This path does not impose a default turn expiry. Bridget must retain its explicit
native mission deadline because it coordinates durable remote work. The fix reuses
that existing authority instead of adding another timer or retry mechanism.

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

- Optimized macOS build: 1m25s; signed and verified.
  SHA-256 e817c59d0294cacf7294852a9b67bf34ec430b4352560ba265ee274dc0c2c57d.
  Active master PID 74260 reports build 5ac9e79599ef.
- Optimized Linux build: 1m51s.
  SHA-256 1d8a5cbdee32757f942322ae14490ee4cf24393b8add7e0c3fb1543b0f08e42b.
  Active worker PID 292436 uses the new executable.
- T3 and its bridge retain their original processes. Three live remote wrappers
  retained their PIDs and birth times across worker replacement.
- The incident task payload fingerprint remained identical after master activation.
  No business task was replayed, cancelled or edited.
- The first macOS status read preceded service readiness; the next read confirmed
  the expected build. No second restart was needed.
- Staging cleanup initially rejected a signing-only hash difference. The compiled
  executable received the same verified signature as the installed copy; their
  hashes then matched and the normal installed cleanup ran. No guard was weakened.
- Politique received the delivery guidance through Bridget. Recipient acknowledgment
  is not claimed.

The fix applies to new delivery projections. An already-injected provider turn
retains its original absolute deadline. No four-hour live model run was performed.
The affected terminated task remains failed; a separately authorized recovery
must reuse its preserved outputs and must check detached business processes.
