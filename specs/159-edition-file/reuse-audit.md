# Réemploi159A — candidat CACHE

Décision : **PASS5/5** le 2026-10-12T13:33:41+00:00. Ce verdict porte sur le réemploi A13. Il ne vaut ni Analyze frais ni GO code.

Feature : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file
Source de base : e52a96240b6168bf4466712e1bda8852b4ecb43f, intégration déclarée par ROOT. Installation158B non attestée par cette lecture.

## Synthèse

33 items A confrontés. Décisions : CREER 10, REUTILISER 23. Aucun doublon évident non arbitré. B14 reste hors gate.

## Réutilisations et créations justifiées

| ID | Item | Décision | Preuve et motif |
|---|---|---|---|
| R01 | AgentDefinition.held_queue | CREER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/registry.rs:49 — Champ prévu absent ; étendre la définition existante, faux par défaut. Aucun registre neuf. |
| R02 | Phases held et cancelled | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:539 — Élargir le CHECK et les lecteurs de la saga existante ; pas deuxième file. |
| R03 | queue_rank | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:532 — Colonne prévue INTEGER PRIMARY KEY AUTOINCREMENT dans send_deliveries ; réemploi allocateur SQLite. MAX+1 écarté car purge fait reculer. |
| R04 | recipient_agent et turn_gate | CREER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:129 — Deux colonnes prévues dans la même table ; aucune équivalente repérée par nom/responsabilité. |
| R05 | Migration SQLite v7 | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store_schema.rs:4 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:795 — Étendre la migration et le préflight existants. Préserver delivery_id UNIQUE NOT NULL et liens d’exécution lors de reconstruction. |
| R06 | Index destinataire/phase/rang | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:885 — Même nom idx_send_deliveries_recipient_phase, extension (recipient_instance_id,phase,queue_rank) conserve les deux préfixes existants ; pas index doublon. |
| R07 | Register.turn_attest | CREER | /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/plan.md:37 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:5622 — Champ booléen fermé faux/omis pour compatibilité. Register exact non relu intégralement ; noms recherchés et raccord wrapper attesté. |
| R08 | TurnState et signal is_busy | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:5237 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/acp.rs:775 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/claude_stream_json.rs:514 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/codex_app_server.rs:1329 — Sources de vérité transport existantes ; aucun nouveau modèle de présence. |
| R09 | busy_period_start/last_attested_at/turn_attest de présence | CREER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:5240 — Données nécessaires à l’attestation ; préserver busy_since legacy, que chaque vrai TurnState busy remet actuellement à zéro. |
| R10 | Réattestation ≤30s | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:4698 — Minuterie heartbeat existante ; contrôler échéance aussi lors de trafic continu. Le seul bras timeout ne suffit pas : décision ROOT retenue. |
| R11 | should_hold(facts) | CREER | /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/plan.md:1 — Décision pure métier FR002 prévue. Aucune primitive identique trouvée ; utiliser faits existants et garde autonome, sans nouvelle autorité. |
| R12 | begin_send_delivery_inner annoncé | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:89 — Nom absent aujourd’hui ; véritable primitive à greffer : begin_send_delivery_with_guidance. Préserver transaction Immediate et réservations. |
| R13 | Émission ledger et tracked_requests atomique | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:151 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store.rs:170 — Même transaction ; table réelle tracked_requests, pas requests. held ne doit pas publier une émission avant remise. |
| R14 | release_next_held(agent,now) | CREER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:403 — Méthode de la saga existante nécessaire à libération ordonnée ; aucune méthode held déjà présente, pas worker/file autonome. |
| R15 | adopt_held_for_agent | CREER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:227 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:5994 — Étendre réaffectation/recovery existants avec identité/génération. Conserver exclusions réveils158B et missions natives. |
| R16 | settle_held | CREER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:597 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:653 — Clôture bornée de la même saga ; indeterminate seul ne clôture pas le suivi métier. Pas nouveau journal d’issue. |
| R17 | held_ids dérivé SQL | CREER | /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/plan.md:63 — Petit ensemble d’IDs demandé pour filtres de rappels ; ni corps copiés ni deuxième file. Reconstitution depuis saga. |
| R18 | Rappels trois paliers et balayage | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:5252 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:4698 — Étendre boucle/temporisation existantes ; pas scheduler ni thread neuf. Palier3 clôture avant balayage suivant contrat. |
| R19 | AutonomousEffect::HeldRelease | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/plan.md:39 — Extension de garde autonome existante ; nom nouveau sans nouveau moteur ni autorité. Corps complet de la garde hors plages fraîches. |
| R20 | Pause durable et reprise | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/execution_store.rs:1655 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:12718 — Réemploi control_pause_interruptions et reprise après pause ; lecture SQL dans même TX nécessaire si décision atomique, pas appel Store à connexion distincte. |
| R21 | Register/recovery/transition de tour | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:5622 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:6534 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:5994 — Raccords existants de connexion et d’activité ; aucune boucle de reprise propre à held. |
| R22 | QueueList daemon | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:10749 — Étendre vue existante, filtrage d’autorité avant aperçu, limite200. Ne pas rendre file étrangère visible. |
| R23 | bridget_queue MCP | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/plan.md:16 — Outil public existant à enrichir ; schéma MCP complet non relu cette passe. Pas nouvel outil A. |
| R24 | Ordre dispatching existant | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:417 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:466 — Remplacer ordre rowid/delivery_id par rang là où prescrit ; queue_rank alias rowid préserve identité des anciens rangs copiés. |
| R25 | Délais métier et acquittements | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:11324 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/ledger_requests.rs:699 — Primitives de suivi/réponse existantes ; ne pas démarrer délai d’une émission tenue. ACK/ledger gardent leurs preuves, aucun ACK neuf. |
| R26 | DeliveryRejected/indeterminate fournisseurs | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/wrapper.rs:6658 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/t3code.rs:1850 — Retours existants réels, y compris request_closed. Transport T3 complet non relu, aucun modèle exécuté. |
| R27 | Liens send_delivery_execution_links | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:862 — FK vers delivery_id conservée. DROP parent exécute cascade : sauvegarde et restauration dans migration transactionnelle obligatoires, pas table métier ajoutée. |
| R28 | ON CONFLICT et clés de remise | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:129 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:546 — INSERT courant simple à colonnes nommées, sans upsert send_deliveries ; unicité métier composite et delivery_id inchangées. Aucun last_insert_rowid trouvé dans recherche ciblée. |
| R29 | Tests migration/reprise T004/T009 | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/tasks.md:29 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/tasks.md:42 — Étendre cas déjà prescrits pour purge/rang/FK/réattestation ; aucun scénario de confort proposé, exécution ROOT ultérieure. |
| R30 | held159_e2e.rs | CREER | /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/tasks.md:43 — Fichier de tests explicitement prévu, composition fixtures existantes ; pas exécution ni montage fournisseur dans cette passe. |
| R31 | Backup/rollback T011 | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/tests/core_089_migration_test.rs:12 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/tasks.md:44 — Réemploi backup SQLite cohérente et recette isolée ; ancien binaire réel158B après reçu, pas preuve runtime inventée. |
| R32 | Baseline et latence T001/T010 | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/tasks.md:14 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/tasks.md:43 — Tests/mesures prescrits uniquement. Pas baseline relancée ni p95/SC004 mesuré ici. |
| R33 | Séparation A13/B14 | REUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/tasks.md:15 — A T001–T013 seulement. QueueEditV1/queue_edits/édition/priorité/repli B exclus. B permutation négative transactionnelle future seulement, aucun code B. |

## Existant pertinent et duplications

La primitive réelle est begin_send_delivery_with_guidance. La table réelle est tracked_requests. La reprise158B exclut déjà ses réveils et les missions natives. Les liens d’exécution et leurs FK doivent survivre à la migration. Aucun deuxième worker, service, magasin ou outil public nécessaire.

## Mémoires, règles et specs proches

Skill et ponts constitution/standards lus. Mémoire /Users/moi/.Codex/projects/bridget/memory/MEMORY.md et prérequis /Users/moi/Nextcloud/10.Scripts/64.bridget/.specify/scripts/bash/check-prerequisites.sh absents lors de cette passe. Aucun outil installé. Code queue/guidage156 et réveils158B ciblé ; les prélectures historiques sont du contexte, pas une Analyze actuelle.

## Journal de recherche

Un passage rg sur les noms A et responsabilités dans crates/scripts/docs/Cargo.toml ; aucun nom held/rank/turn_attest existant. Lectures adaptatives des primitives réelles. Manifeste fournit plages effectivement vues et SHA à la capture finale. Pas crédit de lecture pour les seules empreintes ou résultats rg.

## Arbitrages

- A1 RETENU_ROOT : T006 vérifie échéance même sous trafic continu en réutilisant la minuterie heartbeat ; pas greffe seulement timeout.
- A2 RETENU_ROOT : Ancienne formule MAX+1 était O(N), pas O(log N) ; aucune indexation supplémentaire pour maquiller le coût.
- A3 VERIFIE_STATIQUE : Étendre index nonunique existant (recipient_instance_id,phase) avec queue_rank, même nom ; deux préfixes inchangés, aucune dépendance INDEXED BY trouvée.
- A4 RETENU_ROOT_FAISABLE_STATIQUE : FR008 monotone à vie : queue_rank INTEGER PRIMARY KEY AUTOINCREMENT, delivery_id UNIQUE NOT NULL. Copie explicite anciens rowid. SQLite conserve watermark malgré DELETE/cascade ; plus de SELECT MAX au chemin chaud.
- A5 CONTRAINTE_MIGRATION : Le lien FK déjà présent impose conserver/reconstruire ses lignes avant DROP du parent ; FK ON, même migration transactionnelle, restaurer indices et liens, pas nouveau registre métier.
- A6 B_HORS_GO : B pourra permuter les rangs existants via négatifs temporaires dans TX puis restaurer valeurs ; ne pas poser CHECK positif incompatible. UPDATE ne relève pas sqlite_sequence : ne jamais fabriquer des rangs positifs au-delà des valeurs existantes. Aucun code B.

Le rang automatique ne recule pas après purge. Cette propriété vient de [SQLite AUTOINCREMENT](https://www.sqlite.org/autoinc.html). Le DROP du parent peut supprimer les liens enfants ; la migration doit les préserver. Voir [SQLite DROP TABLE](https://www.sqlite.org/lang_droptable.html). La saturation de l’entier doit rester un refus sûr, jamais un rebouclage. Aucun benchmark ni migration exécutée ici.

## Alignements documentaires avant Analyze

- MAX+1 à remplacer par allocateur SQLite AUTOINCREMENT prévu ; ajouter conservation FK/liens en migrationv7.
- Index recipient_rank annoncé : étendre même idx_send_deliveries_recipient_phase, pas coexistence.
- Clarifier échéance de réattestation contrôlée aussi sur trafic continu.
- Remplacer requests par tracked_requests et nom fictif begin_send_delivery_inner par vraie primitive à greffer.
- Actualiser références WT150/base156/ancien binaire/absence de relance avec futur cadre159A et preuve réelle158B.
- SC001 compatibilité : preuve des résultats réels et limites héritées, aucun GREEN global supposé.

## Gate avant tasks / code

- [x] G1 Chaque item A extrait et confronté aux recherches de noms/responsabilités et surfaces ciblées ; limites explicites.
- [x] G2 Aucun doublon évident non arbitré ; même saga, même index, minuterie, garde et outils existants.
- [x] G3 Skill, bridges et références locales lues ; mémoire et prérequis absents déclarés.
- [x] G4 Couplages de base158B et prédécesseurs transport/queue/pause vérifiés sur code intégré ; installation non attestée ici.
- [x] G5 Divergences du plan motivées dans candidat CACHE : cadence trafic continu, index étendu, AUTOINCREMENT à vie et migration des FK. Application documentaire ROOT requise avant Analyze.

## Limites

- Lecture ciblée uniquement, pas audit global ni Analyze159A frais. Ancienne décision PASS2026-10-11 non reprise comme preuve.
- Aucune commande Cargo/test/DB/service/Git ni mesure/performance. Documentation SQLite officielle consultée pour la faisabilité uniquement.
- Basesource e52a96240b6168bf4466712e1bda8852b4ecb43f attestée par ROOT ; HEAD non vérifié par cet agent. Installation158B encore en cours selon dernier message reçu.
- Spec159 plage60–71 non recréditée ; reuse-audit historique seulement1–45 ; protocol/Register/MCP et garde autonome non relus intégralement. Corps géants N/daemon/wrapper non lus intégralement.
- Recherches absence bornées crates/scripts/docs/Cargo.toml puis couplages ciblés. B14 hors gate et hors code.
- La vérification supplémentaire AUTOINCREMENT est statique et documentaire ; migration/FK/rollback et coûts réels restent à prouver par tests déjà prescrits.
- Borne initiale7min dépassée ; collecte générale arrêtée sur demande ROOT. Seule vérification supplémentaire autorisée2min sur AUTOINCREMENT/FK/INSERT/rowid réalisée.

Outputs : /Users/moi/.cache/bridget-reuse159A-20261012/reuse-audit159A.md ; /Users/moi/.cache/bridget-reuse159A-20261012/manifest.json ; /Users/moi/.cache/bridget-reuse159A-20261012/reuse-audit159A.json
