# Contre-revue adverse de l'implémentation 102 — cursor-listen

Date : 2026-09-17. Demandes : 22:06 à Bridget-enhance (Codex, tour terminé en erreur
à 22:09), relance 22:12 (erreur immédiate à 22:12), puis 22:22 à cursor-listen
(client Cursor, UUID 04c521fe-3282-41e2-bbc7-e1590ce3c351, projet 67.ListeToMe),
réponse reçue à 22:26. Fournisseur/client déclaré différent de Claude ; l'identité
commerciale du modèle n'est pas attestée. Lecture seule confirmée par le relecteur.

## Question posée

Cinq points sur le worktree 102 (spec, analysis §Converge, implementation, code et
tests nommés) : atomicité/rejeu, courses des sollicitations, lecture/reçus,
neutralisation des notices et erreur uniforme, minimalisme. Verdict attendu APPROVE /
APPROVE_WITH_CHANGES / BLOCKED avec fichier et fonction par objection.

## Verdict reçu : APPROVE_WITH_CHANGES

Aucun BLOCKED : « pas de chemin convaincant de perte d'entrée, de rejeu exact
divergent, ni d'acquittement d'une génération par l'accusé d'une autre ».

| Objection | Vérifiée comment | Retenue | Raison / action |
|---|---|---|---|
| MED — `apply_ack` n'apaise pas une ligne `in_flight` : une alerte peut encore arriver après lecture confirmée | plan.md §6 (« un envoi déjà en vol peut encore arriver… lecture vide, aucun travail requis ») et FR-018 (alerte déjà remise non rappelable) ; store `apply_ack` ne touche qu'une ligne sans remise active ; marquer `satisfied_by_read` en vol casserait la corrélation `settle` par génération | Partiellement (documentation) | Comportement voulu par le plan ; phrase ajoutée dans commandes.md « Alertes, états et limites ». Aucun rappel de remise en vol. |
| LOW — rejeu du reçu actif sans budget d'octets | `thread_read` rejoue exactement `base_seq+1..=through_seq`, borne figée lors de la première page bornée ; les entrées sont immuables donc la page rejouée est identique octet pour octet | Non | Asymétrie sans effet ; un budget appliqué au rejeu pourrait au contraire couper une page déjà promise. |
| LOW — `load_wake(...).unwrap_or_default()` après UPSERT dans `thread_post` | code lu ; une absence ne peut être qu'une incohérence de base | Oui | Remplacé par `StoreError::Invariant("sollicitation absente après écriture")` : la transaction échoue au lieu de publier un état vide. |
| COUV — V30/V36 documentaires, pas de recette wrapper réel avec faux fournisseur | implementation.md (limites) | Oui, comme limite visible | Maintenu hors périmètre de cette recette ; consigné dans analysis.md et la remise. |
| OK — courses post/ack/indeterminate/échéance | V08, V09, V24, V25, V27 | — | — |

Aucune objection n'a déclenché de régénération de fichier ; une correction ciblée
(une ligne) et une phrase de documentation. Recette complète relancée après la
correction (voir implementation.md, T030).
