# Quickstart - SPEC-079

## Validation automatisée

Depuis:

`/home/moi/bridget-referent/.worktrees/session-079-continuite-travaux-redemarrage`

1. `cargo test -p bridget-transport`
2. `cargo test -p bridget-daemon execution_store --lib -- --test-threads=1`
3. `cargo test -p bridget-daemon idempotency --lib -- --test-threads=1`
4. `cargo test -p bridget-daemon daemon --lib -- --test-threads=1`
5. `cargo test -p bridget-daemon wrapper --lib -- --test-threads=1`
6. `bash scripts/test-bridget-ronde-dispatch.sh`
7. `cargo fmt --check`
8. `cargo clippy --workspace --all-targets -- -D warnings`
9. `git diff --check`

## Preuve de continuité isolée

1. Démarrer daemon et wrapper fixtures.
2. Envoyer un message UI `trigger_turn`.
3. Attendre l'accusé de remise et l'état running.
4. Arrêter le daemon fixture et le provider fixture.
5. Redémarrer le daemon et réenregistrer le wrapper avec le même agent géré.
6. Vérifier une seule continuation reconstructed et un seul nouveau delivery_id.
7. Vérifier le corps exact et la même référence projet.

## Preuve de ronde isolée

1. Créer deux ProjectBinding fixtures.
2. Activer la ronde de A et laisser B sans politique.
3. Exécuter deux fois la même occurrence.
4. Vérifier une seule soumission pour A et aucune pour B.
5. Désactiver A et avancer d'une occurrence.
6. Vérifier zéro nouvelle soumission et aucune modification des exécutions existantes.
## Pilotage opérateur

Les décisions sont locales au daemon et toujours épinglées à la génération:

1. `bridget project-round list`
2. `bridget project-round status --project <ID>`
3. `bridget project-round enable --project <ID> --binding-generation <N>`
4. `bridget project-round disable --project <ID> --binding-generation <N>`
5. `bridget project-round dispatch` pour une occurrence manuelle globale.

Une désactivation ne stoppe aucun agent et ne modifie aucune exécution. Un
rattachement neuf revient à `disabled` jusqu'à une nouvelle décision explicite.

## Installation future du scheduler

La skill ne modifie aucune unité de production. Lors de la livraison explicite:

1. construire et installer le binaire Bridget contenant le contrat v1;
2. installer le dispatcher depuis
   `/home/moi/bridget-referent/bridget/scripts/bridget-ronde-dispatch.py`;
3. faire pointer l'unique timer existant vers ce dispatcher avec le chemin
   absolu du binaire Bridget;
4. vérifier `project-round list` avant d'activer la première politique;
5. ne créer aucun timer ni daemon par projet.

Le binaire et le dispatcher doivent appartenir à la même livraison. En cas de
retour arrière, désactiver les politiques puis restaurer l'ancienne commande du
timer global.

