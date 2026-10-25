# Tâches — SPEC147 panneau Bridget vivant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-1).

Plan et gate de réutilisation validés par le principal. Les templates/helpers locaux sont absents ; génération manuelle selon l'outline de speckit-tasks. Les quatre corrections de revue sont contractuelles : ready premier protégé, nouveau snapshot S commun, refus métier sans retry infini et sélection effacée seulement sur refus confirmé du fil.

## Complément approuvé US10 — adaptateur de bureau V2, même session147

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-3).

## Phase1 — Préparation

- [x] T001 Consigner les bases146, les deux worktrees147, les propriétaires Rust/RPC/Web et les limites de non-intervention dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/implementation.md ; résultat observable : sources142/145 et services actifs inchangés, commandes de test et données privées de fixture identifiées.
- [x] T002 Préparer les fixtures synthétiques, reçus RED/GREEN et cible Cargo isolée /Volumes/SD1TO/bridget-build-147-rust dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json ; commandes Cargo directes avec CARGO_TARGET_DIR exact, sans appel au nettoyeur scripts/build.py qui pourrait retirer des worktrees fusionnés hors périmètre. Aucun succès ni compteur fabriqués.

## Phase2 — Fondations bloquantes

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-4).

## Phase3 — US1 / US147-01 : abonnement automatique (P1)

But : liste et détail ouverts reflètent les changements sous deux secondes en transport sain, sans clic ni polling de contenu.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-6).

## Phase4 — US2 / US147-02 : sélection mémorisée (P1)

But : chaque conversation retrouve son choix sans clic, y compris hors première page.

Test indépendant : A → B → A → B, choix indépendants, contextes multi-projets/environnements et snapshot store ancien.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-7).

## Phase5 — US3 / US147-03 : reprise et sécurité (P1)

But : rattraper les manques, refuser les données non autorisées et libérer toute ressource quand la vue cesse.

Test indépendant : transport coupé avec changements manqués, refus métier, binding indisponible puis rétabli, fermeture/page masquée et dix cycles.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-8).

## Phase6 — US4 / US147-04 : lecture stable (P2)

But : conserver les gestes et les pages utiles lors d'une actualisation ; ne jamais restaurer une ancienne instruction courante.

Test indépendant : page2 dépliée, page3 en chargement, remplacement de consigne puis nouvelle tête ; focus, texte sélectionné et copie inchangés pour les éléments inchangés.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-9).

## Phase7 — Qualité et clôture documentaire

- [x] T025 Exécuter format/lint/typecheck/tests/build ciblés aux paquets touchés, contre-revue puis audit sécurité/performance/minimalisme, et consigner leurs limites dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json ; aucune erreur ouverte masquée, aucune revue locale qualifiée d'inter-fournisseurs, aucun build global de projets étrangers.
- [x] T026 Consolider implementation.md, résultats, ADR du canal humain et convergence des artefacts dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/implementation.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/docs/decisions/147-suivi-humain-bridget.md ; cocher uniquement les preuves réelles et déclarer séparément ce qui n'est pas installé/activé. Aucun commit, fusion, push, déploiement, restart ou cleanup sans autorisation distincte.

## Phase8 — US5 / US147-05 : destinataires sollicités (P2), ajout approuvé

But : comprendre les sollicitations dans la ligne auteur/date existante, sans confondre demande et livraison.

Test indépendant : none/targets/all, noms autorisés absents et plusieurs targets effectifs ; ligne compacte et copie exacte du corps.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-10).

## Dépendances et exécution parallèle

T001/T002 → fondations T003/T004 puis T005/T006. US1 utilise ce contrat fermé. US2 est testable par store/read146 indépendamment du flux, mais sa recette combinée attend US1. US3 renforce les ressources de US1. US4 dépend du suivi et conserve la sélection US2.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-11).

US5 réutilise les lectures autorisées146 et la ligne existante. T027 précède T028, puis T029/T030. La lane Web est seule propriétaire du panneau ; l'ajout attend son point de synchronisation et ne crée pas de writer concurrent. Les T001–T026 et leurs IDs restent inchangés.

Exemples après fondations : T007 puis T008 Rust, T009 RPC et T010 Web peuvent préparer leurs RED dans leurs lanes disjointes ; T007/T008 ne sont pas simultanées car daemon.rs est commun. T015 Web n'écrit pas le panneau pendant T010. T018 Rust et T019 RPC/Web suivent leurs propriétaires. Les mocks restent aux frontières, et l'intégration T014/T021/T024 clôt leur différence avec le réel.

## Stratégie d'implémentation et responsabilité

Le premier incrément fournit un flux humain buildable et observable, puis sa consommation groupée. Les étapes suivantes ajoutent sélection et reprise, puis stabilité du segment historique. Le besoin validé comprend les deux améliorations ; US1 seule n'est pas une livraison finale acceptable.

Retirer la sélection locale désormais redondante lors de T016. Aucun helper vide, store dérivé ou abstraction générique n'est créé. La garde commune T005 réduit la maintenance ; la portée T012/T020 prévient les fuites ; le snapshot atomique T023 porte une vraie garantie de pagination.

Tests avant tâche suivante. Les preuves de réussite sont datées, attribuées et limitées aux commandes exécutées. Les commits par tâche prévus par le workflow restent différés tant qu'une autorisation de commit n'est pas donnée ; cette préparation ne revendique aucune livraison.

## Gate Converge final

Le principal a réalisé deux passes de convergence lecture seule. Le hash des tâches est resté identique avant/après ces passes : 04379ec81e413bfbddfe7a3afca5b055cca6bb00a83ee4a45a32e10037fdf026. Ce hash décrit l’état gelé avant le cochage final, pas le présent fichier modifié.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-12).

## Réouverture approuvée — US147-06/07

Les30 tâches précédentes et leurs preuves restent intactes. Le principal a validé le plan UUID US6 puis ses reçus RED/GREEN/régressions : T031–T034 sont cochées. Le plan MCP US7 est validé avant son code ; ses preuves restent ouvertes. Aucun changement de production ni restart autorisé.

## Phase9 — US6 / US147-06 : UUID sans astuce de casse (P1)

But : formes hyphénées36 majuscule/minuscule/mixte identiques avant idempotence/SQL, sans modifier corps, résolution des noms ou contrats humains.

Test indépendant : base privée et vrais appels agent ThreadRequest/MCP plus parsing CLI. Rejouer Create/Post/Close/ACK avec UUID possédant a–f ; comparer effets, références, reçus et corps. Zéro modèle ni agent actif.

- [x] T031 [US6] Écrire et observer les RED comportementaux dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs : sept cas couvrant Create, Post/copie, Close replay, ACK/reçu, membres/targets, formes invalides et invariants humains. Varier UUID hyphénés36 majuscule/minuscule/mixte avant hash/SQL ; exiger un seul effet au replay, relations canoniques et corps exact. Aucun test décoratif du helper seulement.
- [x] T032 [US6] Écrire et observer les RED parsing/résolution CLI dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/cli.rs : membre/notify UUID majuscule normalisé sans appel résolveur ; noms et préfixes conservent leur parcours existant ; UUID déclaré invalide refuse sans mutation. Vérifier le montage ThreadRequest/MCP existant sans créer une autorité humaine.
- [x] T033 [US6] Étendre require_uuid pub(crate) dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs et le réutiliser depuis thread_members dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/cli.rs. Valider hyphénation36 eq_ignore_ascii_case puis retourner minuscule avant idempotence/SQL ; normaliser aussi ACK avec ReceiptInvalid préservé. Garder canonical_uuid humain/curseurs strict et noms/préfixes/corps inchangés. T031/T032 GREEN, sans nouveau helper/dependency/migration.
- [x] T034 [US6] Exécuter les tests ciblés et non-régressions102/127/145/146/147, CLI/MCP/replay/mauvaisUUID, format/lint adaptés aux fichiers touchés et consigner les reçus attribués dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/implementation.md. Vérifier zéro mutation des fixtures refusées, corps exact, erreurs fermées et ancienne consultation humaine intacte. Ne cocher que des commandes exécutées ; aucun restart, configuration production ou appel de modèle.

Dépendances : gate UUID validé → T031 → T032 → T033 → T034, toutes prouvées. Même propriétaire Rust, pas writer concurrent. Le gate distinct US7 autorise T035–T041. Les helpers/templates absents restent un fallback manuel constaté ; aucune exécution fictive.

Portée T031–T034 : seuls commandes et outil de fils partagés create/post/read/ack/history/show/close, UUID thread/operation/membres/targets/reçus. Pas normalisation d'acteur d'autorité, IDs opaques send/autreoutil, noms/préfixes partiels ou canonical_uuid humain. Les références membres/targets sont minuscules canoniques, pas des preuves d'identité.

## Phase10 — US7 / US147-07 : montage Claude/GLM attesté (P1)

But : vrai catalogue Bridget monté avant le premier prompt new/resume, identité native attestée, opt-out/homonymes/permissions conservés. Test indépendant : options SDK de fixture plus vraie façade MCP OS privée, sans modèle ; preuves distinctes.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-13).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-14).

### US7 — amendement de conformité avant clôture

Écart d'auto-revue RPC accepté par le principal : le status SDK précédait les métadonnées, donc un disabled explicite pouvait échouer si status rejetait, au lieu du skip fermé attendu. RED dédié puis garde metadata avant status ; flags/identity ne doivent déclencher aucune I/O. Même besoin existant, T049 ajoutée sur GO parce que T035–T041 étaient déjà cochées. Cochages antérieurs restent des preuves de la version préamendement, pas une clôture de cette correction.

## Phase11 — US8 / US147-08 : ajout membres et historique complet (P1)

But : ajouter à un fil ouvert sans perdre l'histoire ni envoyer d'ancienne mission. Plan accepté par le principal ; gate détaillé soumis à sa lecture avant code.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-15).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-16).

## Phase12 — Corrections US7 validées en isolé, même contrat

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-17).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-18).

## Phase13 — US9 / US147-09 : choix natif et messages directs (P2)

Plan et gate5/5 item27 validés par le principal avant code. Aucun nouveau composant/store/API/helper/dépendance.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/tasks.md#extrait-19).

Dépendances US9 : gate natif item27 accepté → T051 RED → T052 GREEN → T053 qualité/recette → T054 convergence. Lane Web seule propriétaire du panneau/test, pas écriture concurrente ; RPC/server et Rust membership disjoints. Chaque nouveau build Web est coordonné, pas doublé à l'aveugle.
