# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/remaining-dependencies.md](../../../../../../../specs/174-audit-incidents/validation/remaining-dependencies.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. The requirement ranges of `minreq` 2.14.1 (`rustls ^0.21.1`, `rustls-webpki ^0.101.0`, `webpki-roots ^0.25.2`) keep every resolution on the affected lines. No patched release exists on those lines.
2. Moving to `minreq` 3.0.0 is a major migration. It changes the TLS stack from rustls 0.21 to 0.23 and changes the client API used by `crates/bridget-daemon/src/t3code_contract.rs`, `t3code_mcp.rs` and `decision_service.rs`. It also contradicts the `=2.14.1` pin and the decision 163 rule of no additional TLS client and no insecure mode (`specs/163-decision-admission/plan.md`, line 63). This needs a recorded architecture decision, not a lock update.
3. Alternatives examined and rejected:
   - Adding `webpki-roots` 1.x as a direct dependency has no effect. `minreq` 2.14.1 builds its own root store from its dependency: `minreq-2.14.1/src/connection/rustls_stream.rs` lines 10 and 33 use `webpki_roots::TLS_SERVER_ROOTS` of 0.25.
   - Feature `https-rustls-probe` (rustls with OS root store through `rustls-native-certs`) does not enable `webpki-roots`, but keeps rustls 0.21.12 and `rustls-webpki` 0.101.7. It changes the trust anchors per machine. The OS store contents were not verified on the deployment hosts, so it is not demonstrated safe.
   - Features `https-native` (native-tls) and `https-bundled` (OpenSSL, vendored) add a second TLS stack, which decision 163 excludes.

<a id="extrait-2"></a>

## Extrait 2

- T3 base URL: always `http://127.0.0.1:<port>` (`crates/bridget-daemon/src/t3code_contract.rs`, `base_url`).
- T3 MCP endpoint: must equal `http://127.0.0.1`, `localhost` or `[::1]` on the runtime port with `/mcp` (`crates/bridget-daemon/src/t3code_mcp.rs`, `validate_endpoint`).
- Decision service: accepts `https://` endpoints from operator configuration (`crates/bridget-daemon/src/decision_service.rs`, `decision_endpoint_allowed`; `decision_services` in `crates/bridget-daemon/src/registry.rs`).
