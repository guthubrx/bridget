# Quickstart de validation - SPEC-073

## Préconditions

- Travailler dans le worktree de la feature.
- Ne jamais utiliser un agent de production pour le premier essai d'arrêt.
- Lancer un daemon et un relais UI isolés ou utiliser un agent jetable géré.

## Validation statique

```bash
git diff --check
node --test crates/bridget-daemon/assets/ui/app.js
cargo test -p bridget-daemon ui::
cargo test -p bridget-daemon --test ui_relay_test
```

## Parcours interface

1. Survoler une ligne : aucune fiche ne doit apparaître.
2. Sélectionner la ligne : le fil change, aucune fiche ne s'ouvre.
3. Activer les trois points : la fiche de l'agent s'ouvre sans changer le fil.
4. Vérifier le logo, le produit, l'éditeur, le mode et les faits attestés.
5. Fermer avec Échap : le focus revient aux trois points.
6. Rouvrir puis cliquer hors du panneau : il se ferme.
7. Ouvrir au clavier et parcourir tous les contrôles avec Tab.

## Parcours de sûreté

1. Ouvrir un agent TMUX ou externe : l'action est indisponible et expliquée.
2. Ouvrir un agent géré : choisir « Décommissionner ».
3. Annuler : vérifier qu'aucun ordre n'est journalisé.
4. Recommencer et confirmer sur un agent jetable.
5. Vérifier l'état d'attente sans double envoi.
6. Vérifier le verdict exact, puis le déplacement vers les agents arrêtés.
7. Ouvrir l'agent arrêté et confirmer que son historique reste visible.

## Contrôles de non-régression

- Le brouillon courant reste intact pendant toutes les ouvertures.
- Le compteur non lu ne change pas à l'ouverture de la fiche.
- Le dernier message et l'heure restent lisibles.
- Les quatre logos locaux répondent sans accès tiers.
- Les tests du mécanisme `StopOrder` restent verts.

## Déploiement ultérieur

Après validation et décision explicite de livraison :

```bash
cargo build --release -p bridget-daemon
systemctl --user restart bridget-daemon.service
systemctl --user restart bridget-ui.service
```

Le redémarrage ne fait pas partie de l'implémentation automatique de cette
feature.
