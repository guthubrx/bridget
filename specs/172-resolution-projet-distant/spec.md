# 172 — Préparation et résolution du projet distant

Statut : In Progress. Correction demandée et élargie le 18 octobre 2026.

## Défaut observé
Le parent « trajets bus » indique Cartae et son dossier, sans project_id.
Bridget écarte toutes les cibles malgré une correspondance opus2d existante.
Un nouveau projet exige également une correspondance et un dépôt préparés
manuellement. L'utilisateur demande le parcours automatique, autonome de T3.

## Comportement attendu
- Hériter du dossier et des droits attestés du parent, T3 ou fournisseur natif.
- Résoudre une correspondance existante sans identifiant demandé au modèle.
- Pour un nouveau dépôt Git, transférer le commit demandé sur la liaison SSH
  déjà installée, préparer la destination et enregistrer la correspondance.
- Accepter un commit local non publié. Réutiliser le dépôt distant et envoyer
  uniquement les nouveaux objets lorsque le commit précédent est disponible.
- Conserver les fichiers de travail distants. Leur actualisation partagée reste
  contrôlée par le worker existant (propreté et absence de tâche concurrente).
- Reprendre après interruption sans créer de second projet.
- Passer ensuite par les contrôles de droits, le routage et l'admission existants.
- Conserver l'identifiant résolu et le résultat au rejeu de la même demande.
- Rendre un refus précis pour une destination occupée, une ambiguïté ou une
  liaison indisponible. Aucun repli silencieux hors contrainte humaine.

## Limites conservées
Le transfert suit le commit demandé, comme la préparation Git de T3 : les
modifications non commitées ne sont pas exportées. Aucun secret fournisseur,
profil personnel ou contenu des répertoires additionnels n'est transféré.
Les dossiers hors Git déjà déclarés gardent leur parcours existant ; la création
automatique décrite ici concerne les dépôts Git. Aucun service T3 n'est requis.

## Recettes
Bridget est déjà déclaré. Revue, situé dans
/Volumes/8TB2/50-repos-archives/11.Repositories/revue, ne l'est pas.
Le banc isolé couvre l'absence de dépôt publié et l'absence de T3. La recette
réelle doit prouver le lancement et le résultat Cartae depuis un parent Revue.
