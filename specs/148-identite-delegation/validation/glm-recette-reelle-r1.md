# Rapport GLM — Recette réelle native_delegation_real_glm — R1

- **Date** : 2026-10-10, 06:45 → 07:04 (budget 30 min, consommé ~19 min)
- **Testeur** : agent GLM indépendant (lecteur seule du dépôt)
- **Verdict global** : ÉCHEC de la recette réelle (RC=101, SIGABRT). Étapes préalables 1 à 4 : PASS.
- **Worktree** : `.worktrees/session-148-identite-delegation` (dépôt non modifié, sources intactes)

## Limites déclarées

- Le parent est une **fixture Codex synthétique**. Aucun modèle Codex réel n'est exécuté.
- L'enfant GLM est réel (daemon, MCP, flotte, wrapper réels). Aucun serveur T3 ne fait partie du chemin natif de l'enfant. Le fait que le testeur soit lui-même lancé par T3 est hors du chemin testé.
- Aucun effort (effort level) n'a été fourni : le catalogue source n'en déclare pas.

## Environnement d'exécution

- Répertoire privé de preuves : `/tmp/bridget148-test-064518` (umask 077, perms 0700, TMPDIR privé dédié).
- Aucun autre build cargo concurrent (vérifié par `ps` au départ).
- Recette exécutée avec les variables exactes du brief (`BRIDGET_148_REAL_GLM=1`, registre `/Users/moi/.cache/bridget-core/agents.json`, `AGENT_TYPE=glm`, `MODEL=glm-5.3`, hash commande `dd8dee…2a8e`).

## Étapes 1 à 4 — PASS

| Étape | Commande | Résultat | RC |
|---|---|---|---|
| Tests lib ciblés | `--lib native148` | 23 passed, 0 failed | 0 |
| Tests lib ciblés | `--lib matrice_fr009` | 1 passed | 0 |
| Tests lib ciblés | `--lib mcp_identity` | 15 passed | 0 |
| Tests lib ciblés | `--lib t3code_mcp` | 6 passed | 0 |
| E2E | `--test native_delegation_e2e` | 1 passed, 1 ignored | 0 |
| E2E | `--test t3_session_identity_env` | 1 passed, 1 ignored | 0 |
| Format | `cargo fmt --all -- --check` | 0 diff | 0 |
| Clippy | `--workspace --lib -- -D warnings -A clippy::too_many_arguments` | 0 warning, 0 error | 0 |
| Compilation | `--test native_delegation_real_glm --no-run` | binaire `native_delegation_real_glm-888c55e9145aadf8` | 0 |

Note clippy : l'exception `-A clippy::too_many_arguments` est explicite (catégorie préexistante, ancrée `store/threads.rs:728`). Je n'affirme pas un zéro-warning strict sans exception. Les 1496 tests workspace ne sont pas répétés : seules 3 syntaxes et le format ont changé depuis le dernier PASS complet, et les ciblés couvrent leurs effets.

## Étape 5 — Recette réelle : ÉCHEC

Commande exacte du brief, `-- --ignored --nocapture`. Sortie : `running 2 tests` — voir « défaut tertiaire ».

- `support::performance_daemon_worker` : **FAILED** (collatéral, voir ci-dessous).
- `native148_parent_codex_fixture_delegates_once_to_real_glm` : **FAILED** à `native_delegation_real_glm.rs:463` — mission réelle échouée, statut `"failed"`, erreur `"agent CLI mort"`, `result: null`, modèle demandé `glm-5.3`, `child_agent_id b4430ae1…`, `task_id a38d6454…`, `message_id 4207ae9d…`.
- Abort du process pendant le nettoyage : panique dans un destructeur (`OwnedChild::stop`, ligne 71) pendant le déroulement → SIGABRT. RC cargo 101.

Conséquences observées : `effective-model.jsonl` vide (aucun modèle annoncé par l'enfant), nonce jamais lu (l'enfant n'a jamais démarré), **aucun `receipt.json` généré** (échec avant l'écriture, ligne 540). Annulation/reprise : non couvertes par cette exécution.

## Diagnostic — cause racine (auth indisponible dans l'environnement isolé)

Aucun correctif appliqué. Données runtime conservées. Chaîne causale prouvée :

1. Le stderr managé de l'enfant (`state/managed-stderr/b4430ae1…/979465a9…-g1/stderr.log`) contient deux fois : `ERREUR gclaude: ZAI_API_KEY non défini dans l'environnement.`, encadré par `fournisseur mort — relance 1/5` puis `relancé (tentative 1/5)` et `mort — relance 2/5`. Le wrapper a relancé, la clé restait absente.
2. `support::isolated_command` fait `env_clear()` et fixe `HOME=<root>/provider` (répertoire vide). Il ne transmet pas `ZAI_API_KEY`. C'est voulu : la recette ne copie pas de secrets.
3. `gclaude` (structure lue sans afficher de secret) : si `ZAI_API_KEY` est vide, il tente `security find-generic-password -s ZAI_API_KEY 2>/dev/null || true`. La sortie est vide si l'appel échoue → erreur fatale observée.
4. Sondes empiriques du trousseau, codes de sortie seuls, aucune valeur affichée :
   - environnement normal → RC=0 ;
   - `env -i` avec HOME redirigé + PATH restreint → **RC=44** (item introuvable) ;
   - `env -i` avec HOME réel → RC=0.
5. Conclusion : **HOME est la variable déterminante**. Le trousseau de connexion ne se résout pas sous le HOME isolé du profil de test. La route trousseau prévue par la recette ne peut donc pas fonctionner telle quelle : `security` ne trouve pas l'item, `gclaude` n'obtient jamais la clé, le CLI enfant meurt, la mission échoue.

`ZAI_API_KEY` est présent dans l'environnement du testeur, mais ni la recette ni moi ne le copions (règle anti-copie de secrets respectée). Conformément au brief : auth indisponible → refus et rapport, sans modification de l'auth.

## Défaut secondaire — abort au nettoyage (assertion `OwnedChild::stop`)

Faits : la panique de nettoyage est à `native_delegation_real_glm.rs:71` : `command.contains(env!("CARGO_BIN_EXE_bridget")) && !command.to_lowercase().contains("firefox")`. L'assertion ppid (ligne 64) a PASSÉ : le processus visé était vivant et enfant direct du test. Le message `&&` ne permet pas de désigner l'opérande fautif ; la valeur observée de `command` n'est consignée nulle part (assert sans format). Sonde empirique : un enfant `bridget mcp` isolé fraîchement lancé montre bien le chemin complet dans `ps -o command=` (journal `log-probe-mcp.txt`), donc l'assertion passerait pour un tel processus. Le mécanisme exact reste **non déterminé** — l'abort a détruit le contexte. Effet certain : un échec de test propre se transforme en SIGABRT et court-circuite le reste du nettoyage (le daemon du fixture est resté orphelin).

## Défaut tertiaire — `--ignored` exécute aussi le worker de support

`-- --ignored` lance TOUS les tests ignorés du binaire, y compris `support::performance_daemon_worker` (inclus via `mod support`). Ce worker exige `BRIDGET_PERFORMANCE_DAEMON=1` et échoue sinon (panique `idempotent.rs:604`, `Err(NotPresent)`). Cet échec est collatéral et se produirait même si la recette principale réussissait. Piste (non appliquée) : `--exact native148_parent_codex_fixture_delegates_once_to_real_glm --ignored --nocapture`.

## Post-mortem

- **Résiduel** : le daemon du fixture (PID 87973, `…/session-148-identite-delegation/target/debug/bridget daemon`, orphelin ppid 1) a survécu à l'abort. Checklist appliquée : PID identifié par `ps` ✓, pas Firefox ✓, autorisation du brief (PID connu, pas de groupe inconnu) ✓, `kill` simple sans `-9` ✓, attente 3 s ✓, libération vérifiée ✓. Aucun autre résiduel du run (`AUCUN_RESIDUEL_DEBUG`). Les processus bridget/claude préexistants (production, autres sessions) n'ont pas été touchés. Firefox : jamais visé, jamais tué.
- **Runtime conservé** : fixture complet `/private/tmp/ng148-8a8f559e77/` + instantané dans le répertoire privé (`runtime-ng148-8a8f559e77/`, 2,1 Mo : fleet.json, agents.json, bridget.db+wal, stderr managé, relay.py, daemon.log). Les fichiers `agent-pids/87993` et `88023` (tentatives gclaude) existaient au premier relevé puis ont été retirés par le daemon encore vivant — relevé initial dans le transcript.
- **Sources intactes** : les 8 hashes SHA-256 (native_delegation.rs, wrapper.rs, mcp.rs, 3 fichiers de test, 2 fixtures) sont identiques début/fin (`diff` vide). Aucun Git, aucune édition du dépôt.
- **Settings et commande inchangés** : `gclaude` `dd8dee…2a8e` (identique à l'attestation), `~/.claude-glm/settings.json` `ca6b9b4b…5bab`, avant = après. Aucun secret lu, affiché, copié ou modifié.
- **Chemins de preuve** : `/tmp/bridget148-test-064518/` (logs `log-lib-*.txt`, `log-e2e-*.txt`, `log-fmt-check.txt`, `log-clippy.txt`, `log-compile-real.txt`, `log-recette-reelle.txt`, `log-sondes-trousseau.txt`, `log-probe-mcp.txt`, hashes début/fin, identification kill).

## Recommandations (à décider en amont — aucune appliquée par le testeur)

1. Route auth : rendre le trousseau résolvable depuis l'environnement isolé (par exemple HOME réel limité à la résolution de la commande, ou résolution amont côté fixture sans jamais écrire la clé sur disque). Le choix appartient au concepteur de la recette.
2. Nettoyage : rendre l'assertion de `OwnedChild::stop` non-abortante en contexte d'échec (constituer la valeur observée dans le message, distinguer les deux opérandes, ou verrouiller le kill derrière un diagnostic consigné).
3. Recette : cibler le test principal avec `--exact` pour ne pas exécuter le worker de support.

## Conclusion

Étapes 1 à 4 : PASS sans réserve. Recette réelle : ÉCHEC reproductible diagnostic — cause racine identifiée et prouvée (trousseau non résolvable sous HOME isolé → `ZAI_API_KEY` absent → CLI enfant mort → mission `failed`). Deux défauts annexes documentés (abort au nettoyage, `--ignored` trop large). Aucun masquage d'échec ; données runtime et preuves conservées.
