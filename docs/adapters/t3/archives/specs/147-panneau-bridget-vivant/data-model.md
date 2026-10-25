# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/data-model.md](../../../../../../specs/147-panneau-bridget-vivant/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Clé : chaîne JSON du triplet `[environmentId, projectId, threadId]`. Les trois composants viennent du contexte T3 attesté ; aucun concaténat ambigu avec séparateur libre.
- Valeur : UUID canonique du fil Bridget, ou absence d'entrée si aucun choix.
- Validation : valeur UUID valide, structure de clé conforme au triplet attendu. Une donnée persistée invalide n'est pas restaurée.
- Ancien snapshot : champ absent → table vide. Aucun corps n'est migré.
- Suppression : refus d'accès au fil confirmé, fil inexistant confirmé ou suppression du contexte selon la réconciliation native du store.
- Indisponibilité : conserver la référence. Elle n'autorise pas l'affichage d'un ancien corps lors d'une reprise non revalidée.

<a id="extrait-2"></a>

## Extrait 2

Un suivi possède : connexion Client négociée, conversation T3, projet résolu, agent/instance attestés, génération opaque UUID, ressource de sortie et invalidation en attente.

<a id="extrait-3"></a>

## Extrait 3

canonical_uuid du contrat humain145/146/147 et les curseurs restent stricts. La normalisation ne vaut aucune attestation d'identité ni permission. ACK garde son refus ReceiptInvalid. Le montage MCP US7 suit le plan natif validé, pas une identité injectée par un outil T3. Aucun champ d'identité dans stdio/env MCP ; les preuves OS privées restent distinctes des fixtures de configuration SDK.
