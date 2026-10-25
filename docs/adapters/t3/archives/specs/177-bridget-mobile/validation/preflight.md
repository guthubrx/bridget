# Archives techniques de l’adaptateur T3

Document d’origine : [specs/177-bridget-mobile/validation/preflight.md](../../../../../../../specs/177-bridget-mobile/validation/preflight.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Le daemon ouvre sa base sous verrou exclusif et ne remplace pas une socket active : daemon.rs:6761.
- Les vues humaines existantes exigent une liaison de conversation et une présence locale : daemon.rs:18115.
- La réattestation valide exactement un endpoint loopback correspondant au port du runtime local : t3code_mcp.rs:120.
- Le même contrôle est réappliqué avant l'admission et avant la remise distante : native_delegation.rs:4017 et le chemin de démarrage distant.
- Un simple tunnel vers le maître ne rend pas ce contrôle portable.
- L'envoi humain nu possède déjà son propre chemin idempotent sans usurper un agent : daemon.rs:12120.
