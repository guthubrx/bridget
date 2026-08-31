# Quickstart de validation future: SPEC-067

Ce guide utilise uniquement des secrets synthétiques et des extensions fixtures.
Aucun credential réel ne doit entrer dans les preuves.

## Préconditions

- SPEC-066 prouvée sur deux projets fixtures.
- Racines privées temporaires pour extensions et secrets.
- Catalogue hôte fixture fourni par chemin absolu avec source_revision,
  UID/GID et listes de projets autorisés.
- Trois providers fixtures ou modes sans dépense externe.
- Valeurs sentinelles générées pour détecter les fuites.

## Parcours 1 - Proposition et approbation

1. Créer un profil projet fixture avec trois AgentProfiles.
2. Ajouter deux ExtensionRefs et trois SecretRefs synthétiques.
3. Afficher la vue locale et vérifier chaque métadonnée.
4. Rechercher les valeurs sentinelles dans la vue et les stores: zéro match.
5. Approuver localement et vérifier le digest exact.
6. Vérifier runtime_policy_version et chaque source_revision dans la vue.

## Parcours 2 - Extensions

1. Préparer l'environnement avec les extensions épinglées.
2. Vérifier les montages read-only depuis chaque provider fixture.
3. Tenter une écriture et un accès à un catalogue non approuvé.
4. Modifier un octet de fixture et vérifier `recreate_required` avant spawn.

## Parcours 3 - Secrets et fuites

1. Utiliser file, directory et process-env synthétiques.
2. Lancer l'agent fixture et vérifier le comportement attendu.
3. Scanner Docker inspect, ps, stores, logs, métriques, UI et artefacts.
4. Vérifier zéro occurrence exacte des sentinelles.
5. Vérifier l'avertissement de visibilité intra-projet.
6. Faire émettre une sentinelle après démarrage et vérifier que JournalWriter
   ne reçoit jamais les octets bruts.
7. Répéter avec la sentinelle coupée au milieu, au retour ligne et sur chaque
   canal, puis vérifier que le mutant sans état de flux échoue.
8. Modifier hors rotation un fichier secret puis un membre du répertoire, et
   vérifier la divergence de SecretSourceStamp avant toute lecture ou montage.
9. Faire produire un incident runtime délégué contenant la sentinelle dans la
   donnée fournisseur, puis vérifier zéro octet brut dans la trame SPEC-068, le
   store, la notification, le rejeu et l'acquittement; seuls le code fermé, la
   référence pseudonymisée et le `ProjectReference` sont autorisés.

## Parcours 4 - Rotation et rollback

1. Proposer une nouvelle génération d'un secret.
2. Vérifier le refus tant qu'un agent est actif.
3. Arrêter, approuver, recréer et lancer.
4. Prober l'ancienne génération: absente.
5. Désactiver le profil, recréer sans montages puis revenir host.
6. Ré-approuver un profil, rebind le projet puis tenter un spawn.
7. Vérifier `stale` et un refus avant lecture ou montage; ré-approuver seulement
   sur la nouvelle binding_generation et le nouveau policy_digest.
8. Vérifier que tout agent déjà actif termine sur son ancienne génération sans
   arrêt implicite tandis que toute nouvelle admission reste refusée.
9. Rejouer après switch backend et après changement de runtime_policy_version;
   vérifier `stale` avant résolution de source.

## Parcours 5 - Contrat fournisseurs

Exécuter les mêmes oracles sur Codex app-server, Claude stream-json et Cursor
ACP. Aucun appel payant n'est requis pour la validation de structure.

## Validation qualité prévue

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
