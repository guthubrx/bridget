# Converge164 — Passage1 sur source R4

**Verdict : NON CONVERGED. Une tâche corrective T021 ajoutée.**
**Statut : In Progress, 2/21.** Aucun cochage d’implémentation, de recette ou de livraison. ROOT confirme le finding sur lecture fraîche.

## Entrées et méthode

Fallback manuel de Phase6 : lecture spec/plan/tasks/checklist/contrat, puis code réel et preuves existantes. Deux lectures : couverture des exigences, puis fidélité des détails et limites. Aucun test, compilateur, lint, build, Git ou service lancé par cette revue ; seules les écritures documentaires autorisées suivent la lecture. Les trois correctifs précédents (frontières normales, Markdown natif et reconnexion/gaps/cut) ont été relus, pas présumés.

Source : `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r4-final.json`, SHA-256 `0814d096b28b5281fa640b28942b234ad9d5b11a018415be31a6006f8bc1d641`. Douze sources exactes, vérifiées avant/après la lecture ; six deltas R3/R4. Les deux fixtures163 rapprochées restent une dépendance distincte du périmètre164.

Preuves contrôlées :

- `/Users/moi/.cache/bridget-delivery164-20261013/tests/affected-test-results164-r4.json` : 36 PASS/0 FAIL, projection7/réducteur10/web14/mobile5.
- `/Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-r4-summary.json` : format final RC0, lint15 avertissements/0 erreur, TypeScript runtime/mobile RC0.
- `/Users/moi/.cache/bridget-delivery164-20261013/tests/types-web164-final.json` : SHA-256 `7e3bf25a886c286d775b58832ac9f1cdd767e12747552c96b5d06708a6257b3a`, RC0 en12,8336s, une seule commande, log vide. Douze sources et métadonnées54 inchangées avant/après.
- `/Users/moi/.cache/bridget-delivery164-20261013/tests/web164-r1.json` : huit tests Markdown/permissions dans le groupe web21 R1 conservés, pas rejoués en R4 ; ces deux sources Markdown sont inchangées depuis R3. Ne pas ajouter8 au total36 comme une nouvelle campagne44.
- `/Users/moi/.cache/bridget-delivery164-20261013/tests/format-semantic-qualified164-r4.json` : équivalence des expressions et enfants JSX rendus qualifiée par Babel7.29.7 RC0. La garde AST stricte RC1 est conservée ; la qualification n’est ni égalité AST stricte ni preuve d’exécution native.

## Sources pour la matrice

Les références H/W/M/C/R/E/T ci-dessous désignent ces chemins complets :

- H : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/packages/client-runtime/src/bridgetJournalPresentation.ts
- W : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/web/src/components/BridgetTaskJournal.tsx
- M : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/mobile/src/features/threads/BridgetTaskJournal.tsx
- C : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/web/src/components/ChatMarkdown.tsx
- R : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/packages/client-runtime/src/state/orchestration.ts
- E : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/packages/client-runtime/package.json
- T : /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/vite.config.ts

Oracles exacts et résultats : fichiers/noms dans `/Users/moi/.cache/bridget-delivery164-20261013/tests/affected-test-results164-r4.json`. Les noms abrégés ci-dessous reprennent leurs intitulés uniques.

## Couverture des17FR

| Exigence | Réalisation source | Preuve ou gate | Verdict P1 |
|---|---|---|---|
| FR-001 | Douze sources T3 seules ; H1 importe les contrats T3, aucun Rust | Inventaire de transfert et diff relu ; aucune dépendance publique neuve | Satisfaite par source |
| FR-002 | W260–276, M551–553 : tâche native existante ; aucun tour/provider fabriqué | « présente les fragments exacts et les activités sans fabriquer un tour fournisseur » | Satisfaite par source |
| FR-003 | H92–115,374–460,468–500 ; clés au premier seq et projection commune | « garde le premier rejeu, expose un conflit et réemploie les blocs clos » | Satisfaite par source |
| FR-004 | H290–294,301–325,398 ; join sans séparateur, domaine session/livraison | « concatène les fragments Unicode exacts sans séparateur inventé » | Satisfaite par source |
| FR-005 | H301–325 ; W455–464 ; M572–581 | « distingue les frontières de livraison et session des extraits réellement coupés », deux oracles de reprise partielle | Satisfaite par source |
| FR-006 | C2426,2444,2727,2927,3203,3341,3452 ; W194/618 ; M75–94,226–247 | Markdown inerte R1 et mobile « rend titres, code et liens dans le lecteur sélectionnable aux préférences T3 » ; recette future | Satisfaite par source, limite native conservée |
| FR-007 | H245–287 ; W195–213 ; M279–299 | « ne corrèle pas started et completed et n’invente aucun succès » | Satisfaite par source |
| FR-008 | H399 garde les références originales ; W166–176 et M334–344 omettent v/ts au rendu | Les oracles n’ouvrent pas un original complet ni un conflit de métadonnée | **Écart C164-01/T021** |
| FR-009 | W436–437,485–497 ; M551–624 : métier/contact séparés de caughtUp | Tests statut/panne et métadonnées distantes | Satisfaite par source |
| FR-010 | W418–434,499–525 ; M530–550,652–695 ; barre/parent existants hors delta | Tests annulation native/refus, aucun tour T3 ; lecture seule conservée | Satisfaite par source |
| FR-011 | W325–355 ; M452–482 : abort/cursor et fusion atomique ; H57–83 compare métadonnées | Reprises aprèsSeq5 PASS ; conflits détectés mais témoin incomplet à l’écran | **Écart C164-01/T021** |
| FR-012 | H4–5,109–115,130–180 ; R135–168 ; W380–408 ; M512–515 | Borne4096, gap64, pruning hors fenêtre, headCut sticky ; tests pression/éviction | Satisfaite par source |
| FR-013 | W398–413,595–634 ; M503–525,697–729 | Tests résultat16KiB/pagination100/copie chargé/refus | Satisfaite par source |
| FR-014 | H partagé importé par W/M ; rendu adapté à chaque plateforme | Tests projection commune et deux composants ; recette T017 future | Satisfaite par source |
| FR-015 | W139–224,530–634 ; M102–247,254–349,596–735 | Styles/préférences/sélection et détail replié ; navigation, largeur et rendu réels attendent S164-12 | Source présente, recette future |
| FR-016 | H40–115,374–500 ; W139/185/438 ; M226/254/555 ; E export seulement | Pureté, mémo/identités, montage brut différé ; complexité ci-dessous | Satisfaite par source |
| FR-017 | Aucun Git/paquet/runtime modifié par la feature à ce stade | T015–T020 existantes : après source163 validée/intégrée/poussée, push fork, signature/install/cleanup | Gate de livraison future, pas nouvelle tâche |

## Couverture des6SC et4US

| Critère | Preuve / limite P1 |
|---|---|
| SC-001 | Unicode exact, premier rejeu conservé, reprises web/mobile aprèsSeq5 sans fusion à travers G1/G2 |
| SC-002 | Texte/activité/inconnu/gap/résultat présents et testés ; détails originaux incomplets → T021. Recette visuelle distincte encore future |
| SC-003 | États métier, résultat et contact distincts ; actions protégées conservées, oracles existants PASS |
| SC-004 | Rendu inerte relu, oracle Markdown conservé et mobile stub ; absence d’I/O nouvelle. Recette intégrée T017 future |
| SC-005 | Fenêtres et ordre bornés ; retention/cut/replay vérifiés par code et36 PASS ; aucune persistance/RPC nouveau |
| SC-006 | Contrôles ciblés et trois TypeScript prouvés. Contre-revue/audit/T015–T020 restent futurs ; aucun succès de livraison présumé |
| US1 | Segments cohérents et outils reconnus ; source/test présents |
| US2 | État/contact/continuité préservés ; finding de reconnexion fermé sur R4 |
| US3 | Historique et résultat présents ; défaut d’accès à l’original complet → T021 |
| US4 | Projection commune, clients adaptés, Bridget indépendant ; paquet et recette future |

## Finding actif C164-01

**FR-008/FR-011, T007, SC-002/US3.** Les deux vues reconstruisent seulement seq/session_id/event/message_id/payload. Les champs natifs v et ts sont retenus en mémoire mais ne sont pas consultables. Le témoin conflictuel rend seulement son payload. Or H57–69 compare aussi ts/session_id/event/message_id : une séquence répétée avec même payload et ts différent est détectée, mais le lecteur ne peut pas examiner cette différence.

Correction minimale, confirmée par ROOT : afficher l’enveloppe originale complète et le témoin concurrent complet sous la disclosure existante, via JSON.stringify(event,null,2). Garder le premier événement, ses références et les bornes actuelles. Aucun rendu actif ni nouvelle persistance. Étendre les deux bancs Journal149 existants pour ouvrir le détail, vérifier v/ts et distinguer premier/témoin pour un conflit de ts seulement. Quatre fichiers code/test autorisés ; aucun nouveau banc ou helper. Rejouer seulement ces deux bancs et les contrôles affectés autorisés par ROOT, pas les36 tests ni la suite Markdown inchangée.

T021 est ajoutée sous Convergence. Les vingt tâches initiales et leurs cases restent conservées. Refaire T013 sur la source corrective stable et ses preuves ; la contre-revue externe/audit T014 suit seulement la convergence finale.

## Minimalisme, complexité et hypothèses

La projection pure coûte O(n+c) sur la fenêtre4096, avec au plus64 lacunes ; les fragments sont joints une fois, sans copie profonde. La fusion coûte O((n+m)log(n+m)) car elle trie une seule fois les séquences reçues hors ordre. Le rapprochement de lacunes est borné par64, la coupe n’a que deux bornes. Les blocs clos gardent leurs références quand le préfixe et le contexte ne changent ; un contexte de lacune modifié justifie une reprojection. Les détails JSON sont montés seulement ouverts. Aucune performance native mesurée n’est affirmée.

Les nouvelles unités restent la projection/merge commune et son oracle, justifiées par deux clients et la contrainte de parité. Les adaptations web/mobile portent des règles de sécurité et les préférences natives existantes ; aucun framework, fournisseur, runtime, service ou bibliothèque ajouté. L’export E porte seulement le sous-chemin. T ajoute une seule entrée dans la liste interop native avec sévérité error inchangée, car les props Markdown natives ne prennent pas className.

Hypothèses : séquences propres au journal d’une tâche, enveloppes natives validées par les contrats actuels, lacunes explicites seules faisant autorité. Le journal ne prouve pas des tours fournisseur ni un statut distant frais. Aucune donnée média/outil ne s’exécute. Aucun import T3 n’entre dans Bridget.

## Limites et prochaine action

36 PASS de tests ciblés ne prouvent ni rendu natif iOS/Android, ni capture Electron, ni paquet installé. AST stricte RC1 et qualification Babel RC0 restent deux faits distincts. Aucun GREEN global, recette, audit code final, push, installation, nettoyage ou changement de runtime T3 déduit.

Prochaine action : ROOT fait exécuter T021 sur Cartae, recueille les deux bancs affectés et un transfert stable, puis relance la lecture Converge. Les gates dépendantes163/livraison164 continuent en parallèle dans leurs périmètres.
