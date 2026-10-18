# 172 — Résolution du projet distant

Statut : In Progress. Correction demandée le 18 octobre 2026.

Le fil « trajets bus » a demandé Cartae avec le cwd opus2D et le SHA
9ecd0abeaec1a05511effd10376e041e097df206. Le champ project_id facultatif était
absent. observe_routing_candidates écartait silencieusement toute cible distante.
La correspondance opus2d était pourtant configurée sur le maître.

Le maître doit résoudre le projet depuis la machine, l'hôte d'origine attesté
et le cwd déjà limité au projet parent. Une correspondance unique suffit.
Un identifiant explicite reste strict. Une absence, une ambiguïté ou un
décalage d'origine doit produire un refus nommé, sans lancement ni gain de droits.
Le choix résolu est figé dans le reçu et conservé au rejeu.

Hors périmètre : provisionner automatiquement les nouveaux projets,
copier des données métier, modifier les droits ou substituer une autre machine.
