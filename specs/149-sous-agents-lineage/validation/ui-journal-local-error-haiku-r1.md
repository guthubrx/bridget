Note finale du principal : le typecheck web global conserve 10 erreurs préexistantes et zéro nouvelle erreur dans le périmètre149. Le « 0 » d’une ancienne section ci-dessous concerne le périmètre modifié et ne prouve pas un typecheck global sans erreur.

# Session 149 - Journal de tâche Bridget : erreur locale `journal_unavailable` (haiku r1)

## Rectification du principal après lecture

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md#extrait-1).


## Résumé

- Un test a été ajouté pour le cas « tâche annulée + journal indisponible ».
- Le test lit le journal par le vrai chemin de lecture (« Lire le début »), puis reçoit un refus `journal_unavailable`.
- Le message affiché ne parle plus de service Bridget indisponible. L'historique reste affiché. Le bouton « Arrêter » reste absent.
- Le code de production n'a pas de changement de logique. Seule la mise en forme d'une ligne a changé (formateur).
- Résultat : 25 tests passent sur les 4 fichiers UI 149 (24 en baseline, plus 1 nouveau). Lint ciblé : 0 erreur.

## Périmètre

Fichiers modifiés par cette phase :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md#extrait-2).

Non modifiés : autres rapports, tâches, Git, services, configuration, base de données, Cargo, `pnpm install`.

## Production : empreintes et diff

| Fichier | SHA-256 avant | SHA-256 après |
|---|---|---|
| `apps/web/src/components/BridgetTaskJournal.tsx` | `4aa5bba24da4d36e446741c88790f4602ec1d0e4dd8bf9fd2942f0456a3ed112` | `4abac314791a3ad77d8d63e399ec5c9c3e6188df1ebaff13aef426ae90195ff5` |

Diff produit par `vp fmt` (aucun autre changement) :

```diff
-  const subject = code === "journal_unavailable" ? "Journal de la tâche indisponible" : "Bridget indisponible";
+  const subject =
+    code === "journal_unavailable" ? "Journal de la tâche indisponible" : "Bridget indisponible";
```

Copie d'avant formatage : `/Users/moi/.cache/ui-journal-149-haiku-r1-backup/BridgetTaskJournal.before-fmt.tsx`.

Conséquence : l'empreinte a changé. L'ancienne approbation de source n'est plus valable. Une revue Sonnet finale doit être refaite sur l'empreinte `4abac314...`.

## Test ajouté

Titre : « tâche annulée : journal indisponible lu sans annoncer Bridget indisponible, historique conservé ».

Déroulé :

1. Montage avec `task.status = "cancelled"`. Le droit d'opérer est accordé (`canOperate = true`).
2. Livraison d'un événement réel (`ligne 7`) par le flux de suivi.
3. Clic sur « Lire le début » : une vraie lecture `action: "journal"` part (`afterSeq: 0`).
4. Réponse refusée : `BridgetLineageError` avec le code `journal_unavailable`.

Vérifications :

- Un élément `role="status"` contient exactement : « Journal de la tâche indisponible (journal_unavailable). Le contenu déjà lu est conservé. »
- Le texte ne contient ni « Bridget indisponible » ni « Lineage Bridget indisponible ».
- L'historique reçu (`ligne 7`) reste affiché.
- Le statut « Annulé » s'affiche.
- Les contrôles « Reconnecter » et « Lire le début » restent présents.
- Le bouton « Arrêter » est absent, alors que le droit d'opérer est accordé. C'est donc l'état terminal qui le masque, pas une permission.

Les messages existants (`snapshot_changed`, `store_unavailable`, `command_failed`) gardent leur libellé. Leurs tests passent sans changement.

## Résultats des commandes

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md#extrait-3).

| Commande | Dossier | Avant modification | Après modification |
|---|---|---|---|
| `vp test run --project unit` sur `BridgetTaskJournal149` et `ThreadRelationshipsControl.bridget149` | `apps/web` | 13 / 13 (2 fichiers) | 14 / 14 (2 fichiers) |
| `vp test run` sur `orchestration.bridget149` | `packages/client-runtime` | 9 / 9 | 9 / 9 |
| `vp test run` sur `BridgetTaskJournal149` | `apps/mobile` | 2 / 2 | 2 / 2 |
| `vp lint` (2 fichiers ciblés) | racine | non lancé | exit 0 |
| `vp fmt --check` (2 fichiers ciblés) | racine | 1 fichier à reformater | OK |
| `tsc --noEmit -p .` | `apps/web` | non lancé | 10 erreurs, hors de mes fichiers (voir limites) |

Total des 4 fichiers UI 149 : 25 / 25 passent (24 en baseline, plus 1).

Le quatrième fichier UI 149 est celui de `apps/mobile`. Je l'ai inclus dans la série, car le brief parle de « 4 fichiers UI client 149 » et le total de 24 correspond.

## Régressions et périmètre

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md#extrait-4).

## Limites et risques

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/ui-journal-local-error-haiku-r1.md#extrait-5).

## Fichiers de preuve (hors dépôt)

- `/Users/moi/.cache/ui-journal-149-haiku-r1-backup/BridgetTaskJournal.before-fmt.tsx`
- `/Users/moi/.cache/ui-journal-149-haiku-r1-backup/tsc-web.log`
- `/Users/moi/.cache/ui-journal-149-haiku-r1-backup/lint.log`

Empreinte du test : avant `c8d782e2162f90f3031861782b3d4bfd02fa70a3e0c510edc3bd070754412de2`, après `69e672229f7c6903dfefcc9e54164c5b0a7f1f680f0577149310ace697950a90`.
