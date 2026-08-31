# Quickstart de validation future: SPEC-066

Ce guide décrit les preuves futures. Il ne lance aucun conteneur dans le run de
spécification.

## Préconditions

- SPEC-065 et SPEC-075 intégrées sur une même base fixture.
- Docker Engine de test observé.
- Image fixture épinglée par digest, sans credential.
- Politique runtime fixture fournie par chemin absolu, avec policy_id/version,
  UID/GID numériques et state root temporaire.
- Deux projets temporaires, dont un avec deux worktrees.

## Parcours 1 - Prepare, inspect, recreate

1. Préparer le projet A.
2. Inspecter la politique complète et comparer au contrat.
3. Supprimer manuellement le conteneur fixture.
4. Observer la dérive, puis recréer explicitement.
5. Comparer empreintes dépôt/state avant et après.
6. Vérifier HOME/XDG sous `/var/lib/bridget-project`, le socket explicite sous
   `/run/bridget/runtime/bridget.sock` et la lecture d'une fixture `0600` par
   l'UID configuré.

## Parcours 2 - Deux agents partagés

1. Lancer deux agents fixtures dans deux worktrees du projet A.
2. Vérifier le même container id et deux générations distinctes.
3. Envoyer un message via Bridget dans les deux sens.
4. Interrompre un tour fixture et vérifier l'issue corrélée.
5. Émettre successivement un incident délégué `warning` puis `failed`.
6. Redémarrer le consommateur avant acquittement, puis vérifier l'ordre, le
   rejeu par curseur, l'acquittement idempotent et le même `ProjectReference`
   avant et après redémarrage.

## Parcours 3 - Isolation entre projets

1. Préparer le projet B.
2. Inspecter les montages depuis A et B.
3. Rechercher explicitement racines et state roots croisés.
4. Vérifier l'absence de home global, `/` et Docker socket.
5. Faire joindre l'ingress B par un wrapper A et vérifier un refus avant toute
   inscription.
6. Présenter une génération ou un environment_epoch forgé sur le bon socket et
   vérifier le même refus sans effet.

## Parcours 4 - Panne et rollback

1. Rendre Docker indisponible dans la fixture.
2. Demander un agent et vérifier l'absence de fallback host.
3. Restaurer Docker, arrêter les agents, choisir host.
4. Lancer un agent host et vérifier la même identité projet.
5. Changer policy_version puis backend et vérifier
   `runtime_policy_changed`, epoch incrémenté et absence d'admission ancienne.

## Parcours 5 - Course spawn et lifecycle

1. Suspendre une fixture entre admission et docker exec.
2. Demander stop, remove ou switch-backend dans un premier run.
3. Inverser l'ordre dans un second run.
4. Vérifier que le spawn gagne et le lifecycle refuse, ou que le lifecycle
   gagne et aucun exec ne démarre; aucun troisième état n'est admis.

## Parcours 6 - Autorité d'image

1. Construire la fixture avec `docker build --iidfile` et utiliser cet
   identifiant local immuable.
2. Refuser le même build référencé uniquement par tag.
3. Tester séparément une référence `repository@sha256` si une fixture de
   registre locale est disponible; aucun accès Internet n'est requis.

## Validation qualité prévue

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Les preuves seront déposées sous
`specs/066-environnement-partage-projet/evidence/`.
