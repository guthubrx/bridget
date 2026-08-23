# Contre-revue adverse — cxbridget — session 007

**Date** : 2026-08-22 · **Round 1**
**Croisement** : relecture par un moteur distinct de celui de l'auteur
**Question posée** : défauts logiques, risques sous-estimés, trous de
couverture du plan 007 (transport ACP, client JSON-RPC maison, registre,
capture de réponse) — points sensibles annoncés : D-201, D-206, FR-007, stdio
multi-threads.
**Verdict reçu** : `APPROVE_WITH_CHANGES` (8 objections, les n°1, 3, 4, 7
signalées comme les plus bloquantes).

| # | Objection | Vérifiée comment | Retenue | Raison / correction appliquée |
|---|---|---|---|---|
| 1 | `deliver()` bloquant rendrait FR-007 impossible ; lecteurs multiples corrompraient le démultiplexage | lecture `wrapper.rs:817` (le thread d'écoute appelle `deliver()` inline) — confirmé | **oui** | D-204 réécrit : lecteur unique propriétaire de stdout, writer sérialisé, table `request_id → waiter`, worker de tours, `deliver()` non bloquant ; répercuté sur T704 |
| 2 | « 7 méthodes » sous-estime le contrat JSON-RPC (hors-ordre, ids mixtes, `error`, EOF, lignes invalides, écritures concurrentes) | analyse — exigences JSON-RPC 2.0 exactes | **oui** | matrice de conformité ajoutée à research R-004, fixtures exigées pour les 3 adaptateurs dans T704 |
| 3 | D-206 logiquement faux (tour `reply=no` sans échéance ; demande plus courte en file ne doit pas annuler le tour actif ; double signalement daemon/transport) | relecture D-206 initial + mécanisme d'expiration 003 (le daemon prévient déjà l'émetteur à l'échéance) | **oui** | D-206 réécrit : daemon autorité unique du cycle de vie ; `session/cancel` limité au tour actif dont le message a expiré ; timeout de transport configurable pour les notifications ; purge des expirés en file |
| 4 | file non bornée, sans purge ni politique de panne | relecture data-model — confirmé absent | **oui** | politique de file ajoutée au data-model (capacité 32 par défaut, refus motivé, purge, échec des `reply=yes` à la mort) ; FR-007 amendé ; tests ajoutés à T705 |
| 5 | TOML contredit « aucune nouvelle crate » | `Cargo.toml` (aucun parseur TOML) — confirmé | **oui (déjà traité)** | arbitrage TOML→JSON appliqué au reuse-audit *avant* réception de la revue ; complété par la validation atomique demandée |
| 6 | aucun champ `version` au registre alors que la spec promettait « version constatée vs attendue » | relecture data-model/spec — incohérence confirmée | **oui** | promesse retirée de la spec : garantie limitée au pin `@paquet@x.y.z` dans `args` + négociation de version ACP à l'`initialize` (option minimale retenue plutôt qu'une sonde générique, Article XIX) |
| 7 | `mark_answered` exécuté avant disjoncteur/dedup/routage : une réponse refusée clôt quand même la demande | lecture `daemon.rs:1200-1330` — **confirmé** (`mark_answered` en 1221-1231, garde-fous en 1233+, livraison en 1318+) | **oui** | D-208 créé : transition `answered` déplacée après livraison réussie ; décision associée : `in_reply_to` traverse le refus DND (réponse sollicitée) ; correction et tests dans T705 — défaut préexistant, corrigé car le flux ACP l'amplifierait |
| 8 | couverture manquante : `stopped` ≠ `unreachable`, busy/idle après reconnexion, équipier fédéré réel | relecture spec/plan — partiellement confirmé (edge cases présents mais sans transitions ni tests automatisés) | **oui, partiellement** | table de transitions d'état ajoutée au data-model ; tests reconnexion-pendant-tour / arrêt propre / mort d'adaptateur ajoutés à T709 ; fédération : réutilisation de `scripts/test-federate-ssh.sh` exigée dans T711, repli manuel consigné — un test fédéré automatisé complet reste au-delà du périmètre raisonnable de la session (assumé) |

## Round 2

**Verdict reçu** : `APPROVE_WITH_CHANGES` — les 8 corrections du round 1 sont
jugées couvertes ; 6 objections nouvelles + nettoyage éditorial. Conclusion du
relecteur : « après correction, je n'ai pas d'objection au démarrage ».

| # | Objection | Vérifiée comment | Retenue | Correction appliquée |
|---|---|---|---|---|
| 1 | propriétaire de la file contradictoire (D-204 : transport ; data-model/T705 : wrapper) — deux files casseraient FIFO/cancel/backpressure | relecture croisée des artefacts — contradiction réelle | **oui** | `AcpTransport` propriétaire unique de `turn + queue` (D-204, data-model, T704) ; T705 réécrit : le wrapper branche et relaie |
| 2 | refus synchrone de file pleine impossible (le daemon accuse réception au push) ; annulation textuelle impurgeable par id | flux daemon relu (Ack au push vers wrapper) — cohérent | **oui** | D-209 créé : `CancelDelivery { id, reason }` et `DeliveryRejected { id, reason }` typés ; overflow = échec asynchrone terminal motivé ; data-model et T705 alignés |
| 3 | bypass DND forgeable par `in_reply_to` arbitraire | raisonnement — valide (mark_answered déplacé après routage, la validation manquait) | **oui** | D-208 durci : validation non mutante (état ouvert, participants croisés) avant bypass ; test id forgé/participants incohérents dans T705 |
| 4 | T704 exigeait les fixtures des 3 adaptateurs avant leur validation en T707/T708 — ordre inexécutable | relecture tasks — confirmé | **oui** | T704 = générique + fixtures Codex (du spike T701) ; T707 ajoute Claude ; T708 ajoute Gemini, chacun contre toute la matrice |
| 5 | preuve SC-003 invalide (Gemini déjà dans les défauts ; `agents.json` hors Git) | relecture T703/T708 — confirmé | **oui** | T708 réécrit : fixture de registre temporaire versionnée (Codex/Claude), ajout dynamique de Gemini, zéro `.rs` au diff |
| 6 | « non vérifié avec raison » ne peut satisfaire FR-014 | relecture T711 — exact au regard de la règle « livraison partielle = In Progress » | **oui** | T712 créé : gate fédération bloquant, séparé de la non-régression locale T711 |
| éd. | résidus TOML / « version attendue » / ancien timeout (plan lignes 52-58-88-117-120-139, spec FR-006 et entité) | `grep` — confirmé | **oui** | tous remplacés par JSON, pin `@x.y.z` dans la commande, négociation ACP, D-206 reformulé |

**Issue** : corrections des rounds 1 et 2 intégralement appliquées (14/14
objections retenues, 0 rejetée). Notification de clôture envoyée au relecteur.
Le paquet est prêt pour délégation d'implémentation.
