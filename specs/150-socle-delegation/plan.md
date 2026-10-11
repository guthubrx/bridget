# Plan - Session150 socle de la délégation native

**Spec** : `specs/150-socle-delegation/spec.md`
**Branche** : `session-150-socle-delegation`
**Worktree** : `.worktrees/150-socle-delegation`
**Date** : 2026-10-11 (révisé après contre-revue Codex, verdict BLOCKED → corrigé)

## Résumé

Quatre corrections dans le moteur de délégation native, sans changer le
contrat d'appel 149 :

1. rétention par arbre, conditions de purge vérifiées au moment de la
   suppression, borne totale de 4096 tâches, pierre tombale de 30 jours
   (R1-R4, R9-R10) ;
2. ouverture du magasin sans plafond et admission sur les seules limites
   actives (R5-R6) ;
3. refus `referent_paused` à l'admission pendant une pause (R7) ;
4. délai de réponse par tâche, borné et hérité (R8).

La suspension des missions pendant une pause passe à la session152 (voir
`spec.md`, « Constat transmis à la session152 »).

## Contexte technique

- Langage : Rust 2024, workspace à 3 crates. Travail dans `crates/bridget-daemon`
  et `crates/bridget-transport` (protocole).
- Stockage : SQLite (rusqlite), base `bridget.db`, même connexion que le
  magasin principal.
- Tests : recette `umask 077` (quickstart).
- Aucune nouvelle dépendance.

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | `Delegate.reply_timeout` optionnel |
| `crates/bridget-daemon/src/delegation.rs` | champ `reply_timeout`, comptages actifs, pierre tombale au rejeu |
| `crates/bridget-daemon/src/delegation_lineage.rs` | ouverture sans plafond, table pierre tombale, arbre purgeable, suppression atomique, génération |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | admission (pause, délai, comptages, purge de borne), délai dans `tick` et le suivi, contrôle « sous-agent vivant » |
| `crates/bridget-daemon/src/referent_control.rs` | `AutonomousEffect::NativeDelegation` |
| `crates/bridget-daemon/src/daemon.rs` | purge et recopie des accusés à l'entretien (pas au démarrage) |
| `crates/bridget-daemon/src/idempotency/send_delivery.rs` | lecture explicite de la phase d'une remise |
| `crates/bridget-daemon/src/delegation_mcp.rs` | schéma `reply_timeout` |
| tests des mêmes modules, `delegation_lineage_tests.rs`, `native_delegation_permissions149_tests.rs` | tests `native150_*`, inversion du test de volume |
| `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | délai, refus, pause, rétention |
| `docs/decisions/150-retention-et-delai-delegation-native.md` | ADR des décisions R1-R9 |
| `docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md` | ADR de la feuille de route 150-158 |

Aucun nouveau module Rust.

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | deux ADR prévus |
| VIII Forensic | OK | défauts prouvés par lecture du code et contre-revue ; tests rouges avant correction pour la purge et l'admission |
| XV Test-before-next | Adapté | le pipeline interdit le commit automatique ; chaque tâche est testée avant d'être cochée |
| XVI Worktree | OK | `.worktrees/150-socle-delegation` |
| XVIII Complexité | OK | admission O(n) → O(log n) ; purge O(k·d) par passage, k ≤ 128 arbres, d = taille d'arbre bornée par 4096 |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Minimalisme (Article XIX)

- Pas de module, pas d'abstraction, pas de dépendance.
- La valeur 4096 existante devient la borne de rétention.
- La pause réutilise la garde unique de l'ADR 027.
- Deux gardes de plafond supprimées (`initialize_lineage`, admission cumulée).
- Rejetés : enum fermée des refus natifs, migration par lots (cas impossible),
  suspension des missions dans cette session (mécanisme partagé avec 152).

Potentiel minimalisme : environ 15 lignes supprimées.

### Responsabilité future (Article XX)

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- La contre-revue et le sort de chaque objection sont dans
  `adversarial-review-codex.md`.
- Non vérifiable dans ce dépôt : la réconciliation de liste côté T3 après purge.
  R9 contourne ce point par un snapshot complet, déjà exigé par le contrat 149.

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | 4097e délégation refusée aujourd'hui ; délégation admise pendant une pause aujourd'hui |
| Unitaire | arbre purgeable (table de vérité R2), deux arbres du même parent, remise `OutcomeUnknown` puis ligne purgée, avis jamais envoyé, entrée `Stopped` décommissionnée avant suppression, lancement en cours, pierre tombale, comptages, refus, délai explicite et implicite d'enfant, capacités |
| Ouverture | 10 000 lignes semées → ouverture réussie |
| Non-régression | filtres `native148`, `native149`, `delegation149`, `spec_087` |
| Suite complète | recette `--workspace --no-fail-fast` avant clôture |

## Risques

| Risque | Parade |
|---|---|
| Purge d'un arbre avec remise en vol | exiger la phase `acked` ; test avec `OutcomeUnknown` |
| Purge d'un arbre dont l'enfant tourne | contrôle présence + flotte + lancement sous verrou ; test de relance après nettoyage |
| Lecteur T3 garde une tâche fantôme | nouvelle génération + signal après commit |
| Remises indéterminées qui saturent la borne | refus `retention_saturated` explicite, documenté |

## Livraison

Code sur disque dans le worktree, sans commit automatique ni relance de
service. Diff cumulé enregistré dans `validation/diff-150.patch`.
