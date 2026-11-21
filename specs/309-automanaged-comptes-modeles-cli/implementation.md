# Implémentation - Spec 309 : Comptes regroupés, modèles auto-référencés et CLI automanaged

## Livraison active

- Commits livrés sur `main` : `7658e08e` (`316-automanaged-comptes-modeles-r1`) puis `fef5e9fa` (`317-automanaged-limit-r1`).
- Binaire macOS ARM64 (`monordinateur`) : `/Users/moi/.local/lib/bridget-communication/317-automanaged-limit-r1/bin/bridget` (sha256 `b348c96783a6daed41ac2ef0730b863400630e4950f502e8c75f64851550a485`, retour qualifié `316-automanaged-comptes-modeles-r1`).
- Binaire Linux Debian 12 (`cartae`, `sysle2`, `ksstore`) : `/home/moi/.local/lib/bridget-communication/317-automanaged-limit-r1/bin/bridget` (sha256 `57653ee4868eed5806e2c61d123eea66879311ab19e8665be231e757bdbac0d0`, `required_glibc: 2.34 <= 2.36`, retour qualifié `316-automanaged-comptes-modeles-r1`).

## Mesures et résultats en direct

1. **Regroupement des vrais comptes** (`bridget accounts status` et `bridget accounts ui`) :
   - 16 vrais comptes affichés au lieu de 23 connexions de routage (`[couvre: ...]` pour les doublons historiques `-haiku213`, `-terra213`, `-models62`).
   - Une seule sonde d'authentification par couple `(machine_id, engine, command, profile_dir)`.
2. **Auto-référencement additif des modèles** (`bridget automanaged sync --models`) :
   - Plafond `MAX_ROUTING_CANDIDATES` / `MAX_CANDIDATE_PROBE_V2_CANDIDATES` porté de `128` à `512` (identifiants `c001..=c512`, `MAX_ROUTING_DECISION_BYTES = 256 * 1024`) après constat réel de `validation_refused: candidate_limit` lors de l'ajout de 88 modèles Antigravity sur `cartae` et `monordinateur`.
   - Ajout additif réussi de `+88` modèles (`+22` par connexion Antigravity, portant chaque connexion à 27 modèles) sur `/home/moi/.cache/bridget-core/agents.json` (`cartae`) et `/Users/moi/.cache/bridget-core/worker-monordinateur-agents.json` (`monordinateur`), avec `last_error: null`.
   - Catalogue live `bridget delegate --catalogue --observed --json` : `antigravity-cartae`, `antigravity-sysle2` et `antigravity-ksstore` remontent `status=reported_models`, `auth=reported_present`, `27` modèles.
3. **Mises à jour automatiques des CLI (`automanaged` actif par défaut)** :
   - Exécution automatique via `bridget-updates` (`scripts/update.py`, compatible Python 3.9.6) toutes les 6 h par défaut.
   - Mises à jour réellement exécutées sur la flotte : `claude` `2.1.327 -> 2.1.330` sur `monordinateur`, `cartae`, `sysle2` et `ksstore` ; `codex` `0.170.0 -> 0.171.0` sur `sysle2` ; `agy` `1.5.14` à jour partout.
4. **Incident Parix `e0ea9aa9-4f8f-4e89-b1af-ade0666dffc4`** :
   - Diagnostic prouvé sur `sysle2` : `claude auth status` rapporte `loggedIn: false`, et Claude 2.1.330 émet le texte `Failed to authenticate: this login reached its time limit and cannot be renewed.` avec `is_api_error_message: true`.
   - Correctif livré dans `fef5e9fa` (`crates/bridget-transport/src/claude_stream_json.rs`) : reconnaissance du nouveau libellé et du champ `is_api_error_message` vers `api_error:oauth_session_expired`.

## Vérifications exécutées

- `make fmt` et `make lint` (`clippy --workspace --all-targets --locked --features bridget-daemon/test-support -- -D warnings`) : succès sans avertissement.
- Tests unitaires ciblés : `routing168_ids_et_probe_v2_sont_fermes`, `spec258_*`, `registry::tests::routing168*`, `permission_projection214_tests::*` (17/17), `automanaged::tests::*`, `decision_service::tests::*` (18/18) : tous verts.
- Vérification ABI Linux (`scripts/check-linux-abi.py`) et validation bidirectionnelle des formats (`storage_check` entre `316` et `317`) : compatibles.
- Ménage : `/tmp/bt` supprimé localement, `/tmp/bridget-build-317` et le cache `/mnt/storage/bridget-build/bridget-cache/bridget-build-e9d92961f213995d` supprimés sur `sysle2`, ancienne release `312-claude-auth-catalogue-r1` retirée de `cartae`, `sysle2` et `ksstore`. Sur `monordinateur`, `315-comptes-ui-popup-close-r1` et les versions antérieures encore référencées par des processus `bridget mcp` en cours sont conservées sans interruption.
