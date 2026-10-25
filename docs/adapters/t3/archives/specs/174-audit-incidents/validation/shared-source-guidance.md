# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/shared-source-guidance.md](../../../../../../../specs/174-audit-incidents/validation/shared-source-guidance.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Livraison : source finale 06b1da2336f18245565bb53076412e9ea7dcb532 poussée sur main.
Binaire macOS optimisé et signé, compilation finale 189,98 secondes, code retour 0.
SHA-256 : e49e231b877a18225d11dd2e29831a4469c8852e0eda942df0bd9d8b22ea186c.
Seule la build finale a été activée sur le maître. Le MCP officiel confirme
`running=true`, `build_id=06b1da2336f1`. Une réinscription transitoire du lien de
ce fil a précédé ce contrôle. T3 et le worker distant sont restés en place.
Les 16 tâches du parent incident ont conservé leurs instances et erreurs ; une
ronde est devenue result_available sans erreur entre les deux observations.
