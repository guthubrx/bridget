# Cleanup safeguards

Automatic removal now preserves worktrees with ignored files, including local evidence and generated output. An incomplete process inventory (nonzero lsof status or diagnostics) prevents removal. The inventory command no longer suppresses warnings.

The regression uses real temporary Git repositories. Before the fix, the ignored receipt was deleted and an incomplete process observation allowed removal. The two new cases fail on the old source. After the fix, all 23 targeted cleanup tests pass. No production worktree was removed.

T3 reference: GitVcsDriverCore.removeWorktree runs an explicit removal, without force by default, and reports Git failure. Bridget reuses Git removal but needs additional guards because maintenance is automatic. Git alone does not preserve ignored output.

Remaining boundary: a process can begin work after the observation. This is not an atomic reservation with the task registry. Production cleanup still requires owner closure and a fresh inventory; the published plan does not authorize mass removal.
