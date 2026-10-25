# Contrat101

Outils publics existants conservés : bridget_events et bridget_journal.
Aucun argument `from`/identité native ajouté aux outils privés.

`events types` annonce les sources et les types effectivement observables.
`events sub` conserve event/agent/file/once/ttl_secs et refuse nommément
source introuvable, indisponible ou incompatible. Sans agent précis, couverture
limitée aux sources compatibles déclarées ; aucune promesse universelle.
`events list` expose état de surveillance, instance daemon et limites de remise.
La déconnexion d'une source n'est pas un turn_ended.

Les wrappers annoncent leurs capacités sur la connexion primaire attestée ;
un client auxiliaire ne peut ni annoncer des capacités ni produire des faits.
Les clients anciens sans annonce sont explicitement non compatibles.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/101-abonnements-t3/contracts/observation.md#extrait-1).

Fenêtre100 conservée : événements reçus par le daemon après abonnement.
Une fin de tour n'atteste ni succès ni fin de projet. Notifications système
exclues de l'observation afin de ne pas créer de cycles.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/101-abonnements-t3/contracts/observation.md#extrait-2).
