# Quickstart de validation future: SPEC-065

Ce guide décrit les preuves attendues après implémentation. Il ne doit pas être
exécuté pendant le présent run de spécification.

## Préconditions

- SPEC-063 prouvée et SPEC-064 stabilisée.
- Release de test identifiée.
- Racine autorisée temporaire et deux dépôts fixtures.
- Politique `project-root-policy-v1` fixture fournie par chemin absolu au
  daemon, propriétaire et modes vérifiés.
- Daemon et base Maicie de test, jamais les données de production.

## Parcours 1 - Enregistrement et rejeu

1. Enregistrer la première fixture par la surface locale Maicie.
2. Conserver `command_id`, `project_id` et la sortie complète.
3. Redémarrer après chaque frontière de saga dans des runs séparés.
4. Rejouer la même commande et les mêmes octets.
5. Vérifier une seule identité et une seule liaison `host`.
6. Rejouer avec capability absente, mauvais rôle et UID pair divergent, puis
   vérifier un refus avant écriture durable.

Preuve attendue: `evidence/us1-register-replay.md` avec commandes, versions,
identifiants opaques et comptes de lignes des deux stores.

### Course canonique obligatoire

1. Préparer deux alias ou liens symboliques vers la même fixture.
2. Lancer deux enregistrements avec des `command_id` distincts en concurrence.
3. Vérifier une seule liaison et une seule identité `active`.
4. Vérifier que la commande perdante converge vers l'identité gagnante ou
   termine en `registration_conflict` durable et non référençable.

## Parcours 2 - Refus et non-destruction

1. Tenter chemin relatif, absent, trop large et hors préfixe.
2. Rejouer avec politique absente, vide, trop permissive ou modifiable par un
   autre compte et vérifier le fail-closed des seules mutations projet.
3. Empreinter la fixture avant et après chaque tentative.
4. Désactiver la liaison valide.
5. Vérifier que les empreintes et worktrees sont inchangés.

Preuve attendue: `evidence/us2-safety.md`.

## Parcours 3 - Rebind explicite

1. Déplacer manuellement une fixture de test.
2. Observer `path_missing` sans correction automatique.
3. Exécuter un rebind explicite vers la nouvelle racine.
4. Vérifier le même `project_id`, une génération incrémentée et une trace de
   l'ancien chemin.
5. Répéter avec un agent fixture actif et vérifier qu'il termine sur son
   ancienne génération sans arrêt implicite.

Preuve attendue: `evidence/us2-rebind.md`.

## Parcours 4 - Compatibilité et corrélation

1. Lancer un agent historique sans `project_id`.
2. Lancer un agent lié au projet enregistré.
3. Vérifier `unregistered` pour le premier et l'identité stable pour le second.
4. Tenter une délégation projet A vers une exécution projet B et observer le
   refus avant spawn.
5. Redémarrer Bridget, reprendre l'exécution par son curseur et reconstruire la
   projection Maicie; vérifier la même ProjectReference partout.
6. Produire un incident runtime délégué, redémarrer avant son accusé et vérifier
   que le rejeu SPEC-068 conserve la même ProjectReference sans doublon.

## Parcours 5 - Rapprochement review_project

1. Démarrer et migrer les stores avec `review_project` configuré: vérifier
   qu'aucune ProjectIdentity n'est créée.
2. Exécuter la prévisualisation locale et vérifier le projet/racine proposés.
3. Confirmer, puis rejouer la même commande et vérifier une seule identité et
   une seule trace d'audit.

## Parcours 6 - Audit durable

1. Exécuter register, rebind, disable puis un rapprochement confirmé sur des
   fixtures distinctes.
2. Rejouer chaque `command_id` avec les mêmes octets.
3. Vérifier un seul ProjectAuditEvent par mutation effective, la génération
   attendue et l'absence de chemin complet, contenu de dépôt ou secret.

Preuve attendue: `evidence/us3-correlation.md`.

## Commandes de qualité prévues

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
