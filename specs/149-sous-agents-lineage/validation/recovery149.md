# Scénarios dégradés natifs 149 - T039, O4, E2/E3 - ronde r3 (binaire release r8)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent). Aucun commit, aucune case cochée, aucune écriture de production, aucun service de production, aucun modèle, aucun Cargo.
Rapport r2 archivé : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/recovery149-r2.md`. Index des résultats r2 : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r2/INDEX.md`.

## Verdict : PARTIAL

F2 est corrigé et prouvé sans aucune mutation de base. Deux écarts restent. Le principal doit les trancher.

| Id | Niveau de preuve | État | Gravité |
|---|---|---|---|
| F2 (racine retenue bloquée après relance) | RÉEL | **Corrigé** (R9.4.a à R9.4.i : 9/9) | - |
| O4 (fournisseur orphelin après arrêt du daemon) | RÉEL | **BLOCK-O4 : non fermé** si le daemon est relancé aussitôt et si le fournisseur ne répond pas à `turn/interrupt` assez vite (O4.6 FAIL) | moyenne |
| E2 (tâche `queued` d'un propriétaire natif mort) | SIMULÉ (état durable rejoué) | **BLOCK-E2 : confirmé** : la racine attend sans fin, seul `cancel` la libère (E2.b FAIL) | faible en fréquence, moyenne en effet |
| E3 (descendant annulé + exécution active) | RÉEL | **Non bloquant** : la racine se termine (E3.b PASS). Écart d'hygiène F3 : l'exécution durable reste active (E3.c FAIL) | faible |

## Binaire, sources, garde

- Binaire release r8 IMMUABLE : `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-0a29ad9b2cdb`. SHA-256 `0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6`, recalculé au début et à la fin de CHAQUE recette (`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r3/shacheck.log`) : toujours égal au reçu `native149-release-receipt-r8.json`.
- Empreinte de production au début : `3943009ca82db59d...` (égale au reçu r8). **Dérive de source en cours de ronde, hors de ma propriété** : `crates/bridget-daemon/src/wrapper.rs` a changé à 19:19:03 (digest `49f3a304d9504fe6`, reçu r8 : `28e92599eee1bb03`). L'empreinte de production devient `4fa56fcac7b37220...`. Aucun autre fichier. Je n'ai pas modifié ce fichier. Conséquence : toutes les preuves de ce rapport valent pour le binaire r8 seulement. Elles ne couvrent pas le nouveau `wrapper.rs`. Il faudra un nouveau binaire et une nouvelle ronde pour le couvrir.
- Sources T3 : `git diff HEAD` = `59a54267ce7c1d2a` avant et après toute la ronde (inchangé). HEAD `33f6d04e11`.
- Fichier d'empreintes : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r3/fingerprints.json`.

## Comptes précis (dernière exécution de chaque recette)

| Recette | PASS | FAIL | Couches |
|---|---|---|---|
| `recovery149.ts` (T039) | 43 | 0 | 41 RÉEL, 2 OBS |
| `o4-149.ts` (O4) | 8 | 1 (O4.6) | 9 RÉEL |
| `e2e3-149.ts` (E2/E3) | 8 | 2 (E3.c RÉEL, E2.b SIMULÉ) | 6 RÉEL, 3 SIMULÉ, 1 OBS |
| `exec-lag149.ts` (E3d) | 4 | 0 | 1 RÉEL, 3 OBS |

Deux passes antérieures de `recovery149.ts` ont eu des échecs. Ils venaient de MES prédicats, corrigés avant la passe finale : passe 1 (37 PASS, 5 FAIL) avec des fournisseurs imbriqués bloquants qui devenaient des orphelins (cause O4), passe 2 (41 PASS, 2 FAIL) avec des prédicats trop stricts sur l'erreur du descendant (`unreachable` ou « stdout Codex fermé » selon l'ordre d'arrêt). Les résultats bruts sont conservés : `recovery149-premiere-passe.json`, `recovery149-passe2.json` dans `results-r3/`.

## T039 : résultats (43/43)

Les 31 premiers contrôles (R1 à R9.1b) sont inchangés par rapport à la ronde r2 et passent tous sur le binaire r8. Détail dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results-r3/recovery149.json`.

### R9.2 : coupure du daemon pendant une mission en vol (R9.2.a à R9.2.g)

Mission `SLOW_149:9000` en vol. VRAI SIGTERM du daemon (PID vérifié). Relance immédiate.

| Mesure | Résultat |
|---|---|
| État de la tâche dès le démarrage du daemon | `failed/unreachable` (avant la reconnexion du parent) |
| Tours portant la mission / PID distincts | 1 / 1 |
| Lancements fournisseur avant / après | 12 / 12 (t0, 7 s, 15 s) |
| Notice d'échec au parent | 1 |
| Rejeu de la requête | même tâche `failed`, aucun lancement |
| Exécution durable de la mission (R9.2.f) | `running/provider_accepted` rév. 1 -> `unreachable/daemon_restart` rév. 2 ; 12 lignes avant et après ; aucune autre ligne modifiée |

R9.2.g (observation) : le fournisseur bloquant a survécu 5,9 s après la sortie du daemon, `ppid 1`, sans SIGTERM de son wrapper (aucun événement `sigterm` dans son journal). C'est le cas O4 ci-dessous.

### R9.4 : racine retenue, descendant coupé (R9.4.a à R9.4.i), sans mutation de base

Deux racines `waiting_for_children` (résultats `m1` et `m2` déjà capturés, exécutions des racines `completed`), chacune avec un petit-enfant `working`. Le PARENT EST HORS LIGNE pendant la coupure et après la relance.

| Étape | Observé |
|---|---|
| Avant la coupure | racines `waiting_for_children`, résultats `fixture149-answer:m1/m2` ; exécutions des petits-enfants actives (R9.4.a) |
| Daemon relancé, parent TOUJOURS hors ligne | en 4,7 s les deux racines passent `result_available`, résultat INTACT, `result_sent` faux, erreur transitoire `native_result_owner_offline` ; stable 6 s plus tard ; petits-enfants `failed` (R9.4.b) |
| Exécutions durables avant / après (R9.4.c) | 16 lignes / 16 lignes ; aucune rouverte ; aucune encore active ; les exécutions des petits-enfants sont closes (`unreachable/daemon_restart`, ou `failed/provider_failed` quand l'arrêt coopératif du wrapper précède celui du daemon) |
| Lancements et tours (R9.4.d) | inchangés ; `m1`, `m1g`, `m2`, `m2g` : 1 tour chacun ; 0 fournisseur vivant |
| Parent reconnecté (R9.4.e) | UNE remise par racine, corps exact, expéditeur = l'enfant, `result_sent` vrai, erreur effacée, rien du petit-enfant remis, aucun doublon pendant 6 s |
| CLI Lineage (R9.4.f) | racine `result_available` + résultat exact, petit-enfant `failed` |
| 2e redémarrage (R9.4.g) | la remise ACCUSÉE (`m1`) n'est PAS rejouée ; tâches et exécutions identiques ; aucun lancement |
| Contrôle (R9.4.i) | `cancel` d'une racine déjà remise : état inchangé, aucune remise |

Observations : (1) R9.4.h : la remise NON accusée (`m2`) est rejouée UNE fois après le 2e redémarrage, avec le MÊME identifiant de message (`native-result-<tâche>`). C'est une livraison au moins une fois, pas un doublon de résultat. (2) La lecture CLI Lineage pendant que le parent est hors ligne rend `binding_unavailable` (liaison du fil absente) : voulu, pas un défaut.

Pause de 2,5 s entre l'arrêt et la relance dans R9.4 : elle isole F2 de O4 (voir ci-dessous). R9.2 garde la relance immédiate.

### R9.6, R9.5, O6, O7

R9.6 PASS (mort d'un fournisseur en vol, daemon vivant : pas de relance, tâche `failed`, 1 notice). R9.5 PASS (17 nonces, aucune mission portée par deux tours, 8 démarrages du daemon). O6 inchangé : `queued` ne s'expose pas par SIGTERM (voir E2). O7 inchangé : un fournisseur lancé pendant l'arrêt n'a ni mission ni relance.

## O4 : fournisseur et arrêt du daemon (9 contrôles : 8 PASS, 1 FAIL)

Script : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/o4-149.ts`. Résultat : `.../results-r3/o4-149.json`. Fixture conservée : `/Users/moi/.cache/bridget149-native-interop.GyH22l`.

Arbre de processus (O4.0 PASS, 3 cas) : daemon -> `bridget managed-wrapper` (groupe propre) -> fournisseur (enfant DIRECT du wrapper, groupe de processus PROPRE : pgid = son PID, différent de celui du wrapper). La variable d'amorce native est dans l'environnement du wrapper (O4.1 PASS ; macOS n'expose pas `ps -o caught`, le gestionnaire se prouve par le comportement).

Durée de vie après le SIGTERM du daemon (T0 = envoi du SIGTERM ; échantillon 20 ms) :

| Cas | Fournisseur | Relance du daemon | Wrapper mort | Fournisseur mort | Événements du fournisseur |
|---|---|---|---|---|---|
| A | coopératif (lit stdin, battement) | aucune | 1,03 s | 1,03 s | `interrupt` du tour EXACT à 1,01 s, battement stoppé (dernier à 0,93 s), puis `sigterm` |
| B | bloquant (ne lit pas stdin) | aucune | 4,05 s | 4,05 s | `sigterm` à 4,04 s (1 s de lecture + 3 s de borne de `turn/interrupt`) |
| C | ignore SIGTERM 8 s | aucune | 6,38 s | 6,36 s | `sigterm` ignoré à 4,02 s ; le wrapper attend la fin naturelle du tour ; pas de SIGKILL (voulu) |
| **B1** | bloquant (SLOW 20 s) | **immédiate** | **1,61 s** | **18,3 s** | **aucun** `sigterm` : orphelin jusqu'à la fin naturelle du tour |
| A1 | coopératif | immédiate | 1,02 s | 1,02 s | `interrupt` exact puis `sigterm` |
| B2 | bloquant | après 6 s | 4,07 s | 4,05 s | `sigterm` à 4,05 s |

Borne effective mesurée : fournisseur qui répond : environ 1,0 s ; fournisseur bloquant avec daemon qui reste arrêté : environ 4,05 s ; fournisseur qui ignore SIGTERM : jusqu'à la fin de son tour (pas d'escalade).

**BLOCK-O4 (cause minimale, pas de correctif proposé ici).**
1. Le wrapper natif a besoin d'au moins 1 s (lecture à délai de 1 s) puis jusqu'à 3 s (`turn/interrupt` borné) avant d'envoyer SIGTERM au groupe de son fournisseur (`wrapper.rs` boucle puis `stop_native_mission` ; `codex_app_server.rs` lignes 1117-1139).
2. Un daemon relancé réconcilie les groupes périmés au démarrage : `daemon.rs:4549` appelle `reconcile_stale_groups(1 s)`. Pour chaque marqueur vivant, `managed_process.rs:1221-1254` envoie SIGTERM au groupe du WRAPPER, attend `timeout/2` = 0,5 s, puis envoie **SIGKILL** à ce groupe.
3. Le fournisseur dirige un AUTRE groupe (`process_group(0)`). Le SIGKILL du wrapper ne l'atteint pas, et le wrapper n'a plus le temps de l'arrêter. Le fournisseur reste orphelin (`ppid 1`) jusqu'à la fin naturelle de son tour.
4. Données : cas B1 (wrapper mort à 1,61 s, aucun événement `sigterm`, fournisseur mort à 18,3 s = fin de `SLOW_149:20000`). Fixture : `/Users/moi/.cache/bridget149-native-interop.GyH22l`. Journal du fournisseur : `.../evidence/provider.jsonl` dans cette fixture. Journaux du daemon : `.../logs/daemon-*.log` (ligne `arrêt daemon: groupes encore vivants après SIGTERM`).
5. Limites : le cas coopératif A1 passe de justesse (wrapper mort à 1,02 s, relance du daemon terminée à 1,08 s, reconnexion du parent comprise). Je n'ai pas mesuré sous launchd (un délai de relance plus long ferait passer le cas B2). Un vrai Codex orphelin pourrait écrire dans le projet pendant son tour ; non mesuré.

## E2 et E3 : diagnostic sur le binaire r8

Script : `.../native-network-recipes/e2e3-149.ts` ; outil SIMULÉ : `.../native-network-recipes/craft-queued.py`. Résultat : `.../results-r3/e2e3-149.json`. Fixture conservée : `/Users/moi/.cache/bridget149-native-interop.3znVhG`.

### E3 : descendant `cancelling` + exécution active, coupure du daemon (RÉEL)

- Fournisseur du descendant sourd à SIGTERM pendant 25 s : l'annulation individuelle reste `cancelling` (E3.a PASS : état `cancelling`, exécution `starting`, racine en attente).
- Contrôle (E3c PASS, daemon sain) : annulation individuelle -> descendant `cancelled`, exécution close, racine `result_available`, résultat remis UNE fois.
- Coupure en pleine annulation (E3.b PASS) : à la relance le descendant devient `cancelled` en 1,1 s, la racine `result_available` en 2,1 s et sa remise arrive UNE fois ; aucun nouveau lancement ni tour. **L'hypothèse E3 de blocage est réfutée sur ce chemin.**
- Hygiène (E3.c **FAIL**, écart F3) : l'exécution durable du descendant `cancelled` reste ACTIVE (`starting`) après la relance. Elle ne bloque pas la racine ici. Je n'ai pas isolé pourquoi `descendants_busy` l'ignore (hypothèse non vérifiée : le nom de l'équipier n'est plus retrouvé dans la flotte).
- E3b : la fenêtre « résultat capturé / exécution encore active » n'existe pas avec un `turn/completed` tardif : le résultat n'est capturé qu'à la fin du tour (E3b.a, observation). Coupure pendant un tour tardif (E3b.b PASS) : exécution `unreachable`, racine résolue.
- E3d (`exec-lag149.ts`) : sans coupure, 12 tâches sur 12 ont leur exécution close AU MÊME INSTANT que la capture du résultat (décalage 0 ms, L1 PASS). 6 essais de coupure dans la fenêtre : fenêtre jamais vue, aucune conclusion (L3/L4 en observation). **Non expliqué** : dans la première passe de `recovery149.ts`, l'exécution de la racine `m2` (résultat capturé, `result_available` après relance) est restée `running` pendant deux redémarrages. Je ne l'ai pas reproduite avec la précondition « exécutions des racines `completed` avant la coupure ». Données : `results-r3/recovery149-premiere-passe.json`.

### E2 : tâche `queued` dont le propriétaire est un enfant natif (SIMULÉ)

Pourquoi simulé : la fenêtre est la durée entre la transaction d'admission et le `tick`. `handle_with_fact` appelle `tick` dans le même appel (`native_delegation.rs:316-318`). Un SIGTERM laisse 1 s d'arrêt gracieux, pendant lesquelles le `tick` se termine : je le DÉDUIS, je ne l'ai pas mesuré. Seul un arrêt brutal (non fait : pas de `kill -9`) peut y tomber. L'état durable est rejoué exactement : `craft-queued.py` fait, daemon ARRÊTÉ, la transaction de `insert_projected` (`delegation_lineage.rs:149-159`) : 1 ligne `native_delegations` + `seq+1`. Rien d'autre n'existe après une admission non ticquée. La ligne clone l'enveloppe d'un descendant réel (même propriétaire natif, même parent).

| Contrôle | Niveau | Résultat |
|---|---|---|
| E2.a : tâche `queued` EXTERNE (propriétaire = fil T3 A reconnecté) | SIMULÉ | **PASS** : elle repart, termine, remise UNE fois (`xDelivered` 1) ; la tâche terminée d'origine n'est pas rouverte (2 tours `x1` : l'original et le clone) |
| E2.b : tâche `queued` d'un propriétaire natif mort, racine `waiting_for_children` | SIMULÉ | **FAIL (BLOCK-E2)** : après 45 s la racine est toujours `waiting_for_children`, la tâche toujours `queued`, le descendant réel `failed` |
| E2c : même clone, mais le propriétaire est en plein tour (racine `working`) à la coupure | SIMULÉ | **PASS** : racine `failed`, le nettoyage ANNULE la tâche `queued`, 1 notice d'échec, aucun lancement |
| E.rel : issue de secours | RÉEL | PASS : seul `cancel` du parent libère la racine bloquée (`cancelled`) ; le résultat retenu est perdu |

**BLOCK-E2 (cause minimale).** `tick` saute (`continue`) toute tâche `queued` dont le propriétaire n'est pas connecté (`native_delegation.rs` : `if !parent_live { ... continue; }`). Un propriétaire natif ne se reconnecte jamais. `descendants_busy` et `tasks.iter().any(!child.terminal())` comptent cette tâche comme active. La racine, elle, n'est pas `failed` : le nettoyage de `cancel_tree` ne s'applique pas à elle. Conditions réelles : racine dont le tour est FINI et tâche admise mais pas encore lancée au moment d'un arrêt brutal (délégation lancée sans attendre sa réponse). La variante réaliste E2c ne bloque pas. Fixture : `/Users/moi/.cache/bridget149-native-interop.3znVhG` (journaux `logs/daemon-*.log`, ligne de la tâche dans `state/bridget.db`).

## Ce qui n'a PAS été prouvé

- Aucun arrêt brutal (SIGKILL/OOM/coupure de courant) : interdit. E2 est donc au niveau SIMULÉ.
- Pas de launchd. Pas de vrai Codex : le fournisseur est un serveur fermé. Un vrai Codex répond plus vite à `turn/interrupt`.
- Pas de reprise de session de modèle. Le nouveau `wrapper.rs` (19:19) n'est pas couvert.
- Le cas `queued` non engagé en réel après SIGTERM reste couvert par le test unitaire seul (O6).
- Les tests unitaires et l'intégration Cargo de r8 (`daemon` 1492 + `transport` 333 = 1825) viennent du reçu. Je n'ai lancé aucun Cargo.

## Fichiers

Dossier des recettes : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/` (voir `README.md`). Résultats : `results-r3/`. Outils ajoutés : `codex149b.py`, `o4-149.ts`, `e2e3-149.ts`, `exec-lag149.ts`, `craft-queued.py`, `check-r3.sh`, `run-r3.sh`, `fingerprints-r3.py`.
