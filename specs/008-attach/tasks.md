# Tasks : `bridget attach` — voir et piloter un équipier

**Prerequisites** : spec.md (2 rounds + amendements reply=false), plan.md
(5 rounds), data-model.md, contracts/abonnement-attach.md, reuse-audit.md (OK,
base `session-07-transport-acp@48c3bf7`), fixtures journal gelées.
**Révision** : découpage a/b et observables renforcés sur revue des tâches
(APPROVE_WITH_CHANGES, 7 points intégrés).

**Règle** : `cargo test` vert et zéro warning (`cargo clippy --all-targets --
-D warnings`) avant de cocher. Un commit livré en review est immuable.
1 tâche = 1 commit stable (Article XV).

---

## Phase 0 : Fondations

- [X] **T801** [Fondation] **ADR 004 + arbitrages** :
  `docs/decisions/004-abonnement-attach.md`. Décisions consignées : (a) client
  attach = **connexion à rôle dédié**, hors annuaire et hors routage, avec
  **matrice fermée des messages autorisés** à ce rôle (Subscribe/Unsubscribe/
  Send/ping — tout message wrapper-only refusé : Rename, Runtime, Domain,
  Availability…) ; (b) envois depuis la vue en `reply=false` (spec amendée) ;
  (c) l'extension de lecture du journal se fait dans le module `journal.rs`
  par tranches, sans casser l'usage T706. Reporter dans plan/data-model.
  **Observable** : ADR auto-portant ; matrice des messages écrite ; les trois
  décisions tracées.

---

## Phase 1 : US1 — Regarder un équipier travailler

- [X] **T802** [US1] **Variantes de protocole d'abonnement + rôle typé**
  (`protocol.rs`, style D-209) : handshake de rôle attach (T801-a) ;
  souscription `{ agent, window: Today|Seq(n)|Date }` ; acceptation
  `subscription_id` (après acceptation wrapper) ; fragment `{ subscription_id,
  seq, offset, final, bytes }` (256 Kio max) ; `SnapshotCaughtUp {
  subscription_id, through_seq? }` ; `Gap { subscription_id, from_seq, to_seq,
  reason? }` ; `End { subscription_id, reason }` ; refus typés (agent inconnu,
  non-ACP, wrapper absent, canal saturé, date invalide/future/hors rétention,
  message hors rôle).
  **Observable** : tests aller-retour de chaque variante ; compatibilité
  ascendante (agents 007 inchangés) ; message wrapper-only sur connexion
  attach → refus typé testé.

- [X] **T803a** [US1] **Lecture incrémentale du journal + résolution de
  fenêtre** (`journal.rs`) : lecture par tranches bornées avec positions/
  offsets, signalement des lignes illisibles (numéro + offset, `seq` inconnu),
  résolution `Today`/`Date`/`Seq` dans le fuseau de l'hôte (refus typés).
  **Observable** : tests sur les fixtures gelées (rotation 5→6, partial-tail,
  corrupt-line) + fenêtre vide ; l'usage T706 (`valid_events`, récupération de
  `seq`) reste intact.

- [ ] **T803b** [US1] **Worker de relais wrapper** : worker dédié à canal
  borné + **canal de contrôle séparé jamais refusé** ; curseur par abonnement ;
  équité rejeu/suivi par tranches ; suivi par offset (motif sonde runtime) ;
  zéro lecture sans abonné ; jamais bloquant pour l'écrivain du journal.
  **Observable** : tests **falsifiables** — worker de rejeu volontairement
  suspendu (barrière) et commande d'arrêt/désabonnement passant par le canal
  de contrôle dans une borne fixée ; compteur/sonde prouvant zéro lecture sans
  abonné ; saturation du canal de commandes → refus typé remonté ; fenêtre
  vide → `SnapshotCaughtUp` sans `through_seq`.

- [x] **T804a** [US1] **Daemon : rôle attach et cycle d'abonnement** :
  négociation du rôle (identité d'envoi **forcée à « humain » côté daemon**,
  `from` forgé réécrit/refusé), table des abonnements par agent,
  `subscription_id`, générations obsolètes ignorées, relais d'abonnement au
  wrapper, `End` typé (wrapper parti, client parti, désabonnement), nettoyage
  complet à la déconnexion.
  **Observable** : tests — client attach **invisible dans `who`**, `Send`
  tracé humain au ledger, `from` forgé rejeté, réabonnement concurrent avec
  fin tardive d'ancienne génération ignorée, zéro état résiduel après
  déconnexion.

- [ ] **T804b** [US1] **Daemon : tampons, écrivains, Gap et corrélation des
  envois** : handler qui ne fait qu'enfiler (aucune E/S sous verrou global —
  interdiction du motif `daemon.rs:1334`), écrivain dédié par vue à délai
  borné (fermeture motivée de la vue lente), tampon en octets (1 Mio) avec
  lâcher par `seq` entier, `Gap` coalescé hors file ; **table bornée
  `pending_attach_sends { message_id → conn_id, expiry }`** créée pour les
  `Send` de rôle attach, conservée après accusé, routant `Nack` et
  `DeliveryRejected` tardifs vers la connexion, purgée à
  rejet/expiration/déconnexion (sans elle, le daemon ignore un
  `DeliveryRejected` sans demande suivie — US2-5 serait irréalisable).
  **Observable** : tests à barrières — vue à écrivain bloqué fermée sans geler
  le daemon ni les autres vues ; **deux vues recevant exactement le même flux
  puis fermeture sans abonnement/thread/fichier résiduel** (FR-006/SC-004) ;
  dépassement de tampon → `Gap` exact ; rejet tardif routé via la table puis
  purgé.

- [ ] **T805a** [US1] **Client attach : socket, réassemblage, reconnexion**
  (`attach.rs` + sous-commande `cli.rs`) : connexion persistante (lecteur
  unique + dispatch `subscription_id`/`message_id`, writer sérialisé), état
  mémoire (`last_seq`, `initial_window`, `caught_up`, `pending_send`),
  réassemblage borné (4 Mio → `Gap(event_too_large)` consommé jusqu'à `final`),
  resynchronisation.
  **Observable** : tests — fragment > 256 Kio en rejeu **et** en live sans
  boucle ; événement > 4 Mio → **un seul** `Gap` consommé jusqu'à `final` ;
  **snapshot vide → `End` → réabonnement au sélecteur initial → premier
  événement reçu exactement une fois** (FR-008/contrat) ; générations
  obsolètes ignorées côté client.

- [ ] **T805b** [US1] **Client attach : rendu et sanitisation** : préfixes
  horodatés par type de payload v1, liste blanche Unicode (hors `Cc`/`Cf`,
  `␛` visible), continuations indentées, longueur bornée signalée, refus
  motivés (non-ACP → pointer le pane, inconnu → liste des attachables).
  **Observable** : quickstart §1-§2 sur équipier réel ; fixtures hostiles
  (ESC/CSI/OSC, C1, bidi, zero-width, CR, backspace, newline forgée,
  combining) rendues neutralisées — SC-007 ; fixtures journal gelées comme
  entrée des tests de rendu.

---

## Phase 2 : US2 — Parler à l'équipier depuis la vue

- [ ] **T806a** [US2] **Raw mode et restauration** : termios via `libc`
  (`ISIG` désactivé, `0x03` en boucle : restauration puis sortie ; garde RAII
  en filet ; stdin non-TTY → mode dégradé sans raw).
  **Observable** : tests pseudo-TTY (Ctrl-C, EOF, erreur, non-TTY) — terminal
  restauré sur chaque chemin de sortie.

- [ ] **T806b** [US2] **Saisie, envoi et issues corrélées** : tampon de saisie
  applicatif réaffiché sous chaque événement ; envoi `Send` `reply=false` sur
  la connexion attach, `message_id` conservé, `pending_send` nettoyée sur
  issue terminale/expiration/fermeture réelle — jamais à `End` ; issues
  affichées (accusé, DND avec minutes, arrêté, file pleine, rejet différé).
  **Observable** : quickstart §3 ; test « accusé puis rejet tardif » avec flux
  intercalé, y compris après `End` (via `pending_attach_sends` de T804b) ;
  **la saisie partielle reste identique à l'octet pendant un événement hostile
  concurrent** (SC-007) ; scénarios non heureux US2-4/5.

---

## Phase 3 : Critères mesurés

- [ ] **T807** [US1] **Bancs SC-001/SC-002 (local)** : latence
  fin-d'append→rendu p95 < 1 s et max < 3 s à 10 evt/s sur 60 s ; jonction
  rejeu→suivi zéro perte/doublon par continuité de `seq`, rotation de minuit
  simulée comprise.
  **Observable** : bancs reproductibles en test, chiffres dans
  `implementation.md`.

- [ ] **T808** [US3] **Banc SC-005 (budget d'observation)** : faux adaptateur
  déterministe (réutiliser 007-T704), N ≥ 200 tours identiques, dégradation
  p95 de latence d'append < 5 % avec 2 vues vs 0 vue.
  **Observable** : banc en test, chiffres consignés.

- [ ] **T809** [US3] **Gate distant (SC-006) + banc distant** : quickstart §6
  sur environnement aux répertoires **réellement distincts** : US1+US2 vers
  l'équipier distant, wrapper distant arrêté → message d'indisponibilité,
  resynchronisation après coupure de fédération ; **banc SC-001 distant :
  p95 < 3 s à 10 evt/s sur 60 s, chiffres consignés**.
  **Observable** : échange distant + chiffres. **Gate non contournable** :
  sans environnement, non cochée et session `In Progress`.

---

## Phase 4 : Finition

- [ ] **T810** Non-régression, synchronisation 007 et documentation : **gate
  d'intégration — tous les commits 007 finalement validés (T707-T712 compris)
  sont intégrés à la branche 008 avant clôture** (pas de base MVP figée) ;
  suite complète + agents 007 inchangés ; section « Observer un équipier »
  dans `README.md`/`README.en.md` ; `DEPRECATIONS.md` relu ;
  `implementation.md` complet, SC-001..SC-007 pointés avec preuves.
  **Observable** : checklist SC pointée ; `git log` prouvant l'intégration de
  la 007 finale.
