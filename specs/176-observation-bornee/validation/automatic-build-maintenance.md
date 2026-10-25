# Automatic build maintenance — 2026-10-24

Correction within session 176. No new service, scheduler, dependency or worktree.

## Cause

Direct Cargo invocations bypassed the existing maintenance wrapper. Two build
targets used for recent validation were unmarked and therefore excluded from its
budget. The wrapper also returned early on a failed build, before registering
its target or maintaining its caches.

## Change

- Maintain recognized caches before and after a normal Cargo return, including
  a nonzero exit code. Preserve Cargo arguments and exit status.
- Register an existing selected staging target before Cargo, and retry after it
  returns so a newly created target is registered even on failure.
- Do not remove worktrees after a failed build. Existing deletion guards and
  intermediate-directory allowlist are unchanged.
- Require Make or the wrapper in the repository instructions. Reuse the checkout
  cache; use a registered staging directory only when isolation is needed.
- Align the English and French build guides and current installation commands.

The policy remains 10 GiB shared across recognized intermediate caches per
repository and host, or seven days since their latest modification. This is a
maintenance target, not a peak-disk cap: active or unobservable profiles remain.
Raw Cargo, custom/cross-target profiles, installed executable copies and backups
remain outside automatic coverage. Interrupted or killed wrappers cannot promise
an after-build pass; the next wrapped invocation performs the before-build pass.

## adaptateur de bureau comparison

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/176-observation-bornee/validation/automatic-build-maintenance.md#extrait-1).

## Validation

Cartae: CleanupTests and ProcessTests ran 26 tests: 25 passed, one explicitly
skipped because the real CLI process observation was incomplete on that host.
The three new cases passed: cleanup before/after failure, registration of a target
created by a failed build, and preservation of Cargo's result when maintenance
fails. The failure cases exercise an actual child executable and temporary files;
process inventory is isolated in the cleanup fixture.
The existing real open-file and Cargo-lock tests passed. No Rust build or full
suite was required for this Python-only correction.

Documentation check passed: 27 pages, 422 local links, 11 language pairs.
Two known Cartae build targets were registered without changing foreign markers.
Their recognized intermediates totaled 4,810,208,251 logical bytes, below budget.
No active binary or daemon is replaced by this correction.

The real CLI cleanup test also passed on macOS (one test, 0.944 seconds).
The independent read-only review approved the delta without a blocking finding.
It notes an extra cache scan before each build, possible rebuild cost after
pruning, and the need to keep diagnostics outside generated cache directories.
The actual Cartae maintenance pass returned successfully: zero bytes removed,
4.48 GiB retained below the budget.
