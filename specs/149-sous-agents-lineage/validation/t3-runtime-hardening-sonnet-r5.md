# Session 149 - validation ciblée r5 (après qualité production de Sol)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md#extrait-1).

## Verdict : APPROVE

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md#extrait-2).

Limite à lire d'abord : le daemon Bridget de la recette navigateur est SIMULÉ (`bridget_fixture.mjs`, magasin JSON privé) et les données sont synthétiques (`[recette149]`). Cette ronde ne valide ni SC001, ni T037, ni le MCP privé, ni T036/T039 natifs. Je n'ai pas utilisé le binaire natif. Je n'ai pas rejoué la matrice réseau 134 tâches, inchangée : les deltas de Sol ne touchent ni `BridgetLineage.ts` ni le serveur de projection.

## Preuves de non-modification

| Mesure | Avant | Après |
|---|---|---|
| sha256 des 32 fichiers de production (liste triée, `shasum`) | `26fba25527ad0247` | `26fba25527ad0247` (identique) |
| sha256 des 16 fichiers de tests149 | identique | identique |
| `git diff HEAD` (fichiers suivis) | `59a54267ce7c1d2a` | `59a54267ce7c1d2a` |

Les 32 fichiers de production : 26 lineage, 6 permissions, au sens de la liste `git diff --name-only HEAD` + fichiers non suivis hors tests. Aucune source n'a bougé pendant ma fenêtre.

Empreintes des 4 fichiers livrés par Sol (16 premiers caractères) :
- `apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts` : `497bcdbeebe0b98d`
- `packages/contracts/src/bridgetPermissions.ts` : `df03fd3fb71dc3e2`
- `packages/provider-core/src/server/mcpSession.ts` : `2e95140365bef3f8`
- `apps/web/src/components/BridgetTaskJournal.tsx` : `08a4796729bf80f1`

## Compteurs exécutés

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md#extrait-3).

Aucun test unitaire n'a été modifié. Les commandes ont donc été lancées telles quelles sur les fichiers déjà formatés/corrigés par Haiku r2 et Sol.

## Audit ciblé des 4 deltas de Sol

1. **Prédicat C0/C1 (`bridgetPermissions.ts` et `ClaudeAdapterV2.ts`).** Les deux fonctions `hasPermissionControlCharacters` sont textuellement identiques (`diff` vide). Elles testent les unités UTF-16 : `code <= 0x1f || (code >= 0x7f && code <= 0x9f)`. Preuve empirique : `boundary149.ts` décode le vrai `BridgetSessionIdentity` (v1) avec chacune des 65 536 unités UTF-16 et compare à l'ancienne règle `/[\u0000-\u001f\u007f-\u009f]/`. Résultat : 0 écart, 65 refusées (32 C0 + 33 de `0x7f` à `0x9f`), 65 471 acceptées. Bornes : `0x1f` refusé, `0x20` accepté, `0x7e` accepté, `0x7f` refusé, `0x9f` refusé, `0xa0` accepté. Une demi-paire de substitution seule et un caractère astral passent, comme avec l'ancienne règle sans drapeau `u`. Les tests existants ne couvrent que `\u0000` : je n'ai pas ajouté de test (propriété du principal), la preuve est portée par mon script jetable.
2. **`HostProcessPlatform` (ClaudeAdapterV2).** Une seule lecture `yield* HostProcessPlatform` (ligne 1004), passée aux trois captures de contexte de lancement (lignes 1052, 1103, 1160). Plus aucun `process.platform` dans le fichier. Le garde `platform !== "darwin"` (ligne 540) utilise donc la plateforme injectée. G3 (21/21) passe avec la plateforme fournie par le test. Limite : aucune exécution non-darwin native n'a été faite.
3. **Codecs compilés au niveau module.** `encodePermissionLaunchJson = Schema.encodeEffect(Schema.fromJsonString(Schema.Unknown))` (ClaudeAdapterV2) et `decodeBridgetPermissions = Schema.decodeUnknownOption(BridgetPermissions)` (mcpSession) : construits une fois, comportement identique, 14/14 et 21/21.
4. **Journal web (`BridgetTaskJournal.tsx`).** La classe `scrollbar-gutter-both` est sur le conteneur défilant ; le `paddingBottom` mesuré (`calc(0.75rem + Npx)`) et les insets de voie sont inchangés ; pas de `z-index`.

## Recette navigateur (couches réelles et simulées)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md#extrait-4).

Mesures de la dernière ligne, de la barre « lecture seule » et du bouton « Arrêter » au défilement maximal (locator : `role=button[name="Arrêter"]`, `elementFromPoint` au centre du bouton) :

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md#extrait-5).

Dans tous les cas : pas de débordement horizontal, zéro champ de saisie (`textarea`, `[role=textbox]`, `contenteditable`), aucun `<form>`.
Les écarts de gouttière asymétriques à 1280 avec la carte ouverte viennent du padding de voie (`pr` 324 px contre `pl` 48 px), fixe et voulu ; l'écart supplémentaire de chaque côté de la zone est de 6 px, donc la réserve de barre de défilement est symétrique (`scrollbar-gutter: stable both-edges`). Carte fermée : 144 / 144.

Clic réel sur « Arrêter » de l'imbriqué T2 (titre long, 1280, carte ouverte) : le clic passe, le magasin passe `T2` à `cancelling`, un reçu d'annulation est créé, aucun Run ni ProviderTurn (compteurs SQL ci-dessus).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md#extrait-6).

## Captures (privées, absolues)

Dossier : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r5/`
1. `01-racine-1280-scroll-max-carte-details-ouverte.png`
2. `02-imbrique-titre-long-1280-carte-ouverte.png`
3. `03-imbrique-480-scroll-max-sans-composeur.png`
4. `04-imbrique-375-scroll-max-sans-composeur.png`
5. `05-terminale-resultat-journal-1280-scroll-max.png`

## Observations sans correction demandée

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md#extrait-7).

## Commandes

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md#extrait-8).

Sorties brutes des tests, de `tsc` et des empreintes : `/Users/moi/.cache/bridget149-tmp/r5/` (`tests-perm.log`, `perm.json`, `ui.json`, `tsc-server.log`, `tsc-web.log`, `prod-before.sha`, `prod-after.sha`).

Nouvel outil (jetable) : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipes/boundary149.ts` (`5e6737c04b988717`).

## Nettoyage

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r5.md#extrait-9).
