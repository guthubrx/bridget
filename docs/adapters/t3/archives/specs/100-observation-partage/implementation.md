# Archives techniques de l’adaptateur T3

Document d’origine : [specs/100-observation-partage/implementation.md](../../../../../../specs/100-observation-partage/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Vérifié : données Unicode, 50/200 entrées, lacune, délai10s serveur muet,
parité des façades, refus owner/source non autorisés, TTL/once/filtres,
collision et quatre témoins négatifs, saturation, reply et annuaire pendant
writer verrouillé, notification exclue des observations suivantes.
Non vérifié : fournisseurs réels, T3 natif universel, performances production,
fédération des faits, reprise durable après redémarrage et suite globale.
Ce ne sont pas des promesses cachées : couverture et pertes sont documentées.
Code évité : second lecteur JSONL, watcher récursif, moteur de règles libre,
BDD d'abonnements, boucle LLM, mandat et verrou de fichiers.

<a id="extrait-2"></a>

## Extrait 2

Prochaine action recommandée : revue du diff puis recette avec fournisseurs
dans un namespace isolé avant toute installation. La couverture native/T3
universelle et l'adaptation des anciens harnais sont des travaux séparés.

<a id="extrait-3"></a>

## Extrait 3

Limites : la suite workspace intégrale n'a pas été lancée, car ses harnais
SIGKILL/groupes restent interdits. La recette1345/0/49 de la099 est celle de
l'autre intervenant, pas une preuve de cette intégration. La chaîne avec un
fournisseur réel et un retour de notification est testée par tronçons, pas
par un scénario fournisseur bout-en-bout. Aucun comportement universel T3/natif
ou shell n'est annoncé. Sauvegarde binaire099, base SQLite cohérente vérifiée
quick_check=ok, plists inchangés et état du pont avant adoption :
/Users/moi/.cache/bridget-adoptions/100-20260916.iMZZbR

<a id="extrait-4"></a>

## Extrait 4

Adoption : SIGTERM individuel après vérification PID/commande pour le daemon
94616 puis le pont94724. Les plists KeepAlive inchangés ont relancé les seuls
services Bridget, daemon82429 et pont83263 ; l'application T3 et les processus
fournisseurs n'ont pas été arrêtés. Retour des neuf fils T3 (quatre busy/cinq
connected au contrôle), trois anciennes présences gérées toujours stopped.
`bridget status` confirme build-id c680c6ce36c5 sans avertissement de version.
Lecture réelle du journal bdget : une entrée, seq220, complete=true ; contenu
non recopié dans ce reçu. Nouveau MCP : 18 outils au total, dont14 Bridget
et4 Maicie déjà existants ; bridget_journal/bridget_events présents.
