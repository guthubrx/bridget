# Recette du noyau — installation isolée, bascule non automatique

**État actuel :** extraction et validations locales en cours. T007 isole le namespace ; T014–T019 exécutent les coutures daemon/CLI/MCP, T020 a exécuté Codex réel. Claude reste en attente d'une authentification valide ; GLM, SSH et la non-régression finale restent à exécuter. Voir implementation.md pour les preuves, pas les résultats d'anciens chantiers.

Répertoire de travail : `/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core`.

## Avant tout essai de binaire

Choisir une racine courte privée et utiliser le même binaire extrait partout :

```sh
cd /Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core
PATH=/Users/moi/.cargo/bin:$PATH cargo build --locked -p bridget-daemon
bridget_state=$(mktemp -d /tmp/bgcore.XXXXXX)
export BRIDGET_HOME="$bridget_state"
export BRIDGET_SOCKET="$bridget_state/bridget.sock"
/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/target/debug/bridget daemon
```

Le daemon reste au premier plan. Dans les autres terminaux, reprendre EXACTEMENT ces deux chemins d'environnement, pas un nouvel appel à mktemp. L'instance d'agent est créée par sa session, jamais fabriquée depuis le nom affiché. Pour les tests de fournisseur, les harnais préparent un profil privé ; aucune configuration globale MCP/skill n'est importée. Cette recette n'installe ni service launchd ni profil agent.

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

## Changer le nom affiché, sans interface

Depuis une session Bridget enregistrée (même namespace et instance vivante),
`bridget rename "Équipe B"` met à jour le nom projeté par who. L'adresse reste
l'UUID : les envois, réponses, scopes et retries ne changent pas. La commande
utilise la socket, le même chemin prévu pour le tunnel autorisé ; sa recette
distante reste à exécuter à T026. Elle ne consulte jamais une base locale pour
modifier celle du daemon.

Une identité absente, une autre instance, un nom déjà pris ou des caractères
de contrôle sont refusés. Répéter le même nom ne crée pas de nouvelle révision.
Le client répond par JSON (agent_id/display_name/revision), pas par une
instruction d'aller ouvrir des réglages graphiques. Aucun fichier d'identité
du fournisseur n'est réécrit.

## Gates de qualité

Avec Rust 1.92.0 disponible dans le PATH, après isolation des harnais :

```sh
PATH=/Users/moi/.cargo/bin:$PATH cargo fmt --all --check
PATH=/Users/moi/.cargo/bin:$PATH cargo clippy --workspace --all-targets -- -D warnings
PATH=/Users/moi/.cargo/bin:$PATH cargo test --workspace
```

Exécuter les crashs et tests longs sous watchdog global avec sorties conservées et barrières observables. Une suite verte sans les scénarios réels ignorés par défaut n'est pas une recette fournisseur/SSH verte. Reporter les résultats dans implementation.md ; aucune bascule de production automatique.
