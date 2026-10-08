# Analyse de cohérence — SPEC147

Date : 2026-10-08 ; lot US1–US5 validé en isolé le 2026-10-09, puis réouverture approuvée US6/US7. Statut courant : In Progress. Les analyses précédentes restent les preuves du premier lot, pas une validation anticipée des nouveaux ajouts.

## Périmètre et méthode

Artefacts lus : spec, plan, research, data-model, contrat, quickstart, reuse-audit et tasks dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant.

Le helper check-prerequisites.sh et le template Analyze sont absents. Le contexte est résolu par /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/.specify/feature.json. Le rapport suit manuellement l'outline de speckit-analyze ; aucune exécution de helper absente n'est revendiquée. Le principal a demandé explicitement la persistance de ce rapport et les corrections non ambiguës.

Les preuves de codebase des explorateurs sont réutilisées. Les fixtures Rust réelles et les tests RPC natifs ont été préférés aux noms de tests inventés. L'analyse ne déclenche ni compilation, ni daemon, ni agent, ni recette sur données actives.

## Passe1 — Findings et corrections autorisées

| ID | Catégorie | Sévérité initiale | Localisation | Résumé | Correction documentaire |
| --- | --- | --- | --- | --- | --- |
| C147-01 | Cohérence | HIGH | spec FR147-09/SC147-05, data-model transition7/invariants | Refus de contexte pouvait être lu comme preuve de suppression du fil mémorisé. | Seul thread_unavailable confirmé par read choisi retire le UUID. Binding/panne conserve le choix et masque contenu ; projet refuse la vue sans déduire suppression du fil. |
| C147-02 | Cohérence contrat | HIGH | plan section5 et modèle du signal | Ancienne variante avec UUID de fil/kind divergeait du signal global retenu. | version/generation/seq/status seulement, aucune référence de fil ni contenu ; mutations étrangères filtrées au daemon. |
| C147-03 | Garantie de reprise | HIGH | contrat bornes/ready | Saturation avant première écriture pouvait remplacer ready par resync. | ready seq0 premier obligatoire non coalescible, place réservée dans la file16 ; test de rafale>16 avant écriture. |
| C147-04 | Erreur et ressources | HIGH | plan runtime, contrat refus, T019/T020 | Supervisor pouvait reconnecter indéfiniment une erreur métier terminale. | Reprise transport transitoire seulement ; refus métier fermé arrête consommation, nouvelle visite/refresh manuel peuvent revalider. |
| C147-05 | Pagination | HIGH | plan snapshot, contrat UI, T022/T023 | Fusion de tête récente et pages sous ancien snapshot pouvait masquer superseded_by_seq. | Acquérir S, reconstruire segment déjà consulté sous to_seq=S commun, staging puis publication atomique ; invalider pagination ancienne. Test remplacement page2 pendant chargement page3. |
| R147-01 | Réutilisation | MEDIUM | T008/T018 | Nouveau nom de harness CLI alors que fixture et vrai handle_connection/CLI existent. | Réutiliser spec145_human_view_tests dans daemon.rs, fixtures11032/11308 ; aucun nouveau harness ni kill groupe098. |
| R147-02 | Couverture frontière | MEDIUM | T009/T019 | runtime.test seul ne prouve pas les raccords RPC réels. | orchestration.test.ts et server.test.ts filtre Bridget ; state/runtime.test seulement pour le comportement de rétention qui y appartient. |
| V147-01 | Niveau de preuve | MEDIUM | T014/quickstart | Un DOM synthétique pouvait être présenté comme trajet daemon→DOM réel. | Trois preuves distinctes : daemon→CLI réel, CLI147 réel exécuté par service/RPC, runtime→DOM synthétique. Aucun mock qualifié de bout en bout réel. |
| V147-02 | Précondition | LOW | T024/quickstart | Formulation d'autorisation preview encore en attente. | Autorisation de l'aperçu isolé147 reçue par le principal ; tâche non bloquée par une demande déjà satisfaite. |

Ces corrections ont été appliquées dans une étape d'édition autorisée, avant la seconde passe de lecture seule. Elles n'ajoutent aucune fonction métier au périmètre utilisateur.

## Passe2 — Couverture FR

| Exigence | Tâches | Résultat de lecture |
| --- | --- | --- |
| FR147-01 | T007–T014 | Watch humain et consommation visible, refresh manuel conservé. |
| FR147-02 | T007/T011/T013/T014 | Create/Post/supersedes/Close et annuaire, pas nouveau CRUD. |
| FR147-03 | T010/T012/T013 | Signal déclenche lecture ; silence60s sans polling. |
| FR147-04 | T007/T010/T011/T013 | File bornée, coalescence, un in-flight/dirty. |
| FR147-05 | T008/T019–T021 | Nouveau ready/génération, rattrapage, snapshot autorisé. |
| FR147-06 | T018–T021 | Cancel réel, page masquée, visites anciennes rejetées. |
| FR147-07 | T015–T017 | Choix séparé par environment/project/conversation. |
| FR147-08 | T015–T017 | show/history direct du choix hors première page. |
| FR147-09 | T015/T019–T021 | Fil refusé ≠ binding ou transport indisponible. |
| FR147-10 | T003–T006/T018–T020 | Garde commune et scope de lecture, droits revérifiés. |
| FR147-11 | T003/T007/T011/T014/T018 | Signal fermé sans contenu/UUID de fil, diagnostics fermés. |
| FR147-12 | T015–T017 | UUID seul persistant, aucun corps/titre/membre. |
| FR147-13 | T022–T024 | Nouveau S commun, staging atomique, ancien chargement invalidé. |
| FR147-14 | T022–T024 | DOM/clé des lignes inchangées, gestes et copie originale. |
| FR147-15 | T005/T008/T018/T021 | Aucun Register/ACK/message/réveil/modèle ni mutation métier de la vue. |
| FR147-16 | T010/T013/T019/T024 | États explicites, style146 et surfaces ordinaires conservés. |

## Passe2 — Couverture SC

| Critère | Tâches | Vérification prévue |
| --- | --- | --- |
| SC147-01 | T007/T014 | Délai visible sous2s dans transport sain, changements attestés. |
| SC147-02 | T010/T013/T014 |60s sans relecture et rafale100 coalescée, état final exact. |
| SC147-03 | T015–T017 | A→B→A→B et choix hors page1, sans clic supplémentaire. |
| SC147-04 | T015/T017/T019 | Projet/environment et ancienne visite, zéro mélange. |
| SC147-05 | T018–T021 | Rattrapage sans doublon et distinction refus fil/contexte/panne. |
| SC147-06 | T018–T021 | Dix cycles, zéro ressource retenue après fermeture. |
| SC147-07 | T003/T007/T014/T015/T018 | Canaris de confidentialité, schéma fermé et références seules persistées. |
| SC147-08 | T022–T024 | Pages, détails, focus, texte sélectionné et copie ; supersession atomique. |
| SC147-09 | T008/T018/T021/T025 | Comparaison données/cursors/verdicts et zéro action agent/modèle. |

## Constitution, minimalisme et responsabilité future

Règles applicables lues dans /Users/moi/.speckit/constitution.md, le pont projet et ses références. Les deux dépôts gardent leur pile réelle Rust/React/Effect/Vitest. Importer Next.js ou Pytest ne servirait pas ces contrats ; la divergence est explicitement justifiée dans plan/research/reuse-audit.

Isolation147, tests RED avant code et preuve avant cochage sont prévus. Les commits et la livraison restent soumis à une autorisation distincte ; aucune case ne revendique un résultat futur. Les trois lanes ont des fichiers disjoints ; les tâches d'une même lane touchant daemon.rs ou BridgetPanel.tsx sont séquentielles. T020 reste répartie par propriétaire.

Minimalisme : aucun nouveau store générique, framework, dépendance, DB, journal durable, fournisseur ou CRUD. La sélection locale redondante disparaît à T016. La garde commune et la ressource scoped portent des garanties réelles ; la reconstruction atomique évite une fausse simplification de pagination.

Responsabilité future : commandes/tests et ressources observables nommés, contrat wire fermé et borne de chaque queue explicite. L'ADR prévue par T026 documente le seul choix structurant nouveau. Les sources externes étayent les principes ; elles ne prouvent pas les performances ou le code147.

## Métriques finales et statut

- Exigences FR :16, couvertes16/16.
- Critères SC :9, couverts9/9.
- Tâches :26 ; setup2, fondations4, US1 huit, US2 trois, US3 quatre, US4 trois, clôture2.
- Exigences sans tâche :0 ; tâches sans exigence ou gate de constitution :0.
- Findings de passe1 :9, corrigés9 ; findings finaux CRITICAL/HIGH/MEDIUM/LOW ouverts :0.
- Problèmes XIX/XX non justifiés :0 ; duplications évidentes ouvertes :0.
- Forme checklist et ID T001–T026 : conforme ; chemins d'action absolus présents.
- `git diff --check` documentaire : PASS observé. Aucun test de produit ni recette visuelle147 lancé par cette analyse.

## Prochaine action

Le principal doit donner le GO implémentation aux propriétaires Rust/RPC/Web après lecture de ce rapport. Les tâches restent0/26. La recette et les tests doivent produire leurs résultats réels avant toute clôture. Ce PASS documentaire ne vaut ni approbation du code, ni installation, ni activation en production.

## Addendum de convergence autorisé

Le statut0/26 ci-dessus décrit la passe initiale avant GO. Le principal a depuis autorisé les lanes ; les preuves courantes restent dans le journal et les résultats147. Les trois cas observés en implémentation ont été intégrés aux artefacts : ready coalescé côté atome malgré protection wire ; rupture CLI/daemon sous WebSocket T3 sain ; intervalle de plus de50 nouveautés.

Le runtime conserve event/readyGeneration/subscriptionId sous un visitId local. Le wire reste inchangé. La reprise technique du seul CLI réutilise Schedule, initiale plus trois reprises maximum, unavailable/command_failed/timeout seulement ; aucun retry métier ni polling de corps. L'historique relit tête, intervalle jusqu'à l'ancre et segment consulté sous S commun avant publication atomique. Coût O(nouveautés + pages consultées).

Audit de réutilisation étendu18/18 ; gate5/5 conservé. Ces précisions complètent les tâches existantes sans nouvel ID. Elles ne prouvent ni leur GREEN final ni l'achèvement T026. Analyse du diff et recette finales restent en attente des reçus.

## Ajout US5 approuvé le 2026-10-09

US147-05 P2 ajoutée sans réécrire US1–US4. FR147-17/18 et SC147-10 couvrent la ligne auteur/date, targets effectifs, none, all, nom absent, plusieurs destinataires, copie exacte et absence de promesse de livraison. T027 RED → T028 GREEN → T029 QA → T030 convergence, toutes ajoutées ouvertes. Les T001–T026 restent inchangées. Le référentiel courant compte donc18 FR,10 SC et30 tâches ; les métriques16/9/26 plus haut décrivent l'analyse initiale.

Réutilisation : ligne et données autorisées existantes, item16 du gate18/18. Aucun nouveau protocole/API/annuaire/mutation/réveil. Les nouvelles exigences sont mappées à T027–T030, sans exigence orpheline. Cet addendum décrit le gate de conception. Les preuves ultérieures sont consignées ci-dessous.

## Convergence des preuves — état avant le gate final, historique

Référentiel courant : 18 exigences, 10 critères, 5 scénarios utilisateurs et 30 tâches. Les tests finaux couvrent les branches de protocole, d'autorité, de processus, de reprise, de pagination, de sélection et de présentation. Rust 16PASS ; transport 298PASS ; tests threads 16PASS ; T3 contracts/Reader/auth/runtime 136PASS ; WebSocket 1PASS ; Web 116PASS. Les tests ignorés et avertissements préexistants restent dans le journal et les reçus, sans faux résultat zéro alerte.

La recette native isolée utilise des données synthétiques. Elle valide choix hors première page, retour A→B→A, focus, sélection de texte, dépliages, copie brute, recherche, supersession pendant pagination en vol, silence supérieur à 60 secondes et dix cycles de libération. L'interop daemon/socket/CLI/Reader/runtime réelle est une preuve distincte avec relais RPC synthétique. Aucun parcours WebSocket/DOM intégral réel n'est revendiqué.

Les cinq variantes métier US5 ont un RED ciblé, un GREEN 116PASS et une recette native de la ligne auteur/date. Les mesures de modification sous deux secondes valent pour l'opération native saine avec fixture ; les essais d'aperçu inactif plus lents restent consignés. Aucun benchmark de production ni essai 128 clients simultanés n'est prétendu.

Les deux audits v14 sont bornés au diff : 6 fichiers source Rust et 17 fichiers source/test T3. Les constats historiques sont conservés, les contre-revues finales sont APPROVE, le scoring readonly ne trouve aucun défaut résiduel confirmé. Les deux validateurs sortent RC0 avec zéro erreur et zéro warning. Aucun constat documentaire CRITICAL persistant. Cette consolidation a précédé le gate final du principal, désormais validé. T025/T026/T030 sont cochées sur les preuves. Aucun commit, fusion, push, installation ou activation.

## Passe finale code ↔ exigences — lecture seule des propriétaires

Sources de cette passe : `/root/rust147`, `/root/rpc147` et `/root/web147`. Aucun nouveau test ni édition produit. Leurs lignes sont celles des sources147 finales. Aucun symbole ou exigence orphelin constaté ; contributions entre lanes distinguées.

| Exigence | Points d’entrée vérifiés | Preuve / responsabilité |
|---|---|---|
| FR147-01 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:12939 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts:241 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:664 | Signal→stream→relecture native, fixtures interop et recette séparées. |
| FR147-02 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs:568 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:12964 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:253 | Done≠Replayed, mutations disponibles, noms pertinents et tri récent. |
| FR147-03 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:11541 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts:35 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:171 | Aucun pollingcorps, garde idle sans scan, silence natif69s. |
| FR147-04 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:807 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:130 | File16/coalescence/overflow et un groupe courant+dirty ; rafale100. |
| FR147-05 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts:50 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:316 | Retry technique borné, newready, rattrapage sous S. |
| FR147-06 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts:346 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/runtime.ts:346 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:97 | EOF/cancel scoped, visibilité et visite obsolète, dix cycles. |
| FR147-07 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/rightPanelStore.ts:503 | Clé environnement/projet/conversation, choix indépendant. |
| FR147-08 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:220 | show/history explicites hors page1, A103 restauré. |
| FR147-09 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:449 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts:43 | Contexte/panne masque mais gardeUUID ; seul refus confirmé du fil retirechoix. |
| FR147-10 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:12795 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts:105 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/auth/RpcAuthorization.ts:26 | Autorité serveur avant lectures/flux, réattestation avant émission. |
| FR147-11 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-transport/src/protocol.rs:2205 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/bridget.ts:19 | Schémas fermés contrôle seul, canaris et champs inconnus rejetés. |
| FR147-12 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/rightPanelStore.ts:942 | UUID seul réhydraté ; aucune copie de corps/titre/membres. |
| FR147-13 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:332 | Pont nouveautés/anciensegment, to_seq S commun, staging atomique. |
| FR147-14 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:414 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:849 | Identité DOM et keys stables ; recette focus/Range/détails/copie brute. |
| FR147-15 | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:11255 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/ws.ts:2144 | Tables/cursors/verdicts identiques, aucun Deliver/dispatch/ACK/modèle. |
| FR147-16 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:76 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts:43 | Codes neutres, masque/refus, pas refonte visuelle large. |
| FR147-17 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:877 | Auteur→destinataires ou Sans sollicitation, date unique même ligne. |
| FR147-18 | /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:731 ; /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx:827 | Targets effectifs auteur exclu, noms autorisés/fallbackUUID, aucun appel extra. |

| Critère | Preuves finales et limite |
|---|---|
| SC147-01 | Cinq mesures DOM native synthétique saine693.6/875/823.3/620.6/803.3ms ; événements réels Rust testés séparément. Pas performance production ni E2E intégral. |
| SC147-02 | Silence69,382.1ms lectures0 ;100 signaux,239…1 contigu sous S239, un groupe courant/dirty. |
| SC147-03 | A103 hors première page→B201→A103→B201 sansclic ; store/panel tests. |
| SC147-04 | Store clétriple et stalevisits testés ; réponses A→B→A et pagination ancienne ignorées. |
| SC147-05 | Interop CLI réelle reprend nouvelle génération ; UI fixture masque panne/refus puis rattrape240/241 ; seul refus fil effaceUUID. |
| SC147-06 | VraiCLI dix cycles plus DOM natif dix cycles ;0 aprèsfermeture, aucune croissance. |
| SC147-07 | Canaris Rust/Schema et UUID-onlystore ; aucun corps dans signaux. |
| SC147-08 | Anciennepage en vol→supersedes4/242 ;242…1 commun, focus/Range/details/copieraw exact. |
| SC147-09 | Snapshots toutes tables et peer sansDeliver ; WSdispatch0 ; QAexternal0. |
| SC147-10 | RED5fail ciblé, GREEN116, six variantes DOM24px/dateunique/UUIDaccessible, rawbodycopieinchangée. |

Cette passe confirme la couverture18/18 et10/10 avec les limites explicites. Elle ne vaut pas installation. Les tasks ont été gelées pendant les deux passes du principal, puis les30 tâches ont été cochées sur son GO final.

## Gate final du principal

Deux passes Converge lecture seule validées, zéro exigence orpheline, zéro finding actif. Les limites de preuve native/synthétique et les avertissements préexistants sont conservés. Les audits Rust et T3 sont mécaniquement conformes (RC0, zéro erreur/alerte). Session Implemented/Validé en isolé, NON installée et NON activée. Aucun chantier148 incorporé.

## Réouverture — cohérence du périmètre, pas validation anticipée

US147-06/07 P1 approuvées. FR147-19–23 et SC147-11–14 ajoutés sans renuméroter les anciens IDs. Référentiel courant23FR/14SC/7US ; le mapping18/10 précédent ne couvre que US1–US5. Les30 tâches prouvées restent cochées. La planification MCP concrète et l'audit de réutilisation de l'extension sont requis avant tâches supplémentaires. La normalisation d'identifiants n'accorde aucun droit ; les écritures interagents de test n'appartiennent pas au flux humain FR147-15. Aucun défaut exact MCP ni réussite d'extension inventés.

## US9 — référentiel élargi, plan/gate natif acceptés

Ajout utilisateur approuvé dans147 : US9 P2/FR29–31/SC19–20, total31FR/20SC/9US. Plan Web natif et reuseitem27 gate5/5 acceptés avant tâches. FR29–31/SC19–20 correspondent àT051 RED5groupes/T052 code/T053 qualité-recette/T054 convergence ; tokens/parentkeys/ScrollArea/détails réutilisés, pas API/store/helper nouveau. Le choix horspage1/filtre demeure identifiable uniquement via détail autorisé. Aucune couverture code anticipée. T049/T050 clôturées sur294PASS/revues finalesGO, US8/US9 restent ouverts sans faux PASS global.

## Préparation de convergence globale —31FR/20SC/9US/54 tâches

Statut In Progress,43 tâches cochées sur GO précédents. Tableau de préparation, pas un nouveau PASS global ni code exécuté par l'owner documentaire.

| Scénario | Exigences / critères | Tâches et état de preuve |
|---|---|---|
|US1 abonnement | FR01–06/10/11/15/16 ; SC01/02/04–07/09 | T003–T014 et T018–T021, preuves initiales validées ; latency UI synthétique distincte du transport natif. |
|US2 sélection | FR07–09/12 ; SC03/04/07 | T015–T017, A→B→A, horspage1 et UUID seul validés. |
|US3 sécurité/reprise | FR05/06/09/10/15/16 ; SC05/06/09 | T018–T021, preuves natives et DOM synthétique séparées, refus versus panne conservé. |
|US4 lecture stable | FR13/14 ; SC08 | T022–T024, instantané commun, pagination/supersessions/DOM/copie validés. |
|US5 sollicitations | FR17/18 ; SC10 | T027–T030, cibles effectives sans statut de livraison validées. |
|US6 UUID | FR19/20 et FR23 côté UUID ; SC11/12 | T031–T034, RED/GREEN/replay/refus/régressions validés. |
|US7 MCP | FR21/22 et FR23 côté montage ; SC13/14 | T035–T041 et T049/T050,294PASS plus preuves OS3PASS/refresh1PASS séparées et revues finales approuvées. |
|US8 membres | FR24–28 ; SC15–18 | T042–T048, revue fonctionnelle APPROVE ; reçus finaux/audit frais et gate global encore attendus. |
|US9 visuel | FR29–31 ; SC19/20 | T051–T054, RED métier et GREEN provisoire signalés ; correctifs show-date/ScrollArea et preuves finales/recette encore attendus. |

Les audits initiaux du08/10 ne couvrent pas les extensions. Audits frais Rust/T3 du09/10 délégués aux reviewers, reçus et validateurs encore attendus. Ancien timeout CLI de qualification US8 corrigé par propriétaire : sortie64KiB/attente2s/global10s et garde PID. Revue du correctif et succès natif ne valent pas injection de timeout RED/GREEN. Vérification finale code↔tous FR/SC, absence d'orphelin et stabilité byte-à-byte des tâches restent à faire par le principal après gel des sources. Aucun commit/installation/activation/restart ou modèle.

## Converge passage1 principal — lecture seule, avant correction FR31

SHA256 tasks avant contrôle : dbde969d79be9397992de8c061785995f8245455728880793486a70d92c99654, confirmé identique par le principal. Aucun cochage ni édition des tâches pendant cette passe.31FR/20SC/9US/54 tâches,43 cochées sur GO antérieurs.

FR01–18/SC01–10 : table de code initiale et121 tests de conservation réutilisés. Sources vérifiées par les owners gelés et le principal ; aucun nouveau test exécuté par l'owner documentaire. Références actuelles des extensions :

- UUID/ACK : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs ; CLI : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/cli.rs.
- MCP : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeMcp.ts ; Adapter : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.ts.
- Store : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/store/threads.rs ; Daemon : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs.
- Refresh : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/t3code_identity.rs ; Panel : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx.

| Exigence | Code / preuve responsable |
|---|---|
|FR19 | UUID493 require_uuid, forme36hyphénée et minuscule avant hash/SQL ; tests UUID. |
|FR20 | ACK1031/canonicalhumain350, CLI1526, replay/refus/corps exacts et résolveur noms inchangé. |
|FR21 | MCP120/176/188, Adapter5060→5174, montage/catalogue et propriété prépublication. |
|FR22 | MCP133/138/147/202, Adapterfinalizer5063→publication5176 ; refus avant I/O/status et cleanup. |
|FR23 | Daemon tests MCP natifs, Refresh1204, SDK294PASS et diagnosticreadonly ; preuves distinctes, pas modèle. |
|FR24 | handle_with_change/AddMembers et Storetransaction ; créateur/ouvert/16/dedup/replay. |
|FR25 | Daemongardeunion et Storerecheck/migrationv3 ; refus sans mutation. |
|FR26 | Daemon12417 interop CLI/MCP réelle, histoire137entrées/3pages/readACK/history exacts. |
|FR27 | Storecursor0 et Daemon exclusiondispatch ;12417 ancienneswakes/entries/reads/timestamps stables/allfutur/0Deliver. |
|FR28 | Daemonpublicationwatchpostcommit/nochange-replay-refus silencieux, membership11PASS et revueAPPROVE. |
|FR29 | Panel599/621/631/633 tokens/focus, tests358/385. |
|FR30 | Panel525/529/535 pinunique autorisé et760/785/795 bas sansheader/messagesseq, tests411/492. |
|FR31 | Panel655/658 detailsfrère et660/662 titres/membres complets wrapping après correction ; loading/mask conservés et nouveau test full authorized title. |

SC11/12 : UUID10tests/replay/non-régressions, aucune autorité déduite de la casse. SC13/14 : SDK294PASS, MCPnative3PASS, refresh1PASS/régress22+14 et isolation, pas chaîne modèle E2E. SC15–18 : membership11PASS/transport3PASS/régress103PASS2ignored et interop natif137entrées, silence/no-replay/audiences/migration. SC19/20 : cinq RED métier puis121PASS/build, nouvelle correction FR31 et recette native encore ouvertes.

Le mapping initial ne trouvait pas d'exigence sans source/test. La revue approfondie du passage1 confirme ensuite un MEDIUM FR31/US9acceptation5 : titre/membres tronqués, details limités aux dates/état. Le principal autorise correction minimale dans details existant avec textes complets qui reviennent à la ligne et RED longtitre/multimembres. Aucune tâche nouvelle, T051–T054 restent ouvertes ;121PASS et build30.12s deviennent pré-correction. Conserver ce finding historique même après son éventuelle correction.

| Gates réellement restants | État |
|---|---|
|FR31 titre/membres complets | RED1FAIL puisGREEN122PASS et statiquesRC0 ; main relit les deux paragraphes wrapping. Correctif code reçu, validation native dans QA restante. |
|Recette native US9 | Aperçu interrompu après resize ; evaluate échoué ne prouve pas0h2. Charte/clavier/DOM/copie complets encore à reprendre en isolé. |
|Audit frais T3 extensions | Final encore à recevoir. Audit Rust neuf validé RC0/0erreur/0warning par reviewer et principal, A/zéro actif limité à US6/US8/T040 et8fichiers/hunks ; pas toutSPEC147 ni réutilisation grade08/10. |

Passage2 et GO final à consigner après ces gates. Commentaire coût de ClaudeMcp : annotation seule vérifiée, sans benchmark ou comportement nouveau. Aucun commit/install/activation/restart/modèle.

## Gate partiel par scénario — US6 validé, US7 conception validée

Le principal a validé require_uuid493, canonical_uuid350 et thread_members1519 avant implémentation UUID. FR19/20↔SC11/12↔T031–T034 correspondent aux seules actions de fils partagés. Sept cas comportementaux, parsing CLI et non-régressions102/127/145/146/147 ; aucune normalisation de l'acteur d'autorité ni des IDs send opaques. Après reçus et contre-revue APPROVE, le principal autorise34 tâches cochées. Deux passes Analyze main, avant/après resserrage de portée, ne laissent aucun finding CRITICAL persistant. FR23 part UUID relève de T031–T034 ; FR21/22 et montage FR23 relèvent de T035–T041, désormais conçues mais non validées GREEN. Aucun faux PASS global des extensions.

Le diagnostic assaini regional-wrkr-1/claude_glm/.claude-glm constate configuration sans Bridget et injection T3 t3-code seulement ; aucun secret ni modèle lancé. Après gate US7, FR21/22/23↔SC13/14↔T035–T041 possédaient reçus SDK292PASS/1SKIP, façade MCPnative3PASS et qualification refresh1PASS avec régressions22/14. Deux revues locales finales APPROVE puis GO principal permettaient leur cochage. Ce paragraphe est historique, avant T049/T050 et les gates globaux ci-dessous. Les preuves demeurent distinctes, pas SDK+modèleE2E.

## Converge passage2 final —31FR/20SC/9US

Le principal confirme CONVERGED après relecture du code gelé, notamment Panel660–665 pour FR31. Toutes les31 exigences et20 critères possèdent source/test/recette dans le périmètre de chaque preuve. Aucun orphelin, aucune tâche manquante, zéro finding actif. Les paragraphes de préparation précédents conservent leurs états provisoires ; le présent gate est courant.

US8 :11PASS, transport3PASS, régressions103PASS/2ignored et10PASS ; interop native CLI/MCP privée137entrées/3pages, silence d'adhésion et cibles futures corroborés. US7 final294PASS/1SKIP, preuves MCP OS3PASS et refresh1PASS distinctes. US9 final122PASS, build post-FR31RC0, recette native finale PASS : charte claire/sombre, titre/membres complets, clavier, pin/recherche/horspage/A→B→A, pagination et stabilité de copie/détails ; masques sans fuite.

Audits frais Rust A et T3 A99.86, zéro actif ; validateurs relus/re-exécutés par principal RC0/0erreur/0warning. Historique MEDIUM corrigé conservé, pas grade initial réutilisé ni dépôt global certifié. QA /Users/moi/.cache/t3-spec147-preview.457DX8/us9-final-native.json lue intégralement. Panneau CSS360px, viewport1843×1152, captures1280×800 ; composants/CSS/store réels, RPC/données/clipboard synthétiques. Aucune chaîne transport/provider/modèle E2E intégrale revendiquée.

SHA256 tasks avant/après les passes : dbde969d79be9397992de8c061785995f8245455728880793486a70d92c99654, strictement identique. Le principal autorise ensuite seulement le cochage des onze dernières tâches.54/54, Implemented/Validé en environnement isolé. À cette clôture, NON installé et NON activé ; aucun commit/fusion/push/restart/cleanup.

## Autorisation de livraison distincte, après convergence

L'utilisateur autorise ensuite commit/fusion/push/installation et redémarrage T3. Livraison EN COURS par principal, sans PR T3 ni cleanup142/145. Ce mandat ne transforme pas les preuves isolées en preuves de production. Reçus de sauvegarde ciblée app/binaire/DB Bridget (pas DB T3), installation/hashes/processus et smoke encore attendus. Aucun finding de développement rouvert ni task décomptée ;54/54 demeure validé.
