# Recette140 — Isolée et sans déploiement

## Racines de travail

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/quickstart.md#extrait-1).

## Vérifications ciblées

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/140-bulles-compactes/quickstart.md#extrait-2).

## Recette observable

Les cinq familles sont à droite. Le logo est local. Une seule ligne apparaît au départ. Cliquer puis utiliser Entrée/Espace : le corps s'ouvre, le focus et le chevron restent cohérents. Ouvrir « Détails techniques » : le texte complet est visible. Copier : le texte original demeure identique. Les pièces jointes et actions restent accessibles sur leurs chemins existants.

Tester thèmes clair/sombre et largeur320. Tester un long nom, un long titre et un long corps : pas de débordement horizontal de la bulle. Passer d'un fil/message à un autre et recycler la liste : un autre message ne récupère pas l'état ouvert précédent. Vérifier l'ancrage du défilement.

Une sollicitation ancienne affiche « Fil partagé ». Une nouvelle sollicitation remise à un membre affiche le vrai titre normalisé. Un message ordinaire, une citation ou une enveloppe incomplète garde le rendu ordinaire. Aucune ouverture n'émet d'ACK ni de relance.

## Limites de la recette

Un build et un aperçu ne prouvent pas une installation. Aucun commit ou redémarrage n'est automatique. Une défaillance de l'aperçu est consignée avec son erreur ; elle n'est pas transformée en preuve visuelle. Les procédures de livraison et de reprise des sessions restent distinctes.
