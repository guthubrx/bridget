# Recette140 — Isolée et sans déploiement

## Racines de travail

Artefacts et code Rust : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/140-bulles-compactes.
Frontend : /Users/moi/11.Repositories/t3code-local/.worktrees/140-bridget-compact.

## Vérifications ciblées

1. Exécuter les tests de logique MessagesTimeline avec fixtures des cinq familles et faux positifs. Ajouter les cas titre absent, hostile et JSON invalide. Constater RED avant code, puis GREEN.
2. Exécuter les tests Rust existants étendus pour le champ facultatif et la remise de titre. Contrôler membre/non-membre, normalisation, vieux lecteur et canon/journal inchangés. Employer un BRIDGET_HOME et une socket isolés.
3. Exécuter format, lint et TypeScript sur les fichiers web touchés, puis build web ciblé. Consigner les commandes exactes et codes retour dans implementation.md lors de leur exécution.
4. Dans l'aperçu isolé, rendre le vrai MessagesTimeline avec données synthétiques. Ne pas relier cet aperçu à /Users/moi/.t3/userdata ou au daemon actif.

## Recette observable

Les cinq familles sont à droite. Le logo est local. Une seule ligne apparaît au départ. Cliquer puis utiliser Entrée/Espace : le corps s'ouvre, le focus et le chevron restent cohérents. Ouvrir « Détails techniques » : le texte complet est visible. Copier : le texte original demeure identique. Les pièces jointes et actions restent accessibles sur leurs chemins existants.

Tester thèmes clair/sombre et largeur320. Tester un long nom, un long titre et un long corps : pas de débordement horizontal de la bulle. Passer d'un fil/message à un autre et recycler la liste : un autre message ne récupère pas l'état ouvert précédent. Vérifier l'ancrage du défilement.

Une sollicitation ancienne affiche « Fil partagé ». Une nouvelle sollicitation remise à un membre affiche le vrai titre normalisé. Un message ordinaire, une citation ou une enveloppe incomplète garde le rendu ordinaire. Aucune ouverture n'émet d'ACK ni de relance.

## Limites de la recette

Un build et un aperçu ne prouvent pas une installation. Aucun commit ou redémarrage n'est automatique. Une défaillance de l'aperçu est consignée avec son erreur ; elle n'est pas transformée en preuve visuelle. Les procédures de livraison et de reprise des sessions restent distinctes.
