# Contre-revue du plan 175
Date : 2026-10-23. Verdict reçu : APPROVE_WITH_CHANGES.
Revue indépendante Claude, Task 21495cee-734f-4dd9-99b1-23b81f1f1024, résultat intégral lu puis ACK et clôture confirmés. Lecture seule, aucune exécution.

| Point | Décision |
|---|---|
| 1. Commande worktrees avant purge/prune | Accepté ; sortie dédiée et test d'absence de mutation exigés. |
| 2. Erreur Git/lsof jamais zéro | Accepté ; inconnu explicite, y compris absence et timeout. |
| 3. Un seul évaluateur de motifs | Accepté ; inventaire et nettoyeur partagent l'évaluation. locked/prunable conservés. |
| 4. base-ref transmis | Accepté ; aucune base main implicite dans l'inventaire après choix explicite. |
| 5. Activité prioritaire | Accepté et précisé : tâche non terminale, continuation possible ou résultat non remis avant rapport. Arrêt non confirmé garde un motif distinct. holds_slot n'est pas requis car cleanup_done est contrôlé séparément. |
| 6. Suppression du rappel final | Non retenu : le rappel est une consigne constante qui invite à relire le statut ; aucun motif dynamique ni autorisation de retrait n'est figé dans le résultat. Le budget UTF-8 et le format d'une seule ligne restent testés. |

La revue porte sur le plan et le code antérieur : ses points 1 à 4 sont des risques de mise en œuvre, pas des incidents attribuables à une commande déjà publiée.
Recommandations retenues : status avec --no-optional-locks ; comparaison par composants et dossier compté affiché ; seuil Python documenté ; erreurs explicites ; capacité Git distinguée des réservations du maître.
Le terme machine candidate est conservé pour compatibilité interne et accompagné d'une action d'examen, jamais d'un booléen removable.
Le modèle décrit les données ; le contrat porte les exemples et règles publiques. Aucun second service, store ou nettoyeur n'est introduit.

## Contre-revue du code final
Task06d96a63-a077-444e-a445-2e1f28a4ded1, Claude automatique, lecture seule. Verdict APPROVE_WITH_CHANGES, sans défaut bloquant de protection. Résultat lu intégralement, ACK et clôture. D1/D3/D4 corrigés et tests ciblés réussis ; D2 écarté par lecture du contrat Rust de comptage brut sous cwd canonique. Aucune nouvelle ronde nécessaire pour ces retouches locales ; contrôle final ROOT distinct et déclaré. Les suggestions de supprimer les notes sont écartées : capacité non garantie et absence de droit de retrait sont deux limites différentes.
