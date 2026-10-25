# Archives techniques de l’adaptateur T3

Document d’origine : [specs/181-questions-mobile/analysis.md](../../../../../../specs/181-questions-mobile/analysis.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| ID | Sévérité | Finding | Résolution vérifiée |
|---|---|---|---|
| A1 | HIGH | « dans un environnement natif isolé » pouvait imposer l’arrêt de l’application utilisateur | Exigence précisée : environnement de recette sans composant/connexion T3, hôte utilisateur préservé ; T018/T020 |
| A2 | HIGH | Le nouvel outil MCP ne doit pas donner un détour à une preuve enfant155 | Plan précise refus aux preuves déléguées et réemploi du principal humain uniquement pour répondre ; T004/T005/T006 |
| A3 | MEDIUM | Notification complexe et action directe non distinguées | Plan précise une demande simple compatible RemoteInput, ouverture du formulaire sinon ; T015/T016/T019 |
| A4 | MEDIUM | Catalogue des anciens clients MCP et code installé pouvaient être confondus | Plan distingue nouveaux lancements et processus chargés ; T020/T021 et reçu final |
