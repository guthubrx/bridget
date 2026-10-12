# Contre-revue Claude — plan156 R1

Date : 2026-10-12. Verdict historique de préparation : APPROVE_WITH_CHANGES.
La revue porte sur la base antérieure à la livraison155 ; ses lignes de code
restent historiques. Le principal a appliqué neuf artefacts corrigés avec
vérification des SHA avant application dans le worktree156. Aucun code ou
runtime156 n’est validé par cette revue ; T001/T002 sont des gates distincts.

Sources réellement lues :
/Users/moi/.cache/bridget-plan156-20261012-r1/review.json.
/Users/moi/.cache/bridget-plan156-20261012-r1/review.md.
/Users/moi/.cache/bridget-plan156-20261012-r1/response.md.
Le chemin fourni sans extension /Users/moi/.cache/bridget-plan156-20261012-r1/review
n’existe pas ; review.json est la sortie brute disponible. Rien n’est recréé.

| Finding | Décision conservatrice appliquée aux tâches existantes | Gate / limite |
|---|---|---|
| F1 HIGH | Aucun repli interrupt pour steer agent, même deadline fournie ; timeout humain validé et échéance du tour restent distincts | T002(a), T013 ; pas de septième vérification ou nouveau nom d’oracle |
| F2 HIGH | mid_turn_ok couvre DeliverIdempotent et Deliver historique ; UI credential/instance primaire et système interne conservés ; rôle Client immuable | T002(b,f), T003/T014/T021 ; pas origin=Human fabriquée |
| F3 MEDIUM | steer+reply=true donne invalid_intent avant connexion/réservation | T007/T010 existants étendus |
| F4 MEDIUM | Interruption utilisateur terminale malgré152 ; mission_active maintenu pour l’agent, humain explicitement autorisé exempt | T002(d), T020 ; dépendance finale corrigée sans nouveau besoin |
| F5 MEDIUM/LOW | Projet avant admission guidance ; admitted après acceptation durable, refus tardif jamais compté | T015/T018 existants ; effet fournisseur reste unconfirmed |
| F6 LOW | FIFO conservée après interrupt : un message antérieur peut porter le prochain tour | T013 existant, sans priorité inventée |
| F7 LOW | Rôle immuable sur la même connexion ; reconnect agent prouvé ne devient pas humain ; autre connexion Attach brute hors modèle | T002(e), T016 ; aucune preuve humaine ou registre neuf |
| F8 LOW | QueueList lit uniquement send_deliveries dispatching ; Send historiques et files internes exclus | T024–T026 ; outil unique en lecture seule |
| F9 LOW | Worktree156 créé par root après155 ; chemin daemon/native_delegation.rs ; six vérifications a–f | Lignes historiques remplacées uniquement après preuve fraîche |

Le principal conserve 31 tâches T001–T031. Les cases et noms d’oracles ne sont
pas modifiés par la reprise documentaire. Les deux HIGH ont leur correction
documentaire ; cela ne prouve aucune correction runtime encore exécutée.

## Relecture Analyze fraîche

La pré-analyse indépendante base155-fresh a identifié U1 (délai exact), C1
(coût COUNT), I1 (mission_active et bridget_cancel), M1 (minimalisme non mesuré).
La reprise précise les deux lectures T006 déjà prévues : couple count+LAST,
cible count+THIRD_LATEST ; la garde reçoit now et les dates, pas COUNT seul.
last+30 et third+600 <=now sont admis. Aucun état/table/service supplémentaire.
Les comptes SQL restent inclusifs ; retry_after exact réutilise guidance_events.
Les 31 IDs, cases non cochées et 87 références d’oracles sont conservés à cette
correction. Les faits de T002 et l’audit d’existant frais sont encore attendus.

LU : revue brute, review.md, response.md, artefacts actuels, pré-analyse et
arbitrages du principal sur la projection temporelle et les trames.
NON LU par ce rédacteur : T3 complet, nouveaux résultats d’exécution156 ou
inspection runtime ; les prélectures source des owners restent attribuées.
Aucune promesse d’Analyze PASS courant ou de réutilisation gate 5/5 n’est copiée
sans la preuve fraîche de ces owners. Aucun nouveau test, helper ou outil.

Empreintes de provenance :

- /Users/moi/.cache/bridget-plan156-20261012-r1/review.json : SHA256 a54aec422215b9d1e4399dca29efb9122abe302c6207e597d4adb2ac69b9c2f2.
- /Users/moi/.cache/bridget-plan156-20261012-r1/review.md : SHA256 17a78c6b554bc2a9ac2c14d755e51512584c1b396e7e0ec29b13fc2f0cf633e5.
- /Users/moi/.cache/bridget-plan156-20261012-r1/response.md : SHA256 cdccaddbdb6a599cc8b3cf291bf4e60e56945bc78a0a5eeb965f74d8c460c5c1.

## Revalidation conservatrice de F1/F2/F8 sur la base155

T002(c) révèle une hypothèse historique erronée : Cursor n’a pas de guidage
actif natif dans le bundle effectivement installé149 et peut convertir steer
en interruption. Root retient Claude seul ; Cursor steer_unavailable et
busy attend, même mid_turn_ok=true. Les mêmes oracles T012/T021/T029 couvrent
ce cas, sans nouveau besoin ni question utilisateur. La preuve complète du
paquet et les autres axes T002 sont attribués à l’owner de lecture fraîche.
QueueList est ajouté à l’action pure pour le refus child_proof déjà prévu
T005/T024, sans guidance_event, quota ni écriture de pause. Les dates now/LAST
couple/THIRD_LATESTcible précisent le délai exact, sans état dérivé persistant.

## État courant après T001

T001 est acquise et cochée par root : baseline 774 PASS/0 FAIL/7 ignored,
214 sources stables, sur métadonnées be7957ee après livraison155. Les anciennes
phrases «toutes les cases non cochées» décrivent la préparation avant ce reçu.
T002 et les 29 tâches d’implémentation restent ouvertes à ce checkpoint.
Aucune case cochée par le rédacteur. Les corrections normatives FR006/US4/FR022
sont Claude seul ; T029 trace Cursor refusé/busy attend parmi les changements
visibles acceptés. Build/install/activation Bridget autorisés 152–160 avec zéro
mission native non terminale, puis nettoyage ; aucune relance explicite T3.

## Limite de compatibilité REST/V1 du pont T3

La voie REST/V1 Client.dispatch ignore l’ancien booléen et ne sait pas exprimer
deliveryIntent. Elle refuse donc localement toute intention explicite avant
émission : steer_unavailable pour steer, motif de capacité explicite pour
interrupt, avec les erreurs/report_discard existants. L’envoi ordinaire None
reste inchangé. Le paquet installé149 est V2 : le trajet de production visé
n’est pas changé par cette fermeture V1. Aucun protocole, API, type ou nom
d’oracle supplémentaire ; T012 étend seulement les cas déjà prévus.

## Clôture de la revalidation T002 par lecture

Tous les axes a–f sont levés dans le rapport frais d’audit_existant151, sur17
sources égales aux blobs155 et le bundle installé149. Claude seul/Cursor/V1,
provenance UI, rôle immuable, Task/mission et chemins Deliver/Cancel sont
explicités. Aucun modèle ou test n’a été lancé par l’auditeur. Le PASS du gate
reuse est24/24 items et5/5 conditions, pas un PASS d’implémentation156.
Rapports et SHA : research.md, section Vérifications levées ; root vérifie et
coche T002 seul. Le seul Analyze après autofix est celui de prelecture154 sur
le gel courant ; aucun Analyze concurrent n’est déclaré par ce rédacteur.


## Contre-revue implementation156 R1 — 2026-10-12

Fournisseur : claudeAgent ; modèle : claude-opus-5-5. Revue T3 déléguée après Converge2. Verdict reçu : APPROVE, limité au code lu. Aucun CRITICAL/HIGH/MEDIUM annoncé. Aucun test/build exécuté par le relecteur.

Le résultat intégral et ses zones non lues sont conservés dans validation/code-review156-r1.md et validation/code-review156-r1.json.

| Objection | Vérification ROOT | Retenue | Décision |
|---|---|---|---|
| Position globale et limite200 avant filtrage QueueList | daemon.rs9354–9385 : index+1 précède le filtre ; lecture de lignes dispatching limitée200 | Oui, LOW | Limite documentée ; aucune nouvelle interface ni correction de confort dans cette livraison. Un pair ne lit pas les corps étrangers. |
| Restes du suivi de réponse lors d’échec d’écriture admitted, Send historique | daemon.rs19271–19320 et track_reply_cycle9525–9560 : suivi durable/mémoire puis INSERT admitted | Oui, LOW | Risque limité au défaut de stockage. Aucun push après Nack. Conservé au backlog ; aucune fausse preuve de rollback. |
| Cancel d’un pair trace intent_not_permitted pour l’interruption refusée | daemon.rs19750–19811 : garde interruption distincte de cancel_request | Oui, remarque LOW | Refus concerne l’interruption, pas le retrait. Il ne compte pas dans les quotas admitted. |
| prepare_dispatch avant garde consommerait un quota historique | Lecture ROOT entière prepare_dispatch9402–9520, circuit_breaker35–69, dedup25–47, envelope58–85 et router129–154 ; record/mark_sent/mark_relayed après garde daemon10027–10032 et19160–19165 | Non | check est en lecture ; dédup/quarantaine purgent les expirés sans inscription. Le budget hops du message local est décrémenté, sans consommer de quota historique. Aucun échange refusé n’est enregistré comme envoyé par prepare_dispatch. |

Limites : la revue adverse n’a lu ni les tests ni tous les documents, ni ACP/wrapper/façades en entier. Converge2 et les preuves ROOT restent distincts. Les huit échecs hérités du FULL sont conservés, et les trois fixtures corrigées ne transforment pas ce FULL en succès global. Le correctif T032 a reçu93PASS et3E2EPASS ciblés. L’audit final reste à exécuter.
