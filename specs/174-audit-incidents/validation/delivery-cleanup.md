# Delivery cleanup — 2026-10-24

The delivered code was already committed, integrated into main and pushed at
`ecadc8bef35c21abb2e01f4c014c17d682a29d02`. This follow-up closes the remaining
cleanup and records the permanent delivery rule in AGENTS.md. It changes no
runtime code and requires no binary rebuild or service restart.

## Removed

- Cartae: two clean, integrated worktrees and their local branches:
  `fix/174-admission-guidance` and `review177-autostart`.
- Mac: one obsolete local remote-tracking reference,
  `cartae/session-174-audit-remaining`, verified as integrated into main.
- Mac: five unused executable copies, 197,200,032 logical bytes.
- Cartae: eleven unused executable copies, 513,378,464 logical bytes.
- Cartae: 3,549,978,613 logical bytes of validation cache intermediates,
  removed through the existing build cleanup function and its guards.

The binary and cache total is 4,260,557,109 logical bytes (about 3.97 GiB).
Worktree sizes are excluded. Logical size does not establish reclaimed physical
space on cloned, compressed or shared storage.

## Preserved and verified

- Mac has only its main checkout and local main branch.
- Cartae keeps main and the mobile session 177 worktree. That worktree contains
  current, uncommitted source and acceptance work. It was not integrated,
  committed or removed by this cleanup.
- The active build remains `5ac9e79599ef`; build `6d0f01096d10` remains available
  for rollback. Active and rollback executable hashes were checked unchanged.
  Some existing wrappers still use the rollback build.
- Reusable recognized intermediates remain within the existing 10 GiB / 7 day
  budget: 0.76 GiB on Mac and 4.36 GiB on Cartae at observation time.
- Historical source packages, proof files, databases and task outputs remain.

Before worktree removal, Git ancestry, tracked/untracked/ignored cleanliness,
closed task guards and process usage were checked. Removal used Git without
force. Process observations were complete for the actual removal scopes.
The first broad Linux process observation was incomplete and authorized no
removal; subsequent scoped observations were complete. Cache removal also
acquired and rechecked the existing Cargo lock. No process was stopped.

## Permanent completion rule

Every authorized delivery must commit, integrate and push validated work;
check main against its remote; remove unused integrated worktrees and branches;
and clean unused binaries and build intermediates. Keep the active executable,
one verified rollback, current work, proofs and files in use. Report each
retention reason and revisit it at the next delivery.

Cargo's existing automatic cleanup before and after each invocation remains
in place. This completion rule covers the additional Git and installed-binary
checks; it does not add a background deletion service.

Validation: diff whitespace checks and actual Git, process, hash and cleanup
observations. No application tests or full build were needed for this
instruction-only change.
