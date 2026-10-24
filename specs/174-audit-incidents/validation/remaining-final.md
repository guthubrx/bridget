# Audit 174 — remaining findings closed and delivered

Date: 2026-10-24. Status: delivered. This closes the remaining audit work; it does not certify the entire repository.

## Findings

- Four old dependencies removed: rustls 0.21.12, rustls-webpki 0.101.7, unsafe-libyaml 0.2.11 and webpki-roots 0.25.4. The current lockfile scan covers 148 packages with no known vulnerability reported.
- Four permission hypotheses qualified. Identity selection does not replace credential authentication; project policy remains bounded to its checkout; remote positive answers require a current proof. Cross-role identifier equality does not grant authority. The PERM-ROOT conclusion is ROOT's contract qualification, not a new independent approval.
- Reconnection jitter fixed: the upper cap no longer concentrates retries at exactly 30 seconds.
- Result-slot reservation moved out of the shared maintenance tick. The same silent-worker fixture changed from 5,002 ms to about 1 ms per tick. One reservation stays in flight at a time. Pause/resume notification uses the existing one-second writer budget.
- Verification failures are distinct from proven revocation. Task status reports delivery_state and delivery_error separately from the historical decision. Wake and artifact publication preserve outcome_unknown after an ambiguous transmission; no automatic duplicate is created.

## Validation

The integrated targeted run recorded 336 passes, one transient process-argument observation failure and nine ignored cases. That one failure passed in isolation without a code change. The final review corrections passed 18 targeted tests plus three bounded-write/recovery tests. The complete remote delegation integration test passed. No full-suite global success is claimed.

The audit runner completed all 14 controls on source 3755f5454aee2031d2905140fab1157e06c70d09. Its report was finalized and validated: decision pass, no active confirmed finding and no unqualified candidate. The previous audit evidence remains historical. The zero-penalty score of 100 is not a quality score. The 39 applicable quality criteria were rated; three disaster-recovery/backup/bascule criteria were outside this change. The twelve quality axes are listed below.

Secret scanning found 55 current-file candidates. Each source line exactly matches an already reviewed historical fingerprint: digests, identifiers or synthetic fixtures. The current history scan produced no new finding. No credential was tested against an external provider.

Independent review 93b0bd7d found no authority bypass. Its concrete objections were corrected and tested: late invalidation visibility, bounded recovery writes and the dynamic artifact error code. No second independent approval is claimed.

## Delivery

Optimized release builds for macOS and Linux use source 3755f5454aee2031d2905140fab1157e06c70d09. macOS is ad-hoc signed and verified. Both binaries are installed; master and worker run this build. Subsequent merged installer/documentation commits do not change the compiled Rust sources.

- macOS SHA-256: de497df7e374fea50f6baf9762a851b0cb1b745c3911d58972980455645a4941.
- Linux SHA-256: 34f4260661b6325aa41d57d1ff034706acef8c94602a5e0d7a9f339024c81ffc.
- Master PID after activation: 17132. Worker PID: 4063096.
- The active remote business wrapper retained PID 4045899 and its process birth value. T3 and its bridge were not restarted. Existing client processes keep their loaded client code until their normal restart; daemon-side corrections are already active.

A cleanup-only follow-up, 3dbdd6aa, recognizes the documented macOS lsof rte type as an AF_ROUTE socket. A real capture reproduced the unnecessary refusal. All 65 cleanup tests pass; real inaccessible files and unknown types still block removal. This Python change requires no new Rust binary.

## Cleanup

Eight closed Cartae worktrees and their branches were removed without forcing Git worktree removal. Their unique patches were proved equivalent to main and all branch heads were saved in a verified Git bundle. The temporary Mac build worktree was removed. The active, modified mobile-177 worktree remains protected. Mac has only main; Cartae has main plus that active worktree.

Nine obsolete binary copies were removed. The active binaries and one rollback version per host remain, including the Linux version used by the active business process. About 3.84 GiB of intermediate build files and 212 MiB of old binary files were removed (logical sizes, not claimed physical disk savings). Other useful/active caches and all evidence remain.

One delivery notice was submitted to each of Politique and monuments; recipient acknowledgment was not required or claimed. Their project missions were not replayed. The garde-propre hook belongs to the existing Opus2D correction task; Bridget does not bypass it.

## Quality axes

| Axis | Observed quality | Rated criteria |
|---|---:|---:|
| Security | 65/100 | 5/5 |
| Model/tool security | 67/100 | 3/3 |
| Reliability | 75/100 | 1/1 |
| Tests | 65/100 | 5/5 |
| Performance | 63/100 | 4/4 |
| Algorithmic complexity | 58/100 | 3/3 |
| Code and error quality | 67/100 | 3/3 |
| Architecture | 69/100 | 4/4 |
| Future maintenance | 75/100 | 3/3 |
| Minimalism | 58/100 | 3/3 |
| Data and behavior fidelity | 75/100 | 3/3 |
| Traceability | 75/100 | 2/2 |

These are scoped evidence-based ratings, not uptime percentages. Remaining limits include large modules, I/O-heavy worker inventory, no representative whole-fleet load campaign and no disaster-recovery qualification. Levels 2 and 3 must not be reported as complete level-4 mastery.
