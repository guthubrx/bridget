# Journal d'implémentation — cycle de vie des demandes

**Spec** : 003-cycle-vie-demandes  
**Branche** : session-03-cycle-vie-demandes  
**Statut** : Implémenté

## Réalisation

- Demandes `--reply` persistées dans SQLite avec états `open`, `answered`, `cancelled` et `timed_out`.
- Annulation idempotente réservée à l'émetteur, avec notification `reply=no` au destinataire et arrêt des rappels.
- Réponses corrélées par `in_reply_to`; les états terminaux ne sont jamais rouverts.
- Demandes ouvertes restaurées lorsque les deux agents se reconnectent après redémarrage.
- Commandes `bridget cancel` et `bridget requests`, et `bridget reply` associe automatiquement la dernière demande reçue.
- Session réouverte : le wrapper publie l'OS (`macOS`, `Linux`, etc.), l'annuaire le conserve et `bridget who` l'affiche dans une colonne alignée ; `bridget agents --json` le restitue aussi.

## Self-review Article XIX/XX

- **Nécessité** : une demande devenue obsolète causait auparavant rappels et réponses forcées inutiles.
- **Simplicité** : une table SQLite et les modules existants remplacent une liste volatile; aucune dépendance ni service ajouté.
- **Hypothèse** : annuler est coopératif et n'interrompt pas un LLM déjà en exécution.
- **Vérifications** : transitions du store, annulation autorisée et interdite, absence de rappel après annulation, restauration d'une demande ouverte, suite complète du workspace.
- **Non vérifié** : essai manuel interactif après redémarrage du daemon de production; il nécessite un redémarrage contrôlé qui déconnecte les agents actifs.
- **Complexité** : opérations par identifiant indexées; seuls les états `open` sont rechargés.

## Tests exécutés

- `cargo test -p bridget-daemon` : succès, 10 tests.
- `cargo test --workspace` : succès, 46 tests.
- `cargo build --release -p bridget-daemon` : succès.
- `git diff --check` : succès.
- Complément OS : `cargo test --workspace` : succès, 46 tests ; `cargo build --release -p bridget-daemon` : succès ; `git diff --check` : succès.
