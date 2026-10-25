# Session 149 - validation delta r4 (F1 sûreté, F2 interface)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md#extrait-1).

## Verdict : APPROVE

Les deux correctifs de Sol sont corrects et prouvés. Aucun finding actif.
Aucune case cochée. Aucun Git, aucune config de production, aucun redémarrage, aucun Cargo.

Limite à lire d'abord : le daemon Bridget est SIMULÉ (`bridget_fixture.mjs`, magasin JSON privé) et les données sont synthétiques (`[recette149]`). Cette ronde ne valide ni SC001, ni T037, ni le MCP privé, ni T036/T039 natifs. Je n'ai pas utilisé le binaire natif.

## Couches réelles et simulées

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md#extrait-2).

## Compteurs exécutés

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md#extrait-3).

`effect_outbox = 4` : ce sont 2 x (`terminal.cleanup` + `preview.cleanup`), statut `succeeded`, créés par les 2 `thread.delete` de MON groupe `deleted` (rejeu ZD-1 lancé deux fois). Zéro effet fournisseur. Cause prouvée par lecture des 4 lignes.

## F1 - audit du delta de Sol (`BridgetLineage.ts`)

Code lu : `unavailable` lit le root par `getThreadRecords(input.threadId)`, retourne sans dispatch si `root.projectId !== input.projectId` ou si `root.bridgetTaskRef !== undefined`, et pour `task_unavailable` exige en plus `child.projectId === input.projectId` et `child.bridgetTaskRef.rootThreadId === root.id`. Un échec de lecture du root est ignoré (`Effect.ignore`). Les quatre codes locaux (`journal_unavailable`, `result_offset_invalid`, `invalid_request`, `envelope_mismatch`) ne dispatchent rien.

### Tests serveur (adaptés et nouveaux)

Le harnais de `BridgetLineage149.test.ts` simulait `getThreadRecords` sans `id` ni `projectId` : 7 tests échouaient (le root n'existait pas). Je l'ai adapté avec des enregistrements scientifiques : le root appartient réellement à `projectId`, n'est pas virtuel ; un enfant porte son marqueur et son projet. Option `threads` pour forger un autre cas. Aucune assertion de sûreté baissée.

5 nouveaux tests F1 :
1. `read` list et show avec `projectId` forgé + vrai root, 6 codes de dégradation : 0 dispatch.
2. `watch` et `journal` idem : 0 dispatch.
3. Root inconnu ou virtuel, même avec le bon projet : 0 dispatch.
4. `task_unavailable` : enfant connu + projet forgé, ou enfant d'un autre projet : 0 dispatch.
5. Contexte vrai : les 6 codes dégradent toujours le vrai root (list et journal). La sûreté n'est pas affaiblie.

Test de mutation (copies temporaires supprimées, aucune édition de production) :
- Garde root retirée : 3 des nouveaux tests échouent.
- Garde enfant retirée : 2 tests échouent (1 nouveau, 1 ancien).
Les tests détectent donc bien la régression.

### Rejeux contre le vrai serveur (`replay_r4.mjs`)

| Groupe | Résultat |
|---|---|
| ZR : `projectId` étranger + vrai fil, 6 appels (`list`, `show`, `journal` en read, `watch`, `journal` flux, `cancel` operate) | 6 x `project_mismatch`, 0 événement, drapeaux de disponibilité identiques, `list` vrai juste après OK |
| ZT : root inconnu ; root virtuel (fil enfant utilisé comme root) sur `list`/`show`/`watch`/`journal` | 8 x `binding_unavailable`, 0 mutation |
| ZK : tâche connue retirée + projet forgé ; UUID inconnu ; contexte vrai + tâche connue retirée | `project_mismatch` sans effet ; `task_unavailable` sans effet ; ce fil seul passe indisponible, racine intacte ; reprise OK |
| ZG : contexte VRAI + `project_mismatch` NATIF (daemon, autre root), `invalid_output`, `timeout` (daemon 8 s > 6 s) | la racine réelle passe indisponible (5 `thread.metadata-updated` : racine + 4 fils disponibles), reprise COMPLÈTE au seq courant du magasin, 135 fils sans doublon |
| ZG-4 : `journal_unavailable` en contexte vrai | local, racine intacte (portée non élargie) |
| ZD-1 : fil hôte lié puis supprimé | observation : `binding_unavailable`, racine principale intacte, 0 événement. Limite : le fil T5 n'est jamais lié (sa première lecture rend `invalid_output`, probable refus de rattacher des tâches déjà liées à un autre root ; non vérifié) |

Correction d'oracle de mon outil (pas du produit) : ZG-3r comparait le filigrane au seq d'avant `hang`. L'étape `hang` incrémente le seq du magasin ; l'oracle compare maintenant au seq courant du magasin.

### Rejeux r3 sur le serveur r4 (23/23)

Journal 1/4 et 1/134 : 3 événements (tâche modifiée + racine), tâches inchangées 0. Tick journal seul : 1 événement racine. 100 `list` + 100 `show` : 0 événement. Changement de génération : 134 marqueurs, 0 `subagent.updated`. Panne globale puis reprise complète au même seq. Historique 130 `available:false` conservé, 4 `true`. Snapshot partiel : `snapshot_changed`, aucun fil créé. `journal_unavailable`/`result_offset_invalid`/`envelope_mismatch` locaux. Autre root : 0 événement.

### Matrice réseau (38/38)

Scopes read, operate, settings:write, 13 charges invalides ou à propriétés en trop, mauvais root, tâche ou fil, cancel rejoué, enveloppe différente : tous conformes.

### UI avec contexte forgé (navigateur preview)

Fil hôte ouvert, `Lineage · 1 running`, « Arrêter » actif. Appel RPC forgé (jeton read, projet étranger + vrai fil) : refus `project_mismatch`, aucun changement d'interface (captures 06 et 07). C'est l'inverse du finding de r3 (capture r3 n°10).
Contraste : `project_mismatch` NATIF réel (magasin d'un autre root) : « Lineage · Bridget indisponible », « Arrêter » absent (capture 08). Après restauration et resynchronisation, le bouton « Arrêter » revient. La bannière « Lineage Bridget indisponible. Reconnecter Bridget » reste jusqu'au clic sur « Reconnecter Bridget », puis disparaît (capture 09). C'est la reconnexion manuelle du flux watch voulue par le test existant « propose la reconnexion sur échec connu » ; je la note comme observation, pas comme défaut.

## F2 - audit du delta de Sol (`BridgetTaskJournal.tsx`)

Code lu : `useLayoutEffect` mesure `[data-chat-composer-overlay]` dans son `[data-chat-canvas]`, stocke la hauteur, applique `paddingBottom: calc(0.75rem + Npx)` sur le conteneur défilant (remplace `py-3`), `ResizeObserver` avec `disconnect` au démontage. Pas de `z-index`.

### Tests (nouveaux, jsdom absent : react-test-renderer avec `createNodeMock`)

- Hauteur 40 px -> `calc(0.75rem + 40px)` ; redimensionnement à 72 px suivi sans rechargement ; `disconnect` appelé une fois au démontage.
- Sans barre flottante : `calc(0.75rem + 0px)`.

### Mesures dans le vrai navigateur (journal long, 36 lignes sur T1, 31 sur T2, 22 + résultat sur adaptateur de bureau)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md#extrait-4).

La dernière ligne (#36) est entièrement au-dessus de la barre à défilement maximal (bas de ligne 728, haut de barre 740). En r3, environ 40 px étaient masqués.

Clic réel sur « Arrêter » de la tâche imbriquée T2 (titre long, 1280 px, carte « Thread details » ouverte, locator `role=button[name='Arrêter']`) : le clic passe, T2 passe à `cancelling` dans le magasin. Aucun ProviderTurn, aucun Run (compteurs SQL ci-dessus).

Je n'ai pas rejoué 1024 et 768 en navigateur dans cette ronde. Je n'ai pas testé le mobile natif ni iOS.

## Captures (privées, absolues)

Dossier : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui149-screenshots-r4/`
1. `01-f2-racine-scroll-max-1280-derniere-ligne-au-dessus-barre.png`
2. `02-f2-racine-scroll-max-480-adapte-sans-composeur.png`
3. `03-f2-racine-scroll-max-375-adapte-sans-composeur.png`
4. `04-f2-imbrique-titre-long-1280-carte-ouverte-arreter-cliquable.png`
5. `05-f2-tache-terminale-resultat-journal-scroll-max-1280.png`
6. `06-f1-hote-avant-appel-forge-lineage-disponible.png`
7. `07-f1-hote-apres-appel-forge-lineage-toujours-disponible.png`
8. `08-f1-autorite-reelle-project-mismatch-natif-lineage-indisponible.png`
9. `09-f1-reprise-apres-reconnecter-lineage-disponible.png`

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md#extrait-5).

## Observations sans correction demandée

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md#extrait-6).

## Fichiers et empreintes (sha256, 16 premiers caractères)

Sources de production évaluées, NON modifiées par moi :
- `apps/server/src/bridget/BridgetLineage.ts` : `787109f44ddf9a52`
- `apps/web/src/components/BridgetTaskJournal.tsx` : `16004d224bac63ef`
- `git diff HEAD` du WT (fichiers suivis) : `fc9fa99f4ada2dca`, base `33f6d04e116430bf7f0011902d6af3868163f2d1`

Fichiers que j'ai modifiés (tests et outils seulement) :
- `apps/server/src/bridget/BridgetLineage149.test.ts` : `7ba2fb545502e0dc` (avant : `961e14b4acae4ee8`)
- `apps/web/src/components/BridgetTaskJournal149.test.tsx` : `70a9528c10d53ce2` (avant : `7708cb454a685d8b`)
- `ui-recipes/replay_r4.mjs` (nouveau) : `0f38b914527c53a9`

## Commandes

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md#extrait-7).

## Nettoyage

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md#extrait-8).
