# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/remaining-transport.md](../../../../../../../specs/174-audit-incidents/validation/remaining-transport.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Copies lues dans `/home/moi/.cache/bridget-174-remaining/t3`.

<a id="extrait-2"></a>

## Extrait 2

- `supervisor.ts` (`retryDelayMs`, ligne 104) : échelle fixe plafonnée, sans gigue. Ce fichier ne fournit donc pas de modèle de gigue à reprendre.
- `session.ts` (`onDisconnect`, `RpcClientError`) : toute coupure devient une erreur transitoire de type `transport`. T3 ne distingue pas une coupure avant ou après écriture. Cette distinction, qui porte `outcome_unknown`, n'a donc pas d'équivalent dans T3.
- `transport.ts` : `sanitizeThreadErrorMessage` masque les erreurs de transport dans l'interface. Ce masquage ne doit pas s'appliquer à `outcome_unknown`. Nous ne l'avons pas modifié ; les consommateurs d'interface restent à vérifier.
