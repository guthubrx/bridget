# Audit de réutilisation — 104-recherche-echanges

## Décision

Statut: PASS
Date: 2026-09-16. Lecture code main1738a072 + specs102, revue indépendante Explore.
PASS porte sur le plan documentaire et la réutilisation, pas sur du code exécuté.

## Synthèse

Deux index d'accès ledger, deux opérations de protocole et actions de lecture/recherche dans les modules existants ; aucune nouvelle table/dépendance/service.
Le plan a été écrit après l'exploration, pas refactoré silencieusement après découverte
d'un doublon. Aucun arbitrage produit bloquant ni duplication évidente laissé ouvert.

## Réutilisations correctement identifiées

Racine des preuves relatives du tableau :
/Users/moi/Nextcloud/10.Scripts/64.bridget/

| Élément proposé | Existant et preuve | Décision |
|---|---|---|
| Recherche corps | store/ledger_requests.rs:719 search_messages, aucun appel rg | ÉTENDRE/REMPLACER cette primitive, pas doublonner |
| Normalisation | même fichier:26 fold_for_search, :49 folded_body_sql | RÉUTILISER Rust, retirer SQL divergent si inutilisé ailleurs |
| Portée participant | même fichier:685 participant_messages | RÉUTILISER règle sender/target et la renforcer sur chaque nouvelle action |
| Page récente | ledger.rs:17 read_projection, cli.rs:4821, mcp.rs:935 | PRÉSERVER le comportement ancien, étendre par action |
| Lecture exacte | PK(id,target) store.rs:59, pas API exacte trouvée | CRÉER l'opération dans ledger existant, pas une table |
| Index candidats | store.rs:68 idx_ledger_ts et :69 idx_ledger_conv insuffisants pour les deux participants | CRÉER deux index sender/target, coût écritures à mesurer |
| Read-only | store_schema.rs:33 et daemon.rs:1449 ouverture READ_ONLY | RÉUTILISER le pattern, ne pas ouvrir Store en écriture sur chaque recherche |
| Fils | spec102 data-model.md discussion_entries/members (non implémenté) | DÉPENDRE de102 et réutiliser son contrôle, pas simuler des tables en prod |
| Surface agent | bridget_ledger existant | ÉTENDRE sans nouvel outil MCP |


## Existant potentiellement pertinent non mentionné

Aucun après intégration de l'exploration. Anciennes fonctions Maicie et docs de handoff
ne sont pas des services de passation/recherche à réactiver. Les primitives d'artefact
ont été évaluées explicitement, pas ignorées.

## Duplications évidentes

Aucune dans le plan retenu. Interdit à l'implémenteur de recréer Send, une base de recherche,
un générateur de résumé, une file de notifications ou un contrôle d'accès parallèle aux fils.
Rechercher encore par nom ET responsabilité avant toute nouvelle création non prévue.

## Mémoires et règles applicables

Constitution globale1.8, AGENTS.md, .specify/memory/constitution.md et standards.md lus.
Articles VII/XVI/XVIII/XIX/XX : décision tracée, isolation, coût borné, réutilisation et
preuves de comportement. mem absent ; mémoire projet Bridget absente. Research/ADR
servent de repli documentaire, pas d'équivalent DevKMS prétendu.

## Specs livrées applicables

089 cœur de communication,094parité des accès,099envoi fiable,100journal/observations.
082/083artefacts évalués comme existant mais non réutilisés pour élargir les droits.
101observationsT3 et102fils restent dans leurs worktrees ; ne pas déduire leur présence
dans main des seuls documents. Relation exacte102 définie dans spec.md/plan.md.

## Journal de recherche

Explore lecture seule par nom handoff/bundle/search_messages et responsabilité search/ledger/
artifact publish/read/share/journal ; inspection des appels, des tests et de la rétention.
Commandes : rg -n, rg --files, git status/worktree ; aucun test/runtime lancé.
Sources web primaires et limites capturées dans research.md ; données envoyées synthétiques.

## Arbitrages

Décisions minimales arrêtées dans research.md. Pas de nouvelle dépendance.

Arbitrages ajoutés pendant l'implémentation (gate anti-doublon, 2026-09-18) :

| Item créé | Existant trouvé (preuve) | Issue | Justification |
|---|---|---|---|
| `ledger::search` (module dans `crates/bridget-daemon/src/ledger.rs`) | `ledger.rs:read_projection` (projection récente) | CRÉER à côté | La projection n'a ni validation, ni curseur, ni relecture ; l'étendre aurait mêlé deux contrats. Même fichier, comme prévu par le plan. |
| `store::ledger_requests::search` (module inline) | `ledger_requests.rs:search_messages` (LIKE SQL), `fold_for_search` | REMPLACER + RÉUTILISER | `search_messages` sans appelant retirée ; `fold_for_search` conservée et réécrite sur `fold_char`, règle unique de repli. |
| `Store::open_read_only` | `store_schema.rs:validate_existing` (READ_ONLY ponctuel) | CRÉER (12 lignes) | Le pattern existant valide un schéma ; il fallait une connexion réutilisable avec `busy_timeout` 100 ms, sans `init_schema`. |
| `ReadPermits` / `ReadPermit` | aucun compteur de permis de lecture (`rg permit`) | CRÉER | Compteur atomique + RAII, 30 lignes, testé unitairement ; un sémaphore externe serait une dépendance. |
| `store::threads::load_thread_for_member` rendu `pub(crate)` | même fonction, privée | RÉUTILISER | Contrôle d'appartenance 102 partagé, aucune duplication de la règle. |
| Index `idx_ledger_target_page(target, ts, id)` | plan : `(target, ts, id, target)` | CRÉER (variante) | Colonne finale redondante ; EXPLAIN identique. |
| `inert_text` (CLI) | aucun helper de neutralisation ANSI (`rg is_control` dans cli.rs) | CRÉER (5 lignes) | Exigence FR-015 ; utilisé pour extraits, identifiants et fragments. |
La politique existante du ledger n'est pas présentée comme un nouvel espace confidentiel.
Si l'implémenteur rencontre un équivalent nouveau après intégration d'autres branches,
réauditer avant création ; ne pas invoquer ce PASS pour forcer un doublon.

## Gate avant tasks

- [x] Tous les services/composants/tables/endpoints/dépendances proposés sont recensés.
- [x] Recherche par nom et responsabilité exécutée, preuves localisées.
- [x] Réutilisations/limites des specs livrées et prévues distinguées.
- [x] Aucune duplication évidente ni arbitrage utilisateur non résolu.
- [x] Plan fondé sur l'existant ; génération des tâches autorisée.

