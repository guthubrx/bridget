# Recherche technique 025

**Date** : 2026-08-25

## Décision — Mesurer les arbres Git, pas le worktree

La documentation officielle de `git diff` distingue la comparaison entre deux
commits de la comparaison avec le worktree et documente les options qui
interdisent les aides de diff et conversions externes.

Source : https://git-scm.com/docs/git-diff

**Choix** : commits complets, référence complète, `--no-ext-diff`,
`--no-textconv`, sorties de chemins terminées par NUL et aucune opération de
checkout.

**Alternative écartée** : lire les fichiers du worktree après avoir vérifié le
SHA. La paire mesurée pourrait alors différer de la paire classée.

## Décision — Version et DDL dans une transaction vérifiable

SQLite réserve `user_version` à l’application et garantit la visibilité des
écritures au commit de transaction. Une version applicative ne prouve donc
pas, seule, que le DDL correspondant existe.

Sources :

- https://www.sqlite.org/pragma.html#pragma_user_version
- https://www.sqlite.org/lang_transaction.html

**Choix** : vérifier la forme de schéma v19, appliquer le DDL v20 et avancer la
version dans une même transaction. Une preuve de version sans forme attendue
est un refus sans écriture.

**Alternative écartée** : boucle qui inscrit toutes les versions intermédiaires
jusqu’à 20. Elle peut certifier un DDL absent.

## Décision — Journaliser la décision, pas les données qui l’ont déclenchée

Les recommandations de journalisation consultées demandent de valider les
données venant d’une autre zone de confiance et d’exclure ou masquer les
secrets, contenus sensibles et chemins lorsqu’ils ne sont pas nécessaires.

Source : https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html

**Choix** : identifiants de règle, chemins complets résolus, identifiants de
constat, SHA et énumérations. Aucun patch, contenu source, texte complet ou
trame externe.

**Alternative écartée** : conserver l’extrait qui a déclenché la règle. Cela
faciliterait le diagnostic mais recréerait la fuite de confidentialité que le
quatrième germe doit prévenir.
