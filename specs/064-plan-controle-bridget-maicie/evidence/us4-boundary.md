# Preuve US4 - frontière mission et exécution

Date de vérification : 2026-08-29T14:41:02Z

## Résultat

PASS en environnement isolé. Maicie conserve une copie opaque et cursée des
faits Bridget. Bridget lit exclusivement un instantané JSON public atomique et
compile sans dépendance productive vers Maicie.

## Faits prouvés

- `ExecutionReference` porte délégation, soumission, exécution, instance et
  identité fournisseur sans commande de contrôle.
- `persist_execution_projection_from_signal` transforme un signal Bridget déjà
  corrélé en projection Maicie avec fraîcheur et curseur. Le scénario de test
  écrit un Gap au curseur 21 et vérifie que l'objectif ne change pas.
- Les états runtime `completed`, `unreachable` et `stale` sont enregistrés
  comme faits et ne ferment ni ne rouvrent objectif ou délégation.
- Maicie publie un contrat JSON versionné par écriture temporaire, sync puis
  renommage. Le lecteur Bridget accepte l'absence comme état dégradé explicite,
  sans l'interpréter comme absence attestée de mission.
- La dépendance `maicie` est limitée aux dépendances de test du daemon. La
  vérification de l'arbre de dépendances normales ne contient aucun package
  Maicie.

## Commandes et verdicts

```text
cargo check -p maicie                                      PASS
cargo check -p bridget-daemon                              PASS
cargo test -p maicie --test contract execution_projection  PASS, 7 tests
cargo test -p maicie ui_projection --lib                   PASS, 4 tests
cargo test -p bridget-daemon --test mission_boundary_test  PASS, 1 test
cargo test -p bridget-daemon wrapper::prompt_tests --lib   PASS, 19 tests
cargo tree -p bridget-daemon --edges normal --prefix none  PASS, Maicie absent
```

## Limites vérifiées et non vérifiées

- Aucune activation, aucun déploiement ni redémarrage de production n'a été
  effectué.
- La preuve d'indisponibilité Maicie est contrôlée: projection publique absente
  et compilation Bridget sans dépendance Maicie. Elle ne simule pas une panne
  d'un daemon de production.
- La projection stocke des faits runtime; elle ne constitue pas une décision
  d'acceptation fournisseur, de consommation ou de clôture métier.
