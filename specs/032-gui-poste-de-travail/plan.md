# PLAN 032 — GUI poste de travail

**Créé** : 2026-08-25 21h15 · **Spec** : `spec.md` · **Cible** : utilisable au
réveil du 26/08.

## 1. État mesuré de l'existant

| Élément | Mesure | Conséquence |
|---|---|---|
| `crates/bridget-daemon/src/ui.rs` | 780 lignes | socle réutilisable |
| Routes | `/v1/snapshot`, `/v1/watch`, `/v1/journal` | **rien à créer** |
| Lecture seule | **une seule condition** : `if request.method != "GET" → 405` | changement localisé |
| Authentification | **jeton existe déjà** (`UiRelayConfig.token`, UUID) | rien à concevoir |
| Écoute | loopback, port éphémère | rien à changer |
| `UiSnapshotV1` | `agents`, `open_requests`, `missions`, `recovery_losses` | **la liste de gauche est déjà servie** |
| `UiJournalEventV1` | enveloppe `DaemonToWrapper` | le fil est déjà servi |
| Front | **153 lignes HTML/JS en dur dans le Rust** | à remplacer, pas à étendre |

**Conclusion structurante : le socle serveur existe. Le travail est à 80 % côté
page, à 20 % côté relais.**

## 2. Découpage en lots parallélisables

Découpés **là où les fichiers ne se touchent pas**.

### L0 — Contrats *(bloquant, un seul agent, court)*

Fige la **forme exacte** des données échangées, avant que quiconque code.
Sans ce lot, celui qui écrit la page invente une forme que le relais ne produira
pas.

**Produit** : `contracts/` — forme d'un envoi, d'une trace inter-agents, d'un
niveau, d'un accusé.
**Fichiers** : `specs/032-gui-poste-de-travail/contracts/*`
**Ne touche aucun code.**

### L1 — Écriture au relais *(le cœur)*

Lever la lecture seule : accepter `POST /v1/send`, transmettre au daemon,
répondre. Le jeton existant protège la route.

**Fichiers** : `crates/bridget-daemon/src/ui.rs`
**Dépend de** : L0 (forme de l'envoi).
**Critères** : AC6.

### L2 — Trace inter-agents *(projection)*

Calculer depuis le ledger : qui a parlé à qui, direction, nombre, position
chronologique. **Ajout au snapshot**, pas une route nouvelle.

**Fichiers** : `crates/bridget-daemon/src/ui.rs` (structure `UiSnapshotV1`)
**⚠ Conflit avec L1** — même fichier. Voir §3.
**Critères** : AC3.

### L3 — Raisonnement Codex

Brancher `item/reasoning/summaryTextDelta`, `summaryPartAdded`, `textDelta`
dans l'aiguillage existant.

**Fichiers** : `crates/bridget-transport/src/codex_app_server.rs`
**Référence** : `~/11.Repositories/t3code/packages/effect-codex-app-server`
**Critères** : AC7, AC8.

### L4 — Raisonnement Cursor

Cesser d'écarter `agent_thought_chunk` (`acp.rs:1500`). Ajouter aussi
`tool_call` / `tool_call_update` pour le niveau 1.

**Fichiers** : `crates/bridget-transport/src/acp.rs`
**Référence** : `~/11.Repositories/t3code/packages/effect-acp`
**Critères** : AC7, AC8.

### L5 — Page : liste + fil *(le gros morceau)*

Liste d'agents, fil en bulles, séparateurs, événements système, charte.

**Fichiers** : nouveau `crates/bridget-daemon/assets/ui/` (html, css, js)
**Dépend de** : L0.
**Critères** : AC2, AC9.

### L6 — Page : saisie, trace, dépli, comportements

Zone unifiée, envoi, dépli de trace, panneau latéral, **défilement qui ne saute
pas**, **reconnexion visible**.

**Fichiers** : mêmes assets que L5 — **séquentiel après L5**, ou même agent.
**Critères** : AC1, AC4, AC5, AC10, AC11.

## 3. Ordre et conflits — imposé par les faits

**L1 et L2 touchent le même fichier.** Trois options, une seule est saine :

- ~~parallèle~~ → conflit garanti sur `ui.rs` ;
- **L1 puis L2, séquentiel** → retenu ;
- fusionner → un lot trop gros pour une relecture ciblée.

**Leçon appliquée du 25/08** : deux lots assemblés ont révélé des défauts
qu'**aucun ne montrait seul**, et un lot vert en aval **masquait** le défaut
d'un lot intermédiaire. D'où **une étape de composition explicite** (§5).

```
L0 (contrats)
 ├─► L1 ──► L2        [ui.rs, séquentiel]
 ├─► L3               [codex_app_server.rs]
 ├─► L4               [acp.rs]
 └─► L5 ──► L6        [assets, séquentiel]
              │
              └─► COMPOSITION ──► relecture finale
```

**Aucun lot ne touche la base de données** — donc **aucune contrainte d'ordre de
migration**, contrairement à la chaîne d'aujourd'hui. À **vérifier** au moment
des tâches, pas à supposer.

## 3bis. Emprunts — méthode et frontière

**`~/11.Repositories/t3code` est sous licence MIT.** Reprise de code autorisée
avec conservation de la mention de copyright. Dans les faits marginal —
TypeScript/React d'un côté, Rust/JS nu de l'autre.

> **RÈGLE STRUCTURANTE : conception de Grok Bot, rendu de T3 Code.**

**Pourquoi cette séparation, et elle commande le modèle de données :**

| | à gauche | modèle mental |
|---|---|---|
| **T3 Code** | des **conversations** — fils de travail, éphémères, qui se closent | *un sujet, une discussion* |
| **Grok Bot** | des **agents** — permanents, avec identité et métier | *un collègue, joignable* |
| **Nous** | **des agents** | `rc1` existe indépendamment de ce dont on parle avec lui |

**Conséquence sur la ligne de liste** — elle porte plus que chez eux :
nom · **état** (vivant / occupé / éteint) · **machine** · dernier message ·
heure · pastille.
*Motif : c'est ce qui dira d'un coup d'œil qui travaille et qui est mort — ce
qui a manqué toute la journée du 25/08.*

**Leur « fil clos » ne se transpose pas** : chez nous ce n'est pas la
conversation qui se clôt, **c'est l'agent qui s'arrête**. Bandeau : « cet agent
est arrêté ; envoyer un message le relance ».

**Ce qu'on relève chez eux, avant d'inventer :**

| quoi | où | pour |
|---|---|---|
| noms d'événements des protocoles | `packages/effect-acp`, `packages/effect-codex-app-server` | L3, L4 |
| formes de données | `packages/contracts/src` | L0 |
| **valeurs de style et vocabulaire** | `apps/web/src/index.css` — `--color-border-subtle`, `--chat-composer-glass-surface`, `--code-background`, `--app-chrome-background` | L5, L6 |

**Relever avant d'inventer.** Leçon du 25/08 : le référent a cherché sur le web
et mesuré notre propre code **sans jamais regarder les dépôts voisins**, alors
que l'implémentation de référence était sur la machine.

## 4. Répartition

| Lot | Auteur | Machine | Relecteur | Machine |
|---|---|---|---|---|
| L0 | 1 agent | distante | — *(contrats, relus par tous)* | — |
| L1 → L2 | 1 agent | distante | 1 agent | **locale** |
| L3 | 1 agent | distante | 1 agent | **locale** |
| L4 | 1 agent | distante | 1 agent | **locale** |
| L5 → L6 | 1 agent | distante | 1 agent | **locale** |

**Auteurs à distance, relecteurs en local — et ce n'est pas du confort.**
Mesure du 25/08 : plusieurs défauts n'ont été trouvés **que** parce que le
relecteur était sur un autre système — un chemin accepté sous Linux, refusé sous
macOS ; une évasion refusée sous macOS **par accident**, acceptée sous Linux.

**Régime : un relecteur par lot.** Pas de jury — mesure du 25/08 : douze lots
sur douze en jury complet alors que le critère en donnait quatre, sans que
personne le voie.

**Effectif : 5 auteurs + 4 relecteurs.** 6 agents vivants, tous `codex` sur la
machine distante → **il faut en réveiller sur la machine locale**, et au moins
un d'un autre fournisseur pour la contre-revue.

## 5. Procédé imposé — tiré de la journée du 25/08

Chaque lot **déclare** :

1. **son périmètre de test** — le paquet **et ses dépendants** : ce qui casserait
   *si on modifiait ce paquet*, jamais ce qu'il consomme. Le paquet se détermine
   **par le diff** (`git diff --name-only`), jamais par le nom du lot.
   Closures mesurées, reprises telles quelles du registre 011 :
   `bridget-core` → tout le workspace · `bridget-transport` → transport + maicie
   + daemon · `maicie` → maicie **+ daemon** · `bridget-daemon` → **daemon seul,
   personne n'en dépend**.
   *Corrigé le 25/08 23h31 : ces lignes avaient été recopiées à l'envers ici et
   dans `tasks.md`, faisant de `maicie` un dépendant de `bridget-daemon` alors
   qu'il en est la dépendance. Le registre, lui, était juste.*
2. **l'univers listé avant comptage** — `-- --list` puis le compte ;
3. **le compte** `passed` / `failed`, **jamais le code retour** ;
4. **le nom du test mort** sous mutant, **jamais son cardinal** — un cardinal
   identique peut cacher **deux substitutions** ;
5. **ce qu'il ne couvre pas**, en clair.

**Les agents se parlent directement.** Le référent ne sert pas de relais : il a
transmis des chemins illisibles en faisant l'intermédiaire, et deux agents qui
se sont coordonnés seuls sont allés plus vite.

**Étape de composition explicite** avant la relecture finale : assembler les
six lots et **mesurer chaque tête isolée ET la composition**.

## 6. Ce qui n'est pas fait cette nuit

Reporté et **écrit pour ne pas se perdre** : jauge de contexte par agent (source
identifiée : `usage_update` en ACP), alerte « demande lente », coût par agent
(**89 entrées, 89 « inconnu »**), pièces jointes, sections, recherche.

## 7. Risques

| Risque | Parade |
|---|---|
| L5/L6 sous-estimés — 153 → ~2000 lignes | un seul agent sur les deux, séquentiel ; charte en règles pour éviter les allers-retours de goût |
| L3/L4 : le flux n'arrive pas comme documenté | **implémentation de référence lisible** sur la machine ; mesurer avant de conclure |
| Découverte à 4 h du matin | L3/L4 sont **facultatifs pour AC1-AC6** — l'interface est utilisable sans eux |
| Contre-revue impossible | 6 agents vivants du **même fournisseur** → en réveiller un autre, ou le noter |
