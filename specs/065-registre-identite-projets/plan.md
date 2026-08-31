# Plan d'implémentation: Registre et identité des projets

**Branche**: `session-065-programme-environnements-projet`
**Spec**: `specs/065-registre-identite-projets/spec.md`
**Date**: 2026-08-29
**Statut du plan**: proposé, non implémenté

## Résumé

La première tranche ajoute une identité projet durable sans changer le mode
d'exécution. Maicie conserve l'autorité métier et pilote une saga idempotente.
Bridget conserve l'autorité technique sur le chemin canonique et le backend,
qui reste fixé à `host`. Les contrats runtime reçoivent un `project_id`
optionnel afin que l'existant continue sans migration globale.

Cette tranche est volontairement utilisable seule. Elle améliore déjà le
classement, le diagnostic et la corrélation, même si les specs 066 et 067 ne
sont jamais implémentées.

## Contexte technique

| Élément | Choix |
|---|---|
| Langage | Rust, workspace existant |
| Persistance Maicie | SQLite privée dans `plugins/maicie/src/store.rs` |
| Persistance Bridget | SQLite existante dans `crates/bridget-daemon/src/store.rs` |
| Contrat public | protocole versionné dans `crates/bridget-transport/src/protocol.rs` |
| Entrée utilisateur | commande locale Maicie, puis outbox vers Bridget |
| Transport de mutation | requête/résultat dédiés Maicie vers Bridget, capability `project_registry_v1` |
| Racines admises | politique hôte Bridget v1, chemin absolu explicite, fail-closed |
| Backend | `host` uniquement |
| Compatibilité | `project_id: Option`, anciennes commandes inchangées |
| Dépendances externes | aucune |

## Architecture et autorités

```text
Commande locale d'enregistrement
             |
             v
Maicie: ProjectIdentity + ProjectRegistrationCommand
             |
       outbox rejouable
             |
             v
Bridget: ProjectBinding(project_id, canonical_root, backend=host)
             |
       ProjectBindingResult
             |
             v
Maicie: état de liaison projeté, jamais vérité runtime
```

| Fait | Autorité | Projection admise ailleurs |
|---|---|---|
| Identité, nom métier, statut métier | Maicie | référence opaque dans Bridget |
| Racine canonique, backend, santé liaison | Bridget | état frais ou indisponible dans Maicie |
| Domaine historique | Bridget wrapper | affichage seulement |
| Objectifs et délégations | Maicie | aucune mutation depuis Bridget |
| Génération et exécution | Bridget | référence dans Maicie après SPEC-064 |

## Réutilisation de l'existant

- Étendre l'outbox et les commandes idempotentes Maicie dans
  `plugins/maicie/src/store.rs` et `plugins/maicie/src/app.rs`.
- Étendre le protocole négocié dans
  `crates/bridget-transport/src/protocol.rs` avec des variantes dédiées au
  contrôle Maicie vers Bridget; ne pas réutiliser `ServiceRequest`, dont la
  direction existante est Bridget vers Maicie.
- Étendre la persistance Bridget dans `crates/bridget-daemon/src/store.rs`.
- Réutiliser la validation de chemin absolu et normalisé de
  `plugins/maicie/src/config.rs`, puis canonicaliser côté Bridget qui possède
  l'hôte réel.
- Réutiliser `SpawnOrder.cwd`, la génération figée et les refus structurés de
  `crates/bridget-daemon/src/cli.rs`, `fleet.rs` et `lifecycle.rs`.
- Conserver `domain` comme métadonnée issue de `wrapper.rs`.
- Rapprocher `review_project` par migration explicite, sans l'utiliser comme
  registre général.
- Étendre les surfaces durables réelles de SPEC-064, notamment
  `execution_store.rs`, `desired_state.rs`, les ordres et leases de flotte,
  ainsi que les références et projections d'exécution Maicie.
- Étendre également les liens et incidents runtime délégués livrés par
  SPEC-068 dans `idempotency.rs`, `fleet.rs`, `protocol.rs` et `daemon.rs`.

## Découpage technique

### Lot 1 - Contrats et modèle de domaine

- Ajouter les contrats `ProjectRegistration`, `ProjectBinding`, la politique
  fermée des racines et leurs issues.
- Négocier `project_registry_v1` sur une connexion locale authentifiée par rôle
  et UID, avec une réponse corrélée par `command_id`.
- Ajouter `ProjectIdentity` au domaine Maicie et `ProjectBinding` au domaine
  technique Bridget.
- Définir les états et transitions de `data-model.md`.

Gate: les tests de contrat refusent champs inconnus, identités divergentes,
chemins relatifs et rejeux divergents.

### Lot 2 - Persistance et saga Maicie

- Persister une identité `pending_binding`, la commande idempotente et l'outbox
  dans la transaction Maicie; interdire toute référence métier avant liaison.
- Reprendre la première commande non terminale au prochain appel CLI.
- Ne jamais déduire une réussite Bridget depuis la seule présence d'une outbox.
- Promouvoir uniquement l'identité gagnante après issue Bridget; faire
  converger ou terminaliser explicitement toute intention perdante.

Gate: crashes injectés avant et après commit conservent une identité unique.

### Lot 3 - Liaison Bridget

- Charger une fois au démarrage la politique de racines depuis le chemin
  absolu fourni au daemon, vérifier propriétaire et permissions, puis valider
  la racine sur l'hôte réel: existence, canonicalisation, type, préfixe
  autorisé et unicité. L'absence ou l'invalidité de la politique ferme les
  mutations projet, pas la route host historique.
- Persister la liaison et l'historique de rebind dans le store existant.
- Persister ProjectAuditEvent dans la même transaction que register, rebind,
  disable ou rapprochement confirmé; un rejeu exact conserve exactement un
  événement pour la mutation effective.
- Rendre une issue idempotente à Maicie, y compris l'identité et la génération
  déjà liées lors d'une collision canonique.

Gate: une racine valide donne une liaison `host`; toutes les formes interdites
sont refusées avant effet.

### Lot 4 - Administration et projections

- Ajouter les actions locales list, status, rebind et disable.
- Ajouter `review-project reconcile --dry-run`, puis une confirmation locale
  idempotente; aucune réconciliation automatique au démarrage ou en migration.
- Projeter `project_id`, `domain` et fraîcheur comme faits distincts.
- Conserver les usages historiques en `unregistered`.

Gate: les trois scénarios indépendants passent, y compris après redémarrage.

## Stratégie de persistance

Maicie ajoute des tables versionnées pour les identités projet et commandes
d'enregistrement. Bridget ajoute des tables versionnées pour les liaisons et
leur historique. Les deux migrations sont additives. Aucun accès croisé aux
fichiers SQLite n'est admis.

La clé d'idempotence est le `command_id` utilisateur. La racine canonique est
également unique côté Bridget. Deux commandes distinctes peuvent préparer deux
identités non actives, mais la contrainte atomique Bridget élit une seule
liaison. Son issue rend `existing_project_id` et `binding_generation`; Maicie
active la gagnante et conserve la perdante en conflit durable ou rattache la
commande à la gagnante. Une enveloppe différente pour la même clé rend
`EnvelopeMismatch`.

## Compatibilité et migration

- `project_id` reste optionnel dans `SpawnOrder`, les générations, les
  soumissions et les projections.
- La même ProjectReference est persistée dans `SpawnOrder`, `SpawnLease`,
  `DesiredEquipier`, `ExecutionSnapshot`, `ExecutionReference`,
  `ExecutionProjection`, les ordres Maicie persistés et les curseurs de reprise.
  Elle est également persistée dans `AgentLinkRecord`, `AgentLinkEvent`,
  `DelegatedRuntimeEventRecord` et `DelegatedRuntimeEventFrame` livrés par
  SPEC-068. Une migration additive représente l'historique sans projet par
  `None`.
- Une commande historique sans projet suit exactement la route actuelle.
- `review_project` est rapproché seulement si l'opérateur lance une migration
  explicite; aucune identité n'est créée au démarrage.
- Une liaison désactivée reste lisible et n'est pas supprimée physiquement.
- La spec 066 ajoutera un backend `docker`; cette spec refuse toute autre
  valeur que `host`.

## Sécurité

- Canonicaliser avant de vérifier les racines autorisées.
- Refuser les racines elles-mêmes trop larges telles que `/` et `/home`.
- Ne jamais suivre une nouvelle cible de lien symbolique après persistance sans
  constater une divergence.
- Ne jamais exposer le contenu du dépôt dans logs, métriques ou événements.
- Les commandes distantes, MCP ou UI ne peuvent pas enregistrer, rebind ou
  désactiver un projet dans cette tranche: frappe locale uniquement.
- Le socket local vérifie le rôle Maicie, la capability négociée et l'UID pair
  égal à celui du daemon avant de décoder une mutation de registre.

## Observabilité

- Logs structurés: `command_id`, issue, composant, backend, durée et code de
  refus. Pas de contenu ni de secret.
- Audit durable: ProjectAuditEvent transactionnel pour chaque mutation
  effective; identifiant déterministe afin qu'un rejeu ne double pas la trace.
- Métriques bornées: total par opération/issue, durée de saga et nombre de
  liaisons par état. Aucun `project_id` comme label Prometheus.
- Projection: dernière tentative, fraîcheur, état et prochaine action.

## Stratégie de test

1. Tests unitaires des validateurs et transitions.
2. Tests de contrat du protocole public.
3. Tests SQLite de migration et unicité.
4. Tests de saga avec crash injecté à chaque frontière.
5. Tests d'intégration CLI sur dépôt et worktree temporaires.
6. Tests de non-destruction par empreinte avant/après.
7. Suite de compatibilité historique du spawn sans `project_id`.
8. Scénarios Gherkin lisibles dans `tests/features/065-registre-identite-projets.feature`.
9. Reprise d'un snapshot et d'un curseur SPEC-064 avec conservation exacte de
   ProjectReference.
10. Prévisualisation puis rapprochement idempotent de `review_project`.
11. Rejeu d'un incident runtime délégué SPEC-068 avec ProjectReference
    inchangée et accusé idempotent.

## Déploiement et réversibilité

- Livrer d'abord les lecteurs compatibles, puis les écrivains.
- Activer l'enregistrement uniquement après validation des migrations.
- Le rollback ignore les nouveaux champs et conserve les nouvelles tables.
- Aucun projet historique n'est migré automatiquement.
- Aucun redémarrage de production n'appartient à ce run de spécification.

## Constitution Check

| Règle | Verdict | Décision |
|---|---|---|
| Français | PASS | Tous les artefacts sont en français. |
| Cycle SpecKit | PASS | Specify, Plan, Audit Existing, Tasks et Analyze précèdent le code. |
| Worktree | PASS avec avertissement | Worktree isolé, mais plus de cinq worktrees existent déjà. |
| Réutilisation | PASS | Stores, outbox, protocole, spawn et projections existants sont étendus. |
| Minimalisme | PASS | Aucun Docker, daemon, crate ou dépendance nouvelle. |
| Responsabilité future | PASS | Autorités, état partiel, migration et rollback sont explicites. |
| Observabilité | PASS | Logs, métriques et état de saga sont conçus avant le code. |
| Complexité | PASS | Index uniques par id et racine, aucune recherche quadratique. |

## Livrables de conception

- `spec.md`
- `research.md`
- `data-model.md`
- `contracts/project-registry-v1.md`
- `contracts/project-root-policy-v1.md`
- `quickstart.md`
- `reuse-audit.md`
- `tasks.md`
- `analysis-report.md`
- `implementation.md`

## Gate avant implémentation

- Créer un nouveau worktree d'implémentation depuis `d589b24` ou un descendant
  propre contenant SPEC-068; le présent worktree documentaire, en retard de 19
  commits au 2026-08-30, ne doit pas servir de base au code.
- L'absence de `.specify` ou de l'outil `specify` ne bloque pas T001 et ne doit
  déclencher ni installation ni mise à jour. Les documents versionnés sous
  `specs/065-registre-identite-projets/` sont la source de vérité.
- SPEC-063 est prouvée sur la route réelle.
- Les identifiants d'exécution de SPEC-064 sont stabilisés.
- Les contrats et persistances d'incidents délégués de SPEC-068 sont présents.
- Refaire l'inventaire exact des types et migrations SPEC-064 sur cette future
  tête et confirmer la matrice ProjectReference avant T004.
- `reuse-audit.md` reste `PASS` contre la tête d'implémentation.
- Analyze ne laisse aucun finding CRITICAL.
- L'utilisateur approuve le programme 065-067.
- Aucune implémentation ne commence dans le présent run.
