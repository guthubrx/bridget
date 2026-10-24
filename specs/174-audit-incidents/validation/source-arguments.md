# Source arguments — 24 October 2026

## Defect and correction

The MCP schema did not explain that cwd is the source directory on the
calling parent's host. It also described expected_base_commit only as a
40-to-64-character string, while the parser accepted 40 or 64
hexadecimal characters and the Git engine only supports 40 (SHA-1). The previous invalid-input diagnostic gave no remedy.

Both delegation tools now distinguish source cwd from destination machine_id.
Automatic admission documents and constrains the full SHA format. The parser
now rejects unsupported 64-character hashes before connecting, and gives a
constant actionable error without
echoing the supplied input. The English and French guides explain these
arguments before the first admission, including Shared HEAD and Worktree
base_ref. Permission checks, routing and idempotency are unchanged.

The existing workspace_preflight_technical recovery already correctly explains
the source/destination distinction and is unchanged.

## T3 reference

At ef86c2bbc3388be8fc593db23a323e3d80b58509, GitVcsDriverCore.createWorktree
executes Git in input.cwd and passes a separate worktreePath.
resolveCommit uses rev-parse --verify to resolve a full commit identifier.
Bridget retains that distinction and additionally maps the source to a remote
host. It does not guess a source path or rewrite an admitted envelope.

## Targeted validation

- spec174_source_arguments_expliques_et_sha_valide_avant_connexion: 1 PASS.
  Checks both tool descriptions, malformed and abbreviated hashes rejected
  before connection, constant diagnostic, uppercase/lowercase SHA-1 and early rejection of SHA-256.
- spec168_mcp_auto_delegate_negocie_v2_et_preserve_le_schema_ferme: 1 PASS.
  Existing socket-based negotiation and closed-schema regression.
- check-docs.py: 27 pages, 422 local links, 11 language pairs PASS.
- Final combined filter delegation_mcp::tests::spec1: 3 PASS, including legacy
  wire compatibility and refusal against an older daemon.
- No full suite and no application mission replay.

## Direct-message observation

Independent read-only analysis confirms that project_unknown is an advisory
warning, not a delivery refusal. A targeted read of the active daemon store
found the two reported message receipts already acked. Both supplied message
identifiers target the same child. Receipt acknowledgement does not establish
that the provider acted on the supplement. The guides now explain this
distinction and the existing task_continue path for a continuable task.

The raw receipt identifiers and operational readings stay outside Git.

## Independent review

The review identified the pre-existing client/engine format mismatch:
workspace_git::sha and lifecycle::resolve_source_base require 40 characters.
The final schema, parser, diagnostics and guides now use that same supported
format. This is an earlier explicit refusal, not new SHA-256 support.
The reviewer accepted the source/destination guidance and constant diagnostic.
The responsible maintainer applied this correction and reran the targeted tests.
