# SPEC-164 — Conversations T3 des tâches Bridget distantes

**Statut :** Implemented
**Priorité :** P1
**Périmètre :** présentation du connecteur T3/Bridget, web/desktop et mobile
**Tâches :** 22/22
**Pipeline :** Livré — G8/G9 acceptés, conservations justifiées.
**Date :** 2026-10-13
**Autorisation :** demande explicite de développement parallèle sur Cartae. Aucune nouvelle validation du périmètre n'est attendue.

## Problème

Le journal d'une tâche distante apparaît comme une suite de blocs JSON. Chaque fragment de texte occupe une ligne avec sa date et sa séquence. Il devient difficile de lire le raisonnement, les commandes et le résultat. L'utilisateur veut consulter ce travail comme une conversation T3 familière, sans rendre Bridget dépendant de T3.

## Solution

Le connecteur client T3 transforme le journal natif déjà lu en blocs de conversation. Il regroupe les fragments de texte compatibles, affiche le Markdown avec les primitives existantes et rend les commandes lisibles dans des blocs repliables. Les données originales restent consultables. Le fil virtuel existant et les actions natives restent les seules autorités. La vue demeure en lecture seule.

## Histoires utilisateur

- **US1 — Lire le travail distant.** Je lis des messages cohérents, du Markdown et des commandes reconnaissables, au lieu de fragments JSON.
- **US2 — Comprendre l'état réel.** Je vois la machine, le moteur, le statut et le dernier contact. Une interruption du journal ne ressemble pas à une fin de tâche.
- **US3 — Retrouver les preuves.** Je consulte les données originales, l'historique paginé et le résultat durable. Je peux revenir au suivi sans perdre les contrôles existants.
- **US4 — Utiliser chaque client.** Je bénéficie du même regroupement sur web, desktop et mobile. Bridget fonctionne toujours sans T3.

## Exigences fonctionnelles

- **FR-001.** Modifier seulement le connecteur et les clients T3. Aucune modification Rust, daemon, protocole, journal ou stockage Bridget. Aucun nouveau service, fournisseur, modèle ou lancement de tâche.
- **FR-002.** Réutiliser le fil virtuel Bridget existant. Ne créer ni conversation de premier niveau, ni tour fournisseur, ni session fournisseur, ni résultat T3 synthétique.
- **FR-003.** Produire une projection commune et déterministe des événements du journal. L'ordre vient de `seq`, jamais de l'heure. Les clés des blocs restent stables lorsque du texte s'ajoute.
- **FR-004.** Regrouper les fragments reconnus `kind: text` avec leur `content` exact. Ne pas ajouter d'espace, supprimer du texte ou normaliser le Markdown. Ne fusionner que des fragments contigus du même domaine de session et d'une identité de livraison compatible. Deux `message_id` présents et différents imposent une frontière. Le `message_id` est une identité de livraison Bridget, pas une identité de message fournisseur. Sa présence commune ne prouve pas un même message. Une identité absente autorise seulement un bloc de texte contigu, sans inventer de message fournisseur.
- **FR-005.** Une commande, un événement non textuel, un changement de session/livraison, une lacune ou une troncature ferme le bloc courant. Une page ne permet pas d'inventer le début ou la fin d'un message. Si le début est absent, l'afficher comme extrait.
- **FR-006.** Rendre le texte et le résultat avec le Markdown T3 existant adapté à la lecture seule. Les chemins distants sont du texte copiable. Ils ne désignent pas automatiquement le système de fichiers de la machine T3. Aucun HTML, script, action de modèle, contenu d'outil ou aperçu distant ne s'exécute.
- **FR-007.** Reconnaître les formes de commande/outil réellement présentes dans le journal. Afficher leur nom ou commande et leur sortie dans une présentation repliable cohérente avec T3. Une réussite, un échec ou une durée n'apparaît que si une preuve explicite existe dans l'événement. Ne jamais corréler deux appels par proximité, nom identique ou date.
- **FR-008.** Les événements inconnus, hérités ou mal formés restent accessibles comme détails techniques sûrs. Chaque bloc donne accès aux événements originaux et à leurs séquences. Ne supprimer aucun événement silencieusement.
- **FR-009.** Conserver le moteur, le modèle, la machine et le dernier contact natifs disponibles. Conserver les distinctions « en cours », « dernier état connu », « journal indisponible » et « résultat disponible ». Le suivi du journal ne prouve ni la disponibilité du worker ni la fin de la mission.
- **FR-010.** Garder le lien parent, l'arrêt natif contrôlé par permission, la reconnexion et les refus existants. Garder la barre de lecture seule. Ne pas ajouter de composeur, d'approbation, de reprise fournisseur ou de fork.
- **FR-011.** Conserver le suivi visible/focalisé, l'annulation des lectures à la fermeture ou au passage en arrière-plan et la reprise au curseur déjà lu. Un rejeu ne duplique ni le texte ni les outils. Une séquence répétée avec un contenu incompatible ne doit pas écraser silencieusement une preuve. Détecter ce conflit avant déduplication : garder le premier événement et un détail conflictuel borné. Signaler aussi une coupe de tête opérée par le conteneur lui-même.
- **FR-012.** Conserver les limites existantes : fenêtre vive de 4 096 événements, pages de journal de 100 événements et limites de transport inchangées, résultat par fenêtres de 16 KiB et plafond natif de 256 KiB. Signaler l'éviction et les lacunes. Ne pas construire une copie illimitée du journal dans un second store. Un historique éphémère de lacunes borné dans le réducteur existant est autorisé. La pression sur cette borne ne doit pas permettre une fusion à travers une lacune oubliée.
- **FR-013.** Conserver lecture du début, page suivante, retour au suivi, lecture et copie du résultat chargé. Le résultat durable reste identifié séparément du texte de progression. Un résultat paginé incomplet porte une indication visible et n'est pas présenté comme complet.
- **FR-014.** Appliquer le même modèle de présentation aux clients web/desktop et mobile. Les primitives natives peuvent différer. Les règles de regroupement, les refus et les données sources sont identiques.
- **FR-015.** Présenter une hiérarchie familière de conversation : texte principal lisible, activité outil secondaire, état compact, détails techniques repliés. Réutiliser les couleurs, espacements et composants T3. Conserver sélection/copie du texte, navigation clavier, largeur mobile et défilement volontaire. Ne pas ramener un lecteur au bas lorsqu'il lit l'historique.
- **FR-016.** La projection reste locale, pure et bornée. Elle n'appelle aucun RPC par événement, ne duplique pas la persistance, et ne modifie pas l'orchestrateur ni les contrats publics du journal. Le petit delta de lacunes dans l'état client existant porte seulement la continuité de lecture. Les blocs clos conservent leur identité ; les détails bruts sont montés seulement à l'ouverture. Documenter la complexité du regroupement et éviter les recherches répétées dans une boucle.
- **FR-017.** Livrer sur le fork T3 après intégration de 163, avec contrôles ciblés, contre-revue, audit, paquet signé, installation, push et nettoyage des seuls travaux 164. Aucun push upstream, aucune PR non demandée et aucun redémarrage de T3.

## Critères d'acceptation

- [ ] **SC-001.** Les fragments `"workflow"`, `" Spec"`, `"Kit"`, `" et"` d'une même suite compatible donnent exactement `"workflow SpecKit et"` dans un seul bloc lisible. Rejeu et reconnexion ne le doublent pas.
- [ ] **SC-002.** Les fixtures texte/commande/erreur/résultat ont un rendu cohérent sur web et mobile. Un événement inconnu demeure consultable. Les lacunes, extraits et résultats incomplets restent visibles.
- [ ] **SC-003.** Une tâche en cours, un résultat disponible et une perte de contact donnent trois situations distinctes. L'arrêt, la reconnexion, l'historique et les permissions conservent leurs comportements antérieurs.
- [ ] **SC-004.** La lecture d'un journal ne lance aucune commande ni aucun fournisseur. Un chemin Cartae ne devient pas un lien vers un fichier du Mac. Les charges Markdown/HTML et outils sont rendues sans action implicite.
- [ ] **SC-005.** Une fenêtre de 4 096 événements reste bornée et conserve l'ordre exact après regroupement, pagination et rejeu. Aucun état durable ou RPC supplémentaire n'est ajouté pour la présentation.
- [ ] **SC-006.** Les contrôles ciblés requis passent ou leurs limites réelles sont explicitement traitées avant livraison. Les sources sont intégrées et poussées, le paquet est signé et installé, les processus T3 existants sont inchangés et le nettoyage 164 est attesté.

## Hors scope

Composeur distant, messagerie bidirectionnelle, nouvelles actions fournisseur, nouvelle admission Bridget, changement de placement ou de décision163, nouvel événement natif, modification des journaux Bridget, capture d'un historique illimité, réécriture de MessagesTimeline, nouvelle bibliothèque Markdown, nouveau service de synchronisation, interface de flotte, redémarrage T3.

## Dépendances

- **SPEC149** : fils virtuels Lineage, lectures natives, journal durable, actions protégées et lecture seule.
- **SPEC162** : exécution distante et placement affiché.
- **SPEC163** : livraison T3 à intégrer avant 164 ; la fonctionnalité164 ne dépend pas du moteur de décision163. Ses fichiers sont isolés.

## Limites

Le journal n'est pas un contrat de conversation fournisseur. Une forme inconnue n'autorise aucune inférence. Sans identifiant explicite, seuls les segments textuels contigus sont regroupés. La vue peut afficher un extrait, et le résultat natif reste la preuve finale. L'installation sans redémarrage livre le paquet sur disque ; l'interface active adoptera ce paquet à la prochaine relance autorisée.

## Périmètre source attesté R3

Le transfert R3 porte sur douze fichiers T3, dont `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/vite.config.ts`. Ce dernier ajoute seulement le conteneur Markdown mobile à la liste existante qui autorise `allowUniwindTheme`. Les props du renderer Markdown natif ne peuvent pas consommer `className`. La règle garde sa sévérité `error` ; aucune règle globale ni dépendance ne change. Le plan contient l’inventaire complet. Aucun fichier Rust, protocole ou stockage Bridget n’entre dans ce périmètre.

Le checkpoint de preuve distingue R3, le retour code R4 et les contrôles R4. Les gates TypeScript web, relecture du transfert R4, Converge, recette et livraison restent à traiter. Ces checkpoints ne cochent aucun critère ni tâche ; progression inchangée **In Progress, 2/20**.

## Checkpoint Converge passage1

Source R4 relue et TypeScript web final RC0 en12,8336s. **NON CONVERGED** : une correction T021 de FR-008/FR-011 est ajoutée. Les détails omettent v/ts et le témoin concurrent rend seulement son payload ; afficher les enveloppes originales complètes dans les détails existants. La reconnexion/gaps/cut est résolue. Source/tests détaillés : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p1.md`. In Progress2/21 ; aucun critère coché ni livraison présumée.

## Checkpoint P2 — convergence source R5b

**CONVERGED sur source, zéro tâche ajoutée. In Progress2/21 cases acceptées au checkpoint.** C164-01/T021 fermé par enveloppes originales et témoins entiers, rendu seulement après ouverture, oracles web/mobile v1 conformes et ts distincts. Passage1 NON CONVERGED conservé. ROOT coche après acceptation ; aucune case modifiée ici.

Source12 Mac/snapshot/remote stable selon `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r5b-precheck.json` SHA-256 `27e132d5f6ab1c2cdf2e362319b04314032c56991968d3960188c9b6f95f2228`. Deux fixtures MessagesTimeline héritées163 restent distinctes des douze sources164 et protégées.

Contrôles `/Users/moi/.cache/bridget-delivery164-20261013/tests/checks164-r5b-summary.json` SHA-256 `3c09f3ec97c0e5eeb3417481d5e3f23ec37769c9a14c307d68ecddd8b036f1f6` :21PASS/0FAIL(15web6mobile), format4/AST stricte/qualification Babel RC0, lint4 RC0 avec8avertissements/0erreur, TypeScript web1,8040s/mobile3,0063s RC0. Les sources/métadonnées54/deux fixtures163 restent inchangées pendant les contrôles. Aucun replay des36, du helper/réducteur/runtimeTC ou Markdown inchangé. Ne pas présenter les preuves conservées comme une nouvelle suite totale.

Rapport `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p2.md`. Prochaine gate T014 contre-revue externe/audit, puis T015–T020 source163/Git/paquet/recette/install/cleanup. Recette web/mobile/native, livraison et runtime restent futurs. Aucun test/service/Git/code relancé ou modifié par cette lecture documentaire.

## Checkpoint contre-revue R1 et correction R6

APPROVE_WITH_CHANGES reçu sur R5b. F1MEDIUM classification par propriété héritée, F2LOW compteur de replay et F3LOW coupe certaine de préfixe sont confirmés et portés par T022. F4warnings React restent non bloquants. P2 CONVERGED sur R5b conservé historiquement ; la nouvelle source R6 et ses contrôles restent à attester. **In Progress2/22**, aucun critère ni tâche coché. Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r1.md`. Aucun P3, recette, livraison ou runtime futur annoncé.

## Checkpoint R6 postformat/P3 — source corrigée et prouvée

R6 final Cartae direct, transféré Mac et stable14empreintes. Manifest `/Users/moi/.cache/bridget-delivery164-20261013/tests/validation164-r6-final.json` SHA-256 `10ba64b098085bb53cef6669f53d02bae7919fa86482d03a99f1b19aa9efa89c`. Source corrective directe selon source164-r6-code.json ; missionc5ce2e7a-f726-44c4-8b87-bcc94e926005 cancelled, fournisseur natif non auteur des edits. Les anciennes admissions/DND restent historiques.

**ConvergeP3CONVERGED sursource, aucune nouvelle tâche.** F1/F2/F3 etT022 corrigés/prouvés ; C164-01/T021 conservé fermé.28PASS7helper/15web/6mobile, TypeScriptruntime/web/mobileRC0, format6/ASTstricte/lint6RC0 ; Babelnonexécuté.9warnings/0erreur, huitReactinchangés et no-accumulating-spread helper déjàprésentR4. Leshashespostformat tests/TC3restentstables et correspondentau transfert ; aucuncontrôle relancé par DOC164.

Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md`. P1/P2 ettasks byte-identiques pendantP3 ; casesROOTtoujours2/22. Contre-revueR2/deltaaudit/T014complète puis Git/QA/paquet/installG8G9 futurs. Aucun succès produit/runtime déduit.

## Checkpoint final R6 — P3 et revue indépendante R2

**P3 CONVERGED sur la source ; R2 APPROVE.** F1/F2/F3 sont fermées. Le compteur représente des occurrences, pas des versions uniques. F4 est accepté et non relu. Rapport et limites : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`.

Audit incrémental de six sources,309lignes lues : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01`. GradeA100 ciblé,0finding actif ; validateur canonique RC0,0erreur/0warning. AuditR5b, P1/P2 et tasks conservés byte-identiques. Aucun nouveau test ni contrôle source relancé. Les28PASS et trois TypeScript RC0 proviennent du manifeste finalR6, pas de la lecture documentaire.

ROOT reste responsable de l'acceptation des cases. Progression documentaire2/22 inchangée. T015–T020, recette UI/native, Git/paquet/installation/cleanup/G8/G9 restent futurs. Aucun succès produit ou runtime présumé.

## Acceptation ROOT — source R6 validée, 2026-10-14

ROOT a accepté les preuves P3 CONVERGED, revue indépendante R2 APPROVE, audit incrémental et validateur canonique RC0. Sur cette instruction explicite, T003–T014 et T021/T022 sont cochées. T001/T002 étaient déjà acceptées. **Progression actuelle : In Progress,16/22.**

Les reçus restent historiques et byte-identiques : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md` décrit le compteur2/22 au moment de sa lecture ; il ne sert pas de compteur courant. Preuves de source finale : `/Users/moi/.cache/bridget-delivery164-20261013/tests/validation164-r6-final.json` et `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r6-final.json`. Contre-revue : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`. Audit : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01/validator-receipt.json`.

Cette acceptation ne relance ni ne réatteste les tests. T015–T020 restent non cochées : intégration Git, paquet, recette UI, installation, nettoyage et reçu final. Aucun succès de livraison ou runtime n'est déduit.

## Acceptation ROOT — T015/T016/T017, 2026-10-14

**In Progress,19/22.** ROOT accepte les trois preuves réelles : intégration/push source T3, paquet signé et recette S164-12 du client isolé. T015/T016/T017 sont cochées. T018 attend la preuve G8 indépendante de ROOT ; T019/G9 et T020 restent ouverts. L'installation sur disque ne prouve pas l'activation du code dans les processus T3.

Source T3 intégrée/poussée : `511190beb307e0952066fe1a4b511cb9dd7adb18`, commit source164 `966f296ac70c3134bc6e91a722cfe86e726f50e3`, fork `refs/heads/local/main-20261009`. Reçu : `/Users/moi/.cache/bridget-delivery164-20261013/git-t3-source164-pushed.json`, SHA-256 `e7685a463e56580b511caa1afb76eb1fa6d7c53612025122506088be677c0259`.

Paquet commun163/164 version0.0.45-local.164 signé, une compilation native. Reçu : `/Users/moi/.cache/bridget-delivery164-20261013/t3-package164-result.json`, SHA-256 `2fb1b8f95140f09676d107f2429be07322c7ff77ff094993f1a82f95dff18984`. R1 conserve2574PASS/2FAIL des oracles ZIP ; R2 rejoue seulement les deux oracles affectés,2PASS/0FAIL. Aucun nouveau succès global2576 annoncé.

Recette S164-12 : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/t017164-final.json`, SHA-256 `fcf03a0e7f86ca30b13497df82a0fd9b512232e5db516eb0bd11f22fb9d47f6f`. Web réel issu du paquet final, données synthétiques, sept captures. Archive66membres vérifiés : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/ui-recipe164-evidence.tar.gz`, SHA-256 `ef4fa8ed527f20ccbc1ba89d467bf02096b67112eae258c1ccd1173786c6cd99`. Viewports1280×900 et390×844 ; le second reste un rendu web mobile, pas une recette React Native. Aucun redémarrage Electron/T3, modèle ou agent réel utilisé dans la recette.

Installation signée sur disque : `/Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json`, SHA-256 `b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0`. Application `/Applications/T3 Code (Local).app` en0.0.45-local.164 ; app.asar SHA-256 `6eb2fbdfed44a603d0a66d81d92fadd6d3486ff1ff43a880fe788c7d39fc923f`. PID704/969 et dates de naissance inchangés. Le code162 reste chargé selon le checkpoint ROOT. Pas d'activation différée ni de faux handshake G8.

## Acceptation ROOT — G8/G9 attestés, 2026-10-14

**Progression actuelle : In Progress,21/22.** ROOT accepte G8 et G9 sur les reçus réels. T018/T019 sont cochées. T020 reste ouverte jusqu’au commit/push documentaire final par ROOT. Aucun push futur n’est attesté.

G8 opérationnel réel accepté sur source Bridget `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` : `/Users/moi/.cache/bridget-delivery163-20261013/g8-t058-operational-final.json`, SHA-256 `5b1c00923f2f307136b5831cc8d3328227b7284deacf44c793d71542e3ceb52c`. Résultat durable `RECETTE163_G8_OK`, un reçu ; OFF explicite prouvé puis replay ON de la même clé inchangé. Le replay ne constitue ni nouvelle tâche ni nouvel appel de décision. La recette HTTP de décision reste celle du laboratoire T040R4. Le provider réel prouvé par cette recette est codex-pro ; autres profils configurés sans nouvelle attestation quota/auth. ROOT atteste daemon31298/pont31420 et workerCartae1454489 actifs, trois routes autorisées Bridget/Roadie/Opus2D. Aucune inspection des capacités en mémoire ni mission de modèle distant supplémentaire déduite.

T3 : source `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée vers le fork/local/main-20261009. Application signée `/Applications/T3 Code (Local).app` installée en0.0.45-local.164 ; PID704/969 et naissances inchangés. Code162 encore chargé, aucun redémarrage ni activation différée. Bridget demeure indépendant de T3.

Nettoyage T3 WT164 et branche attesté : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t3wt164-fresh-stable-retirement.json`, SHA-256 `b0661c19c7243969d90035dfd8a9de062e2a823e5ae2183c78ac8fa6553f6b1b` ; retrait sans force après source intégrée/poussée et garde lsof complète,321991571octets logiques. Les anciens chemins WT164 sont historiques. G9 final : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json`, SHA-256 `08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3`. Onze ressources retirées : trois worktrees, deux stages Mac et six namespaces TMP ; trois branches retirées. Le WT T3 de163 et sa branche sont conservés : inventaire complet282139fichiers réussi, puis inventaire exact changé après archive. La garde refuse le retrait ; aucune cause de changement inventée, aucun contournement.

Conservations Cartae justifiées : `/Users/moi/.cache/bridget-delivery163-20261013/g9-cartae-final-usage-retentions.json`, SHA-256 `cb23362780df36d938f321c054c0fe626f0021194d65dbb78ed30f2e8852d359`. lsof RC0 mais incomplet à cause de FUSEportal `/run/user/1002/doc` ; aucun contournement. Sont conservés `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission`, sa copie `/home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164`, `/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target` et les cinq namespaces `/home/moi/.local/state/bridget-delivery164/tests/tmp-r2`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r3`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r4`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r5b`, `/home/moi/.local/state/bridget-delivery164/tests/tmp-r6`. Pas de retrait distant ni de gain physique revendiqué. Rollback, preuves, DB et travaux étrangers restent protégés.

S164-12 est une recette web réelle du paquet, avec données synthétiques : sept captures et archive66membres SHA vérifiés, archive `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/ui-recipe164-evidence.tar.gz` SHA-256 `ef4fa8ed527f20ccbc1ba89d467bf02096b67112eae258c1ccd1173786c6cd99`. La taille390×844 prouve le web responsive, pas React Native natif ni Electron actif.

Tests sources :28PASSR6 distincts des10réducteurR4 et8MarkdownR1 conservés, soit46preuves de tests qualifiées par provenance, pas une nouvelle campagne46 ni un secondFULL. L'échec historique de rejeu/réalignement reste historique ; sa correction est couverte par l'oracleR6, aucun replay supplémentaire déduit de ce texte. Paquet :2574PASS/2ZIPFAILR1 conservés puis deux seuls oracles ZIP PASSR2 après réarchivage ; pas de nouveauGREEN2576. P1/P2/P3, contre-revues et audits ciblés restent inchangés ; aucun succès global/runtime de T3 inventé.

### Conservations G9 supplémentaires

Les caches `/Users/moi/.cache/t3-final162.2nosz6zd` et `/Users/moi/.cache/t3-final163.zsb7fmfu` sont conservés pour leurs bases SQLite persistantes réelles,20480octets chacune, aux chemins `/Users/moi/.cache/t3-final162.2nosz6zd/source/node_modules/.vite/task-cache/v18/cache.db` et `/Users/moi/.cache/t3-final163.zsb7fmfu/source/node_modules/.vite/task-cache/v18/cache.db`. Ces dépendances ne sont pas conservées sous prétexte d'un futur build163. L'ancien échange `/Applications/.T3 Code (Local).app.new-163` reste utilisé par les processus T3 protégés ; le rollback et les preuves restent conservés. Cache Cargo partagé `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug` conservé, permis lourd rendu ; aucun nouveau FULL ou build163 en attente prétendu. Les trois namespaces SQLite `/tmp/b163.uqd3443p`, `/tmp/b163.dxmfq_la` et `/tmp/b163.07di472q` sont protégés par leurs attestations historiques ; leur existence actuelle n'est pas revendiquée. Aucun gain physique ni modification DB de production revendiqué.

## Clôture documentaire acceptée par ROOT — 2026-10-14

**Statut canonique : Implemented. Progression :22/22. Pipeline : Livré, G8/G9 acceptés avec conservations justifiées.** T020 est cochée sur le push documentaire réel `c57cfd35efa1338cda5256a02c57d71258d8b8d4` vers github/main. Reçu : `/Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json`, SHA-256 `56316fdcb4202865fbb57e7979a11d302bb968e2170453a9523070a490a30390` ; HEAD et référence distante égaux, racine propre à cette observation. Ce commit contient les documents de livraison G8/G9 avant le cochage final. Le dernier commit de ce cochage et son push seront réalisés par ROOT ; aucun hash futur n'est présenté comme acquis. Les checkpoints21/22 précédents restent historiques.

Le build actif Bridget est toujours source `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` ; le commit de métadonnéesc57 est distinct et ne prétend pas une nouvelle compilation. Source T3 `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée au fork/local/main-20261009, application164 signée sur disque. T3PID704/969 reste chargé en162, sans redémarrage ni activation différée.

La recette S164-12 prouve le client web construit avec données synthétiques, en taille desktop et mobileCSS ; elle ne vaut pas Electron actif ni React Native natif. Les tests et les audits ciblés restent qualifiés par leur provenance et couverture partielle. Aucun nouveau Converge, audit, test, build ou succès global GREEN n'est créé par cette clôture. Bridget reste indépendant de T3.
