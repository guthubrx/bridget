# Quickstart opérateur - SPEC-085

## Préconditions

- Docker Engine disponible sur Linux amd64.
- Image Bridget runtime construite et identifiée par digest.
- Politiques racine, runtime et ressources validées.
- Aucun agent actif sur le projet à migrer.

## Activation serveur

1. Construire l'image avec la commande documentée dans `infra/project-runtime/production/`.
2. Vérifier le digest et le test de fumée des outils.
3. Installer les catalogues opérateur sans secret.
4. Prévisualiser le changement du service systemd.
5. Redémarrer Bridget et vérifier que la capacité Docker est attestée.
6. Conserver `host` comme défaut pendant le premier test.

## Validation projet

1. Choisir un projet Host sans agent actif.
2. Prévisualiser `activate_docker` avec la politique approuvée.
3. Confirmer et attendre l'état `ready` attesté.
4. Lancer un agent de test et vérifier checkout, Git, Rust, Node et Python.
5. Arrêter l'agent, puis tester stop/recreate.
6. Revenir explicitement à Host et vérifier que dépôt et worktrees sont inchangés.
7. Redémarrer le daemon et vérifier la réconciliation de l'état.

## Refus à vérifier

- politique absente
- digest d'image divergent
- agent actif
- génération obsolète
- Docker indisponible
- tentative d'argument ou de montage libre

