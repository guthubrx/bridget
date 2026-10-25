# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/spec.md](../../../../../../specs/164-conversation-t3-distante/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Statut :** Implemented
**Priorité :** P1
**Périmètre :** présentation du connecteur T3/Bridget, web/desktop et mobile
**Tâches :** 22/22
**Pipeline :** Livré — G8/G9 acceptés, conservations justifiées.
**Date :** 2026-10-13
**Autorisation :** demande explicite de développement parallèle sur Cartae. Aucune nouvelle validation du périmètre n'est attendue.

<a id="extrait-2"></a>

## Extrait 2

Le journal d'une tâche distante apparaît comme une suite de blocs JSON. Chaque fragment de texte occupe une ligne avec sa date et sa séquence. Il devient difficile de lire le raisonnement, les commandes et le résultat. L'utilisateur veut consulter ce travail comme une conversation T3 familière, sans rendre Bridget dépendant de T3.

<a id="extrait-3"></a>

## Extrait 3

Le connecteur client T3 transforme le journal natif déjà lu en blocs de conversation. Il regroupe les fragments de texte compatibles, affiche le Markdown avec les primitives existantes et rend les commandes lisibles dans des blocs repliables. Les données originales restent consultables. Le fil virtuel existant et les actions natives restent les seules autorités. La vue demeure en lecture seule.

<a id="extrait-4"></a>

## Extrait 4

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

<a id="extrait-5"></a>

## Extrait 5

- [ ] **SC-001.** Les fragments `"workflow"`, `" Spec"`, `"Kit"`, `" et"` d'une même suite compatible donnent exactement `"workflow SpecKit et"` dans un seul bloc lisible. Rejeu et reconnexion ne le doublent pas.
- [ ] **SC-002.** Les fixtures texte/commande/erreur/résultat ont un rendu cohérent sur web et mobile. Un événement inconnu demeure consultable. Les lacunes, extraits et résultats incomplets restent visibles.
- [ ] **SC-003.** Une tâche en cours, un résultat disponible et une perte de contact donnent trois situations distinctes. L'arrêt, la reconnexion, l'historique et les permissions conservent leurs comportements antérieurs.
- [ ] **SC-004.** La lecture d'un journal ne lance aucune commande ni aucun fournisseur. Un chemin Cartae ne devient pas un lien vers un fichier du Mac. Les charges Markdown/HTML et outils sont rendues sans action implicite.
- [ ] **SC-005.** Une fenêtre de 4 096 événements reste bornée et conserve l'ordre exact après regroupement, pagination et rejeu. Aucun état durable ou RPC supplémentaire n'est ajouté pour la présentation.
- [ ] **SC-006.** Les contrôles ciblés requis passent ou leurs limites réelles sont explicitement traitées avant livraison. Les sources sont intégrées et poussées, le paquet est signé et installé, les processus T3 existants sont inchangés et le nettoyage 164 est attesté.

<a id="extrait-6"></a>

## Extrait 6

Composeur distant, messagerie bidirectionnelle, nouvelles actions fournisseur, nouvelle admission Bridget, changement de placement ou de décision163, nouvel événement natif, modification des journaux Bridget, capture d'un historique illimité, réécriture de MessagesTimeline, nouvelle bibliothèque Markdown, nouveau service de synchronisation, interface de flotte, redémarrage T3.

<a id="extrait-7"></a>

## Extrait 7

- **SPEC149** : fils virtuels Lineage, lectures natives, journal durable, actions protégées et lecture seule.
- **SPEC162** : exécution distante et placement affiché.
- **SPEC163** : livraison T3 à intégrer avant 164 ; la fonctionnalité164 ne dépend pas du moteur de décision163. Ses fichiers sont isolés.

<a id="extrait-8"></a>

## Extrait 8

Le transfert R3 porte sur douze fichiers T3, dont `/Users/moi/11.Repositories/t3code-local/.worktrees/164-conversation-t3-distante/vite.config.ts`. Ce dernier ajoute seulement le conteneur Markdown mobile à la liste existante qui autorise `allowUniwindTheme`. Les props du renderer Markdown natif ne peuvent pas consommer `className`. La règle garde sa sévérité `error` ; aucune règle globale ni dépendance ne change. Le plan contient l’inventaire complet. Aucun fichier Rust, protocole ou stockage Bridget n’entre dans ce périmètre.

<a id="extrait-9"></a>

## Extrait 9

Source R4 relue et TypeScript web final RC0 en12,8336s. **NON CONVERGED** : une correction T021 de FR-008/FR-011 est ajoutée. Les détails omettent v/ts et le témoin concurrent rend seulement son payload ; afficher les enveloppes originales complètes dans les détails existants. La reconnexion/gaps/cut est résolue. Source/tests détaillés : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p1.md`. In Progress2/21 ; aucun critère coché ni livraison présumée.

<a id="extrait-10"></a>

## Extrait 10

Rapport `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p2.md`. Prochaine gate T014 contre-revue externe/audit, puis T015–T020 source163/Git/paquet/recette/install/cleanup. Recette web/mobile/native, livraison et runtime restent futurs. Aucun test/service/Git/code relancé ou modifié par cette lecture documentaire.

<a id="extrait-11"></a>

## Extrait 11

APPROVE_WITH_CHANGES reçu sur R5b. F1MEDIUM classification par propriété héritée, F2LOW compteur de replay et F3LOW coupe certaine de préfixe sont confirmés et portés par T022. F4warnings React restent non bloquants. P2 CONVERGED sur R5b conservé historiquement ; la nouvelle source R6 et ses contrôles restent à attester. **In Progress2/22**, aucun critère ni tâche coché. Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r1.md`. Aucun P3, recette, livraison ou runtime futur annoncé.

<a id="extrait-12"></a>

## Extrait 12

Rapport : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md`. P1/P2 ettasks byte-identiques pendantP3 ; casesROOTtoujours2/22. Contre-revueR2/deltaaudit/T014complète puis Git/QA/paquet/installG8G9 futurs. Aucun succès produit/runtime déduit.

<a id="extrait-13"></a>

## Extrait 13

**P3 CONVERGED sur la source ; R2 APPROVE.** F1/F2/F3 sont fermées. Le compteur représente des occurrences, pas des versions uniques. F4 est accepté et non relu. Rapport et limites : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`.

<a id="extrait-14"></a>

## Extrait 14

Audit incrémental de six sources,309lignes lues : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01`. GradeA100 ciblé,0finding actif ; validateur canonique RC0,0erreur/0warning. AuditR5b, P1/P2 et tasks conservés byte-identiques. Aucun nouveau test ni contrôle source relancé. Les28PASS et trois TypeScript RC0 proviennent du manifeste finalR6, pas de la lecture documentaire.

<a id="extrait-15"></a>

## Extrait 15

Les reçus restent historiques et byte-identiques : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/converge164-p3.md` décrit le compteur2/22 au moment de sa lecture ; il ne sert pas de compteur courant. Preuves de source finale : `/Users/moi/.cache/bridget-delivery164-20261013/tests/validation164-r6-final.json` et `/Users/moi/.cache/bridget-delivery164-20261013/remote-code/source-transfer164-r6-final.json`. Contre-revue : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/specs/164-conversation-t3-distante/validation/code-review164-r2.md`. Audit : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/164-conversation-t3-distante/audits/2026-10-14/session-2026-10-14-spec-164-delta-r6-01/validator-receipt.json`.

<a id="extrait-16"></a>

## Extrait 16

**In Progress,19/22.** ROOT accepte les trois preuves réelles : intégration/push source T3, paquet signé et recette S164-12 du client isolé. T015/T016/T017 sont cochées. T018 attend la preuve G8 indépendante de ROOT ; T019/G9 et T020 restent ouverts. L'installation sur disque ne prouve pas l'activation du code dans les processus T3.

<a id="extrait-17"></a>

## Extrait 17

Source T3 intégrée/poussée : `511190beb307e0952066fe1a4b511cb9dd7adb18`, commit source164 `966f296ac70c3134bc6e91a722cfe86e726f50e3`, fork `refs/heads/local/main-20261009`. Reçu : `/Users/moi/.cache/bridget-delivery164-20261013/git-t3-source164-pushed.json`, SHA-256 `e7685a463e56580b511caa1afb76eb1fa6d7c53612025122506088be677c0259`.

<a id="extrait-18"></a>

## Extrait 18

Paquet commun163/164 version0.0.45-local.164 signé, une compilation native. Reçu : `/Users/moi/.cache/bridget-delivery164-20261013/t3-package164-result.json`, SHA-256 `2fb1b8f95140f09676d107f2429be07322c7ff77ff094993f1a82f95dff18984`. R1 conserve2574PASS/2FAIL des oracles ZIP ; R2 rejoue seulement les deux oracles affectés,2PASS/0FAIL. Aucun nouveau succès global2576 annoncé.

<a id="extrait-19"></a>

## Extrait 19

Recette S164-12 : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/t017164-final.json`, SHA-256 `fcf03a0e7f86ca30b13497df82a0fd9b512232e5db516eb0bd11f22fb9d47f6f`. Web réel issu du paquet final, données synthétiques, sept captures. Archive66membres vérifiés : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/ui-recipe164-evidence.tar.gz`, SHA-256 `ef4fa8ed527f20ccbc1ba89d467bf02096b67112eae258c1ccd1173786c6cd99`. Viewports1280×900 et390×844 ; le second reste un rendu web mobile, pas une recette React Native. Aucun redémarrage Electron/T3, modèle ou agent réel utilisé dans la recette.

<a id="extrait-20"></a>

## Extrait 20

Installation signée sur disque : `/Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json`, SHA-256 `b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0`. Application `/Applications/T3 Code (Local).app` en0.0.45-local.164 ; app.asar SHA-256 `6eb2fbdfed44a603d0a66d81d92fadd6d3486ff1ff43a880fe788c7d39fc923f`. PID704/969 et dates de naissance inchangés. Le code162 reste chargé selon le checkpoint ROOT. Pas d'activation différée ni de faux handshake G8.

<a id="extrait-21"></a>

## Extrait 21

T3 : source `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée vers le fork/local/main-20261009. Application signée `/Applications/T3 Code (Local).app` installée en0.0.45-local.164 ; PID704/969 et naissances inchangés. Code162 encore chargé, aucun redémarrage ni activation différée. Bridget demeure natif Bridget.

<a id="extrait-22"></a>

## Extrait 22

Nettoyage T3 WT164 et branche attesté : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t3wt164-fresh-stable-retirement.json`, SHA-256 `b0661c19c7243969d90035dfd8a9de062e2a823e5ae2183c78ac8fa6553f6b1b` ; retrait sans force après source intégrée/poussée et garde lsof complète,321991571octets logiques. Les anciens chemins WT164 sont historiques. G9 final : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json`, SHA-256 `08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3`. Onze ressources retirées : trois worktrees, deux stages Mac et six namespaces TMP ; trois branches retirées. Le WT T3 de163 et sa branche sont conservés : inventaire complet282139fichiers réussi, puis inventaire exact changé après archive. La garde refuse le retrait ; aucune cause de changement inventée, aucun contournement.

<a id="extrait-23"></a>

## Extrait 23

Tests sources :28PASSR6 distincts des10réducteurR4 et8MarkdownR1 conservés, soit46preuves de tests qualifiées par provenance, pas une nouvelle campagne46 ni un secondFULL. L'échec historique de rejeu/réalignement reste historique ; sa correction est couverte par l'oracleR6, aucun replay supplémentaire déduit de ce texte. Paquet :2574PASS/2ZIPFAILR1 conservés puis deux seuls oracles ZIP PASSR2 après réarchivage ; pas de nouveauGREEN2576. P1/P2/P3, contre-revues et audits ciblés restent inchangés ; aucun succès global/runtime de T3 inventé.

<a id="extrait-24"></a>

## Extrait 24

Les caches `/Users/moi/.cache/t3-final162.2nosz6zd` et `/Users/moi/.cache/t3-final163.zsb7fmfu` sont conservés pour leurs bases SQLite persistantes réelles,20480octets chacune, aux chemins `/Users/moi/.cache/t3-final162.2nosz6zd/source/node_modules/.vite/task-cache/v18/cache.db` et `/Users/moi/.cache/t3-final163.zsb7fmfu/source/node_modules/.vite/task-cache/v18/cache.db`. Ces dépendances ne sont pas conservées sous prétexte d'un futur build163. L'ancien échange `/Applications/.T3 Code (Local).app.new-163` reste utilisé par les processus T3 protégés ; le rollback et les preuves restent conservés. Cache Cargo partagé `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug` conservé, permis lourd rendu ; aucun nouveau FULL ou build163 en attente prétendu. Les trois namespaces SQLite `/tmp/b163.uqd3443p`, `/tmp/b163.dxmfq_la` et `/tmp/b163.07di472q` sont protégés par leurs attestations historiques ; leur existence actuelle n'est pas revendiquée. Aucun gain physique ni modification DB de production revendiqué.

<a id="extrait-25"></a>

## Extrait 25

Le build actif Bridget est toujours source `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` ; le commit de métadonnéesc57 est distinct et ne prétend pas une nouvelle compilation. Source T3 `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée au fork/local/main-20261009, application164 signée sur disque. T3PID704/969 reste chargé en162, sans redémarrage ni activation différée.
