# Audit de reutilisation de l’existant — 158B

## Decision

Statut: PASS
Date: 2026-10-12. Gate de réemploi avant tasks seulement ; aucun verdict Analyze ou code B.
Feature dir candidat : /Users/moi/.cache/bridget-plan158B-officiel-20261012.
Source relue : /Users/moi/Nextcloud/10.Scripts/64.bridget, code158A80b43407936110526b28f2c38ea72e1b34055154 livré selon ROOT/AGENTS ; métadonnées9a94334a.

## Synthese

40 items extraits/audités. 0 duplication évidente non arbitrée. 3 nouveaux modules de domaine, 3 tables prévues, 1 outil MCP neuf demandé. Aucun service/thread/dépendance neuf. US5–US7/FR101–107 complets, US8 reportée.
Les cinq artefacts candidats et le patch B sont lus intégralement. Les primitives du code sont relues sur les plages du manifeste ; aucun crédit de lecture intégrale du dépôt.

## Reutilisations correctement identifiees

| Item du plan | Décision | Preuve source ROOT | Motif et limite |
|---|---|---|---|
| Boucle de réveil 1 s | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:5161 | Greffe dans la boucle existante ; aucun timer/thread neuf. |
| Entretien TTL | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:7692/7731 | Étendre maintain_stores ; pas second nettoyeur. |
| wake_schedules (table) | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store.rs:358 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/project_compat.rs:1346 | FR101. Les rondes projet sont liées à project_id/binding_generation ; elles ne portent ni owner/cadence/cible arbitraire ni occurrence B. |
| wake_schedules.rs (module domaine) | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:8351 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs:6705 | Aucun ordonnanceur civil équivalent trouvé. Porte cadence/occurrence et les trois sources prévues, sans service. |
| store/wake_schedules.rs (module Store) | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store.rs:33 | Implémentation du Store actuel pour la table FR101 ; aucune autre connexion ou abstraction de stockage. |
| store/observation_missed.rs (module Store) | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store.rs:38/149 | Faits et métadonnées ne sont pas le snapshot des abonnements. |
| WakeTarget | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs:2898 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp_scopes.rs:57 | self typé résolu au daemon ; les autres requêtes portent des UUID simples, sans cette distinction de portée. |
| WakeCadence | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/project_compat.rs:1346 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs:6705 | Deux cadences métier B ; pas DTO existant pour heure/jours/intervalle. |
| WakeRequest | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs:2363/3061 | Famille fermée dans enum existant. Les ObservationRequest/ThreadRequest ont des contrats différents. |
| WakeResult et refus fermés | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs:4029 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/threads.rs:80 | Résultat métier B ; réemploi du patron fermé et des noms de refus déjà exacts, sans codealias. |
| FrozenWake / source / phase | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:512 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/communication.rs:184 | Preuve récupérable commune scheduled/cli_once/missed_summary. Le canon binaire historique ne contient pas la programmation ; pas décodeur universel. |
| freeze_wake_occurrence | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/threads.rs:1817 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store.rs:450 | CAS/Immediate réemployés comme patron ; le CAS102 porte une séquence de fil, pas expected_next_at. |
| reserve_cli_wake | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:887/1129/1157 | Même registre099. Lire issued_at original, canon/collision puis COUNT60s et INSERT sous une TX. Aucun OperationKind neuf. |
| Lecture pending CLI/issued_at | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:1157/1162 | La requête existante ne charge pas issued_at. Étendre localement lecture et domaine JSON B ; pas getter supposé déjà disponible. |
| Quota durable 60 CLI/min | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:512/525 | Reçus099, scope interne daemon, horloge serveur. Pas seconde table/counter RAM. Coût COUNT à vérifier avant éventuel index. |
| Réservation/remise/ledger099 | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:887 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:54/89 | Canon et enveloppe avant socket ; maintien des frontières de transactions. |
| ACK/unknown/reprise099 | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:768 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:4608 | ACK instance/génération exact. Reprise sur preuve durable ; unknown ne confirme pas le fournisseur. |
| Post et reçu de fil102 | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/threads.rs:577/1050 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/threads.rs:1067/1260 | Utiliser handle_with_change, post privée. UUID102 immuable, ancien reçu restitue message_id. |
| Auteur/membre/projet fil | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/threads.rs:1057 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/threads.rs:1082 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/communication.rs:36 | Owner attesté membre ; aucune origine client ou faux membre système. |
| WakeRow102/alertes partagées | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/threads.rs:1219/1777 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:8550 | Vraie contribution avant intentions ; pas expiry B600s sur la ligne partagée. |
| ScheduledWake/garde pause | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/referent_control.rs:107/131 | Nouvelle variante métier dans garde pure unique. Ne pas utiliser admit_native_delegation/halt A. |
| Route/DND/capacité/départ | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:8334/8392/4532 | Recontrôle au vrai départ et à la reprise ; aucune relance de modèle hors ligne. |
| Identité auxiliaire/propriété | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/communication/client.rs:1180 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:8715 | Identité source authentifiée ; owner du magasin ne vient pas des arguments. |
| Outil bridget_wake schedule | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:620 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp_scopes.rs:57 | FR104 unique outil demandé, base Sends/hint false, self propre ; étendre les mêmes dispatcher/catalogue. |
| bridget_wake list | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp_scopes.rs:57 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/observation.rs:307 | Actions propres WritesOwnState, requête fermée et propriétaire vérifié ; pas lecture globale MCP. |
| bridget_wake cancel | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store/threads.rs:1072 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency.rs:1129 | Mutation propriétaire avec reçu exact ; aucune annulation rétroactive de post/remise. |
| CLI wake unitaire | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/communication/client.rs:66/97 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs:950 | Socket existante ; System produit daemon pour agent direct. Fil exige identité auxiliaire membre attestée. |
| CLI wake list --all | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:10713 | Voie humaine existante, distincte des portées MCP ; aucun flag administratif dans outil. |
| CLI wake cancel humain | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:10713 | Acteur de contrôle/négociation existants ; garder règle terminal interactif du contrat. |
| observation_missed (table) | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store.rs:149 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/observation.rs:76/423 | FR106. Le snapshot et guichet_lifecycle_events ne contiennent pas ces faits/provenances once ; même Store/DB. |
| observation_missed_owners (table) | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store.rs:149 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/observation.rs:143/298 | 8192owners+dates/cutoff dépassent snapshot262144B. Borne séparée et TTL sans prolongation par lecture. |
| Ligne globale pertes de couverture | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/observation.rs:102 | Dans la table metadata prévue, une ligne fixe sans UUID/corps. Counters actuels volatile/skip ne survivent pas à éviction. |
| Notification/provenance typée | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/observation.rs:67/76/491 | Porter owner/subscription/fact_id avant retrait once, jamais parser body comme autorité. |
| Worker et retour typé borné64 | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:1724/1775 | Même worker ; callback corrélé au pending durable. Plan:36 ferme la saturation sans second writer ni verrou daemon depuis writer. |
| Snapshot once + pending facts | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:1750 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/store.rs:51 | Même transaction Store avant enqueue ; pas de consommation once faussement sauvegardée. |
| events missed / ObservationRequest | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs:2363 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:705/2412 | Action Reads dans outil existant ; pagination bornée, owner absent des arguments et pas deleteàlecture. |
| Filtres/silence volontaire/gaps | RÉUTILISER | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/observation.rs:102/423/482 | journal_only et throttle ne deviennent pas des pertes ; lacunes réelles restent datées. |
| Register/DND/épisode retour | RÉUTILISER + ÉTENDRE | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:5681/7668 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/observation.rs:298 | Hook existant couvre seulement restored ; étendre au même épisode persistant, pas une génération par tick. |
| settle_summary_ack/cutoff | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/idempotency/send_delivery.rs:768 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon.rs:8625 | Coordination nécessaire du cutoff propriétaire avec ACK099 validé ; crash entre commits repris par phase acked. |
| Calendrier intervalle/civil/TZ | CRÉER JUSTIFIÉ | /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs:6705 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/Cargo.toml:34 | libc réemployée ; aucun calcul civil équivalent. Contrat68–72 borne domaine,2881x8 conversions horslock/CAS ; CPU non mesuré. |

## Existant potentiellement pertinent non mentionne

project_round_policies et record_project_round_dispatch ont été confrontés au besoin. Leurs clés projet/génération et politique fixe ne remplacent pas wake_schedules owner/cible/cadence. Réemploi de la boucle, refus de fusionner les contrats. Store::guichet_lifecycle_events concerne des demandes de service terminées ; aucun équivalent de missed par abonnement/provenance once.

## Duplications evidentes

Aucune duplication évidente confirmée. Ne créer ni second registre099/102, ni nouvel outbox/worker. Ne pas exposer threads::post privée ; handle_with_change est déjà disponible.

## Memoires et regles applicables

AGENTS.md courant atteste158A livrée. Ponts .specify/memory/constitution.md et standards.md lus ; ArticlesXVIII/XIX/XX404–620 et référence de workflow chargés. ~/.Codex/projects/bridget/memory/MEMORY.md et CLAUDE.md absents. Précondition officielle check-prerequisites.sh absente : fallback manuel, sans installation, hook contourné ou mutation feature.json historique.

## Specs livrees applicables

099 : identité/reçu/ACK/issue ambiguë ; 100/101 : observation et silences/coverage ; 102 : membre/post/UUID/reçu/alertes partagées ; 087 et158A : garde pause/contrôle humain/portées. Corps sources priment sur anciens contrats100/101 dont certaines déclarations de reprise sont historiques. Aucun test de ces specs relancé ici.

## Journal de recherche

Un passage large rg par noms et responsabilités dans crates/skills/scripts : absence WakeSchedule/ScheduledWake/wake_schedules/observation_missed/reserve_cli_wake, puis rapprochement round_policies/099/102/observations. Lecture des puits reserve/begin_send_delivery/ACK, post/commit102, worker/file/once/gaps, Register/DND, tick/entretien, garde et scopes. Inventaire de chaque module/type/table/API/surface candidat ci-dessus.
Manifeste : /Users/moi/.cache/bridget-reuse158B-20261012/readmanifest158B.json. Les recherches rg ne valent pas lecture sémantique de tous les fichiers.

## Arbitrages

RÉUTILISER099/102/Store/worker/scopes ; CRÉER uniquement les responsabilités de domaine manquantes. Metadata owners séparée nécessaire, global fixe dans cette même table. Réserve CLI transactionnelle et issued_at serveur initial. La formule plan:21 « getter issued_at/canon existants » désigne une extension du magasin ; le getter exact issued_at n’existe pas encore, comme research:11 l’explicite.
Les deux réserves initiales sont fermées dans les candidats finaux : canal retour/backpressure/pending durable (plan:36) ; calendrier non-DST avec domaine annoncé et calcul borné (contrat:68–72). Aucun résultat d’exécution calendrier ou COUNT revendiqué.

## Gate avant tasks

- [x] Aucune duplication évidente non arbitrée.
- [x] Chaque item du plan possède une ligne d’audit.
- [x] Mémoires/règles applicables lues ou absence précise signalée.
- [x] Specs proches et primitives actuelles vérifiées.
- [x] Borne physique16/128 et retrait transactionnel des annulées soldées explicités dans les six artefacts officiels ; reçus099/102 conservés.

## Limites et six réponses ArticleXX

1. Nécessaire : FR101–107 et US5–US7 demandés, US8 hors plan tant que consommateur absent.
2. Simple : greffe sur tick/Store/099/102/worker ; nouveaux modules séparent seulement domaine et SQL.
3. Hypothèses : identité/projets attestés, données durables lisibles, calendrier dans le domaine annoncé, mêmeUIDADR011 pour CLIagent.
4. Vérifié : lectures candidates/code ciblé et comparaison de clôture des29 empreintes intermédiaires, sans Git.
5. Non vérifié : testsB, mesureCOUNT/CPUcalendrier, runtime/DB/provider/T3/activation, code futur. Aucune assertion de parité T3 actuelle.
6. Conservé : preuve pending/cutoff/UUID/issued_at nécessaire au crash ; pas helper de confort, ni reçu/quota doublon, ni dépendance supplémentaire.
Le PASS couvre le réemploi du plan candidat, pas sa réalisation. Les trente oracles sélectionnés restent une preuve historique A ; pas baseline B nouvellement exécutée.

## Objection finale ROOT — borne physique FR101

La version finale data-model-158B.md:21–23 conserve cancelled_at et annonce16 programmations actives/owner,128total. Le plan ne ferme pas les reliquats annulés/pending. Des créations/annulations répétées peuvent donc conserver un historique illimité si seul le compteur actif est contrôlé. Choix nécessaire : borne16/128 de toutes les lignes conservées, ou retrait des annulées une fois leur occurrence terminale prouvée via entretien existant. Aucun nouveau registre de cancel. Après retrait, le devenir de la lecture/rejeu cancel connu doit être explicite (reçu099 conservé ou refus de cible supprimée), sans promesse illimitée. Aucune tâche/test/code supplémentaire créé ici. Cette réserve suspend le PASS de réemploi publié quelques instants auparavant.

## Clôture officielle ROOT — réserve FR101 résolue

ROOT a intégré et gelé les six artefacts officiels158B. Le seul delta de borne est relu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/spec.md:263 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/data-model-158B.md:23/81 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/plan-158B.md:92 ; contrat wake-158B:86, research:71 et quickstart:47. La borne porte toutes les lignes physiques, annulées comprises. Retrait dans la TX d’admission seulement des anciennes annulées sans occurrence pending/inflight ; schedule_limit si aucune ligne retirable. Cancel connu rend already_cancelled dans la rétention ; UUID retiré refuse. Aucun nouveau registre ni retrait des reçus099/102.

Les six SHA officiels correspondent au reçu /Users/moi/.cache/bridget-plan158B-officiel-20261012/root-integration-plan158B.json. La réserve précédente est historique et résolue. Les40 items restent inchangés. Gate avant tasks PASS ; aucun code, test, baseline B, Analyze ou livraison B validé par ce rapport. Toutes les écritures de cet agent restent dans /Users/moi/.cache/bridget-reuse158B-20261012. FIN DES LECTURES, CACHE GELÉ.


Clôture ROOT après revueplanR1 :15hypothèses tranchées, contrats de cycle de vie/crash/worker/cutoff précisés. Les40items audités restent inchangés ; aucun service/table/module supplémentaire. GatePASS5/5 maintenu. Empreintes actualisées dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/root-plan-tasks158B.json. Pasde validationB/test/Analyze déduite.
