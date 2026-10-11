# Tâches - Session157 catalogue fiable, quotas, remplacement

**Entrées** : `spec.md`, `plan.md`, `research.md` (R1-R9), `data-model.md`,
`contracts/delegation157.md`, `reuse-audit.md`,
`adversarial-review-codex.md`.
**Racine** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation`.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`).
**Prérequis** : session 150 livrée. Aucune tâche ne dépend de 152. La reprise sur
quota est la session future 160 (`spec.md`).

Règle : une tâche n'est cochée qu'après son test vert. Les tests marqués
« rouge d'abord » sont écrits et vus en échec avant le code. Les lignes citées
viennent de la base `1f2c661d`. Chercher par nom de fonction, car la session 150
les déplace.

## Phase 1 - Préparation

- [ ] T001 Établir la référence : lancer les filtres `native148`, `native149`, `native150`, `delegation149`, `spec_087` de `crates/bridget-daemon` avec la recette ; ajouter le test ignoré `native157_catalogue_latence` (catalogue avec 10 fournisseurs de test, médiane et p95 de 50 appels) et le lancer sur le code actuel ; consigner compteurs et mesures dans `specs/157-catalogue-quotas/validation/baseline.md`
- [ ] T002 Forensic en lecture seule (Art. VIII) : chercher un fait réel expurgé de l'incident du 2026-10-11 (liste des clés présentes, `launch_context` présent ou non, jamais de valeur) pour dire quelle cause de R1 s'est produite ; consigner dans `specs/157-catalogue-quotas/validation/forensic.md` ; aucun appel à un fournisseur ; sans fait réel, les documents disent « scénario de code »

## Phase 2 - Fondations (bloquantes)

- [ ] T003 [P] Ajouter `FallbackTarget {agent_type, model, effort: Option<String>}` et le champ `fallback: Vec<FallbackTarget>` (`#[serde(default, skip_serializing_if = "Vec::is_empty")]`) à la variante `Delegate` de `crates/bridget-transport/src/protocol.rs` ; test : une requête sans ce champ produit exactement les mêmes octets `serde_json::to_vec` qu'avant
- [ ] T004 [P] Rouge d'abord puis implémentation : créer `crates/bridget-daemon/src/provider_limits.rs` (déclaré dans `lib.rs`) avec `record(agent_type, fact, observed_at)`, `quota(agent_type, now)`, la règle d'épuisement (FR-008) et les bornes (8 fenêtres par type, retrait des fenêtres passées à l'écriture) ; annoter la complexité ; la mémoire garde la source de chaque fait ; tests `native157_limits_*` : table de la règle d'épuisement par fenêtre (rejected Claude, 100 %, `unknown` à 50 %, sans heure de retour jamais épuisé) et `native157_limits_statut_codex_global` (statut global atteint + fenêtre courte à 100 % + fenêtre hebdomadaire à 61 % : seule la courte est épuisée, son retour est le sien ; statut global atteint sans fenêtre à 100 % : `limit_reached` vrai, `exhausted` faux), fenêtre périmée ignorée, une fenêtre n'efface pas l'autre, clé par `agent_type`, 9e fenêtre refusée ou la plus ancienne retirée (choisir et tester une règle)
- [ ] T005 [P] Extraire `launch_preflight(registry_épinglé, agent_type, source) -> Result<(), SpawnRefusal>` de `submit_spawn` ET de `prepare_spawn_parts` (`crates/bridget-daemon/src/lifecycle.rs`) : type connu, capacités, observation fournisseur, garde de facturation (`forbidden_environment_variable`), protocole géré, `build_environment` avec l'erreur PROPAGÉE (aujourd'hui ignorée par `if let Ok`), `command_exists` ; les deux fonctions l'appellent ; le contrôle `cwd` reste hors du préflight ; rendre le code de `refusal_record` accessible (`pub(crate)`) ; tests `native157_preflight_*` : binaire absent donne `command_missing`, clé API interdite avec binaire présent donne `billing_guard`, HOME absent avec binaire présent donne `env_unfit`, protocole non géré donne `negotiation_failed`, définition épinglée vérifiée ; tous les tests de lancement existants restent verts

- [ ] T006 [P] Ajouter à `Task` dans `crates/bridget-daemon/src/delegation.rs` le champ `substitution: Option<Substitution>` avec `#[serde(default)]` ; test : un payload 150 sans ce champ se relit avec `None`

## Phase 3 - US1 Le catalogue ne ment plus (P1)

**But** : un seul verdict pour le catalogue et l'admission.
**Test indépendant** : filtres `native157_coherence`, `native157_preflight`.

- [ ] T007 [US1] Rouge d'abord : dans les tests de `crates/bridget-daemon/src/daemon/native_delegation.rs`, écrire deux tests, un par cause : `native157_coherence_t3_claude_sans_launch_context_vers_codex` (le catalogue actuel dit `discovery: true`, l'admission rend `permission_source_unavailable`) et `native157_coherence_launch_context_present_source_illisible` (même code par une autre cause : `recheck_context` échoue) ; écrire la matrice `native157_coherence_matrice` (parent : Claude sans `launch_context`, Claude avec, Claude avec source illisible, Codex, aucun fait ; cible : claude, codex, glm, codex-pro ; posture : 3) qui compare, par les vrais gestionnaires, chaque booléen du catalogue au résultat de l'admission DANS LE CONTEXTE DÉCLARÉ (`verdict_context`) ; écrire les tests de cas hors garantie qui documentent la divergence attendue : `native157_coherence_hors_contexte_fait_limite_a_une_demande`, `native157_coherence_hors_contexte_sous_repertoire` (Claude vers Claude, `cwd` différent du parent), `native157_coherence_hors_contexte_binaire_retire_entre_deux_appels` ; si un test de cause passe déjà, STOP : observer le fait réel avant de continuer (Art. VIII)
- [ ] T008 [US1] Extraire de l'admission de `native_delegation.rs` (bloc du fait attesté, `child_policy`, `for_inherited_delegation`, chemin discovery sans fait) la fonction `resolve_target(st, owner, instance, fact, permission, agent_type, model, effort, posture, ...)` qui rend la cible résolue ou un code, et qui contrôle dans cet ordre le type connu, le modèle et l'effort (avant tout motif remplaçable), puis `launch_preflight`, la politique de l'enfant et le quota ; l'admission l'appelle ; aucun changement de comportement hors préflight ; tests 148, 149 et 150 verts
- [ ] T009 [US1] Réécrire le catalogue sur `resolve_target` : `discovery`, `inherit` et `development` viennent du verdict (avec la posture correspondante), ajouter `can_run`, `constraints[{code, message}]` (messages en français, courts, qui citent la cause : fait sans `launch_context` ou source illisible) et la racine `checks` et `verdict_context` (`cwd` = `cwd_root`, fait non limité à une demande) ; garder `inherit_refusal` et `development_refusal` ; T007 passe
- [ ] T010 [US1] Admission : refuser avant toute création de tâche avec le code du préflight ; tests `native157_preflight_admission_binaire_absent_sans_tache` (0 tâche, clé libre) et `native157_preflight_catalogue_binaire_absent` (`can_run: false`, contrainte `command_missing`)
- [ ] T011 [US1] Mettre à jour la description de `bridget_capabilities` dans `crates/bridget-daemon/src/delegation_mcp.rs` et le test qui fige les clés du catalogue (`native_delegation_permissions149_tests.rs`, test sur les clés) ; vérifier que les clés existantes sont toujours présentes
- [ ] T012 [US1] Refaire la mesure `native157_catalogue_latence` ; comparer à `validation/baseline.md` ; si le rapport dépasse 2 (SC-008), calculer chaque politique une seule fois par fournisseur et dériver les trois postures (R8) puis remesurer ; consigner dans `validation/mesure-catalogue.md`

## Phase 4 - US2 Quotas visibles et respectés (P1)

**Test indépendant** : filtres `native157_limits`, `native157_quota`.
**Dépend de** : T004, T008.

- [ ] T013 [US2] Rouge d'abord : tests `native157_quota_catalogue_epuise` (`quota.exhausted`, `can_run: false`, contrainte `provider_quota_exhausted`), `native157_quota_admission_refuse_avec_resets_at` (0 tâche, clé libre, détail), `native157_quota_perime_ignore`, `native157_quota_inconnu_null`, `native157_quota_survit_fin_presence` (la présence est retirée, la mémoire reste), `native157_quota_nom_enregistre_par_les_trois_sources` (les trois sources écrivent sous le nom du registre)
- [ ] T014 [US2] Ajouter le champ `provider_limits` à `DaemonState` et écrire dans `handle_rate_limit` (`crates/bridget-daemon/src/daemon.rs`) en plus de `Presence.rate_limits` ; la source et l'instant (`observed_at`) sont enregistrés ; les tests existants de `handle_rate_limit` restent verts
- [ ] T015 [US2] Ajouter `failure_with_detail(code, detail)` dans `native_delegation.rs` ; brancher le quota dans `resolve_target` (refus `provider_quota_exhausted` avec `detail.resets_at`) et `quota` dans chaque entrée du catalogue ; T013 passe sauf le détail côté pont
- [ ] T016 [US2] Faire rendre le détail par le pont MCP (`delegation_mcp.rs` : message `code` suivi du JSON de `detail` si présent) ; test de contrat `native157_quota_detail_visible_cote_mcp` ; vérifier que les autres refus gardent le message d'avant

## Phase 5 - US3 Remplacement explicite par le parent (P2)

**Test indépendant** : filtre `native157_fallback`.
**Dépend de** : T008, T015.

- [ ] T017 [US3] Rouge d'abord : tests `native157_fallback_*` : cible indisponible, premier candidat valide choisi ; second candidat si le premier est refusé ; `model_unavailable` jamais remplacé, y compris modèle absent + binaire absent et modèle absent + politique indisponible ; `effort_unavailable` jamais remplacé, y compris effort invalide + quota épuisé (zéro substitution, ordre type/modèle/effort avant préflight, politique et quota) ; test d'ordre des refus du contrat (le premier refus de la liste gagne) ; refus de droits du parent jamais remplacé ; cible disponible, `fallback` ignoré ; sans `fallback`, mêmes codes qu'avant ; doublon, entrée égale à la cible, plus de 3 entrées donnent `invalid_request` ; un remplacement ne donne aucun droit de plus (politique de l'enfant calculée depuis le parent pour chaque candidat)
- [ ] T018 [US3] Dans l'admission de `native_delegation.rs`, après la garde de pause et les gardes de droits du parent, parcourir la cible demandée puis les candidats (au plus 4, O(1)) avec `resolve_target` ; règle FR-013 pour la cible demandée, tout refus fait passer au suivant pour un candidat listé ; écrire la cible effective dans `record.request` (`agent_type`, `model`, `effort`, `fallback` vidé), remplir `substitution` ; `canonical` garde la demande d'origine ; T017 passe
- [ ] T019 [US3] Refus `fallback_exhausted` avec `detail.tried` quand `fallback` est fourni et que tout échoue ; tests `native157_fallback_epuise_liste_les_essais` et `native157_fallback_epuise_sans_tache_cle_libre`
- [ ] T020 [US3] Exposer `fallback` (tableau d'au plus 3 objets fermés, `additionalProperties: false`) dans le schéma `bridget_delegate` et la liste blanche de `delegation_mcp.rs` ; l'analyse des arguments rejette toute forme invalide en `InvalidParams` ; ajouter `features` au catalogue ; tests de schéma et de rejet
- [ ] T021 [US3] `task_view` et `bridget_task_status` rendent `substitution` ; tests `native157_fallback_rejeu_meme_substitution` (le verdict n'est pas recalculé après un changement de quota), `native157_fallback_rejeu_autre_liste_envelope_mismatch`, `native157_fallback_projection_lineage_cible_effective`

## Phase 6 - US4 Compatibilité et finitions

- [ ] T022 [US4] Relancer les filtres de T001 et comparer à `validation/baseline.md` : aucun test 148, 149, 150 ou 087 ne passe de vert à rouge ; vérifier les octets canoniques d'un appel sans nouveau champ ; consigner dans `validation/tests-157.md`
- [ ] T023 [P] Documenter le catalogue (`can_run`, `constraints`, `quota`, `checks`, `verdict_context`, `features`) et sa garantie bornée, `fallback`, `substitution` et les refus dans `docs/delegation-native.md`, `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md`, avec la limite « authentification non vérifiée »
- [ ] T024 [P] Écrire l'ADR `docs/decisions/157-catalogue-quotas-remplacement.md` (contexte, décisions D-A à D-E et D-G à D-J, la reprise sur quota reportée en 160 avec ses six exigences d'entrée, conséquences, statut Accepté)
- [ ] T025 Lancer la suite complète (`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette) ; rejouer seuls les tests connus sensibles à la charge avant de conclure ; consigner les compteurs dans `validation/tests-157.md`
- [ ] T026 Relire le diff complet (Articles XIX et XX : lignes suppressibles, abstractions, chemins d'erreur), écrire `specs/157-catalogue-quotas/implementation.md` (dont ce qui n'a pas été vérifié en réel) et enregistrer `git diff` dans `specs/157-catalogue-quotas/validation/diff-157.patch`

## Dépendances

- T001 et T002 avant tout.
- T003 à T006 en parallèle, avant les phases 3 à 5.
- Phase 3 (US1) avant Phase 4 (US2) : le quota passe par `resolve_target`.
- Phase 5 (US3) après T015 : le remplacement utilise le quota.
- Les phases 3 à 5 touchent `native_delegation.rs` : exécution séquentielle.
- T022 à T026 en dernier ; T023 et T024 en parallèle.

## Exemples de parallélisme

- T003, T004, T005, T006 : fichiers distincts.
- T023, T024 : documentation seule.

## Stratégie

MVP = Phases 1 à 3 : le catalogue dit la vérité et refuse tôt. Puis quotas
(US2) et remplacement (US3). La reprise sur quota est la session 160.

## Charge future (Article XX)

- Réduisent la charge : T008 (une seule logique de verdict, fin de la dérive),
  T010 (plus de tâche créée pour échouer aussitôt).
- Ajoutent de la complexité justifiée : T004 (mémoire des limites, exigée par la
  perte de la présence), T018 (boucle de remplacement bornée à 4).
