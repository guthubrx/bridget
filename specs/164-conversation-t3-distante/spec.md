# SPEC-164 — Conversations adaptateur de bureau des tâches Bridget distantes

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-1).

## Problème

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-2).

## Solution

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-3).

## Histoires utilisateur

- **US1 — Lire le travail distant.** Je lis des messages cohérents, du Markdown et des commandes reconnaissables, au lieu de fragments JSON.
- **US2 — Comprendre l'état réel.** Je vois la machine, le moteur, le statut et le dernier contact. Une interruption du journal ne ressemble pas à une fin de tâche.
- **US3 — Retrouver les preuves.** Je consulte les données originales, l'historique paginé et le résultat durable. Je peux revenir au suivi sans perdre les contrôles existants.
- **US4 — Utiliser chaque client.** Je bénéficie du même regroupement sur web, desktop et mobile. Bridget fonctionne toujours par le parcours natif Bridget.

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-4).

## Critères d'acceptation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-5).

## Hors scope

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-6).

## Dépendances

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-7).

## Limites

Le journal n'est pas un contrat de conversation fournisseur. Une forme inconnue n'autorise aucune inférence. Sans identifiant explicite, seuls les segments textuels contigus sont regroupés. La vue peut afficher un extrait, et le résultat natif reste la preuve finale. L'installation sans redémarrage livre le paquet sur disque ; l'interface active adoptera ce paquet à la prochaine relance autorisée.

## Périmètre source attesté R3

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-8).

Le checkpoint de preuve distingue R3, le retour code R4 et les contrôles R4. Les gates TypeScript web, relecture du transfert R4, Converge, recette et livraison restent à traiter. Ces checkpoints ne cochent aucun critère ni tâche ; progression inchangée **In Progress, 2/20**.

## Checkpoint Converge passage1

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-9).

## Checkpoint P2 — convergence source R5b

**CONVERGED sur source, zéro tâche ajoutée. In Progress2/21 cases acceptées au checkpoint.** C164-01/T021 fermé par enveloppes originales et témoins entiers, rendu seulement après ouverture, oracles web/mobile v1 conformes et ts distincts. Passage1 NON CONVERGED conservé. ROOT coche après acceptation ; aucune case modifiée ici.

Source12 Mac/snapshot/remote stable selon `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r5b-precheck.json` SHA-256 `27e132d5f6ab1c2cdf2e362319b04314032c56991968d3960188c9b6f95f2228`. Deux fixtures MessagesTimeline héritées163 restent distinctes des douze sources164 et protégées.

Contrôles `/Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-r5b-summary.json` SHA-256 `3c09f3ec97c0e5eeb3417481d5e3f23ec37769c9a14c307d68ecddd8b036f1f6` :21PASS/0FAIL(15web6mobile), format4/AST stricte/qualification Babel RC0, lint4 RC0 avec8avertissements/0erreur, TypeScript web1,8040s/mobile3,0063s RC0. Les sources/métadonnées54/deux fixtures163 restent inchangées pendant les contrôles. Aucun replay des36, du helper/réducteur/runtimeTC ou Markdown inchangé. Ne pas présenter les preuves conservées comme une nouvelle suite totale.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-10).

## Checkpoint contre-revue R1 et correction R6

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-11).

## Checkpoint R6 postformat/P3 — source corrigée et prouvée

R6 final Cartae direct, transféré Mac et stable14empreintes. Manifest `/Users/moi/.cache/bridget-delivery164-20261013/tests/validation164-r6-final.json` SHA-256 `10ba64b098085bb53cef6669f53d02bae7919fa86482d03a99f1b19aa9efa89c`. Source corrective directe selon source164-r6-code.json ; missionc5ce2e7a-f726-44c4-8b87-bcc94e926005 cancelled, fournisseur natif non auteur des edits. Les anciennes admissions/DND restent historiques.

**ConvergeP3CONVERGED sursource, aucune nouvelle tâche.** F1/F2/F3 etT022 corrigés/prouvés ; C164-01/T021 conservé fermé.28PASS7helper/15web/6mobile, TypeScriptruntime/web/mobileRC0, format6/ASTstricte/lint6RC0 ; Babelnonexécuté.9warnings/0erreur, huitReactinchangés et no-accumulating-spread helper déjàprésentR4. Leshashespostformat tests/TC3restentstables et correspondentau transfert ; aucuncontrôle relancé par DOC164.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-12).

## Checkpoint final R6 — P3 et revue indépendante R2

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-13).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-14).

ROOT reste responsable de l'acceptation des cases. Progression documentaire2/22 inchangée. T015–T020, recette UI/native, Git/paquet/installation/cleanup/G8/G9 restent futurs. Aucun succès produit ou runtime présumé.

## Acceptation ROOT — source R6 validée, 2026-10-14

ROOT a accepté les preuves P3 CONVERGED, revue indépendante R2 APPROVE, audit incrémental et validateur canonique RC0. Sur cette instruction explicite, T003–T014 et T021/T022 sont cochées. T001/T002 étaient déjà acceptées. **Progression actuelle : In Progress,16/22.**

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-15).

Cette acceptation ne relance ni ne réatteste les tests. T015–T020 restent non cochées : intégration Git, paquet, recette UI, installation, nettoyage et reçu final. Aucun succès de livraison ou runtime n'est déduit.

## Acceptation ROOT — T015/T016/T017, 2026-10-14

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-16).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-17).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-18).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-19).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-20).

## Acceptation ROOT — G8/G9 attestés, 2026-10-14

**Progression actuelle : In Progress,21/22.** ROOT accepte G8 et G9 sur les reçus réels. T018/T019 sont cochées. T020 reste ouverte jusqu’au commit/push documentaire final par ROOT. Aucun push futur n’est attesté.

G8 opérationnel réel accepté sur source Bridget `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` : `/Users/moi/.cache/bridget-delivery163-20261013/g8-t058-operational-final.json`, SHA-256 `5b1c00923f2f307136b5831cc8d3328227b7284deacf44c793d71542e3ceb52c`. Résultat durable `RECETTE163_G8_OK`, un reçu ; OFF explicite prouvé puis replay ON de la même clé inchangé. Le replay ne constitue ni nouvelle tâche ni nouvel appel de décision. La recette HTTP de décision reste celle du laboratoire T040R4. Le provider réel prouvé par cette recette est codex-pro ; autres profils configurés sans nouvelle attestation quota/auth. ROOT atteste daemon31298/pont31420 et workerCartae1454489 actifs, trois routes autorisées Bridget/Roadie/Opus2D. Aucune inspection des capacités en mémoire ni mission de modèle distant supplémentaire déduite.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-21).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-22).

Conservations Cartae justifiées : `/Users/moi/.cache/bridget-delivery163-20261013/g9-cartae-final-usage-retentions.json`, SHA-256 `cb23362780df36d938f321c054c0fe626f0021194d65dbb78ed30f2e8852d359`. lsof RC0 mais incomplet à cause de FUSEportal `/run/user/1002/doc` ; aucun contournement. Sont conservés `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission`, sa copie `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164`, `/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target` et les cinq namespaces `/home/moi/.local/state/bridget-delivery164/tests/tmp-r2`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r3`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r4`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r5b`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r6`. Pas de retrait distant ni de gain physique revendiqué. Rollback, preuves, DB et travaux étrangers restent protégés.

S164-12 est une recette web réelle du paquet, avec données synthétiques : sept captures et archive66membres SHA vérifiés, archive `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/ui-recipe164-evidence.tar.gz` SHA-256 `ef4fa8ed527f20ccbc1ba89d467bf02096b67112eae258c1ccd1173786c6cd99`. La taille390×844 prouve le web responsive, pas React Native natif ni Electron actif.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-23).

### Conservations G9 supplémentaires

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-24).

## Clôture documentaire acceptée par ROOT — 2026-10-14

**Statut canonique : Implemented. Progression :22/22. Pipeline : Livré, G8/G9 acceptés avec conservations justifiées.** T020 est cochée sur le push documentaire réel `c57cfd35efa1338cda5256a02c57d71258d8b8d4` vers github/main. Reçu : `/Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json`, SHA-256 `56316fdcb4202865fbb57e7979a11d302bb968e2170453a9523070a490a30390` ; HEAD et référence distante égaux, racine propre à cette observation. Ce commit contient les documents de livraison G8/G9 avant le cochage final. Les cases fermées sont intégrées et poussées dans `af8998a7d7494913d4420e2dca7354a73bd8a4f5`, vérifié par ROOT sur github/main égal à HEAD et racine propre. Cartae main et worktree sont alignés sur ce commit ; archive25membres vérifiée byte-identique. Reçu : `/Users/moi/.cache/bridget-delivery163-20261013/git-final-cartae-af8998a7-command.json`. Les checkpoints21/22 précédents restent historiques.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/164-conversation-t3-distante/spec.md#extrait-25).

La recette S164-12 prouve le client web construit avec données synthétiques, en taille desktop et mobileCSS ; elle ne vaut pas Electron actif ni React Native natif. Les tests et les audits ciblés restent qualifiés par leur provenance et couverture partielle. Aucun nouveau Converge, audit, test, build ou succès global GREEN n'est créé par cette clôture. Bridget est autonome.
