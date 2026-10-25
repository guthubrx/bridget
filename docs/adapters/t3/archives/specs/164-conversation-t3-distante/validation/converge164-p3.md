# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/validation/converge164-p3.md](../../../../../../../specs/164-conversation-t3-distante/validation/converge164-p3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Racine source complète : `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante`.

<a id="extrait-2"></a>

## Extrait 2

| Finding | Source R6 relue | Preuve affectée réelle | Verdict |
|---|---|---|---|
| SEC-001/F1 — propriété héritée | /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/packages/client-runtime/src/bridgetJournalPresentation.ts :276, hasOwnProperty.call avant lookup | /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/packages/client-runtime/src/bridgetJournalPresentation.test.ts :122, oracle inconnu étendu aux __proto__/constructor/toString/valueOf/hasOwnProperty, bloc technical et original conservé | Résolu ; aucun label objet/fonction transmis au rendu |
| REL-001/F2 — même témoin | Même helper :80, comparaison au témoin courant avant incrément ; web :152, libellé occurrences conflictuelles | Même oracle helper :139–160, référence conservée et compteur0 sur replay identique, autre témoin compteur1 | Résolu ; aucune promesse de versions uniques pour un troisième témoin répété |
| REL-002/F3 — coupe de préfixe certaine | /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/web/src/components/BridgetTaskJournal.tsx :329–343 et /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/mobile/src/features/threads/BridgetTaskJournal.tsx :456–470, prefixGap depuis gaps connus/entrants avant pruning, headCut sticky | Deux oracles existants « conserve les lacunes et la coupe… » étendus : gap déclaré1..40 puis fragment41 → Extrait, même saut sans gap → pas Extrait, puis nouvelle visite afterSeq41 conserve le résultat | Résolu ; aucun gap déduit d’un saut numérique |
| C164-01/T021 | JSON entier original/témoin monté seulement ouvert, non modifié dans le delta | Deux oracles enveloppe entière v1/ts distinct passent dans les bancs R6 | Fermeture conservée |

<a id="extrait-3"></a>

## Extrait 3

| Exigence | Réalisation/proof R6 ou source inchangée | État source |
|---|---|---|
| FR-001 | Manifest12 sources T3, deux fixtures163 héritées distinguées ; zéro Rust/daemon/store/service nouveau | Satisfaite |
| FR-002 | Fil virtuel et statut natif existants, aucun tour/fournisseur fabriqué | Satisfaite |
| FR-003 | H93 : premier conservé/ordre/clé stable ; projectionH472 commune | Satisfaite |
| FR-004 | Fragments Unicode exacts joinH402 ; domaines session/livraison distincts ; oracle helper PASS | Satisfaite |
| FR-005 | H305 frontières ; pages seules ; prefixGap W/M et oracles gap/sansgap PASS | Satisfaite après F3 |
| FR-006 | C inerte et renderer natif préférences/sélection inchangés ; MarkdownR1 conservé/mobileR6 PASS | Satisfaite par code ; nativeQA future |
| FR-007 | Vocabulaire reconnu own-only H276 ; started/completed distincts, pas réussite inventée | Satisfaite |
| FR-008 | Fallback technique pour hérités/inconnus et original/témoin complets lazy ; oracles helper/web/mobile PASS | Satisfaite après F1 |
| FR-009 | Statut/contact métier séparés de caughtUp, tests Journal149R6 PASS | Satisfaite |
| FR-010 | Actions/permissions/readonly/parent/focus existants ; aucun composer ni commandement nouveau | Satisfaite |
| FR-011 | Même témoin non recompté, occurrences explicites ; fusion events/gaps/cut atomique ; reprises PASS | Satisfaite après F2 |
| FR-012 |4096/64/pages100/result16KiB+plafond natif256KiB conservés, prefixcut certain, coupe conservative | Satisfaite |
| FR-013 | Historique/résultat paginé/copie chargé/refus conservés ; testsR6 PASS | Satisfaite |
| FR-014 | Projection commune et adaptations natives ; mêmes règles de frontières/données | Satisfaite par source ; recette future |
| FR-015 | Textes/activité secondaire/détails/lazy/theme/select/scroll volontaire ; pas retour forcé ajouté | Code présent ; T017 future |
| FR-016 | Pureté, blocs clos partagés, O(n+c), tri unique et facteurs bornés ; aucun I/O/event/store public nouveau | Satisfaite |
| FR-017 | Contrat source163 validée/intégrée/poussée avant164, fork/paquet/G8/G9 | T015–T020 futures |
| SC-001 | Texte exact/rejeu/reprises testés surhelper+clientsR6 | Satisfaite par source |
| SC-002 | Formes héritées sûres et préfixes/extraits correctement distingués ; original complet | Satisfaite par source ; recette future |
| SC-003 | Métier/contact/permissions/refus/annulation distincts, testsR6 | Satisfaite par source |
| SC-004 | Inertie read-only conservée ; aucune autorité locale induite par un outil/chemin distant | Satisfaite par source ; nativeQA future |
| SC-005 | Ordre/bornes/pagination/continuité, aucune persistance/RPC supplémentaire | Satisfaite par source |
| SC-006 | Contrôles source et Converge prouvés ; T014R2/auditdelta/produit encore distincts | Gates produit futures |
| US1 | Texte cohérent et inconnus sûrs | Satisfaite source |
| US2 | État réel, gaps/reprise et préfixe incomplet visibles | Satisfaite source |
| US3 | Détails/témoins, historique et résultat accessibles sans perte | Satisfaite source |
| US4 | Deux clients, Bridget indépendant ; livraison contractuelle prévue | Satisfaite source, produit futur |

<a id="extrait-4"></a>

## Extrait 4

**P3 CONVERGED sur la source ; R2 APPROVE.** F1/F2/F3 sont fermées. Le compteur représente des occurrences, pas des versions uniques. F4 est accepté et non relu. Rapport et limites : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`.

<a id="extrait-5"></a>

## Extrait 5

Audit incrémental de six sources,309lignes lues : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01`. GradeA100 ciblé,0finding actif ; validateur canonique RC0,0erreur/0warning. AuditR5b, P1/P2 et tasks conservés byte-identiques. Aucun nouveau test ni contrôle source relancé. Les28PASS et trois TypeScript RC0 proviennent du manifeste finalR6, pas de la lecture documentaire.
