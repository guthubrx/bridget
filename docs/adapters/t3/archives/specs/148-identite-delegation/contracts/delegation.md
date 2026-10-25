# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/contracts/delegation.md](../../../../../../../specs/148-identite-delegation/contracts/delegation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les grants humains passent par le protocole de contrôle natif et la CLI humaine.
Ils ne sont pas accessibles aux quatre outils MCP. Les secrets de session T3
sont exclus de l'environnement enfant même si `pass_env` les demande.
La révocation est conservée par AgentId stable jusqu'à un nouveau grant humain.
Une reprise d'instance exige une preuve native et le départ de l'ancienne
présence et de sa connexion primaire. Une erreur de capture persistante produit
`native_result_not_persisted`, sans accusé ni livraison directe hors de la saga.
