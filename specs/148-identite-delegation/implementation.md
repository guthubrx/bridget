# Implémentation148

Statut : Implémentée et livrée sur disque. Aucun processus de production relancé.

Le daemon porte une saga native durable. Le registre fixe le fournisseur, le
modèle, l'effort et la posture avant lancement. La flotte gère les processus.
La mission et le résultat utilisent les remises idempotentes existantes.
Pour un enfant natif neuf, les faits durables d'identité et de Git sont joints
à sa mission dans un seul tour fournisseur. L'enveloppe et sa corrélation restent
inchangées. Une relance conserve la carte de reprise séparée déjà existante.
L'état durable distingue remise, travail, attente des enfants, résultat,
annulation et nettoyage. La définition du fournisseur est figée à l'acceptation.

Les droits sont natifs. Les demandes MCP ne choisissent jamais leur parent.
Un grant humain est inaccessible depuis le MCP. Sa révocation ferme les nouvelles
demandes et tout retour implicite aux droits hérités. La reprise d'une instance
gérée exige une preuve de propriété vivante. Les lectures restent sans mutation.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/implementation.md#extrait-1).

## Validation achevée

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/implementation.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/implementation.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/148-identite-delegation/implementation.md#extrait-4).

## Limites connues

La posture d'écriture Claude/GLM est refusée par le registre actuel. Le catalogue
annonce `development_protocol_unavailable`. La posture d'inspection reste
disponible sous les droits natifs du parent. Aucun confinement fictif n'est promis.
Les anciens processus continuent à utiliser leur code déjà chargé.
