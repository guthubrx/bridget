# TÂCHES 032 — GUI poste de travail

**Créé** : 2026-08-25 21h30 · **Gate `reuse-audit.md`** : `OK`, franchi.
**Cible** : interface utilisable au réveil du 26/08.

> **Objectif vérifiable en une minute :** ouvrir l'interface, voir ses agents
> dans une liste avec leur état, écrire à l'un d'eux, recevoir sa réponse dans
> le fil — et **qu'un message qui arrive pendant la saisie ne touche ni le
> texte ni la position de lecture.**

## Procédé imposé à tous les lots

Tiré des défauts mesurés le 25/08. **Une livraison qui l'ignore est refusée.**

1. **Périmètre de test** : le paquet **et ses dépendants** — ce qui casserait
   *si on modifiait ce paquet*, jamais ce qu'il consomme.
   **Sens de la flèche, corrigé le 25/08 23h31** : `bridget-daemon` **consomme**
   `maicie` en dur ; `maicie` est donc sa **dépendance**, pas son dépendant.
   Un diff limité à `bridget-daemon` a pour closure `bridget-daemon` seul —
   personne ne l'importe. Inversement, un diff dans `maicie` **doit** faire
   jouer `bridget-daemon`.
   *La formulation antérieure désignait le mauvais côté et faisait jouer une
   closure plus large que nécessaire : coûteux, et faux comme règle.*
2. **Univers listé avant comptage** : `-- --list` puis le compte.
3. **Le compte** `passed` / `failed` — **jamais le code retour**.
4. Sous mutant : **le NOM du test mort**, jamais son cardinal.
   *Un cardinal identique peut cacher deux substitutions qui se compensent.*
5. **Deux sha mesurés** — `HEAD` et `origin/<branche>` — identiques.
6. **Ce que le lot ne couvre pas**, écrit en clair.
7. Matériel rendu après **`ps` ET `lsof`**, dans cet ordre : **éteindre puis
   supprimer** — sinon l'espace n'est pas rendu.

**Les agents se parlent directement.** Le référent ne relaie pas : il a
transmis des chemins illisibles entre deux machines en servant d'intermédiaire.

---

## L1 — Écriture au relais *(cœur — AC6)*

**Fichier** : `crates/bridget-daemon/src/ui.rs` · **Contrat** : C1

- [ ] T1.1 Accepter `POST /v1/send` — `serve_connection` refuse aujourd'hui
      tout sauf `GET` (une seule condition).
- [ ] T1.2 Valider le corps selon C1 ; codes d'erreur **de l'ensemble fermé**,
      jamais de prose libre.
- [ ] T1.3 Transmettre au daemon par la socket, rendre `delivery_id`,
      `issued_at`, `status`.
- [ ] T1.4 Appliquer le **jeton existant** (`UiRelayConfig.token`) à la route.
- [ ] T1.5 Oracle : envoi valide → 202 + `delivery_id` non vide.
- [ ] T1.6 Oracle : destinataire inconnu → `unknown_recipient`, **rien
      d'archivé**.
- [ ] T1.7 Oracle : `GET` sur `/v1/send` → 405 *(contrôle positif : la route
      ne relâche pas la garde des autres méthodes)*.
- [ ] T1.8 Mutant : retirer la validation du destinataire → **nommer le test
      mort**.

> **Interdit** : exposer une route d'approbation de profil ou de routine.
> **Frappe humaine obligatoire** (ADR 011).

## L2 — Trace inter-agents *(AC3 — après L1, même fichier)*

**Fichier** : `crates/bridget-daemon/src/ui.rs` · **Contrat** : C2

- [ ] T2.1 Agréger `LedgerEntry` (`sender`, `target`, `ts`, `id`) en
      `peer_exchange`. **Aucune donnée nouvelle, aucune table.**
- [ ] T2.2 Calculer `direction` (`in` / `out` / `both`) et `count` — `count`
      **toujours présent, même à 1**.
- [ ] T2.3 Placer `at` à sa **position chronologique réelle**.
- [ ] T2.4 Ajouter à `UiSnapshotV1` et pousser par `/v1/watch`.
- [ ] T2.5 Oracle : deux messages A→B et B→A entre deux bulles → **une** trace
      `both`, `count: 2`.
- [ ] T2.6 Oracle : un seul message → `count: 1`, direction correcte.
- [ ] T2.7 Mutant : forcer `direction` à une valeur fixe → **nommer le test
      mort**.

## L3 — Raisonnement et actes Codex *(AC7, AC8)*

**Fichier** : `crates/bridget-transport/src/codex_app_server.rs` · **Contrat** : C3
**Référence MIT** : `~/11.Repositories/t3code/packages/effect-codex-app-server`

- [ ] T3.1 **Lire la référence avant d'inventer.**
- [ ] T3.2 Brancher `item/reasoning/summaryTextDelta`, `summaryPartAdded`,
      `textDelta` → `reasoning`.
- [ ] T3.3 Brancher `item/commandExecution/outputDelta` → acte `command`.
- [ ] T3.4 Brancher `item/fileChange/patchUpdated` → acte `file`.
- [ ] T3.5 Brancher `item/plan/delta` → acte `plan`.
- [ ] T3.6 `item/*/requestApproval` → acte `approval` — **affiché, jamais
      validable**.
- [ ] T3.7 Oracle : flux avec raisonnement → `available: true` + résumé.
- [ ] T3.8 Oracle : flux **sans** raisonnement → `available: false`.
      *Contrôle positif : l'instrument voit la présence avant de juger
      l'absence.*
- [ ] T3.9 Mutant : ignorer `item/reasoning/*` → **nommer le test mort**.

## L4 — Raisonnement et actes Cursor *(AC7, AC8)*

**Fichier** : `crates/bridget-transport/src/acp.rs` · **Contrat** : C3
**Référence MIT** : `~/11.Repositories/t3code/packages/effect-acp`

- [ ] T4.1 **Lire la référence avant d'inventer.**
- [ ] T4.2 `acp.rs:1500` rejette tout sauf `agent_message_chunk` — **cesser
      d'écarter** `agent_thought_chunk`.
- [ ] T4.3 Brancher `tool_call` et `tool_call_update` → acte `tool`.
- [ ] T4.4 Oracle : `agent_thought_chunk` reçu → `reasoning.available: true`.
- [ ] T4.5 Oracle : aucun chunk de pensée → `available: false`.
      *Cas documenté : chez Gemini il n'est **jamais** émis.*
- [ ] T4.6 Mutant : rétablir le filtre → **nommer le test mort**.

## L5 — Page : liste et fil *(AC2, AC9)*

**Fichiers** : `crates/bridget-daemon/assets/ui/` *(à créer)* · **Contrats** : C4, C2

- [ ] T5.1 Créer `assets/ui/` — `index.html`, `app.js`, `theme.css`.
- [ ] T5.2 **Retirer les 153 lignes embarquées** de `ui.rs` ; servir depuis
      `assets/`. *(Pas de duplication : l'embarqué disparaît.)*
- [ ] T5.3 Liste d'agents : nom · **état** · **machine** · dernier message ·
      heure · pastille.
      **Ce sont des AGENTS, pas des conversations** — d'où état et machine.
- [ ] T5.4 Tri par dernier message ; agents arrêtés en section **masqués**.
- [ ] T5.5 Fil : bulles agent à gauche, utilisateur à droite ; séparateurs de
      date ; événements système **en gris, centrés**.
- [ ] T5.6 Consommer `/v1/snapshot` puis s'abonner à `/v1/watch`
      — **abonnement AVANT snapshot**, comme le fait la route.
- [ ] T5.7 Charte, **relevée sur T3 Code (MIT)** :
      `apps/web/src/index.css` — `--color-border-subtle`,
      `--chat-composer-glass-surface`, `--code-background`,
      `--app-chrome-background`.
- [ ] T5.8 Contrôle positif de charte : **aucune bordure**, **un seul accent
      coloré**, **deux niveaux de gris de texte**.

## L6 — Page : saisie et comportements *(AC1, AC4, AC5, AC10, AC11, AC12)*

**Fichiers** : mêmes assets — **après L5, même agent** · **Contrats** : C1, C2, C5

- [ ] T6.1 Zone unifiée, **boutons à l'intérieur** : à gauche machine et
      `reply`, à droite activité et envoi.
- [ ] T6.2 **AC1 — un message qui arrive ne modifie ni le texte en cours, ni
      quoi que ce soit d'autre. Rien ne part sans appui.**
- [ ] T6.3 **AC10 — le défilement ne saute jamais.** Si l'utilisateur lit plus
      haut : position figée, bouton « nouveaux messages » ; **c'est lui qui
      descend**.
- [ ] T6.4 **AC11 — coupure visible.** `relay_state` → bandeau, reconnexion
      automatique. *Le daemon a été redémarré deux fois le 25/08.*
- [ ] T6.5 Trace : clic sur la ligne → dépli si `count <= 3`, sinon panneau ;
      **même geste**. Clic sur l'avatar → conversation de l'agent.
- [ ] T6.6 Panneau latéral pour ce qui ne tient pas dans une bulle.
- [ ] T6.7 Niveaux : replié par défaut avec la durée ; dépli → actes ;
      raisonnement **imbriqué** dans les actes.
- [ ] T6.8 **AC8 — « raisonnement non fourni »**, jamais un vide.
- [ ] T6.9 Bandeau agent arrêté : « envoyer un message le relance ».
      *Ce n'est pas un fil qui se clôt, c'est un agent qui s'arrête.*
- [ ] T6.10 **AC12 — `bridget ui` suffit.** Aucune compilation, aucune
      installation.
- [ ] T6.11 Vocabulaire : **« injecté » / « en vol »**, jamais « reçu ».

## L7 — Page : contenu et chronologie lisibles *(AC14, AC15, AC16)*

**Fichiers** : `crates/bridget-daemon/assets/ui/` · **Contrats** : C2,
`/v1/journal` existant

- [x] T7.1 Indexer les corps `turn_start` et `prompt_dispatched` par
      `message_id`, depuis `from_seq=0` pour traverser minuit, sans modifier le
      relais Rust.
- [x] T7.2 Projeter chaque message entrant riche en bulle à droite, qu'il
      vienne de l'utilisateur ou d'un autre agent, et la réponse de l'agent
      courant en bulle à gauche, avec leur texte exact.
- [x] T7.3 Résoudre chaque `delivery_id` d'une trace depuis les journaux des
      deux participants ; le dépli rend les textes disponibles, ou un état
      d'indisponibilité explicite, jamais les identifiants.
- [x] T7.4 Afficher l'heure locale sur chaque bulle et chaque trace ; grouper
      par journée locale tout en triant sur le `ts` d'émission.
- [x] T7.5 Oracle de présence sur les textes exacts avant l'oracle d'absence
      des identifiants ; mutant nommé et lecture réelle dans un navigateur.
- [x] T7.6 Écrire explicitement les limites du lot : aucune nouvelle donnée,
      aucune route, aucun changement d'authentification ou de relais Rust.

## Composition — après les sept lots

- [ ] TC.1 Assembler et mesurer **chaque tête isolée ET la composition**.
      *Le 25/08, un lot vert en aval masquait le défaut d'un lot
      intermédiaire.*
- [ ] TC.2 Vérifier qu'**aucun lot n'a touché la base** — donc aucune
      contrainte d'ordre. **À mesurer, pas à supposer.**
- [ ] TC.3 Parcours complet : ouvrir · voir la liste · écrire · recevoir ·
      recevoir pendant la saisie sans être dérangé.

## Répartition

| Lot | Auteur | Machine | Relecteur | Machine |
|---|---|---|---|---|
| L1 → L2 | `rc1` | distante | `relec6` | locale |
| L3 | `jc6` | distante | `relec7` | locale |
| L4 | `cursor3` | locale | `relec8` | locale |
| L5 → L6 | `jc1` | distante | `cursor4` | locale |
| L7 | `cartae0` | locale | — | — |
| Composition | `rc5` | distante | — | — |

**Auteurs et relecteurs sur des machines différentes quand c'est possible** :
plusieurs défauts du 25/08 n'ont été trouvés **que** pour cette raison.
**Un relecteur par lot, pas de jury** — douze lots sur douze en jury complet le
25/08 quand le critère en donnait quatre.

**Contre-revue adverse** : `cursor5` (fournisseur différent), sur le plan.
