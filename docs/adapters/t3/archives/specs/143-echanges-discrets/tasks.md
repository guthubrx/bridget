# Archives techniques de l’adaptateur T3

Document d’origine : [specs/143-echanges-discrets/tasks.md](../../../../../../specs/143-echanges-discrets/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-07. Statut : Implemented — non installé, non activé.13/13 tâches vérifiées. T001–T008 conservent les preuves historiques du socle346/audit A. US4 a ses preuves392/contrôles/recette/revue distinctes. Convergence principale finale CONVERGED à11:39:54 sur tâches12/13 gelées ; coche T013 après verdict en phase documentaire distincte.
Frontend : `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/`.

<a id="extrait-2"></a>

## Extrait 2

- [x] T001 — Baseline puis RED ciblés dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.test.ts` et `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.test.tsx` : formes réelles des deux fournisseurs, erreurs/refus/inconnu, styles, dépliage/clavier, copie et ordinary. Résultat : baseline puis échecs de comportement consignés. FR01–10, SC01–05.

<a id="extrait-3"></a>

## Extrait 3

- [x] T002 [US2] — Projection sortante typée locale dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.ts` : seulement les deux formes bridget_send attestées, destinataire sûr et fallback natif pour isError/is_error/refus/ambiguïté. Aucun verdict de livraison. Résultat : tests logique GREEN. FR03–07/09, SC02/04.
- [x] T003 [US1] — Styles des entrées dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx` : sans fond/bordure, padding moindre, familles reconnues et groupes141 ; corps, copie, états et actions conservés. Résultat : assertions styles et contenu GREEN. FR01/02/08/09, SC01/03.
- [x] T004 [US2] — Intégration sortante dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx`, sur PlainWorkEntryRow : logo/toggle existants, ligne neutre, données complètes au dépliage et erreurs natives. Aucun nouveau panneau Sources. Résultat : sorties compactes reconnues, autres natives. FR03–09, SC02–04.
- [x] T005 [US3] — Interactions et régressions dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.test.ts` et `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.test.tsx` : clavier/focus/toggle/ancrage, copie entrante exacte, valeurs JSON sorties accessibles, ordinarytools/messages inchangés. Résultat : deux suites ciblées GREEN sans assertions uniquement snapshot. FR02/04/07–10, SC02–05.

<a id="extrait-4"></a>

## Extrait 4

- [x] T009 [US4] — Fixer le préfixe strict, les frontières explicites de note utilisateur, la résolution sûre des noms et la politique streaming dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/plan.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/contracts/discreet-presentation.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/data-model.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/reuse-audit.md` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/143-echanges-discrets/specs/143-echanges-discrets/analyze.md`. Résultat : lecture intégrale principale puis GO vers11:12 CEST, gate/Analyze PASS documentaire avant code. FR11–16, SC06–08.
- [x] T010 [US4] — Ajouter des tests RED dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.test.ts` et `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.test.tsx` : relais strict/ambigu, notes visibles, streaming, nom/ID, souris/clavier, copie et rendu assistant conservé. Résultat reçu du principal : baseline346 PASS en4,39s, puis16 nouveaux échecs RED (14logique/2UI),346 anciens tests PASS ; preuve reçue vers11:10 CEST. FR11–16, SC06–08.
- [x] T011 [US4] — Implémenter la projection locale et son rendu dans `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.logic.ts` et `/Users/moi/11.Repositories/t3code-local/.worktrees/143-bridget-discreet/apps/web/src/components/chat/MessagesTimeline.tsx`. Ligne à gauche logo/direction/destinataire/« Entre agents », sans cadre, corps replié. Note utilisateur explicite visible, garde conservatrice références/notes/HTML et corps mixte monté CSS pour citations ; focus réutilisé, réédition texte A→B→A repliée. Résultat : gel392 GREEN et revue finale APPROVE, aucun I/O ni backend. FR11–16, SC06–08.
- [x] T012 [US4] — Rejouer les deux suites de tests ciblées et les contrôles format/lint/type/build/diff sur la révision US4 ; conserver les warnings de baseline et séparer les nouveaux compteurs du socle346. Résultat :392 PASS (283 logique/109 UI), principal5,40s à11:32:57 et relecteur6,08s à11:32:20 ; format1,361s PASS, lint0erreur/22warnings baseline, types PASS, build49,45s PASS/6141modules avec chunk warning existant, diff PASS. FR10–16, SC05–08.
- [x] T013 [US4] — Vérifier la nouvelle présentation dans le seul aperçu isolé autorisé : relais replié/déplié à gauche, note visible, noms/IDs, streaming, ordinary, clavier/focus/copie et largeur étroite. Résultat : recette native isolée PASS à11:33–11:35, focus réel et citation DOM/résolution conservés,320px de conteneur sans débordement ; copie exacte unitaire, pas clipboard hôte E2E ni viewport mobile ou navigation citation E2E. Revue APPROVE ; cleanup ownservices11:35:22 ; convergence principale CONVERGED11:39:54, tâches12/13 byte-identiques et sources inchangées. Coche après verdict en phase documentaire distincte. Aucun restart, commit ou déploiement. FR10–16, SC05–08.
