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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/validation/converge164-p1.md#extrait-1).

Oracles exacts et résultats : fichiers/noms dans `/Users/moi/.cache/bridget-delivery164-20261013/tests/affected-test-results164-r4.json`. Les noms abrégés ci-dessous reprennent leurs intitulés uniques.

## Couverture des17FR

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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/validation/converge164-p1.md#extrait-2).

## Limites et prochaine action

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/validation/converge164-p1.md#extrait-3).

Prochaine action : ROOT fait exécuter T021 sur Cartae, recueille les deux bancs affectés et un transfert stable, puis relance la lecture Converge. Les gates dépendantes163/livraison164 continuent en parallèle dans leurs périmètres.
