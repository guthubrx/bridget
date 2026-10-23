# Source preflight diagnostics — 2026-10-23

The source directory belongs to the calling parent host. A destination path from another host can fail before admission. The MCP client previously exposed only `workspace_preflight_technical`, hiding the guidance available in the daemon.

## Delivered change

Source commit: `748d687033eb14405a04005f210824dbca69b27b` (initial correction `f9d6f67b980b6844da4f3eafc261d3aff4e1d155`).

The existing error code now carries static `detail.message` and `detail.recovery` fields. They explain the source-host requirement and ask the caller to inspect any existing task before retrying. Raw Git stderr and private arguments are not exposed. Known `workspace_branch_exists` and `workspace_limit` causes retain specific guidance. Unknown technical causes use a neutral statement rather than claiming that Git could not start.

No routing, permissions, admission state, receipt, retry or worker protocol changes. The maintainer did not create or replay a business mission.

## T3 comparison

Read `GitVcsDriverCore.ts`: `gitCommandContext`, `isMissingGitCwdError`, command spawning with `cwd`, and `branchExists`. T3 distinguishes a missing working directory from an existing branch. Bridget additionally needs to explain source versus destination hosts. No automatic path remapping was added.

## Validation

Four targeted tests passed: the source-preflight projection, MCP detail rendering, and two existing base-reference regressions. Coverage includes a real absent source directory, static text without raw stderr, branch and capacity causes, and an unrelated prefix left unchanged.

The MCP rendering test uses a response fixture. It is not an end-to-end admission test. No full suite was run.

Independent review of the initial correction identified misleading generic guidance for branch and capacity refusals. The final correction preserves both causes and uses neutral generic wording. The final four tests passed after that change; no second independent review is claimed.

## Activation

Optimized, signed macOS build `174-748d6870` was installed and confirmed by the official daemon status. Artifact SHA-256: `db5470a1cb96fb43b650b65e6bfddeb089a22eca633095e97e31b2721f1b0c81`.

Only the master restarted. The bridge, T3 and remote worker were preserved. The worker remains on its previous optimized build because no worker production code changed. Private activation and test receipts are retained outside Git. The project owner was notified; message submission does not prove that the recipient has read it.
