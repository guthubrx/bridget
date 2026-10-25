# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/data-model.md](../../../../../../specs/145-bridget-thread-viewer/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La conversation T3 et son projet proviennent du modèle de lecture serveur. `project.workspaceRoot` est l'autorité de racine. La session du fournisseur peut être dormante. Le lecteur humain ne réclame pas un processus fournisseur actif.

<a id="extrait-2"></a>

## Extrait 2

`T3ThreadBindingFact` possède `version:1` et `t3_thread_id:UUID`. Le lien primaire le publie après `Registered` et `report_project_context`. La connexion possède le fait. Le daemon vérifie route, identité vivante, agent UUID stable, instance UUID stable et projet actif T3 canonisé. La déconnexion retire l'index. L'absence ou l'ambiguïté ferme l'accès. Aucun stockage durable nouveau.

<a id="extrait-3"></a>

## Extrait 3

La preuve source T3 utilise un `HashSet` en mémoire `t3_project_connections`, car le tuple `communication_projects` perd cette source. Un fait autre/invalide retire la preuve. La déconnexion retire preuve et liaison. Une ambiguïté reste un refus explicite.

<a id="extrait-4"></a>

## Extrait 4

Clé : environnement authentifié + conversation T3 + fil Bridget + génération de requête. Données transitoires : liste chargée, sujet, détail sélectionné, pages de messages, positions de page, recherche locale et erreur. Les corps ne sont pas persistés dans le store natif de navigation.
