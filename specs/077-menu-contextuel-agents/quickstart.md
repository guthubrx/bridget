# Quickstart - SPEC-077 Menu contextuel des agents

## Tests automatisés

Depuis le worktree de la SPEC:

1. node --test crates/bridget-daemon/assets/ui/app.js
2. cargo test -p bridget-daemon ui --lib -- --test-threads=1
3. git diff --check

## Parcours manuel isolé

1. Ouvrir l'interface avec un jeton de test.
2. Cliquer sur les trois points d'un agent actif et vérifier le menu compact.
3. Fermer, puis faire un clic droit sur la ligne et vérifier les mêmes actions.
4. Fermer, focaliser la ligne et utiliser Maj+F10.
5. Parcourir les commandes avec les flèches, Début et Fin.
6. Épingler l'agent et vérifier son déplacement sans saut de scroll.
7. Marquer comme lu et vérifier la disparition du compteur sans disparition de message.
8. Masquer l'agent et le restaurer depuis Agents masqués.
9. Recharger la page et vérifier la restauration des préférences.
10. Vérifier la matrice actif, arrêté et non géré.
11. Ouvrir une confirmation puis l'annuler avec Échap.
12. Vérifier qu'aucune action locale n'a déclenché de requête réseau.

## Réversibilité

Supprimer la clé locale bridget.ui.agent-sidebar-preferences.v1 restaure l'ordre et la visibilité par défaut. Cela ne modifie ni agents, ni messages, ni secrets.
