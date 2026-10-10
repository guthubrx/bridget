# Preuves natives réseau et reprise - session 149 - ronde r4 (tour final ciblé)

Date : 2026-10-10. Testeur : Sonnet 5.5 (sous-agent de test). Aucun modèle, aucun Cargo, aucune mutation de production, aucun Git.
Ce rapport ne remplace ni `recovery149.md` ni `interop149.md`. Il ferme trois findings de r3 sur le binaire release r9.

## 1. Verdict

| Finding r3 | Niveau | Résultat |
|---|---|---|
| O4.6 relance rapide, interruption bloquée | RÉEL | **PASS** 6/6 |
| E2 `queued` à propriétaire natif perdu | RÉEL (effets du daemon) + SIMULÉ (état de départ) | **PASS** 6/6 (5 RÉEL, 1 SIMULÉ) |
| F3 clôture CAS avant `cancelled` | RÉEL | **PASS** 4/4 |
| Total | | **16 PASS, 0 FAIL**, 0 SIM manquant, 0 processus résiduel |

Aucun défaut de produit bloquant. Deux observations nommées sont listées en section 5 (pas de correctif demandé ici).

## 2. Binaire et empreintes

| Élément | Valeur |
|---|---|
| Binaire | `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-release/bridget-abc850858975` |
| SHA-256 | `abc850858975fb5c7d733eb052be4fff6e3cb24537ce8b7d4475019b79abf6d8` |
| Contrôle début et fin de chaque recette | OK à 6 reprises (`native-network-recipes/results-r4/shacheck.log`), OK en contrôle final à 22:08:44 |
| Empreinte de production | `b2b87458cf3cec7989debeb91352b838c417c3eb48329d57099bec4c9a0d5f29`, identique à chaque contrôle |
| Reçu | `native149-release-receipt-r9.json` |

## 3. Résultats par recette

Scripts : `native-network-recipes/{o4-r4,e2-r4,f3-r4}.ts`, lancés par `run-r4.sh`. Résultats bruts sans secret : `native-network-recipes/results-r4/*.json`.
Ports : 14816 (O4), 14817 (E2), 14818 (F3). Les trois étaient libres avant et après.

### 3.1 O4.6 - relance immédiate, fournisseur à interruption bloquée (RÉEL, 6/6)

Montage : daemon réel, wrapper réel, faux app-server `codex149b.py` avec `SLOW_149:20000` (sommeil non interruptible). SIGTERM individuel du daemon, puis relance sans attendre le wrapper.

| Mesure | Valeur observée |
|---|---|
| Arbre avant coupure | daemon 55973 > wrapper 55994 (groupe propre) > fournisseur 55999 (groupe propre 55999, ppid = wrapper). Naissance : 22:03:45 pour les deux |
| Daemon 1 sorti | 1100 ms après le SIGTERM. Le wrapper et le fournisseur vivaient encore à ce moment |
| Fournisseur : SIGTERM reçu du wrapper | 4,07 s, non ignoré |
| Wrapper et fournisseur morts | 4076 ms (même échantillon) |
| Daemon 2 prêt | 4083 ms, donc après la mort du wrapper. Il a attendu l'arrêt coopératif (borne 8 s), sans SIGKILL |
| Prompts du nonce | 1. Lancements de fournisseur : 1 avant, 1 après |
| État durable | tâche `failed/unreachable`, exécution `unreachable/daemon_restart`, marqueur natif supprimé |
| Processus restants 3 s plus tard | 0 |

Le cas de r3 (orphelin 18,3 s) est fermé. Le cas 6,38 s signalé dans le brief tombe aussi sous la borne.

### 3.2 E2 - `queued` imbriqué à propriétaire perdu (6/6)

Fixture de départ : racine réelle `waiting_for_children` avec résultat réel `fixture149-answer:e1` et exécution racine `completed`. Le descendant réel est `working`.
**SIMULÉ** (E2r4.b) : daemon arrêté, transaction exacte d'admission rejouée par `craft-queued.py` pour deux lignes `queued` : un clone du descendant (propriétaire = enfant natif, parent = racine) et un clone externe (propriétaire = fil T3 A). Après la relance, la recette n'écrit plus rien en base.

| Étape | Niveau | Observé |
|---|---|---|
| E2r4.a avant coupure | RÉEL | racine + résultat capturé, descendant `working`, exécution active |
| E2r4.b état simulé | SIMULÉ | deux `queued` insérés, aucune exécution, racine et descendant intacts |
| E2r4.c parent hors ligne | RÉEL | résolution en 1075 ms. `queued` imbriqué `failed/unreachable`, descendant `failed/unreachable`, racine `result_available`, résultat intact, `result_sent` faux (erreur transitoire `native_result_owner_offline`), stable 6 s, 0 lancement. `queued` externe resté `queued` (parent absent) |
| E2r4.d exécutions | RÉEL | 3 lignes avant, 3 après. Aucune rouverte, aucune active. Exécution du descendant : `unreachable/daemon_restart` |
| E2r4.e reconnexion | RÉEL | remise du résultat racine **1 fois**, corps exact, expéditeur = l'enfant, `result_sent` vrai, rien du descendant. `queued` externe repris : `result_available`, 1 remise, tâche d'origine non rouverte (2 tours `x1`). Pas de faux échec |
| E2r4.f 2e redémarrage | RÉEL | 0 remise rejouée, tâches et exécutions identiques, 0 lancement, 0 fournisseur vivant |

Le blocage de r3 (racine en attente sans fin, résultat perdu) ne se reproduit pas. Niveau exact : l'état de départ `queued` est simulé. L'arrêt de l'admission par un vrai SIGTERM n'est pas atteignable (déjà établi en r3). Je ne prétends pas qu'un crash naturel l'ait produit.

### 3.3 F3 - exécution fermée avant `cancelled` (RÉEL, 4/4)

Annulation par la CLI Lineage (l'annulation MCP d'un descendant est refusée `native_delegation_refused`, comme en r3). Invariant échantillonné toutes les 100 ms : « jamais `cancelled` avec exécution active ».

| Étape | Observé |
|---|---|
| F3.1 daemon sain (135 échantillons) | `cancelling/starting` puis `cancelled/failed`. 0 violation. Exécution fermée `failed/provider_failed`. Racine `result_available` |
| F3.2.a fenêtre | descendant `cancelling` avec exécution `starting` (état exact du finding r3), racine `waiting_for_children` |
| F3.2.b après SIGTERM du daemon en plein `cancelling` | descendant `cancelled`. Exécution `unreachable/native_cancelled_restart` (révision 1). 0 violation sur tous les échantillons. Racine `result_available`, 1 remise, 0 nouveau lancement |
| F3.2.c témoin terminal | exécution `completed` (révision 2, motif `completed`) identique avant/après. Aucune exécution terminale rouverte |

Trois tables capturées avant/après dans `results-r4/f3-r4.json` (`notes.f32Before`, `notes.f32After`) : `native_delegations`, `executions`, `agent_links`.
Remarque de calibrage : le fournisseur sourd de r3 (`IGNORE_TERM_149:25000`) bloque le démarrage du daemon (voir 5.1). J'ai utilisé `IGNORE_TERM_149:6000 SLOW_149:6000` pour rester dans la borne de 8 s.

## 4. Comptes

| | PASS | FAIL | SIM | Limites |
|---|---|---|---|---|
| O4.6 | 6 (dont 1 OBS) | 0 | 0 | une seule relance rapide, un seul fournisseur bloquant |
| E2 | 6 | 0 | 1 (E2r4.b, état de départ) | crash naturel de la fenêtre admission/tick non atteignable |
| F3 | 4 | 0 | 0 | fournisseur sourd à 6 s seulement (8 s et plus : voir 5.1) |
| **Total** | **16** | **0** | **1** | |

## 5. Observations nommées (aucune ne bloque ces trois findings)

### 5.1 Un fournisseur natif sourd à SIGTERM plus longtemps que la borne coopérative empêche le démarrage du daemon (RÉEL, par conception)

Fixture `/Users/moi/.cache/bridget149-native-interop.ISmsPZ` (première tentative F3, logs `logs/daemon-2.log`).
Cas : descendant avec `IGNORE_TERM_149:25000`, SIGTERM du daemon, relance immédiate. Le nouveau daemon sort avec :
`daemon error: statut bootstrap invalide: arrêt coopératif natif incomplet pour e533f90e-... (pgid 69721)`.
Le marqueur natif est conservé, aucun SIGKILL, aucun fournisseur relancé. C'est le comportement de la section 3.1 de la revue de sources (`native-restart-final-source-review-sonnet-r2.md`, point O-1 : refus fermé). Conséquence d'exploitation : le daemon reste arrêté tant que le fournisseur ne meurt pas. Je ne propose pas de correctif. Le principal décide si la procédure d'exploitation doit le dire.

### 5.2 Lien de flotte `agent_links` laissé `open` pour des tâches terminées par la reprise (RÉEL, non comparé à r8)

| Fixture | Lien resté `open` (rev 1) | Tâche |
|---|---|---|
| `.../bridget149-native-interop.LhJlCa` (O4) | oui | `failed/unreachable` |
| `.../bridget149-native-interop.MyCzuH` (E2) | racine et descendant | racine `result_available` remise et accusée ; descendant `failed/unreachable` |
| `.../bridget149-native-interop.dMsmZc` (F3) | racine du cas coupé | `result_available`, remise, `cleanup_done` vrai |

Les liens des tâches terminées sans reprise sont `closed` (rev 2), y compris le `cancelled` du cas F3.2. Je n'ai ni asserté ni cherché l'effet : aucun lecteur testé ici ne dépend de cet état. À noter seulement. Si cet écart compte, c'est un point pour le principal, pas pour moi.

## 6. Hygiène

| Point | État |
|---|---|
| PID signalés | uniquement ceux de la recette, un par un, parent et commande vérifiés, SIGTERM, jamais `-9`, jamais `pkill`. Aucun SIGTERM ignoré |
| Processus fixture restants | 0 (contrôle par chemin de fixture et par noms `codex149b`/`managed-wrapper`) |
| Ports 14816, 14817, 14818 | libres |
| Configs et preuves | `agents.json` et `proof-*.json` (credentials) supprimés des quatre fixtures. DB et logs conservés comme preuve |
| Dossiers `/tmp/b149n.*` de la ronde | supprimés (4, tous créés pendant mes recettes) |
| Fixtures d'autrui (Sonnet r7 et autres) | non touchées |
| Première tentative F3 | échec de montage de la recette (annulation MCP sans repli CLI) puis refus de démarrage du daemon (5.1). Corrigée côté recette seulement. Les données de ce premier passage ne sont pas conservées dans `results-r4` (écrasées par le second passage) |
| Production | aucun fichier modifié. Empreinte identique au début et à la fin |

## 7. Fichiers

| Chemin absolu | Rôle |
|---|---|
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/o4-r4.ts` | O4.6 |
| `.../native-network-recipes/e2-r4.ts` | E2 |
| `.../native-network-recipes/f3-r4.ts` | F3 |
| `.../native-network-recipes/run-r4.sh`, `check-r4.sh` | lancement et contrôles SHA/empreinte |
| `.../native-network-recipes/results-r4/` | résultats JSON, sorties, `shacheck.log` |
