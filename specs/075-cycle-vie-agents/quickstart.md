# Quickstart de validation - SPEC-075

## Préconditions

- Worktree: `/home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents`
- Aucun daemon de production n'est redémarré par ce quickstart.
- Les essais processus utilisent des répertoires et sockets temporaires.

## Validation statique

```bash
cd /home/moi/bridget-referent/.worktrees/session-075-cycle-vie-agents
cargo fmt --check
git diff --check
```

## Tests ciblés

```bash
cargo test -p bridget-transport lifecycle
cargo test -p bridget-daemon desired_state
cargo test -p bridget-daemon fleet
cargo test -p bridget-daemon lifecycle
cargo test -p bridget-daemon ui
cargo test -p bridget-daemon --test ui_relay_test
node crates/bridget-daemon/assets/ui/app.js
```

## Preuve intégrée

1. Démarrer un daemon de test sur une socket temporaire.
2. Spawner un agent fixture persistant.
3. L'arrêter et attester zéro processus du groupe.
4. Vérifier `who`: même nom, `stopped`, géré.
5. Redémarrer le daemon de test: même résultat, aucune reprise.
6. Relancer: même nom, génération supérieure, un seul groupe.
7. Décommissionner: absence de `who`, état `decommissioned` caché dans
   `fleet.json` et nom réservé.
8. Redémarrer: absence maintenue.
9. Vérifier que le journal de session antérieur existe encore.

## Migration pré-déploiement

Avant le premier redémarrage avec le schéma 4, exécuter la commande d'adoption
sur l'instantané du daemon courant. Vérifier la liste des noms adoptés et
refusés. Aucun nom refusé n'est modifié automatiquement.
