# Preuve US1 - environnement Docker de projet

Date: 2026-08-30
Branche: session-066-environnement-partage-projet
Base: main 74234641fe04293c007717b8f1e3879823f5d382

## Oracle exécuté

`cargo test -p bridget-daemon --test project_runtime_integration_test -q`

Résultat exact: 1 passed, 0 failed.

L'oracle construit d'abord `infra/project-runtime` par `docker build --iidfile`
et ne consomme ensuite que l'image ID locale obtenue, sans tag d'autorité.

## Faits prouvés

- image locale immuable, labels projet/génération/politique/contrat et
  attestation post-démarrage;
- rootfs en lecture seule, capabilities supprimées, no-new-privileges, réseau
  bridge sans port, limites CPU, mémoire et PID;
- UID/GID effectifs, HOME et les quatre racines XDG dans le state root projet;
- écriture effective de l'agent dans le state root et lecture d'une fixture
  `0600` par le même UID;
- socket Unix explicite `/run/bridget/runtime/bridget.sock` présent dans le
  conteneur et sans dérivation depuis HOME;
- montages projet/state root/socket inspectés, conteneur supprimé puis recréé;
- contenu du projet et du state root conservé après recréation;
- aucun conteneur de test avec le label `bridget.project_id=project-066` ne
  subsiste après l'oracle.

Aucun projet réel, daemon de production ou service utilisateur n'a été modifié.
