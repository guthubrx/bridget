# Deprecation register

[English](DEPRECATIONS.md) · [Français](DEPRECATIONS.fr.md) · [Documentation](README.md)

This register records paths retained during a migration and historical removals.
A dated decision is not evidence that a running service has been upgraded.

| Path | Replacement | Removal condition or status |
| --- | --- | --- |
| Wire field `AgentInfo.rate_limit` (one optional window) | `AgentInfo.rate_limits` (a vector); legacy wire input is read as one element | After every who/UI/MCP consumer reads `rate_limits`. |
| Executable allowlist in `wrapper.rs` | Private `agents.json` registry | Removed in T703. |
| Untransported stderr rendering in `wrapper.rs` | Structured ACP delivery or application log | Removed in T705. |
| Ignored SC-005 microbenchmark in `acp.rs` | Two real attach views in `tests/sc005_attach_budget.rs` | At removal of the historical ACP coverage in a test cleanup cycle. |
| npm bridges `@zed-industries/codex-acp` and `@zed-industries/claude-code-acp` | Native `codex app-server` and `claude_stream_json` drivers | Removed on 2026-08-24 (G10); refused as `SpawnRefusal::EnvUnfit` on managed and wrapper entry points. |
| ACP as the investment path for Codex/Claude features | Native drivers for model, quota and cost support | No new Codex/Claude features through ACP after G10. Generic ACP remains for supported Cursor/Gemini paths. |

The deprecated Zed bridges were third-party npm packages reached through generic
ACP, not a separate internal transport implementation. Removal preserves native
Codex/Claude registry defaults and rejects those packages in managed spawn and
`--equipier` / `launch_acp_with_status`. Matching normalizes basenames and
`@version` suffixes, including absolute executable paths.

Generic `acp.rs` remains. The historical specifications 007/009/010 and ADR 010
retain the original package-pin context.

## Historical migration inventory

| Area | Decision |
| --- | --- |
| Native defaults in `registry.rs` | Retained. |
| Generic ACP protocol | Retained for providers still using it. |
| Companion-service test definitions using the old bridges | Replaced with native drivers. |
| Managed runtime `npx` test | Replaced with `fixture-acp`. |
| `npx` plus `fixture-acp` in fleet/CLI/parity/protocol tests | Retained as a test adapter, not a Zed bridge. |
| Managed-process and MCP-identity `npx` tests | Retained for process-group lifecycle. |
| Historical specifications and ADR context | Retained without rewriting the old decision. |
| `attach.rs` error labels naming old bridges | Renamed; they were labels, not launch commands. |

Daemon-managed lifecycle is additive: terminal-wrapper launch remains supported.
Compact `attach` rendering applies only to TTY output. Non-TTY rendering
remains the compatibility path and is not deprecated.
