# Quickstart - Validation de la ronde par projet

## Tests automatisés

Depuis la racine du worktree :

1. `cargo test -p bridget-transport spec_079_contrat_ronde_projet_est_ferme_versionne_et_rejouable --lib`
2. `cargo test -p bridget-daemon spec_079_politique_ronde_absente_idempotente_et_epinglee_au_rebind --lib -- --test-threads=1`
3. `cargo test -p bridget-daemon spec_081 --lib -- --test-threads=1`
4. `cargo test -p bridget-daemon --test ui_relay_test -- --test-threads=1`
5. `node --check crates/bridget-daemon/assets/ui/app.js`
6. `node crates/bridget-daemon/assets/ui/app.js`
7. `cargo fmt --check`
8. `cargo clippy --workspace --all-targets -- -D warnings`
9. `git diff --check`

## Validation de lecture

1. Démarrer un daemon et son relais UI avec au moins deux projets actifs.
2. Activer la ronde d'un seul projet par la commande existante.
3. Appeler `GET /v1/projects` avec le jeton du relais.
4. Vérifier que les deux projets portent une génération et un objet `round`.
5. Vérifier que seul le projet activé a `round.enabled=true`.
6. Vérifier qu'aucune donnée fournisseur ou secrète n'est ajoutée.

## Validation de mutation

1. Ouvrir le menu du projet par les trois points, le clic droit, puis `Maj + F10`.
2. Vérifier que « Ronde de vigilance » présente le même état dans les trois cas.
3. Activer la ronde et vérifier que le contrôle devient occupé sans changer immédiatement la ligne.
4. Après la réponse, vérifier `actif · ronde activée` et le détail « prochain cycle global, au plus 7 min ».
5. Désactiver puis vérifier que les agents et travaux restent inchangés.
6. Rejouer le même `command_id` et vérifier la même projection sans nouvelle révision.
7. Envoyer le même `command_id` avec un état différent et vérifier le refus sans changement.

## Validation des limites

1. Ouvrir le menu, reconnecter le projet par une autre session, puis demander la mutation avec l'ancienne génération : refus explicite.
2. Retirer le projet : l'action de ronde devient indisponible et explique que le projet est inactif.
3. Réactiver ou reconnecter le projet : la nouvelle génération apparaît non configurée et désactivée.
4. Arrêter le daemon pendant une mutation : la dernière valeur confirmée reste visible avec une erreur.
5. Retirer temporairement la capacité `ProjectRoundPolicyV1` d'une fixture : la liste refuse de fabriquer un état.

## Validation du dernier passage

1. Activer un projet et exécuter une occurrence du dispatcher global.
2. Rafraîchir la liste et vérifier occurrence, instant d'observation et état `deposited`, `refused` ou `indeterminate`.
3. Rejouer la même occurrence : aucune régression d'instant.
4. Tenter une occurrence plus ancienne : elle ne remplace pas la dernière occurrence.
