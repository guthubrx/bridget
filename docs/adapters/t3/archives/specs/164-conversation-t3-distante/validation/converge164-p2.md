# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/validation/converge164-p2.md](../../../../../../../specs/164-conversation-t3-distante/validation/converge164-p2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Douze sources164 T3, mêmes responsabilités que P1. Seuls les deux conteneurs Journal et leurs deux bancs149 changent de R4 à R5b. Aucun fichier Rust, contrat public, stockage, service ou runtime Bridget modifié. Deux fixtures MessagesTimeline corrigées par163 sont des dépendances héritées distinctes, pas deux nouvelles sources de la feature164 :

<a id="extrait-2"></a>

## Extrait 2

- /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/web/src/components/chat/MessagesTimeline.logic.test.ts — SHA-256 `77b995db090b5a17050f0fc42c66ee0903e16bc5f60bc9dea6f140f06e56befd`.
- /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/web/src/components/chat/MessagesTimeline.test.tsx — SHA-256 `ac77fe1ed9886ab24e179c31ef0b310e125a43d7c4858873ac5fc89ef10f3d13`.

<a id="extrait-3"></a>

## Extrait 3

| Élément | Source et réalisation | Oracle et preuve |
|---|---|---|
| Original et témoin web complets | /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/web/src/components/BridgetTaskJournal.tsx : lignes135–166, JSON.stringify entier sous open | /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/web/src/components/BridgetTaskJournal149.test.tsx : ligne341, « rend l’enveloppe complète et le témoin ts/v seulement après ouverture des détails » PASS |
| Original et témoin mobile complets | /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/mobile/src/features/threads/BridgetTaskJournal.tsx : lignes309–333, JSON.stringify entier sous open, texte sélectionnable | /Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx : ligne365, même intitulé PASS |
| Premier conservé / conflit metadata | Helper H57–83 et H92–115 inchangé ; ts est comparé et le premier seq conservé | Les deux oracles chargent first/conflicting avec même seq/payload et ts distinct ; dates absentes avant ouverture puis visibles ensemble avec v/session/message/payload |
| Reconnexion / lacunes / coupe | W322 et M449 fusion atomique héritée R4 ; H130–180 pruning hors fenêtre et coupe conservative | Les deux oracles de reprise partielle PASS dans les bancs R5b ; helper/réducteur R4 restent inchangés |

<a id="extrait-4"></a>

## Extrait 4

H/W/M/C/R/E/T désignent les sept chemins complets déjà définis dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p1.md`. Les quatre fichiers changés ont été relus ; huit autres sources sont identiques à R4 et leurs lectures restent valides. Les positions W/M ci-dessous sont celles de R5b.

<a id="extrait-5"></a>

## Extrait 5

| Exigence | Réalisation / preuve | État P2 |
|---|---|---|
| FR-001 | Manifest12, source T3 seulement ; deux fixtures163 identifiées séparément | Source satisfaite |
| FR-002 | W246, M537 : tâche native existante, aucun provider/tour fabriqué | Source satisfaite |
| FR-003 | H92–115,374–500 ; projection déterministe, clés stables, premier conservé | Source satisfaite |
| FR-004 | H290–325,398 : texte exact/session/livraison, oracle Unicode R4 inchangé | Source satisfaite |
| FR-005 | H301–325 ; W441/M558 pages indépendantes ; reprise R5b PASS | Source satisfaite |
| FR-006 | C inerte inchangé, W180/604, M226–247 ; Markdown R1 conservé et mobile préférences PASS | Source satisfaite, limite native conservée |
| FR-007 | H245–287 ; états command explicites, activités distinctes ; oracles R4 inchangés | Source satisfaite |
| FR-008 | W154/162 et M320/330 : enveloppes originales/témoins complets et sûrs | **Finding fermé, deux oracles R5b PASS** |
| FR-009 | W422/483, M539/588 : métier et contact distincts du suivi ; tests statut/contact PASS | Source satisfaite |
| FR-010 | W404/487, M516/638 : arrêt natif, permissions, reconnexion, parent/barre existants | Source satisfaite |
| FR-011 | W322 et M449 : events/gaps/cut atomiques ; témoins complets, abort/cursor existants | **Finding fermé, reprise et détails PASS** |
| FR-012 | H4–5,109–180, R135–168 ; W372/394, M500/501 :4096/64/100/16KiB ; plafond natif256KiB inchangé | Source satisfaite |
| FR-013 | W362/384/581 ; M489/683 : historique/résultat/copie chargé ; tests pagination/refus PASS | Source satisfaite |
| FR-014 | H commun à W/M, rendus locaux adaptés ; parité et préférences PASS | Source satisfaite ; T017 future |
| FR-015 | W135–210/M102–247 : détails repliés, Markdown/sélection/styles ; pas défilement forcé ajouté | Source présente ; recette T017 future |
| FR-016 | H pur O(n+c), merge tri unique ; memo/ref/détails différés ; E export et T liste native ciblés | Source satisfaite, minimalisme justifié |
| FR-017 | Contrat livraison fork après source163 ; T015–T020 existantes | Gate produit future, pas écart source ni nouvelle tâche |
| SC-001 | Unicode exact/rejeu et reprises préservés par tests/lecture | Source satisfaite |
| SC-002 | Textes/activités/inconnus/gaps/extraits/résultat et détails complets couverts | Source satisfaite ; recette réelle future |
| SC-003 | Métier/contact/actions/refus distincts ; tests Journal149 concernés PASS | Source satisfaite |
| SC-004 | Markdown/outil sans action ; deux plateformes et détails complets inertes | Source satisfaite ; capture native réelle future |
| SC-005 | Bornes/ordre/coupes/conflits vérifiés, zéro store/RPC ajouté | Source satisfaite |
| SC-006 | Contrôles source prouvés ; revue externe/audit/paquet/Git/G8/G9 existants à exécuter | Gate produit future |
| US1 | Texte cohérent, Markdown et activités reconnaissables | Source satisfaite |
| US2 | Statut/contact/continuité réels conservés | Source satisfaite |
| US3 | Détails complets accessibles, historique et résultat distincts | Source satisfaite après T021 |
| US4 | Projection commune et Bridget indépendant ; livraison prévue sans restart T3 | Source satisfaite ; produit futur |

<a id="extrait-6"></a>

## Extrait 6

Lecture limitée au périmètre164 et aux preuves déjà exécutées par ROOT. Aucun test/build/lint/Node/Git/service relancé. Aucun GREEN global ni constat sur les changements163R7 externes à l’UI164. Pas de recette sur appareil, paquet, install, source poussée, cleanup ou runtime T3 validé par ce rapport.
