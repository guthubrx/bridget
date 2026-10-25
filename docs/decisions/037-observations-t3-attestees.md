# ADR 037 — Observations adaptateur de bureau attestées et interruption visible

Date : 2026-09-16. Périmètre : session 101. Décision implémentée dans la branche ;
validation de production distincte. Complète les ADR 034 et 036, sans les
réécrire comme si leurs limites historiques n'avaient jamais existé.

## Contexte

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/037-observations-t3-attestees.md#extrait-1).

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/037-observations-t3-attestees.md#extrait-2).

## Décisions

### Identité réelle, sans redémarrage fournisseur

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/037-observations-t3-attestees.md#extrait-3).

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/037-observations-t3-attestees.md#extrait-4).

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/037-observations-t3-attestees.md#extrait-5).

### Faits et capacités avant abonnement

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/037-observations-t3-attestees.md#extrait-6).

Les capacités sont annoncées par une connexion principale attestée. Un
auxiliaire ne peut ni annoncer une capacité ni produire un fait. Le catalogue
expose les sources compatibles ; les anciens wrappers sans annonce ne sont
pas présumés compatibles. Refuser la souscription visant un agent inconnu,
indisponible ou incapable de produire l'événement. Un filtre général ne couvre
que les sources déclarées, jamais universellement tous les agents.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/037-observations-t3-attestees.md#extrait-7).

### Persistance d'une interruption, pas promesse de continuité

Conserver une trace bornée des abonnements dans le Store existant, sans créer
de bus de notifications durable. Après redémarrage, restituer les abonnements
non expirés comme `interrupted`, avec avertissement au retour du propriétaire
et état consultable dans `events list`. Aucun redémarrage automatique : un
nouvel abonnement explicite reprend sur les faits futurs seulement.

Pendant la vie du daemon, la perte d'une source produit `source_unavailable` ;
son retour peut rétablir `active`. Les notices signalent perte, reprise ou
changement de couverture, sans rejouer la lacune. Elles respectent les mêmes
limites de remise et le mode ne-pas-déranger que les notifications ordinaires.
L'état consultable ne dépend donc pas de la réception de la notice.

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/037-observations-t3-attestees.md#extrait-8).

## Compromis et limites

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/037-observations-t3-attestees.md#extrait-9).

## Vérification et adoption

[Note technique de l’adaptateur](../adapters/t3/archives/docs/decisions/037-observations-t3-attestees.md#extrait-10).
