# Analyse de cohérence - SPEC-070

**Primitive indisponible** : les skills `speckit-analyze` ne sont pas
installées dans le worktree et `specify` est absent du serveur. Analyse manuelle
effectuée le 30 août 2026.

## Vérifications croisées

| Exigence | Plan | Tâches | Verdict |
|---|---|---|---|
| Pas d'accusé agent fictif | garde-fou `prompt_dispatched` | T005, T007 | couvert |
| Activité agent réelle | projection de tour actif | T005, T006 | couvert |
| Délai de 60 s dissocié | daemon puis transport | T001 à T004 | couvert |
| Erreur corrélée | projection + identité de message | T007 | couvert |
| Notification opt-in et clic | Notification API + cible interne | T008, T009 | couvert |
| Pas de backend supplémentaire | ADR-016, audit de réutilisation | toutes | couvert |

## Findings

### F01 - Statut « traitement démarré » non prouvé - Corrigé au plan

Le code actuel bascule ce statut dès `prompt_dispatched`. Le plan et T007 le
remplacent par une attente neutre tant qu'aucun événement fournisseur n'a été
reçu.

### F02 - Echéance de réponse et d'exécution confondues - Corrigé au plan

Le défaut de 60 secondes est posé avant la livraison. T001 à T004 le séparent
et couvrent l'arrêt propre au seul vrai plafond fournisseur.

### F03 - Clic de notification non garanti - Accepté

La norme rend le clic meilleur effort. La cible interne existante reste le
repli obligatoire, ce qui évite de promettre une navigation que la plateforme
ne peut garantir.

### F04 - Risque d'exposer une commande - Corrigé au plan

L'activité compacte utilise une catégorie sûre dérivée du type d'acte. Les détails
restent sous contrôle du rendu existant.

## Conclusion

Aucun finding CRITICAL ni besoin d'arbitrage. Les corrections sont ciblées,
sans nouveau service ni dépendance. La première tâche exécutable est T001.
