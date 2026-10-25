# Archives techniques de l’adaptateur T3

Document d’origine : [specs/158-reveils-controle/plan.md](../../../../../../specs/158-reveils-controle/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Décision | Choix | Alternative rejetée | Raison |
|---|---|---|---|
| Vue globale | requête de contrôle, acteur humain exigé | drapeau sur `lineage` ; `ControlStateV1` seule | autorité humaine ; la capacité s'accorde à tout client |
| Arrêt et pause | deux états, une garde commune | règle séparée dans l'admission | ADR 027 : garde unique |
| Idempotence de l'arrêt | reçus bornés portant la réponse, annulation une seule fois | `last_command_id` ; rejeu qui réannule | A, B, rejeu de A remettrait l'arrêt ou annulerait une tâche C neuve |
| Stockage de l'arrêt | ligne unique | genre dans `control_events` | reconstruction de table à `CHECK` en production |
| Portées | table par action + test | types à la T3 | ~30 déclarations à réécrire |
| Plafond | `effective_posture`, durée de vie de l'enfant | posture demandée, tâche non terminale | une posture omise ; un enfant garde la vie entre rondes |
| Fils pour `discovery` | `read` et `ack` permis, `post` refusé | tout fil interdit | lecture sans `ack` rejoue la même page |
| Budget de tours | retiré | compteur d'usage actuel | non dédupliqué, Claude seul, remis à zéro par ronde |
