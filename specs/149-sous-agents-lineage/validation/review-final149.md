# Revue finale 149 (T041) - Sonnet 5.5 high

## Complément final du principal (2026-10-10)

Ce complément prévaut sur les verdicts datés ci-dessous. Le corps de la revue reste historique.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-1).

## Actualisation du principal — portée historique

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-2).

Date : 2026-10-10. Relecteur : sous-agent de revue (Sonnet 5.5, effort high).
Cette revue ne coche aucune case. Elle ne lance aucun modèle réel, aucun service, aucun Git, aucune installation, aucun Cargo. Elle n'édite aucun fichier de production ni de test.

## 1. Verdicts (deux niveaux séparés)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-3).

SOURCE_ONLY_APPROVE veut dire ceci : le code lu respecte le contrat validé. Les tests exécutés prouvent les niveaux annoncés plus bas. Il ne dit rien du comportement des modèles réels ni du MCP privé authentifié sur le réseau.

Conditions de validité : les empreintes de la section 2 doivent rester identiques. Une nouvelle correction de production rouvre cette revue pour les fichiers touchés.

## 2. Code exact relu (empreintes vérifiées aujourd'hui)

Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage`. Base `6807c22b7ada683f757486a6382aeda170ec68ab`. L'arbre n'est pas committé (41 entrées `git status`).

- Empreinte des 196 fichiers `.rs`, `.toml`, `Cargo.lock` de `crates/` et de la racine : `e6f45086b23974604667c93d3b34755d7d73dc5dadbe4dde94c1e09f34285408`. Je l'ai recalculée avec l'algorithme du reçu. Elle est **identique** au reçu `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native149-debug-receipt.json`.
- Les 37 digests individuels du reçu sont tous identiques. Aucun `.rs` n'est plus récent que le binaire debug `bridget-823e8a5fab8a`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-4).

- `git diff HEAD` des fichiers suivis : `fc9fa99f4ada2dca` (16 premiers caractères), **identique** au rapport r4.
- `apps/server/src/bridget/BridgetLineage.ts` : `787109f44ddf9a52`. `apps/web/src/components/BridgetTaskJournal.tsx` : `16004d224bac63ef`. Les deux sont identiques au rapport r4.
- Aucun fichier `.ts` ou `.tsx` n'est plus récent que le rapport r4.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-5).

## 3. Ce que j'ai fait moi-même

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-6).

Exécutions (lecture seule, sans modèle) :

| Contrôle | Résultat |
|---|---|
| `claude --version` et `claude --help` (sondes sans modèle) | 2.1.296 ; `--permission-prompts host\|none`, `--permission-mode`, `--tools`, `--allowedTools` existent |
| `claude --permission-mode default --version` | accepté (alias caché) ; `bogus` refusé : le CLI valide bien les choix |
| Recherche de chaînes dans le binaire CLI 2.1.296 (lecture seule) | la liste interne est `acceptEdits, auto, bypassPermissions, default, dontAsk, plan` et `manual` est normalisé en `default` (`rh(e)=e==="manual"?"default":e`). L'événement `system` porte donc `default`. Le Rust, qui n'accepte pas `manual`, est correct. |
| `tsc --noEmit` : `packages/contracts`, `packages/client-runtime`, `packages/provider-core`, `apps/mobile` | 0 erreur chacun |
| `tsc --noEmit` : `apps/server` | 16 diagnostics, **tous dans des fichiers que le diff ne touche pas** : `BridgetRustInterop.test.ts` (2), `BridgetRustInterop.testkit.ts` (12), `BridgetRustInteropObserver.test.ts` (1), `provider/Drivers/CodexMcp.ts` (1). Ce sont des règles de style du plugin Effect (imports Node, `Date.now`). Aucune n'est une erreur de type. Le compte est le même qu'en r4. |
| `tsc --noEmit` : `apps/web` | 10 diagnostics, tous dans `MessagesTimeline.logic.test.ts` (7) et `MessagesTimeline.test.tsx` (3), non touchés par le diff |
| `vp test run` : journal web, `ThreadRelationshipsControl.bridget149`, `orchestration.bridget149`, journal mobile | **24 PASS / 0 FAIL, 4 fichiers** (2,47 s). Le chiffre « 22 » de r4 ne compte pas le mobile (3 fichiers, 9+9+4). Le mobile ajoute 2 tests. |
| Recomptage des logs r4 natifs | daemon 1471 PASS / 0 FAIL / 61 ignorés ; transport 325 / 0 / 2 ; observer en série 10 / 0 ; **0 avertissement de compilation** |
| Ports de recette 14775 et 15735 | libres |

Non exécuté, avec la raison :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-7).

## 4. Revue du contrat, point par point

Les noms de rapports sans dossier se trouvent dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/`. Les références de ligne Rust sont dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/` sauf mention contraire.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-8).

## 5. Findings

**Zéro finding actif de sûreté, de correction ou de contrat.**

### Observation O1 - non bloquante, non mesurée (performance, Article XXII)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-9).

### Gaps de preuve réels et minimaux

Chaque gap a un niveau de preuve actuel, ce qui manque, et le plus petit test utile. Aucun ne bloque SOURCE_ONLY_APPROVE. Le principal décide s'il les ferme avant le cochage.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-10).

### Documentation à corriger par le principal (hors de mes fichiers)

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/tasks.md`, en-tête : « G-P REQUEST_CHANGES ciblé r3 » est périmé. `permissions-contract-deltas-r4.md` donne APPROVE.
- Même fichier : coquilles `àS149-33`, `Phase0`, `profondeur8`, `liste100/128KiB`, `pages16KiB/100`.

## 6. Scénarios S149-29 à S149-33 : preuve exacte par niveau

Les tests sont lus. Les noms sont ceux du code. Tous apparaissent `ok` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-r4-final-daemon.log` ou `native-r4-final-transport.log`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-11).

## 7. Limites réel et synthétique

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/review-final149.md#extrait-12).

## 8. Correction de la matrice de preuves

La matrice intermédiaire `proof-map149.md` contenait des erreurs de classement. Je l'ai réécrite. Les corrections de fond :

- T022 : « Non prouvé » était faux. Deux tests Rust nommés existent et passent.
- T029 : « Non prouvé » était faux. Sept tests `thread.stop` existent dans `Orchestrator.bridget149.test.ts` et passent. Le classement ne cherchait que les rapports.
- T016 à T023 : la colonne « non nommés » ignorait des tests nommés (profondeur, atomicité, rollback, watch, pages de journal, annulation, rejeu).
- T001 et T002 : classés partiels à tort. Leur critère est documentaire et il est rempli.
- T040 : classé partiel à tort. Le critère est une recette UI web avec preuves visuelles, et elle existe (r2, r3, r4).
- Le « 22 tests UI » de r4 n'inclut pas les 2 tests mobiles. Total vérifié aujourd'hui : 24.
- Les chemins tronqués `.../` sont remplacés par des chemins absolus.

## 9. Pour le principal

- Verdict sources : SOURCE_ONLY_APPROVE. Candidats au cochage sur cette base, au niveau de preuve de `proof-map149.md` : T001, T002, T004, T005, T007, T008, T010, T016 à T019, T021 à T023 (T020 partiel : gap G8), T024 à T031 (T029 compris), T033, T040. Restent partiels : T006, T009, T011, T012, T013, T014, T015, T020, T032, T034, T035 (voir `proof-map149.md`). Le principal décide. Cette revue ne coche rien.
- À fermer ou accepter avant le cochage : G1 à G9 (aucun ne bloque).
- À attendre : T036 à T039 (pending chez d'autres propriétaires), puis T042 (`quickstart.md`, `implementation.md`), T043 à T045.
- Fichiers de ce propriétaire libérés : `review-final149.md`, `proof-map149.md`, `ui-recipe149.md`.
