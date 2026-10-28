# Session198 — Reprise native après refus de démarrage

Incident : tâche bd3dabf6-426e-4ca6-8bdf-92d0e0e6a506, demande opus2d-cartae-suite-cameras-20261028-01. Start retourne ResourceLimit sans reçu de commande. Le rejeu ne reprend pas la tâche.

## Résultat requis
Conserver tâche, clé, enfant logique, mission, ronde, worktree, fichiers et choix de flotte. Rendre le domaine exact d’un nouveau refus de capacité ; un ancien refus sans preuve reste inconnu. Après refus Start éligible, un rejeu strict propriétaire peut reprendre sans nouveau décideur ni nouvelle tâche. Aucune reprise d’un Release ou Start d’issue inconnue sans preuve durable fermant l’ancienne commande.

## Garanties
Revalidation des droits actuels du parent sans élargissement, source/base, mapping, définition et contrôle pause/gel. Identités et historique conservés. Réponse perdue, concurrence, annulation, ancienne époque et ancienne commande tardive ne peuvent produire deux fournisseurs. Un reçu déjà existant est observé, jamais effacé. Une absence de fichier ne constitue pas seule une preuve d’arrêt. Ancien pair : refus explicite sans effet. Aucun restart ou changement de store réel pour installer le candidat.

## Limites de preuve actuelle
Maître/worker183 actifs. Le relevé ultérieur du store (5276439 octets comptés sur16777216 ;1117 entrées sur4096) ne prouve pas la capacité à l’instant du refus. File worker, quota store et pending maître sont des causes possibles non distinguées dans le retour historique.
