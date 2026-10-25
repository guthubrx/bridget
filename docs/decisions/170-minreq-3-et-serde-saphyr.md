# ADR-170 — Upgrade minreq and replace serde_yaml

[Français](170-minreq-3-et-serde-saphyr.fr.md)

**Date:** 2026-10-24. **Status:** Accepted for session 174.

## Context

Audit 174 left four dependency findings open. Three came from minreq 2.14.1: rustls 0.21.12, rustls-webpki 0.101.7 and webpki-roots 0.25.4. The fourth, unsafe-libyaml 0.2.11, came through serde_yaml. The migration removes these old dependency lines.

ADR-163 designated minreq as the single HTTP boundary. This decision updates its exact version pin and retains that boundary: one client, only https-rustls, no permissive TLS mode.

## Decision

1. Pin minreq to =3.0.0, disable default features and enable only https-rustls.
2. Replace serde_yaml with serde-saphyr 2.0.0. Disable default features and enable only deserialize. RUSTSEC-2024-0449 lists this alternative.
3. Parse pin files with from_str_with_options and reject_unsupported_tags: true. The include and properties features are not compiled. An !include tag is rejected.
4. Keep the parser's default duplicate-key rejection and resource budget.
5. Preserve lazy HTTP bodies, zero redirects, header and status-line limits, and the single in-flight request permit. The decision service still does not use with_timeout.
6. Preserve the MINREQ_TIMEOUT environment guard, since minreq 3 still reads this variable.

## Code changes

The HTTP status is now u16. HttpReply::status and ContractError::Http use that type. Headers are now pairs without normalized names. Mcp-Session-Id is read without case sensitivity; duplicate values are rejected.

## Consequences

The lockfile resolves rustls 0.23.46, rustls-webpki 0.103.16 and webpki-roots 1.0.11. unsafe-libyaml and serde_yaml disappear from the graph. The explicit lockfile scan covers 148 packages and reports no known vulnerability at the time of this audit.

The default cryptographic provider adds aws-lc-rs and a C build through aws-lc-sys. Build hosts need a C compiler and, depending on the target, cmake. YAML edge cases may differ from serde_yaml 0.9. The focused pin tests do not certify the whole YAML grammar.

## Alternatives

Keeping minreq 2.14.1 would preserve the old dependency findings. System certificate probing would make trust depend on each host. Native TLS and OpenSSL features would introduce another TLS stack, contrary to ADR-163. serde_yaml forks listed by RUSTSEC-2024-0449 do not resolve that advisory.

## Validation

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/170-minreq-3-et-serde-saphyr.md#extrait-1).
