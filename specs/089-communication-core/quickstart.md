# Recette future — ne pas confondre préparation et installation

**État actuel :** dépôt et conception seulement. Le code copié utilise encore les valeurs par défaut du produit historique ; ne pas lancer son daemon, ses wrappers ou ses scripts de déploiement.

Répertoire de travail : `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core`.

## Avant tout essai de binaire

T007 doit rendre explicites et vérifiables le home, la socket courte et l'identité d'instance du noyau extrait. Le test refuse de démarrer s'ils pointent vers l'installation historique. Les commandes exactes d'essai seront inscrites ici après cette couture, pas inventées avant son implémentation.

Ne pas modifier HOME global, les skills globales, launchd, le registre fournisseur ni le tunnel de la flotte. Utiliser des répertoires temporaires distincts par scénario et ne nettoyer que les processus/chemins créés par celui-ci.

Pour les outils SpecKit qui imposent un préfixe numérique malgré le nom de branche `session-089-communication-core`, sélectionner explicitement `SPECIFY_FEATURE=089-communication-core`. Les scripts officiels ne sont pas modifiés pour contourner leur convention ; feature.json pointe sur le même dossier.

## Scénario local à exécuter (SC-08901/02/05/09/10)

1. Démarrer un daemon isolé puis deux sessions réelles de fournisseurs distincts, permissions et modèle déclarés.
2. Lire who, demander une réponse à la seconde session depuis la première, conserver tous les paramètres de retry.
3. Répondre par MCP avec in_reply_to ; lire le même ledger via CLI ; prouver answered et l'absence de rappel supplémentaire.
4. Rejouer l'envoi à l'identique : même issue, un seul prompt et une seule ligne de ledger ; modifier un champ : refus sans mutation.
5. Attacher le journal, comparer les bytes et séquences ; interrompre le flux puis reprendre sans fraîcheur inventée.

## Scénario SSH à exécuter (SC-08904)

Deux machines réelles, sockets de test distinctes des sockets historiques. Préflight de possession et de disponibilité avant d'établir le transfert. Lire annuaire et ledger depuis le distant, livrer une demande et sa réponse, couper uniquement le tunnel de test, puis reconnecter. Vérifier mêmes identifiants, aucun prompt dupliqué et aucun redémarrage fournisseur indu. Consigner commande SSH exacte, durée, clock skew des mesures et nettoyage. Un gate distant non exécuté reste non validé.

## Gates de qualité

Avec Rust 1.92.0 disponible dans le PATH, après isolation des harnais :

```sh
PATH=/Users/moi/.cargo/bin:$PATH cargo fmt --all --check
PATH=/Users/moi/.cargo/bin:$PATH cargo clippy --workspace --all-targets -- -D warnings
PATH=/Users/moi/.cargo/bin:$PATH cargo test --workspace
```

Exécuter les crashs et tests longs sous watchdog global avec sorties conservées et barrières observables. Une suite verte sans les scénarios réels ignorés par défaut n'est pas une recette fournisseur/SSH verte. Reporter les résultats dans implementation.md ; aucune bascule de production automatique.
