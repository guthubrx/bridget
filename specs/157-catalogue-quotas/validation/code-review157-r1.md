## Verdict : APPROVE_WITH_CHANGES

Aucun défaut bloquant trouvé dans ce que j'ai lu. Deux points demandent une décision avant livraison, et trois points sont mineurs. Je n'ai rien écrit, lancé ni testé.

## Vérifications faites

- **Empreintes** : manifeste, `tasks.md`, `diff-157.patch` (29 fichiers, 103 hunks) et `native_delegation.rs` correspondent au brief.
- **Diff** : 27 fichiers suivis, plus `provider_limits.rs` et l'ADR (non suivis), soit 29 au total. Les deux autres entrées non suivies (`implementation.md`, `validation/`) sont des artefacts.
- **FR-013** : la liste de motifs remplaçables (`native_delegation.rs:1164-1167`) est identique à la spec. `model_unavailable` et `effort_unavailable` ne sont jamais remplacés.
- **FR-014** : l'ordre est respecté dans `target_resolver` : type (934), modèle (935), effort (936), préflight épinglé (940-957), politique (958-967), puis quota en dernier (977).
- **FR-010** : `record_refusal` ne persiste que cinq codes (`delegation.rs:657-666`). Un `provider_quota_exhausted` ne consomme donc pas la clé. Le détail `resets_at` est validé par `failure()` (`native_delegation.rs:74-100`).
- **FR-017 et FR-018** : le rejeu se fait avant les gardes (1057-1063). `canonical` reste l'original et `effective_request` n'est inséré que dans `Task` (1318). Reserve et Commit revalident le candidat épinglé sans nouvelle sélection (1132-1144, 1219-1225).
- **FR-008 et FR-009** : `provider_limits.rs:71-76` applique la règle par fenêtre. Le statut Codex global ne prouve pas l'épuisement.
- **Borne mémoire** : huit fenêtres par type, éviction par `observed_at` puis clé lexicale (`provider_limits.rs:43-54`), conforme à la spec ligne 380.
- **Parseur MCP** : `fallback` est fermé à 1-3 candidats, sans doublon ni égalité avec la cible demandée (`delegation_mcp.rs:132-175`).

## Objections vérifiables

**1. MEDIUM - Clé `substitution: null` ajoutée à la projection hors fallback.**
- Preuve : `native_delegation.rs:260` écrit `view["substitution"] = json!(task.substitution)` pour tous les appels. L'oracle154 a été modifié pour ajouter `"substitution":null` (`native_workspace154_tests.rs:418-423`, diff `main`).
- Problème : FR-019 protège les octets canoniques de la requête, pas la projection de réponse. La spec ne dit pas si la clé doit toujours être présente. Le test « contrat 150 » attend désormais une clé que 150 ne produisait pas. C'est un test ajusté au code, même si le contrat reste cohérent.
- Changement minimal : écrire la clé seulement si `task.substitution` est `Some`. Puis restaurer l'oracle154 à sa version d'origine et le rejouer seul. Sinon, ajouter une phrase explicite dans la spec et dans `docs/delegation-native.md`.

**2. LOW à MEDIUM - Fait « passé » sur une fenêtre déjà connue.**
- Preuve : `provider_limits.rs:38` purge seulement selon le `observed_at` courant. Aux lignes 39-41, un nouveau fait dont le retour est passé retourne sans supprimer l'ancienne entrée de même clé `window`.
- Scénario : `five_hour` est `rejected` à t=90 avec retour 500. À t=100, un fait `five_hour` avec retour 95 arrive. L'ancien fait reste et bloque l'admission jusqu'à 500.
- Changement minimal : avant le `return` de la ligne 39, faire `windows.remove(&fact.window)`. Aucun test n'existe pour ce cas, car `native157_limits_fenetre_perimee` utilise un nom de fenêtre différent.

**3. LOW - Changement de refus sur les lancements existants (hors délégation).**
- Preuve : `lifecycle.rs` ajoute `launch_preflight` dans `submit_spawn_with_policy` et `prepare_spawn_parts` (~371). Le préflight passe avant la vérification de `cwd` et crée l'environnement plus tôt.
- Effet : un `cwd` absent combiné à un environnement invalide renvoie désormais `env_unfit` au lieu de `cwd_gone`. Ce changement n'est pas listé dans les limites de converge.
- Changement minimal : le noter dans la spec ou la documentation.

**4. LOW - Horloge distincte pour le catalogue.**
- Preuve : `native_delegation.rs:1473` lit `unix_timestamp()` une seconde fois, séparément des lectures de quota dans `target_resolver` (977). Au passage d'une fenêtre, `can_run` et `quota` peuvent diverger.
- Changement minimal : capturer l'instant une fois au début du catalogue. Ou déclarer la limite.

**5. LOW - Documentation incomplète du test flaky.**
- Preuve : `tests-157.md:40-42` signale que `native154_creation_quarante_appels_trois_proprietaires` a échoué une fois sous créations concurrentes, puis a passé seul. `converge157.md` n'en parle pas.
- Changement minimal : l'ajouter aux limites de converge.

## Réponses aux questions

- **Code conforme à FR-001 à 020 ?** Vérifié par lecture pour FR-005, 008 à 019. Non vérifié : FR-020 (docs non lues) et le détail de FR-001 à 004 au-delà des plages citées.
- **Cohérence droits, motifs, ordre, rejeu, pin, options 154/155, quota, invalidation Reserve/Commit ?** Oui pour les points lus. Seul le point 1 touche le contrat de réponse.
- **Failure et MCP gardent-ils un détail fermé ?** Oui : `failure()` et le parseur MCP valident la forme.
- **Converge et preuves surévaluent-elles ?** Oui sur deux points : l'affirmation « tous les fichiers correspondent au manifeste » n'a pas été vérifiée par moi (seul `native_delegation.rs` l'a été), et la lecture « intégrale 3628 lignes » porte sur le patch R1, pas sur le patch final de 3653 lignes.

## Non lu ou non testé

- Non lus en entier : `plan.md`, le contenu de `tasks.md`, `reuse-audit.md`, `data-model.md`, `implementation.md`, le contenu du manifeste, `converge157.json`, `docs/delegation-native.md` et les skills.
- Partiellement lus : `task_view` (255-275), `native_permissions.rs` (seulement `child_policy_reader`), `delegation.rs` (seulement les champs de `Task`), les tests natifs157 (seulement leurs noms et ceux de `provider_limits.rs`).
- Non exécutés : aucun test, build, scanner, service ni DB, conformément à la consigne. Les résultats rapportés (2396 PASS, 9 FAIL, etc.) ne sont pas confirmés par moi.
- Pas de GREEN global et pas de réel fournisseur, authentification ou modèle : ces limites sont bien déclarées dans converge.

Le ROOT peut appliquer les points 1 et 2 avant livraison. Les points 3 à 5 peuvent être traités dans la documentation.
