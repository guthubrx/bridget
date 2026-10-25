# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-restart-final-source-review-sonnet-r2.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-restart-final-source-review-sonnet-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Niveau | Verdict |
|---|---|
| Sources : O4.6 (délai natif commun), E2 (queued à propriétaire perdu), F3 (clôture CAS avant `cancelled`), alias Codex hors namespace | **SOURCE_ONLY_APPROVE** - zéro finding bloquant |
| Documents (héritage, discovery explicite, compat148, O1) | **Cohérents** avec le code lu. Aucune contradiction matérielle. |
| Delta T3 `BridgetTaskJournal.tsx` (libellé `journal_unavailable`) | **Approuvé** par lecture. SHA conforme. |
| Preuve d'exécution | **Non faite ici.** Voir section 6. |

<a id="extrait-2"></a>

## Extrait 2

T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/BridgetTaskJournal.tsx` = `4abac314791a3ad77d8d63e399ec5c9c3e6188df1ebaff13aef426ae90195ff5` (conforme au brief). Le test `BridgetTaskJournal149.test.tsx` fait `69e67222...0a90`.

<a id="extrait-3"></a>

## Extrait 3

| Point | Conclusion | Preuve |
|---|---|---|
| Fermer seulement le cas prouvé | Conforme. | `queued_owner_lost` rend `false` sans `parent_task_id` (`:878`). Un `queued` externe (T3 ou autonome) n'a pas de parent natif : `parent_task_id` n'est posé que si `for_child(&owner)` trouve une tâche (`:460-477`). Il garde la reprise normale. |
| Preuve de filiation | Conforme. | Il faut : parent lisible ; `parent.child == task.owner` ; `instance == owner_instance == origin_owner_instance` ; même racine ; lien de flotte de l'instance du parent ; son parent dans {origine, courant} du parent ; `delegation_id` du parent ; rôle `native_delegate` (`:879-893`). |
| Perte définitive | Conforme. | `prepare_restart` ne s'exécute qu'au démarrage (seul appelant : `daemon.rs:3892`, via `reserve_managed_recoveries` en `:4628`). Les enfants natifs sont exclus de toute reprise. Le propriétaire ne revient donc jamais. |
| Effet sur la racine | Conforme. | Le `queued` passe `failed/unreachable`, donc terminal. `descendants_busy` ne le compte plus (`:1298-1300`). Une racine `waiting_for_children` qui a son résultat sort de l'attente (`:1091-1121`) et le résultat reste livrable une fois (clé idempotente inchangée). |
| Ordre sûr à la reprise | Conforme. | CAS d'exécution d'abord, `save` ensuite (`:937-938`). Un arrêt entre les deux se rejoue sans rouvrir un état terminal. |
| Contradiction de filiation | **Fermée, mais bloque le démarrage.** | Parent absent, instance absente ou lien incohérent rendent `Err` (`:879-893`). La tâche n'est pas modifiée. Mais l'`Err` remonte jusqu'à `daemon.rs:4628` et le daemon refuse de démarrer. Voir O-1. |

<a id="extrait-4"></a>

## Extrait 4

La capacité privée de résultat retenu, l'autorité avant idempotence, le CAS de `working`/`failed-unreachable`, le gestionnaire SIGTERM par drapeau et l'arrêt du fournisseur par le wrapper n'ont pas changé de façon visible dans les zones relues. Aucune interférence trouvée avec les quatre deltas. La revue T3 existante n'est pas refaite.

<a id="extrait-5"></a>

## Extrait 5

| Exigence du brief | Conclusion | Preuve |
|---|---|---|
| Héritage par défaut, sans grant humain | Conforme au code. | Doc : posture absente = héritage, « ne demande aucun grant ». Code : avec un fait parent, `child_policy` ne produit jamais plus que le fait (`native_permissions.rs:206-277`) et aucune lecture de grant n'a lieu. |
| `discovery` explicite = lecture seule | Conforme. | Doc l.81. Code : `readonly = posture == Some(Discovery)` (`native_permissions.rs:209`), effet `Discovery` (`:274`). |
| Compat148 sans fait : seulement `discovery` explicite, selon l'ancien grant | Conforme. | Doc l.81-83 et SKILL l.48-50. Code : sans fait, `Discovery` explicite passe par `authorize_cwd` (grant) ; toute autre posture rend `permission_attestation_unavailable` (`native_delegation.rs:429-432`). Pas de repli silencieux. |
| O1 : credential frais sans fait jamais vs credential retiré | Clarifié, cohérent. | `contracts/permissions.md:36-47` et `:441-466` (branche (b)(i)/(ii)) reprennent la formulation de r1. `test-strategy.md` S149-32(b) est aligné. L'annulation humaine native est indépendante du credential T3 (b)(i). |
