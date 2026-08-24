# Tasks 017 — Maicie greffière du catalogue

**Input** : `spec.md`, `plan.md` et `data-model.md` gelés à `b1abd11`, après
la revue hostile fable-5 APPROVE.
**Statut** : préparation d'exécution uniquement. Le code 017 attend le merge
de la session 015 et la fin du chantier 016 ; aucune tâche ci-dessous ne les
contourne.

## Règles d'exécution

Un seul composant Maicie porte la session : pas de couloirs parallèles. Les
tâches se réalisent dans l'ordre des phases 1 à 6 du plan et tout changement de
sémantique du journal gelé repasse par revue hostile. Le journal canonique est
le seul fichier déclaré par projet ; `registre list` est sa seule vue humaine
d'autorité. Aucune tâche ne crée de plan hôte, de tâche, d'issue, de message
libre, de score, de déduplication automatique ou d'état `planned`.

Chaque commit valide `git diff --check`, ses tests ciblés et
`cargo clippy -p maicie --all-targets -- -D warnings`. Le workspace complet est
réservé à T1715. Les tests de mutation indiqués ci-dessous doivent prouver leur
oracle, pas seulement exécuter le chemin vert.

## Phase 1 — Corpus v1 fermé et configuration déclarative

- [x] T1701 Figer le corpus JSONL v1 dans
  `plugins/maicie/tests/fixtures/catalogue-v1/` et les types fermés dans
  `plugins/maicie/src/catalogue.rs` : deux kinds `add|transition`, RFC 3339
  avec fuseau, sévérités et sources fermées, références uniquement par ID.
  **Observable** : les tests de contrat refusent type, champ, sévérité et
  référence inconnus sans préparer d'append (SC-1702).
- [x] T1702 Déclarer et valider le chemin unique du catalogue dans
  `plugins/maicie/src/config.rs` et `plugins/maicie/src/catalogue.rs` : chemin
  explicite, fichier régulier non-symlink sous le projet hôte et aucun autre
  artefact de workflow accessible en écriture.
  **Observable** : un chemin hors catalogue, un symlink et `tasks.md` sont
  refusés avant toute écriture ; le catalogue déclaré seul est accepté.

**Gate G1701 — grammaire** : le corpus producteur/lecteur et les refus fermés
sont revus avant d'introduire l'écriture durable.

## Phase 2 — Migration conservatrice et append durable

- [x] T1703 Implémenter dans `plugins/maicie/src/catalogue.rs` le lecteur et
  le writer verrouillés : lecture de collision `(id, bytes canoniques)`, no-op
  pour le retry identique, refus sans mutation pour bytes divergents,
  `O_APPEND`, verrou exclusif et `sync_data`, sans temporaire ni rename.
  **Observable** : SC-1708 prouve une seule ligne pour le replay identique et
  zéro mutation pour le replay divergent ; la mutation qui retire le verrou ou
  `sync_data` fait échouer l'oracle.
- [x] T1704 Prouver dans `plugins/maicie/tests/contract/catalogue.rs` deux
  writers réellement concurrents du même catalogue déclaré, avec deux entrées
  complètes, distinctes et réouvrables.
  **Observable** : SC-1710 produit exactement deux lignes JSONL intactes,
  sans perte ni troncature ; une mutation vers réécriture globale perd une
  entrée et rend le test rouge.
- [x] T1705 Migrer le catalogue-prose réel (environ 50 entrées) par
  `plugins/maicie/src/catalogue.rs` et un corpus sous
  `plugins/maicie/tests/fixtures/catalogue-migration/`, en préservant chaque
  texte verbatim et son identifiant de provenance ; consigner les entrées
  incomplètes uniquement en `pending_qualification`.
  **Observable** : SC-1701 compare le corpus avant/après : 100 % des textes
  historiques sont retrouvés octet pour octet, aucune sévérité/source/récurrence
  n'est inventée et aucune entrée n'est activée par défaut.

**Gate G1702 — journal durable** : migration réelle, idempotence et append
concurrent sont revus ensemble ; aucun code de vue ou de transition ne passe
ce gate sans journal réouvrable.

## Phase 3 — Vue pure du registre

- [x] T1706 Implémenter la réduction et le rendu déterministe `registre list`
  dans `plugins/maicie/src/catalogue.rs`, puis l'appel mince dans
  `plugins/maicie/src/main.rs` : ordre figé open, sévérité, récurrence, gate
  raté, date, ID ; footer exact `N/M/K/P` incluant
  `pending_qualification`.
  **Observable** : SC-1704 compare octet pour octet deux journaux équivalents
  dont l'ordre physique diffère ; le rendu, son footer et l'absence d'écriture
  sont identiques.
- [x] T1707 Exposer `registre add` et `registre list` dans
  `plugins/maicie/src/main.rs`, sans voie de message libre ni logique de tri
  dupliquée.
  **Observable** : un add conforme puis list passent uniquement par le
  composant catalogue ; une tentative de texte libre, de cible par titre ou
  d'écriture hôte est refusée (SC-1702, SC-1705).

**Gate G1703 — vue sans jugement** : revue du golden SC-1704 et de la
frontière lecture seule avant le raccord aux objectifs.

## Phase 4 — Lien d'arbitrage et clôture attestée

- [ ] T1708 Ajouter le fait durable de délégation `(constat_id, objective_id)`
  dans `plugins/maicie/src/domain.rs`, `plugins/maicie/src/store.rs` et
  `plugins/maicie/src/app.rs` : seul un arbitrage humain qui crée une
  délégation peut déclarer ce lien ; IDs exacts seulement.
  **Observable** : une homonymie textuelle, un `constat_id` absent ou un
  objectif non déclaré ne peut créer aucun lien ni aucune transition.
  **BLOQUÉ lot 5** : `domain.rs` / `store.rs` occupés par le couloir B 016.
- [~] T1709 Implémenter dans `plugins/maicie/src/app.rs` et
  `plugins/maicie/src/catalogue.rs` la transition unique `open → delivered`
  depuis une clôture durable attestée : événement reçu ou réconciliation qui
  lit l'état durable du même `objective_id`, jamais horloge, silence ou texte.
  **Observable** : SC-1703 produit une seule transition au rejeu ; SC-1709
  perd l'événement puis réconcilie l'objectif exact avec la même unique ligne.
  **Partiel lot 5** : API journal `reconcile_attested_closures` livrée ; câblage
  `app.rs`/store attend T1708.
- [ ] T1710 Déclencher la réconciliation idempotente **au fil des commandes
  catalogue** dans `plugins/maicie/src/app.rs` et le documenter en une ligne
  dans `specs/017-greffiere-catalogue/quickstart.md` ; n'ajouter ni boucle
  résidente ni polling.
  **Observable** : ouvrir/consulter le registre rattrape une clôture durable
  manquée, tandis qu'aucune activité hors commande ne produit de transition.
  **BLOQUÉ** : dépend de T1708 (lecture états durables). Documenté dans
  quickstart (noyau journal prêt).

**Gate G1704 — transitions factuelles** : les transitions sont revues contre
les deux sources attestées et leur idempotence, avec mutation séparant lien,
clôture ou append.

## Phase 5 — Découvrabilité et frontière de travail

- [ ] T1711 Faire passer dans `plugins/maicie/src/app.rs` toute décision de
  remède par la délégation durable existante, en portant `constat_id` dans son
  entrée canonique ; ne fournir aucune primitive d'envoi libre depuis le
  catalogue.
  **Observable** : SC-1705 refuse une voie message-nu et prouve qu'un travail
  lancé possède délégation, corrélation et reçu durable.
  **BLOQUÉ** : dépend de G1704 / T1708.
- [x] T1712 Mettre à jour la skill Maicie active et
  `specs/017-greffiere-catalogue/quickstart.md` : `registre list` est exigé au
  début de session, avant toute proposition de suite et en première opération
  du rituel de clôture ; les sorties de jalon affichent `N/M/K/P`.
  **Observable** : SC-1706 couvre démarrage et proposition ; SC-1707 couvre
  le rituel et son footer, sans écrire dans un plan hôte.
- [~] T1713 Documenter dans `plugins/maicie/README.md` et `README.md` le
  journal du dû, sa migration, ses interdits, la qualification humaine et les
  limites v1 (pas de déduplication, score, `planned`, runtime résident ou
  adaptateur hôte).
  **Observable** : chaque commande documentée renvoie au journal déclaré et
  aucune documentation ne promet une action automatique hors délégation.
  **Partiel lot 5** : `plugins/maicie/README.md` fait ; README racine reporté.

## Phase 6 — Gate final de session

- [ ] T1714 Exécuter les scénarios complets de
  `specs/017-greffiere-catalogue/quickstart.md` et consigner dans
  `specs/017-greffiere-catalogue/implementation.md` les commandes, durées et
  preuves SC-1701 à SC-1710, dont le corpus réel de migration et l'append
  concurrent.
  **Observable** : chaque SC pointe une sortie reproductible ; les quatre
  décomptes et le zéro-perte verbatim sont explicitement mesurés.
- [ ] T1715 Lancer `cargo test --workspace` puis
  `cargo clippy --workspace --all-targets -- -D warnings`, et conduire la
  revue hostile finale de la session 017.
  **Observable** : la revue mute format fermé, idempotence, append concurrent,
  événement perdu/réconciliation, homonymie et message libre ; chaque mutation
  échoue avec son oracle et la non-régression est consignée.

## Dépendances et stratégie

- T1701 → T1702 → G1701 → T1703 → T1704 → T1705 → G1702.
- G1702 → T1706 → T1707 → G1703.
- G1703 → T1708 → T1709 → T1710 → G1704.
- G1704 → T1711 → T1712 → T1713 → T1714 → T1715.

Le MVP est T1701 à T1710 : un journal fermé, durable, migré, lisible et dont
les clôtures sont seulement des faits attestés. T1711 à T1715 rendent cette
discipline découvrable, vérifiable et prête pour le gel final.

## Lot FR-1711 (arbitrage 2026-08-24)

Table de correspondance écrite dans `spec.md` / `data-model.md`. Émission
automatique hors `domain.rs`/`store.rs` : `registre consign` +
`transcribe_observed_fact` / `consign_observed_fact` pour `gate_failed` et
`review_amender` ; hors table → pending. SC-1711 couvert par test unitaire.
