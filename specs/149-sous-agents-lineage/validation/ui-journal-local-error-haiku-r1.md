Note finale du principal : le typecheck web global conserve 10 erreurs préexistantes et zéro nouvelle erreur dans le périmètre149. Le « 0 » d’une ancienne section ci-dessous concerne le périmètre modifié et ne prouve pas un typecheck global sans erreur.

# Session 149 - Journal de tâche Bridget : erreur locale `journal_unavailable` (haiku r1)

## Rectification du principal après lecture

- Le brief indiquait 10 erreurs web et 16 erreurs serveur préexistantes, avec zéro erreur nouvelle. Les 10 erreurs web de ce tour correspondent à cette baseline. L'hypothèse « cause probable : packages/contracts » ci-dessous n'est pas établie et ne constitue pas un nouveau finding.
- La recette native réelle est déjà décrite dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-native-recipe149-sonnet-r1.md`. Elle a observé `journal_unavailable` après l'arrêt réel d'un enfant. Ce test prouve le nouveau rendu ; il ne remplace pas cette recette.
- Le principal a changé le libellé avant ce tour. L'empreinte « avant » ci-dessous contient déjà ce changement. Haiku a ensuite modifié uniquement sa mise en forme. La nouvelle revue Sonnet porte sur ce petit changement de libellé ; les approbations des autres sources T3 restent distinctes.


## Résumé

- Un test a été ajouté pour le cas « tâche annulée + journal indisponible ».
- Le test lit le journal par le vrai chemin de lecture (« Lire le début »), puis reçoit un refus `journal_unavailable`.
- Le message affiché ne parle plus de service Bridget indisponible. L'historique reste affiché. Le bouton « Arrêter » reste absent.
- Le code de production n'a pas de changement de logique. Seule la mise en forme d'une ligne a changé (formateur).
- Résultat : 25 tests passent sur les 4 fichiers UI 149 (24 en baseline, plus 1 nouveau). Lint ciblé : 0 erreur.

## Périmètre

Fichiers modifiés par cette phase :

- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/BridgetTaskJournal149.test.tsx` : test ajouté (9 tests devenus 10).
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/BridgetTaskJournal.tsx` : mise en forme seule (voir ci-dessous).

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

Environnement : Node v24.13.1 (`/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin`). `node_modules` est un lien symbolique vers le dépôt principal, déjà en place.

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

- Les tests existants sur la bannière « Lineage Bridget indisponible » et sur la reconnexion (« root global ») passent sans changement.
- Pas de nouvelle fixture ajoutée pour un cas « root ordinaire » : la fixture existante du test de reconnexion le couvre déjà.
- Les preuves UI antérieures restent valables sur le même périmètre : seul le libellé d'un cas a changé.
- Aucun test global de la suite T3 n'a été lancé (demande du brief).

## Limites et risques

1. **Mutation non exécutée.** Je n'ai pas relancé le test contre l'ancien libellé, pour ne pas modifier le code de production. Par construction, l'ancien texte commence par « Bridget indisponible (journal_unavailable) ». Il ferait donc échouer les deux assertions de message. Ce point n'est pas prouvé par exécution.
2. **Fixtures mockées.** Le test vérifie le rendu du composant avec une erreur simulée. Il ne prouve pas que le serveur renvoie `journal_unavailable` dans un cas réel. Ce point relève de la recette native.
3. **Aucun modèle ni fournisseur.** Ces tests n'impliquent aucun fournisseur ni modèle. Aucun succès de modèle n'est déduit.
4. **Empreinte de production changée.** Une revue Sonnet finale est requise sur `4abac314...`.
5. **Typecheck web : 10 erreurs hors périmètre.** Elles sont dans `apps/web/src/components/chat/MessagesTimeline.logic.test.ts` (7) et `MessagesTimeline.test.tsx` (3). Ces deux fichiers ne sont pas modifiés dans l'arbre de travail. Le brief indiquait 0 erreur web en baseline, mais je n'ai pas relancé le typecheck avant modification. Cause probable : les types modifiés dans `packages/contracts`. Cette cause n'est pas vérifiée. Mes fichiers ont 0 erreur. Journal complet : `/Users/moi/.cache/ui-journal-149-haiku-r1-backup/tsc-web.log`.
6. **Lint global et suite T3 non lancés.** Le lint a été lancé seulement sur les 2 fichiers ciblés.
7. **Aucune capture d'écran** n'a été produite ni utilisée.

## Fichiers de preuve (hors dépôt)

- `/Users/moi/.cache/ui-journal-149-haiku-r1-backup/BridgetTaskJournal.before-fmt.tsx`
- `/Users/moi/.cache/ui-journal-149-haiku-r1-backup/tsc-web.log`
- `/Users/moi/.cache/ui-journal-149-haiku-r1-backup/lint.log`

Empreinte du test : avant `c8d782e2162f90f3031861782b3d4bfd02fa70a3e0c510edc3bd070754412de2`, après `69e672229f7c6903dfefcc9e54164c5b0a7f1f680f0577149310ace697950a90`.
