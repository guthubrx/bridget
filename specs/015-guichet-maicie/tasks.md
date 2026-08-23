# Tasks 015 — Guichet Maicie : identité joignable et demandes corrélées

**Input** : `spec.md` et `plan.md` gelés par revue hostile.
**Statut** : plan d'exécution uniquement. Aucune tâche ci-dessous n'est
commencée par la présente livraison documentaire.

## Règles de couloir et de contrat

Le contrat 015 est gelé. Une tâche peut préciser un champ, un refus ou un test
déjà décidé ; elle ne peut pas en modifier la sémantique. Toute modification
substantielle revient à la revue hostile avant le code.

| Lot | Propriétaire | Fichiers exclusifs |
|---|---|---|
| A — Bridget | codeur A | `crates/bridget-transport/src/protocol.rs`, `crates/bridget-daemon/src/daemon.rs`, `crates/bridget-daemon/src/store.rs`, `crates/bridget-daemon/tests/guichet_integration_test.rs`, `specs/015-guichet-maicie/contracts/protocole-guichet.md` |
| B — Greffe Maicie | codeur B | `plugins/maicie/src/guichet.rs`, `plugins/maicie/src/store.rs`, `plugins/maicie/src/domain.rs`, `plugins/maicie/src/app.rs`, `plugins/maicie/tests/contract/guichet_domain.rs`, `plugins/maicie/tests/integration/guichet_greffe.rs` |
| C — Adaptateur et CLI Maicie | prospective | `plugins/maicie/src/bridget_client.rs`, `plugins/maicie/src/reconcile.rs`, `plugins/maicie/src/main.rs`, `plugins/maicie/tests/contract/guichet_client.rs`, `plugins/maicie/tests/integration/guichet_gate.rs` |
| Intégration | prospective | `plugins/maicie/Cargo.toml`, `plugins/maicie/src/lib.rs` — modification uniquement par la tâche T1511 |

Tous les lots conservent les règles Maicie : protocole Bridget public seulement,
jamais `bridget.db`, aucune boucle résidente, aucun LLM, aucun processus enfant
Maicie, aucune approbation hors frappe TTY humaine locale. Les validations de
chaque commit sont `git diff --check`, les tests ciblés et `cargo clippy
--all-targets -- -D warnings` sur le périmètre touché ; le workspace complet
est le gate final.

## Phase 1 — Fondation contractuelle non contournable

- [ ] T1501 Formaliser le contrat public gelé dans `specs/015-guichet-maicie/contracts/protocole-guichet.md` : négociation `maicie_guichet`, rôle de service distinct, `ServiceRequest`, `GuichetReply`, `RequestLifecycleEvent`, table de récupération, tailles/canon/refus et matrice Bridget-vs-Maicie ; appliquer sans re-cycle les retouches cosmétiques « émetteur déclaré » dans FR-1504 et « coopérative » dans FR-1515 de `specs/015-guichet-maicie/spec.md`. **Observable** : corpus de trames fixe couvrant les trois opérations, `answered`/`cancelled`/`timed_out`, capacité absente et divergence canonique.
- [ ] T1502 [P] Préparer les fixtures contractuelles producteur↔consommateur sous `specs/015-guichet-maicie/contracts/fixtures/` et `plugins/maicie/tests/contract/guichet_client.rs`, sans code de production : dépôt validé, `request_already_terminal`, deux ordres `delivery_report`/`answered`, six refus et SC-1509. **Observable** : chaque fixture porte l'issue fermée attendue et un commentaire de mutation qui explique quel défaut la ferait échouer.

**Gate G1501 — contrat** : une revue croisée approuve le contrat et les
fixtures avant toute modification des crates Bridget ou de la SQLite Maicie.
Le contrat doit dire explicitement que `maicie_guichet` est une borne de
capacité, non une identité opposable entre processus du même compte.

## Phase 2 — Lot A : guichet durable Bridget

- [ ] T1503 Implémenter la capacité négociée `maicie_guichet` et l'identité de service réservée dans `crates/bridget-transport/src/protocol.rs` et `crates/bridget-daemon/src/daemon.rs` ; seuls ce rôle-capacité et les transitions autorisées accèdent à claim, `GuichetReply` et événements de demande, jamais `from: "maicie"` seul. **Observable** : SC-1509 — une trame qui déclare `maicie` sans capacité est refusée, y compris après reconnexion ; le chemin historique reste inchangé.
- [ ] T1504 Implémenter la persistance du guichet, l'idempotence et la reprise dans `crates/bridget-daemon/src/store.rs` et `crates/bridget-daemon/src/daemon.rs` : `queued → claimed → replied|rejected`, octets exacts, tombstone et réponse reconstruite après crash. **Observable** : trois crashs réels (avant persistance, après claim/avant accusé, après résultat/avant retour) conservent une seule demande ou une issue explicite, sans destinataire Maicie introuvable.
- [ ] T1505 Implémenter la greffe Bridget des demandes corrélées dans `crates/bridget-daemon/src/daemon.rs` et ses tests exclusifs `crates/bridget-daemon/tests/guichet_integration_test.rs` : l'accusé durable d'un `delivery_report(in_reply_to=...)` passe la demande à `answered` et dépose atomiquement `RequestLifecycleEvent`; `cancelled` et `timed_out` déposent aussi leur événement unique. **Observable** : SC-1507 et course timeout/rapport — le rapport tardif reste livrable, la demande terminale n'est jamais rouverte, aucun rappel ne survit à `answered`.

**Gate G1502 — Bridget** : A fournit une démonstration daemon réel de l'agent
qui dépose pendant l'absence de Maicie, du refus de capacité sans usurpation
possible par nom déclaré, et des trois événements terminaux persistants.

## Phase 3 — Lot B : User Story 1, greffe Maicie atomique (P1)

**Goal** : traiter une demande structurée une seule fois dans le registre
Maicie, tout en laissant Bridget propriétaire du transport et de la demande.

**Test indépendant** : une réponse de livraison valide, un timeout concurrent,
un crash entre transaction et réponse, puis une relève fraîche produisent une
seule décision et au plus une transition vers `à_évaluer`, jamais `clos`.

- [ ] T1506 [US1] Définir dans `plugins/maicie/src/domain.rs` et `plugins/maicie/src/guichet.rs` les trois requêtes fermées, les refus métier et les reçus `ReceptionGreffe` / `ReçuCorrélation`; implémenter dans `plugins/maicie/src/store.rs` les clés uniques `request_id` et `(in_reply_to,response_message_id)`. **Observable** : `plugins/maicie/tests/contract/guichet_domain.rs` refuse texte libre, type/champ inconnu, référence absente, approbation distante et enveloppe divergente sans écrire un objectif, une délégation, une approbation ou un SpawnOrder.
- [ ] T1507 [US1] Implémenter dans `plugins/maicie/src/app.rs` le traitement relationnel et la transaction unique fait + reçu + décision + transition `à_évaluer` pour `delivery_report`; traiter `timed_out`/`cancelled` par `request_already_terminal` sans rouvrir ni perdre le hash. **Observable** : `plugins/maicie/tests/integration/guichet_greffe.rs` passe dans les deux ordres rapport/`answered`, après timeout, et aux frontières crash ; il démontre qu'une mutation séparant décision et transition fait échouer l'oracle.

**Gate G1503 — greffe** : B prouve que l'événement `answered` et le rapport
ne créent jamais deux décisions, et que le chemin guichet ne peut ni approuver
ni consommer une activation de profil.

## Phase 4 — Lot C : User Stories 1, 2 et 3, relève pull-only et CLI

**Goal** : rendre le guichet utilisable au prochain appel Maicie, avec statut
et échéance factuels, sans runtime résident ni état caché.

**Test indépendant** : Maicie absente reçoit un dépôt ; une commande locale
relève sous un budget absolu, rend une réponse liée et expose séparément la
décision locale, la remise et le snapshot transport.

- [ ] T1508 [P] [US1] Dès G1501, implémenter dans `plugins/maicie/src/bridget_client.rs` le client public de capacité `maicie_guichet` et les fixtures de T1502 : négociation, dépôt, claim, ack/reply et événements, tous bornés par une même échéance absolue. **Observable** : `plugins/maicie/tests/contract/guichet_client.rs` prouve que chaque phase consomme le budget restant, que la capacité absente refuse avant I/O métier et que les bytes d'un retry sont inchangés.
- [ ] T1509 [US1] Implémenter dans `plugins/maicie/src/reconcile.rs` la relève bornée, le replay exact et la réponse reconstruite depuis le reçu ; intégrer dans `plugins/maicie/src/main.rs` l'ouverture de commande qui relève sans polling ni daemon caché. **Observable** : `plugins/maicie/tests/integration/guichet_gate.rs` couvre les trois crashs, l'épuisement de budget sans perte et l'absence d'une seconde décision ou d'une seconde réponse.
- [ ] T1510 [US2] [US3] Ajouter dans `plugins/maicie/src/app.rs` (appel depuis `plugins/maicie/src/main.rs` sans logique métier dupliquée) les projections JSON déterministes `mission_status` et `deadline_question`. **Observable** : le corpus montre remise locale, observation transport et fraîcheur séparées ; échéance passive sans mots « bloqué », « terminé » ou « en retard » non attestés.
- [ ] T1511 Intégrer les modules et seules dépendances requises dans `plugins/maicie/Cargo.toml` et `plugins/maicie/src/lib.rs`, après gel des interfaces A/B/C. **Observable** : le crate compile avec les trois couloirs sans modifier leurs fichiers exclusifs ; aucune dépendance vers un crate interne Bridget n'est ajoutée.

**Gate G1504 — parcours réel** : agent réel → demande liée déposée alors que
Maicie est absente → commande Maicie → réponse corrélée et demande Bridget
`answered` → greffe Maicie unique. Le même gate tente une approbation distante
et une usurpation sans capacité : les deux doivent échouer dans le binaire réel.

## Phase 5 — Finition, non-régression et revue hostile

- [ ] T1512 Mettre à jour `specs/015-guichet-maicie/quickstart.md`, `plugins/maicie/README.md`, `README.md` et `docs/decisions/003-maicie-compagnon-orchestration.md` avec la capacité, la limite coopérative v1, le modèle pull-only, les refus et la boucle résidente explicitement v2. **Observable** : chaque commande documentée est exécutable ou explicitement marquée future ; aucune ne propose une approbation distante.
- [ ] T1513 Exécuter les non-régressions : `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, corpus de contrat et gate réel G1504 ; consigner les mesures de relève dans `specs/015-guichet-maicie/implementation.md` seulement lors de l'implémentation réelle. **Observable** : SC-1501 à SC-1509 sont pointés par une preuve, y compris p95 de relève sur 100 délégations.
- [ ] T1514 Effectuer une revue hostile finale des lots A/B/C : retirer la capacité, forger `from: maicie`, muter les octets canoniques, inverser rapport/événement, injecter un timeout concurrent, séparer la transaction Maicie et tenter une approbation distante. **Observable** : chaque mutation fait échouer un oracle dédié ; les comportements historiques Bridget/Maicie restent verts.

## Dépendances et stratégie

- T1501 → G1501 → T1503, T1504, T1506 et T1508.
- T1503–T1505 → G1502 ; T1506–T1507 → G1503.
- T1508 peut préparer le harnais dès G1501 ; T1509 attend G1502 et G1503.
- T1510 attend T1507 et T1509 ; T1511 intègre les trois couloirs seulement
  après leurs interfaces gelées.
- T1512–T1514 ferment la session après le gate G1504.

Le MVP réellement utilisable est T1501–T1511. La boucle résidente, GUI/TUI et
automatisations de routine restent hors de ces tâches et nécessitent leur
propre spécification.
