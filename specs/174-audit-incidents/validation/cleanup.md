# Cleanup safeguards

Automatic removal now preserves worktrees with ignored files, including local evidence and generated output. An incomplete process inventory (nonzero lsof status or diagnostics) prevents removal. The inventory command no longer suppresses warnings.

The regression uses real temporary Git repositories. Before the fix, the ignored receipt was deleted and an incomplete process observation allowed removal. The two new cases fail on the old source. After the fix, all 23 targeted cleanup tests pass. No production worktree was removed.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/cleanup.md#extrait-1).

Remaining boundary: a process can begin work after the observation. This is not an atomic reservation with the task registry. Production cleanup still requires owner closure and a fresh inventory; the published plan does not authorize mass removal.
