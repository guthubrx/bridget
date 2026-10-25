# Contrats de correction 099

## Preuve d'identité

Registered peut contenir une preuve privée optionnelle destinée au seul owner.
RegisterAuxiliary présente agent_id, instance_id et credential ; aucune route
canonique n'est créée. Le daemon refuse preuves absentes, erronées, d'une autre
identité ou d'une incarnation terminée. Les auxiliaires existants sont invalidés
avec leur owner. Les rôles Attach et Service ne peuvent emprunter ce chemin.
ClientHello/issuer_scope ne confère pas une identité.

## Envoi

Send classique ne répond pas Ack si l'écriture a échoué. La notification est bornée,
hors verrou global. Une trame partielle est fermée, pas concaténée à une autre.
Les demandes et leur corrélation sont durables avant remise ; issue ambiguë
nommée plutôt que retry implicite pouvant doubler une exécution.

SendIdempotent exige une identité attestée correspondant à from avant admission.
L'identifiant de rejeu ne vaut pas une autorisation d'emprunter une autre identité.

## adaptateur de bureau

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/099-fiabilite-communications/contracts/communication.md#extrait-1).
