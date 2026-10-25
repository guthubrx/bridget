# Contre-revue adverse du plan — SPEC-165 R2

Date : 2026-10-14  
Tâche : `node:delegated-task:command%3Amcp%3Ae3126f2b-7428-43cd-ad5d-737df2712318%3Adelegate-task%3Aspec165-plan-review-r2-20261014`  
Mode : lecture seule

## Verdict reçu

**BLOCKED pour le plan initial.** La solution reste locale, mais elle doit drainer le dernier état terminal avant de fermer.

## Findings retenus

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/adversarial-review.md#extrait-1).

## Invariant corrigé

Un sous-agent terminal sans lien et sans obligation reste déconnecté. Un sous-agent qui possède déjà un lien reçoit son dernier état. Le lien ferme seulement quand le worker publie la même clé comme drainée, ou quand la borne dure expire. Une obligation persistée peut rouvrir le lien afin de finir une corrélation ou de recevoir un ACK.

## Tests exigés

- matrice filiation, état et obligation ;
- parsing V1/V2 et relation inconnue refusée ;
- état `running` puis `completed`, `turn_end`, réponse et ACK avant fermeture ;
- cas silencieux sur cinq lectures ;
- expiration avec obligation conservée ;
- terminal puis réactivation pendant le drain ;
- remise reçue pendant le drain ;
- reprise d'une obligation présente sur disque ;
- worker paniqué, socket fermée et lecteur terminé ;
- 1 000 enfants terminaux sans obligation et 30 racines.

Le plan doit recevoir une nouvelle contre-revue après ces corrections.

## R3 — Verdict et corrections

Date : 2026-10-14  
Tâche : `node:delegated-task:command%3Amcp%3Ae3126f2b-7428-43cd-ad5d-737df2712318%3Adelegate-task%3Aspec165-plan-review-r3-20261014`

**Verdict : APPROVE_WITH_CHANGES.** L'architecture est validée. Trois corrections majeures et trois précisions mineures restent requises avant le code.

| Finding R3 | Réponse intégrée |
|---|---|
| Obligation créée après le scan puis lien mort | Relecture ciblée de l'état après une mort imprévue et après perte du serveur. |
| Scan initial imprécis | Itération sur le premier snapshot réussi avec `load_thread_state(id)` ; aucune inversion des noms compacts. |
| Borne repoussable | `drain_since` appartient au `Link` et ne repart pas sur une clé terminale nouvelle. |
| Tests non isolables | Fonction pure de réconciliation et construction de `Link` à partir de morceaux. |
| Chemin `retain` sans fermeture | Finalisation explicite et `Drop` de sécurité pour tous les retraits. |
| Quiescence périmée | Invalidation sur chaque remise et chaque ajout dans `pending`. |
| Ordre documentaire faux | Relais et journal s'arrêtent dans le worker avant sa fin ; le propriétaire ferme ensuite son clone et joint le lecteur. |
| Tour terminal survenu pendant l'arrêt | Rattrapage si le tour est connu dans `turn_origins` et absent de `ended_turns`. |

Une R4 courte doit confirmer ces corrections avant l'écriture du code.

## R4 — Vérifications runtime finales

Date : 2026-10-14  
Tâche : `node:delegated-task:command%3Amcp%3Ae3126f2b-7428-43cd-ad5d-737df2712318%3Adelegate-task%3Aspec165-plan-review-r4-20261014`

**Verdict : APPROVE_WITH_CHANGES.** Les corrections R3 sont confirmées. Deux gardes supplémentaires sont demandées.

| Finding R4 | Preuve ou correction |
|---|---|
| Filiation HTTP réelle non attestée dans les artefacts | Lecture expurgée du flux V2 réel : chemin `threads[].lineage.relationshipToParent`, 1 121 `subagent`, 10 `fork`, 406 racines. Preuve ajoutée à `validation/runtime-before.md`. |
| Nombre de reprises persistées inconnu | Lecture des 1 093 états compacts présents : zéro `pending`, zéro tour connu non projeté, zéro obligation à rouvrir. |
| Pic possible sur un futur stock d'obligations | Plafond constant de 32 drains simultanés, avec report des obligations suivantes. |
| Idempotence de la finalisation | Test ajouté à T008 avec le chemin de panique. |

Une R5 limitée à ces deux corrections doit donner le verdict final.

## R5 — Approbation finale

Date : 2026-10-14  
Tâche : `node:delegated-task:command%3Amcp%3Ae3126f2b-7428-43cd-ad5d-737df2712318%3Adelegate-task%3Aspec165-plan-review-r5-20261014`

**Verdict : APPROVE.** Aucune objection ne reste ouverte. Le code peut commencer sans décision utilisateur supplémentaire.
