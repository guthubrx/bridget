# Audit 174 — Remaining dependency qualification

Date: 2026-10-24. Branch: `session-174-remaining-dependencies`.

Scope: the four dependency candidates left open by audit 174 (`DEP-rustls`, `DEP-rustls-webpki`, `DEP-unsafe-libyaml`, `DEP-webpki-roots`).

Method: primary advisory pages (RustSec and GitHub Advisory Database), crates.io version metadata, read-only `cargo tree --offline --frozen`, and text searches in the repository and in the locally cached crate sources. No `cargo build`, `cargo test` or `cargo check` was run.

## Outcome

No `Cargo.toml` or `Cargo.lock` change is committed. No minimal compatible fix is demonstrated for any of the four candidates. Every published fix lies behind a major migration of the HTTP client `minreq` (2.14.1 to 3.0.0), which is pinned by decision 163. That migration needs an architecture decision.

## Individual verdicts

| Candidate | Locked version | Verdict |
|---|---|---|
| `DEP-rustls` | 0.21.12 | Open. Version is in the affected range and no 0.21 release is patched. The conditions of RUSTSEC-2026-0346 and RUSTSEC-2026-0349 are not met in this repository. Fix requires `minreq` 3. |
| `DEP-rustls-webpki` | 0.101.7 | Open. RUSTSEC-2026-0104 (GHSA-82j2-j2ch-gfr8) is not reachable. RUSTSEC-2026-0098 and -0099 need a misissued certificate. RUSTSEC-2026-0382 is exposed in version through the bundled emSign root, but only for an operator-configured remote HTTPS decision endpoint. No 0.101 release is patched. Fix requires `minreq` 3. |
| `DEP-unsafe-libyaml` | 0.2.11 | Open, low. Unmaintained (RUSTSEC-2024-0449), no patched version exists anywhere. Used only to parse a local YAML pin file. Not fixable by a version bump. |
| `DEP-webpki-roots` | 0.25.4 | Open. RUSTSEC-2026-0381: the bundled roots include `emSign ECC Root CA - G3`. No 0.25 release is patched, and 1.0.10 is blocked by `minreq` requiring `^0.25.2`. Fix requires `minreq` 3. |

## Advisory evidence

| Advisory (alias) | Crate | Affected range | Patched | Required condition | Reachability in this repository |
|---|---|---|---|---|---|
| RUSTSEC-2026-0346 | rustls | `<0.23.46` | `>=0.23.46` | Custom `ClientSessionStore` that formats session values with `Debug` | No `ClientSessionStore` in `crates/` (text search: 0 hits). `minreq` uses the default rustls builder. Not reachable. |
| RUSTSEC-2026-0349 | rustls | `<0.23.46` | `>=0.23.46` | Preference for a hybrid post-quantum key exchange, plus group memorization before authentication | No hybrid post-quantum group in `rustls-0.21.12/src` (text search: no key-exchange hit). Condition absent for this version. |
| RUSTSEC-2026-0104 (GHSA-82j2-j2ch-gfr8) | rustls-webpki | `<0.103.13` | `>=0.103.13` | Explicit `RevocationOptions` and attacker-supplied CRL bytes | No CRL or revocation code in `crates/` or in `minreq-2.14.1/src` (0 hits). Not reachable. |
| RUSTSEC-2026-0098 (GHSA-965h-392x-2mh5) | rustls-webpki | `<0.103.12` | `>=0.103.12` | Certificate already misissued by a trusted CA, after signature verification | In version. Exploitation needs prior misissuance. Conditional. |
| RUSTSEC-2026-0099 (GHSA-xgp8-3hg3-c2mh, CVE-2026-93601) | rustls-webpki | `<0.103.12` | `>=0.103.12` | Misissued certificate combining name constraints and wildcard names | In version. Exploitation needs prior misissuance. Conditional. |
| RUSTSEC-2026-0382 | rustls-webpki | `<0.103.16` | `>=0.103.16` | Trust anchors accept a path through a distrusted emSign intermediate | `webpki-roots` 0.25.4 bundles `emSign ECC Root CA - G3` (`src/lib.rs`, around line 2928), so the condition can hold. Reached only by remote HTTPS decision endpoints (see below). |
| RUSTSEC-2026-0381 | webpki-roots | `<1.0.10` | `>=1.0.10` | Accepting a chain under the bundled `emSign ECC Root CA - G3` | Bundled in 0.25.4. Same reachability as RUSTSEC-2026-0382. |
| RUSTSEC-2024-0449 | unsafe-libyaml | unmaintained crate (informational) | none | No vulnerability condition; the notice concerns maintenance status | Used by `serde_yaml` 0.9.34+deprecated. The advisory says forks such as `serde_yaml_ng` are affected too. |

## Cargo chains and features

Read-only `cargo tree --offline --frozen -i <crate>`:

- `rustls` 0.21.12 ← `minreq` 2.14.1 ← `bridget-daemon`
- `rustls-webpki` 0.101.7 ← `minreq` 2.14.1 and `rustls` 0.21.12
- `webpki-roots` 0.25.4 ← `minreq` 2.14.1 ← `bridget-daemon`
- `unsafe-libyaml` 0.2.11 ← `serde_yaml` 0.9.34+deprecated ← `bridget-daemon`

`crates/bridget-daemon/Cargo.toml` pins `minreq = "=2.14.1"` with feature `https-rustls` only. In `minreq` 2.14.1 that feature enables `rustls`, `webpki-roots` and `rustls-webpki`. Its requirements are `rustls ^0.21.1`, `rustls-webpki ^0.101.0` and `webpki-roots ^0.25.2`.

crates.io index facts:

- The 0.21, 0.101 and 0.25 lines have no release patching the advisories. Latest: `rustls` 0.21.12, `rustls-webpki` 0.101.7, `webpki-roots` 0.25.4.
- Latest 2.x `minreq` is 2.14.1.
- `minreq` 3.0.0 (3.0.0-rc.0 and rc.1 also exist) requires `rustls ^0.23.35`, `rustls-webpki ^0.103.8` and `webpki-roots ^1.0.4`. These ranges would allow the fixed versions, but no resolution was tested.

## Why no change was committed

1. The requirement ranges of `minreq` 2.14.1 (`rustls ^0.21.1`, `rustls-webpki ^0.101.0`, `webpki-roots ^0.25.2`) keep every resolution on the affected lines. No patched release exists on those lines.
2. Moving to `minreq` 3.0.0 is a major migration. It changes the TLS stack from rustls 0.21 to 0.23 and changes the client API used by `crates/bridget-daemon/src/t3code_contract.rs`, `t3code_mcp.rs` and `decision_service.rs`. It also contradicts the `=2.14.1` pin and the decision 163 rule of no additional TLS client and no insecure mode (`specs/163-decision-admission/plan.md`, line 63). This needs a recorded architecture decision, not a lock update.
3. Alternatives examined and rejected:
   - Adding `webpki-roots` 1.x as a direct dependency has no effect. `minreq` 2.14.1 builds its own root store from its dependency: `minreq-2.14.1/src/connection/rustls_stream.rs` lines 10 and 33 use `webpki_roots::TLS_SERVER_ROOTS` of 0.25.
   - Feature `https-rustls-probe` (rustls with OS root store through `rustls-native-certs`) does not enable `webpki-roots`, but keeps rustls 0.21.12 and `rustls-webpki` 0.101.7. It changes the trust anchors per machine. The OS store contents were not verified on the deployment hosts, so it is not demonstrated safe.
   - Features `https-native` (native-tls) and `https-bundled` (OpenSSL, vendored) add a second TLS stack, which decision 163 excludes.

## Reachability of outbound TLS

Outbound rustls connections happen only for `minreq` calls to `https://` URLs:

- T3 base URL: always `http://127.0.0.1:<port>` (`crates/bridget-daemon/src/t3code_contract.rs`, `base_url`).
- T3 MCP endpoint: must equal `http://127.0.0.1`, `localhost` or `[::1]` on the runtime port with `/mcp` (`crates/bridget-daemon/src/t3code_mcp.rs`, `validate_endpoint`).
- Decision service: accepts `https://` endpoints from operator configuration (`crates/bridget-daemon/src/decision_service.rs`, `decision_endpoint_allowed`; `decision_services` in `crates/bridget-daemon/src/registry.rs`).

The decision service is therefore the only path where a remote certificate is validated by the webpki stack. RUSTSEC-2026-0381 and -0382 matter only when an operator configures a remote HTTPS decision endpoint and an attacker can intercept that connection with a chain under a distrusted emSign intermediate. This is conditional and not demonstrated exploited.

## unsafe-libyaml

`crates/bridget-daemon/src/reprise.rs`, function `load_pin` (line 370), parses a YAML pin file with `serde_yaml::from_str`. The file is read from a local path supplied by the caller (line 135). No patched version exists, so the only remedy is a format or parser migration, which is outside this lot.

## Tests

No code change was made, so no test was required for this note. Any future `minreq` migration needs a consolidated test run, as decision 163 requires for changes to its HTTP boundary.

## Decision needed

- Option A: keep `minreq` 2.14.1, accept the residual risk for RUSTSEC-2026-0381 and -0382 limited to operator-configured remote HTTPS decision endpoints, and record that acceptance.
- Option B: authorize a migration to `minreq` 3.0.0. This changes decision 163 and needs a new decision record, then a consolidated test run.
- For `unsafe-libyaml`: accept the unmaintained crate, or plan a pin-file format change.

## Limits

- Advisory data was read on 2026-10-24 and may change.
- Reachability was established by text search and code reading, not by runtime tests or network probes.
- The contents of the OS root store on each machine were not checked.
- The origin of the pin file path was not qualified.
