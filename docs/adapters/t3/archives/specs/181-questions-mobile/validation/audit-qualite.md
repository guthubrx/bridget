# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/validation/audit-qualite.md](../../../../../../../specs/181-questions-mobile/validation/audit-qualite.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Critère | Niveau /4 | Justification |
|---|---|---|
| Frontières et résolution des dépendances | 3 | DomaineBridget natifBridget, adaptateurconf iné, refus155séparés ; pas de nouvellelib. |
| Maîtrise des cycles | 3 | Lecture des imports et flux : transporttypes->daemonstore, adaptateur->protocole, aucunimportT3dansdomaine. |
| Séparation construction et exécution | 3 | SDK/buildconfig public séparés du namespace/credentials ; binaire/associationpreuve distingués. |
| Assemblages et dépendances dynamiques | 2 | Assemblages concernéspassent etmoteurvocalIntent facultatif ; pas de qualificationtoutesversionshôtes/fournisseurs. |
