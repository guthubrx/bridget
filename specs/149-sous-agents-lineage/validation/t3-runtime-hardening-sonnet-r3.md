# Revue runtime T3 149 - correctifs Sol (3 fichiers), ronde Sonnet r3

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), normal, effort high. Aucun edit de production, aucun Git, aucune case cochée, aucun redémarrage, aucune config de production.

**Verdict : CHANGES_REQUIRED (un seul point résiduel, F1).** Les trois deltas de Sol sont corrects et prouvés. F1 est un défaut de sûreté préexistant que la matrice des refus de contexte met en évidence : un refus de contexte forgé modifie quand même l'état.

## Couches

| Couche | État |
|---|---|
| Serveur T3 du WT (`apps/server/src/bin.ts`, Node 24.13.1, PID 74491, `127.0.0.1:14774`), auth bearer scopée, WebSocket RPC, `BridgetLineage`, `Orchestrator`, projection SQL privée | RÉEL |
| UI web du WT (Vite+ dev `localhost:15734`), navigateur preview T3, onglet à moi (fermé en fin de recette) | RÉEL |
| Daemon Bridget : `bridget_fixture.mjs` sur un magasin JSON privé | **SIMULÉ** (aucun modèle, aucun binaire natif) |
| Données | Synthétiques `[recette149]` |

Le binaire natif r3 n'a pas été utilisé (consigne). Cette recette ne valide ni SC001, ni T037, ni le MCP privé, ni T036/T039 natifs.

Isolation : cache neuf `/Users/moi/.cache/bridget149-ui.r3` (0700), base T3 privée neuve, ports 14774/15734. Les preuves r2 (`/Users/moi/.cache/bridget149-ui`, rapports r2, `ui149-screenshots/`) sont intactes.

## Compteurs réellement exécutés

| Volet | PASS | FAIL | Notes |
|---|---|---|---|
| Lot serveur 149 (5 fichiers), état entrant | 104 | 2 | Les 2 échecs sont d'anciens oracles contredits par le delta Sol (voir Audit) |
| Lot serveur 149 après adaptation et 9 nouveaux tests | **115** | 0 | `BridgetLineage149` 18, `Orchestrator.bridget149` 33 (dont 7 nouveaux) |
| Lot UI/client 149 (4 fichiers, client + web + mobile jsdom) | 22 | 0 | Non modifié |
| Matrice réseau `matrix149.mjs all` sur le nouveau serveur | 38 | 0 | Scopes, 13 charges invalides, mauvais root/tâche/fil, cancel |
| Rejeux ciblés `replay_r3.mjs all` | 22 | 0 | + 1 FINDING (F1) |
| UI navigateur (tableau UI : 9 cas) | 8 | 0 | + 1 FINDING (F1), 10 captures |
| Typecheck serveur `tsc --noEmit` | - | - | 16 erreurs, **toutes baseline** (`BridgetRustInterop*`, `CodexMcp.ts`), 0 dans les fichiers 149 |

Compteurs SQL après toute la recette : `runs=0 run_attempts=0 provider_turns=0 provider_sessions=0 provider_threads=0 provider_session_bindings=0 runtime_requests=0 effect_outbox=0 thread_launch_workflows=0 thread_messages=0`. Aucun ProviderTurn, Run ni exécution produits par la vue.

## Audit explicite du contrat (anciennes assertions)

Le contrat `lineage.md` L141 dit : « un journal absent rend `journal_unavailable`, sans inventer de lignes ». L224 dit : « un indisponible garde l'historique vérifié sans le présenter comme état courant ». Aucun des deux ne fixe la portée. `data-model.md` L98-105 confirme l'historique conservé.

Deux tests de `BridgetLineage149.test.ts` exigeaient qu'un `result_offset_invalid` (lecture `show`) et un `journal_unavailable` (flux journal) marquent la racine indisponible. Le delta de Sol retire ce comportement : ces deux codes ne disent rien de l'autorité ni du transport. Je juge ce changement correct. Je n'ai pas baissé la sûreté : les tests adaptés exigent maintenant 0 dispatch pour les refus locaux (`result_offset_invalid`, `journal_unavailable`, `invalid_request`, `envelope_mismatch`) et **toujours** un dispatch d'indisponibilité pour six codes d'autorité, de transport ou de format (`unavailable`, `project_mismatch`, `binding_unavailable`, `invalid_output`, `store_unavailable`, `timeout`). Ils ajoutent `task_unavailable` : fil connu de ce root invalidé seul ; tâche étrangère ou inconnue sans aucun effet.

Autre ancienne hypothèse : « le marqueur d'un enfant égale le seq global ». Elle est fausse depuis le delta : un enfant inchangé garde le marqueur de son dernier fait (SQL : enfant 3, racine 6). L'oracle valide est `seq enfant <= seq racine`, et `seq racine = seq du magasin`. Aucun test existant ne la portait.

## Résultats des rejeux demandés

Mesure : table `orchestration_events` (SQL lecture seule) et JSON de projection. Sortie brute : `/Users/moi/.cache/bridget149-ui.r3/logs/replay-r3-run1.txt`.

| Replay | Résultat | Preuve mesurée |
|---|---|---|
| Journal 1/4 (fait modifié) | PASS | 3 événements : 1 `subagent.updated`, 2 `thread.metadata-updated` (la tâche + la racine). Les 3 autres tâches : 0 événement |
| Tick journal seul 1/4 | PASS | 1 événement : racine seule |
| Journal 1/134 | PASS | 3 événements (pas environ 270) |
| Tick journal seul 1/134 | PASS | 1 événement : racine seule |
| Création initiale de 130 tâches | PASS | 130 `thread.created`, 130 `subagent.updated`, 1 racine |
| 100 `list` + 100 `show` identiques | PASS | 0 événement |
| Nouvelle génération, faits inchangés | PASS | 135 `thread.metadata-updated` (134 fils + racine), 0 `subagent.updated`, 0 `thread.created`, 134 marqueurs réécrits |
| Panne globale (magasin illisible), puis reprise complète au même seq | PASS | `invalid_output`, racine et 134 fils `available:false` ; reprise : tous `true`, seq 14/14, 0 création, 0 subagent, 134 fils sans doublon |
| Base à 130 historiques (retrait de 130 tâches, même génération) | PASS | 130 fils `available:false` (observé seq 15, marqueur seq 14), 4 `true`, 134 fils conservés, aucun prune |
| `journal_unavailable` (T4, lecture + flux) | PASS | 0 événement, racine et T1/T2/T3 inchangés |
| `result_offset_invalid` | PASS | 0 événement, racine inchangée |
| `task_unavailable` d'un enfant connu de ce root | PASS | ce fil seul `false`, T1/T2/T4 et racine `true` ; reprise au snapshot suivant |
| `task_unavailable` d'une tâche liée à un AUTRE root, et d'un UUID inconnu | PASS | 0 événement, racine 1 et racine 2 intactes |
| Snapshot partiel / mutation entre pages | PASS | `snapshot_changed` après 3 tentatives, nombre de fils inchangé (134), 0 `thread.created`, 0 `subagent.updated` ; test unitaire : snapshot à curseur ouvert refusé sans écriture |
| Snapshot complet identique après indisponibilité | PASS | `available:true` rétabli au même seq, aucun doublon ; un second snapshot identique écrit 0 événement (test unitaire) |

## UI navigateur (preview T3, locators `role=`/`text=`)

Captures : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r3/`

| Cas | Résultat | Preuve |
|---|---|---|
| Lineage : parent + enfant visibles ; sidebar à 3 fils malgré 134 tâches projetées | PASS | `01-lineage-parent-sidebar-3-fils-130-historiques.png` |
| Nested, titre long, 1280x800, carte « Thread details » ouverte : « Arrêter » libre | PASS | `elementFromPoint` rend le bouton sur 3 points ; bouton x 418-475, carte x >= 989 ; `02-imbrique-titre-long-carte-ouverte-1280-arreter-libre.png` |
| Clic réel sur « Arrêter » (imbriqué, carte ouverte) | PASS | reçu natif `cancelling` avec `request_id` UUID frais ; 0 run/tour ; `03-arret-natif-imbrique-annule-sans-bouton.png` |
| Clic réel sur « Arrêter » (racine) | PASS | reçu `cancelling` (2 reçus distincts pour 2 tâches) |
| Carte fermée, 1024, 768, 480, 375 px (titre long) | PASS | 6 états : 3 points de clic sur le bouton, bouton dans la section, aucun débordement horizontal ; `04`, `05`, `06` |
| Pas de composeur | PASS | 0 `textarea`/`contenteditable`/`textbox` dans chaque état ; barre « Tâche Bridget · lecture seule » |
| Panne globale : « Dernier état connu », « Arrêter » désactivé, journal conservé ; reprise : « En cours », « Arrêter » actif, lignes 1-7 sans doublon | PASS | `07-panne-globale-dernier-etat-connu-arreter-desactive.png` |
| Journal absent de T4 : message local, état « Échec », panneau hôte sans « indisponible » | PASS | `08-journal-absent-T4-message-local-etat-echec-racine-disponible.png`, `09-...` |
| Contexte forgé (jeton read) : refus, mais Lineage passe en « Bridget indisponible » | **FINDING F1** | `10-FINDING-contexte-forge-read-rend-lineage-indisponible.png` |

Limites : le viewport 375 px est une fenêtre de navigateur, pas une vraie coque mobile, ni iOS, ni desktop. Les clics natifs (arrêt) passent par le daemon simulé. L'onglet preview est fermé.

## Findings pour Sol (correction minimale)

**F1 - Un refus de contexte forgé met le vrai fil hors service (sûreté, moyenne).**
`BridgetReader.resolveContext` refuse `thread.projectId !== input.projectId` en `project_mismatch` avant tout contact avec le daemon. `BridgetLineage.observeFailure` (`apps/server/src/bridget/BridgetLineage.ts`, branche `default`) envoie alors `bridget.lineage.unavailable` pour `input.threadId`. Cette commande ne vérifie pas le projet.
Reproduction : jeton `orchestration:read` seul, `bridget.lineage.read {projectId: <autre projet>, threadId: <fil racine réel>, action: "list"}`. Résultat : `project_mismatch`, mais 5 `thread.metadata-updated` (racine + 4 enfants) et `available:false` ; l'UI affiche « Lineage · Bridget indisponible », « Dernier état connu » et « Arrêter » désactivé jusqu'au prochain `list` réussi (replay `Z1`, capture 10, sorties `replay-r3-run1.txt`). Un détenteur d'un jeton en lecture désactive donc l'arrêt de n'importe quelle racine. Sur la matrice r2, R1/R3 avaient déjà laissé la racine à `false` sans que personne le voie.
Correction minimale : dans `observeFailure`, ne pas dégrader quand l'échec vient du contexte fourni par l'appelant. Par exemple, pour `project_mismatch` et `binding_unavailable`, relire le fil et ne dispatcher que si `thread.projectId === input.projectId`. Test à ajouter dans `BridgetLineage149.test.ts` avec le stub `getThreadRecords` ; je n'ai pas modifié `BridgetLineage.ts`.

**F2 - La barre « Tâche Bridget · lecture seule » masque la dernière ligne du journal (interface, faible).**
`apps/web/src/components/BridgetTaskJournal.tsx`, conteneur défilant : pas de marge basse. Au défilement maximal, la dernière ligne finit à y=788 alors que la barre commence à y=748 (1280x800) ou 752 (480x800). La dernière entrée est coupée sur environ 40 px (visible en bas de la capture 07, ligne #7). Correction : `pb` du conteneur défilant égal à la hauteur de la barre.

**Observations (pas de correction demandée) :**
- `snapshot_changed` épuisé (3 tentatives) fait passer la racine et tous les fils en indisponible : 135 événements dans le cas à 134 tâches, puis 135 de plus à la reprise. Cohérent avec le contrat (état courant non vérifiable), mais coûteux en cas de mutation soutenue du daemon.
- Une lecture `db_counts.sh` a rendu 137 fils une fois, puis 138 sur cinq lectures suivantes ; non reproduite, aucun événement de suppression. Je ne la qualifie pas de défaut.
- Le message « Le contenu déjà lu est conservé » s'affiche aussi quand rien n'a été lu (T4).

## Non vérifié

MCP privé authentifié, rotation/révocation, identité étrangère, S149-19 (la fixture ignore `--t3-thread`), résultat retenu natif, reprise sans relance native, double lancement natif, SC001, T037. `orchestration.launchThread` sur fil virtuel non lancé (risque de vrai fournisseur ; garde couverte par les tests serveur). Aucune case T036/T039/T040 ne peut être cochée sur cette seule preuve.

## Fichiers changés par moi (tests et outils uniquement), sha256 (16 premiers)

| Fichier | sha256 |
|---|---|
| `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetLineage149.test.ts` | `961e14b4acae4ee8` |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts` | `ba3210d373f8885a` |
| `ui-recipes/replay_r3.mjs` (nouveau) | `602630a9e446094c` |
| `ui-recipes/ev_counts.sh` (nouveau) | `7029efe1d499ff61` |
| `ui-recipes/scenario_step.mjs` (étapes `tick`, `remove-task`, `foreign-root`, `title`) | `e6c2740977309a26` |
| `ui-recipes/rpc149.ts` (cache/port par variables, mode `setup2`) | `4bda99c188b3fd06` |
| `ui-recipes/matrix149.mjs`, `run_recipe_server.sh`, `db_counts.sh` (cache/port par variables) | `fbba255c451be2e1`, `ce71c09a78d47dfb`, `3efb5a5cd70b46f2` |

Sources de production évaluées (non modifiées par moi) : `BridgetLineage.ts` `d58b30821704e664`, `BridgetTaskJournal.tsx` `75bc537b3a4518b0`, `Orchestrator.ts` `d3a770b04090439d`. `git diff HEAD` du WT : sha `fc9fa99f4ada2dca`, 29 fichiers suivis modifiés, 19 non suivis, base `33f6d04e116430bf7f0011902d6af3868163f2d1`.

## Commandes

```
export RECIPE149_CACHE=/Users/moi/.cache/bridget149-ui.r3 RECIPE149_T3_PORT=14774 RECIPE149_WEB_PORT=15734
PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:$PATH
# tests (depuis la racine du WT)
./node_modules/.bin/vp test run apps/server/src/bridget/BridgetLineage149.test.ts apps/server/src/bridget/BridgetReader149.test.ts apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts apps/server/src/orchestration-v2/ProjectionStore.bridget149.test.ts apps/server/src/orchestration-v2/ThreadLaunchService.bridget149.test.ts
# recette (depuis ui-recipes/)
node fixture_store_init.mjs $RECIPE149_CACHE/store.json $RECIPE149_CACHE/proj
zsh run_recipe_server.sh start ; vp dev (apps/web, PORT=15734 T3CODE_PORT=14774)
node --no-warnings matrix149.mjs all ; node --no-warnings replay_r3.mjs all
```

Les jetons `auth session issue` sortent sur stderr (pas stdout) : rediriger `2>&1`.

## État final et nettoyage

Onglet preview fermé. Serveur 74491, shell Vite 76625 et listener 76637 arrêtés un par un en SIGTERM après contrôle de PID, de commande et de cwd. Ports 14774 et 15734 libres, 0 processus fixture. Jetons bearer, jeton d'appairage et jeton de recette supprimés. Conservés : `/Users/moi/.cache/bridget149-ui.r3` (DB privée, `store.json` et copies `store.json.r3*`, `logs/`). Aucun job différé, aucune activation planifiée.
