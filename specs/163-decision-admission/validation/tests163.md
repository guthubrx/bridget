# Vérifications SPEC-163 — résultats ciblés et limites

**Statut courant au 14 octobre 2026 :** Implemented, 58/58 (100 %) ; **Pipeline : Livré** avec conservations justifiées. G8/G9 et documents de livraison sont acceptés ROOT ; T045 clôturée sur le commit documentaire c57cfd35efa1338cda5256a02c57d71258d8b8d4 poussé et vérifié. Source active 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 inchangée ; les marqueurs de clôture 58/58 ajoutés ici restent à committer/pousser par ROOT, sans hash futur anticipé.

## Provenance des résultats T3

Résultats transmis par ROOT et rapprochés du reçu local /Users/moi/.cache/bridget-delivery163-20261013/t3-validation163-final.json, SHA-256 9076676edfd08d77d0e489d8ab740ad9e6fb5fe758a7237c7e5b9a9c557e13e2. Empreinte locale vérifiée par le rédacteur ; aucun test, build ou service lancé par ce rédacteur.

Le premier contrôle, non globalement réussi, reste archivé dans /Users/moi/.cache/bridget-delivery163-20261013/t3-validation163-before-fixture-correction.json, SHA-256 e5c765649cb9e2f1444b0eb3628ee1d967c3171ed6740bc703c7cf46d1c005ed, également vérifié. Aucun échec historique supprimé.

| Campagne ou contrôle | Résultat rapporté | Limite de portée |
|---|---|---|
| Groupe unique des cinq bancs T3 existants | 191 PASS/0 FAIL, RC0, 12,439 s | Fixtures du consommateur sur wire figé ; pas runtime ou interopérabilité T030. |
| Format/lint des sept fichiers | RC0 ; cinq warnings historiques de lint | Ne prouve pas un lint sans warning. |
| Typecheck contracts | RC0, 2,547 s | Contrats du consommateur. |
| Typecheck server | RC0, 40,914 s | Serveur T3 ciblé. |
| Typecheck web initial | RC1 ; dix diagnostics hérités dans deux fixtures MessagesTimeline | Cause lue dans diagnostics/contrat ; aucun replay typecheck de baseline séparé. Blobs identiques au HEAD174a9d04554758aacffb4aade588e61c810faf5a avant correction. |
| Format/lint des deux fixtures corrigées | RC0 ; deux warnings historiques | Delta limité à deux champs. |
| Typecheck web après correction | RC0 ; zéro diagnostic, 9,969 s | Résultat actuel distinct du premier échec. |
| Deux bancs affectés par le correctif des fixtures | 446 PASS/0 FAIL, RC0, 6,621 s | Replays des seuls bancs affectés. Les 191tests du consommateur n’ont pas été rejoués. |

La correction explicitement autorisée par ROOT ajoute providerThreadId:null dans deux lignes de fixtures uniquement : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/web/src/components/chat/MessagesTimeline.logic.test.ts et /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/web/src/components/chat/MessagesTimeline.test.tsx. Le champ optionnel de l’union fork ne satisfait pas le champ nullable requis de dynamic_tool ; la valeur concrète attendue est null. Aucun code de production, cast ou scénario nouveau. Les sources initiales ont été qualifiées identiques à HEAD174a ; le reçu conserve leurs empreintes et les diagnostics.

Les 191résultats du consommateur et les 446résultats des deux fixtures ont des portées différentes. Ils ne constituent pas une suite globale relancée ou une preuve de service actif. Les contrôles préparatoires ne permettent pas le cochage de T031–T035 : interopérabilité/runtime T030 restent attendus.

## Dépendances et projection des modes — préparation contrôlée

Reçu /Users/moi/.cache/bridget-delivery163-20261013/t3-dependencies163-preparation.json, SHA-256 5201168f1eba80280f87f9b3cc6ba1922f7d2757cab57a38dd1f552ee287c585 vérifié localement. ROOT rapporte 24copies indépendantes, 6452 liens internes et 676 rebases ; le reçu détaille comparaison des fichiers rebases et invariants du cache.

La projection ExFAT ne conserve pas exactement les modes Unix : 18 fichiers pyc de 0600 à 0700 qualifiés, sans prétendre une conservation des modes. Les 676 launchers restent exécutables avec les modes attendus. Le paquet final et ses permissions/signatures doivent être contrôlés dans le stage Mac natif prévu pour T038. Ce reçu ne prouve aucune installation, signature finale ou activation.

## Limites générales

Conserver l’échec web initial et ses diagnostics, les warnings historiques et les replays exacts. Aucun test supplémentaire de confort, répétition des 191tests ou banc autonome ajouté. Aucun source Rust en cours considéré validé par ces résultats T3. G8/G9 et validation T030 non déduits. Les résultats sont exécutés par les propriétaires indiqués dans les reçus ; le rédacteur a seulement lu les métadonnées et vérifié les empreintes locales.

## Attestation de reprise effective — complément reçu après le premier archivage

Le reçu d’ajustement est maintenant localisé : origine /home/moi/.local/state/bridget-delivery163/codex-timeout163-adjustment.json ; copie récupérée /Users/moi/.cache/bridget-delivery163-20261013/codex-timeout163-adjustment.json. SHA-256 cb260b8de5d02677dea42a7379bf20eac54d2aa048e8142ac8d9da6d0988a65c vérifié localement. La mention précédente d’absence locale décrit l’instant de première lecture ; elle n’est pas supprimée.

Le chercheur attestation_cartae163, en lecture seule, rapporte la commande génération 470 / instance 308f6e2f-e169-4c0a-9ae3-20941edcedab. Fichier distant /home/moi/.local/share/bridget-worker/cartae/store/commands/cdb55c71c080778a78ce7df3d264caf71e94f5ae2c9e96b278740cb011a8eee8.json ; SHA du snapshot rapporté 2d2e48e8290ccc26100fff4ece5141f63eb774276dbbd4d53a65854c458a36e5. État released, native_ready=true, startup_failed=false. La préparation fige notify_timeout_secs=21600 ; fichier UID 1002/mode 0600.

Wrapper nouveau PID 695011, naissance 143429550 ; Codex PID 695023, naissance 143429571, via Node PID 695015 ; tous UID 1002. Ancien wrapper 627449 absent. Le worker 471321, naissance 142199350, reste actif et inchangé ; KillMode=process, SendSIGKILL=no. Le diff structuré du registre confirme uniquement notify_timeout_secs 2700→21600. Racine worker 0700 ; registre, backup, reçu et credential 0600, sans lien symbolique. Aucun credential recopié.

Ces faits attestent la reprise effective et la définition de lancement figée 21600. Le délai mission 1791940741 correspond au 14 octobre 2026 à 03:19:01 CEST selon ROOT ; le chercheur n’a pas relu cette deadline dans le store maître. Le délai de lancement 1791919437 est distinct. La preuve ne démontre pas l’achèvement du développement ni l’atteinte du délai futur.

La progression reste 4/45, aucun cochage SPEC-163 ajouté. Aucun succès T030, G8/G9 ou recette du décideur déduit. Le rédacteur a seulement vérifié le SHA de la copie locale ; il n’a ni interrogé le serveur ni inspecté ces processus.

## État constaté à 22:10 CEST — seconde mission principale arrêtée

ROOT constate le 13 octobre 2026 à 22:10 CEST que la seconde mission principale ed4af564-69f0-4a6a-8f57-39dc92c57689 est failed « échéance Codex dépassée ». PIDs 695011/695015/695023 absents à ce contrôle. L’attestation précédente released/native_ready et notify_timeout_secs 21600 décrit la reprise antérieure ; elle ne prouve pas que cette nouvelle échéance ne peut pas survenir. Un agent d’attestation diagnostique encore le runtime ; aucune cause supplémentaire n’est déduite ici.

Le second développeur worker reste working selon ROOT : tâche 70aea08d-b411-4141-9f3e-2efc2df13df2, enfant 605c9b05-9e05-4baa-ba1a-a1be59185a1f. Ce statut ne coche aucune tâche ni ne démontre un résultat futur.

ROOT rapporte que le journal distant /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/developpement-cartae163.md décrit douze tests ciblés spec163 daemon PASS et deux tests ciblés transport PASS. Aucun audit global, suite FULL ou achèvement des tâches n’est déduit. Les sources compilées et périmètres exacts restent ceux du journal ; aucune empreinte non vérifiée n’est ajoutée. Le rédacteur n’a ni rapatrié ni synchronisé le worktree distant pendant le développement.

## Limite de la préparation max8 — contrat Laya non validé

Artefact local /Users/moi/.cache/bridget-delivery163-20261013/decision-request-max8.json, SHA-256 a27dabc5e837f64c21bb33e8238d94d48834f977cf451c7378f2d5429ef11b99 vérifié par le rédacteur. Lecture JSON locale : questions est une liste, avec huit candidats c01–c08 ; max_len=1024 et head_max_len=512. Le préflight exige un objet questions pour le contrat Laya : la compatibilité de cet artefact est donc encore invalide, malgré le plafond de huit candidats préparé.

Cet artefact atteste seulement une construction préparatoire locale. Il ne prouve ni tokenisation maximale correcte, ni absence de coupe, ni POST accepté, ni sélection réelle du checkpoint. T040 n’est pas faite et G8 n’est pas déduit. Aucune nouvelle tâche, scénario ou campagne ajoutée ; les corrections relèvent du contrat et de la recette déjà prévus. Progression 4/45 et 45 tâches inchangées.

## Lecture du journal DEV_CARTAE avant seconde reprise — preuve locale du développeur

Copie figée fournie par ROOT : /Users/moi/.cache/bridget-delivery163-20261013/developpement-cartae163-before-reprise2.md, SHA-256 f823a32857ccd039a146fdff41299bd9e6cc14ad4d199c18acda77206bb01216 vérifié. Origine : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/developpement-cartae163.md. Le rédacteur a lu cette copie ; ce n’est ni une contre-validation ROOT des sources ni un replay de tests. Aucun fichier du worktree distant n’a été écrasé ou synchronisé par le rédacteur.

Le développeur rapporte en dernière campagne ciblée : cargo test -p bridget-daemon spec163 --lib -- --nocapture, RC0, 12 PASS/0 FAIL, 2010 filtrés ; cargo test -p bridget-transport spec163 --lib -- --nocapture, RC0, 2 PASS/0 FAIL, 351 filtrés. Son git diff --check est rapporté RC0. Warnings de style/interfaces privées/noms hérités signalés. Aucune FULL workspace, essai Mac, T3, service réel ou production par cette campagne.

Le journal conserve l’échec historique transport codex_app_server::tests::stop_pendant_un_tour_termine_le_groupe_enfant et son rejeu exact échoué, ainsi que l’écart cargo fmt global hors périmètre. Ils ne sont pas transformés en succès par les deux tests spec163. Les SHA de sources listés dans ce manifeste sont déclarés par le développeur ; ils ne sont pas requalifiés en sources compilées contre-vérifiées par ROOT dans ce document.

Le même journal déclare localement les types wire, reçu additif, extraction de projection post-commit, configuration/axes, frontière HTTP et parcours durable/MCP. Ces déclarations ne cochent pas les tâches : le contrat du constructeur Laya reste notamment incompatible à ce checkpoint et les parcours distants ne sont pas encore raccordés. Aucun audit global ou achèvement de SPEC-163 déduit.

Limite de couverture actuelle : probe distant, entretien de reprise, pression/purge et CLI restent hors de la preuve finale ciblée ci-dessus. Le constructeur max8 ne prouve pas la compatibilité Laya et T040 reste non réalisée. Raccord T030/interop, tests finaux, recette et G8/G9 toujours ouverts. Aucun cochage futur sur ces résultats locaux du développeur.

## Convergence passage 1 — six corrections ajoutées par ROOT, pas une clôture

ROOT a ajouté uniquement T046–T051 dans la section Convergence de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/tasks.md. Le rédacteur ne modifie ni ce fichier, ni spec/plan, ni les sources. Périmètre des corrections : placement nullable manquant dans list/show v3 ; cohérence CLI/Reader des versions ; plafonds avant réservation et absence de double compte du reçu propre ; refus de fallback filtré et ordre du permis HTTP ; texte affichable T3 ; validation sanitaire HTTP et délai de reprise T051. Ces corrections servent les contrats existants, sans nouvelle fonctionnalité.

Progression actuelle 4 cochées/51, In Progress. Les mentions antérieures 4/45 et 45 tâches décrivent les checkpoints précédant cette extension de convergence ; elles restent historiques. Aucune des nouvelles tâches ou validation future n’est cochée. G8/G9 et livraison restent ouverts.

Le snapshot de sources du premier passage reste /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163.json, empreinte du fichier 54b4b59d39b45ec39ed549d55e55b1b959c31b8068ad45374ba23a032e3fddf7 vérifiée localement : 23 surfaces, soit 21 sources et 2 preuves selon ROOT. La copie figée du journal est /Users/moi/.cache/bridget-delivery163-20261013/remote-final-developpement-cartae163.md, SHA-256 c218c680d4fa5e03c090ebfeb1d34af1b942fe3f5a09717d10ae1ca9edc7b875 vérifié. Ces empreintes identifient les copies, pas une nouvelle attestation indépendante des sources compilées après modifications.

Les 27 PASS ciblés de la ronde 1, d’abord déclarés par le développeur, sont maintenant attestés indépendamment : 21 bibliothèque daemon, 3 CLI, 1 probe worker et 2 transport. Le reçu indépendant conserve aussi le replay affecté de deux tests CLI et cargo check RC0, sans les compter comme de nouveaux tests du groupe 27. La régression transport historique échouée et les limites sans FULL workspace restent conservées. Cette attestation porte uniquement sur la ronde 1, avant les corrections de ronde 2 ; elle ne valide aucune source R2 en cours. Aucun total global vert déduit.

Le principal a déclaré son premier résultat stable, puis travaille aux corrections de ronde 2. Tâche 65c12f09-412b-46de-92d2-261339bd4244, nouvelle requête root163-converge-p1-corrections-20261013-r2, working selon ROOT. Ce statut ne démontre ni correction achevée ni tests nouveaux. Les anciennes frontières temporaires de fichiers sont déclarées closes par le journal développeur ; les modifications de convergence et leur validation restent à rapprocher.

## Tokenizer offline — réussite préparatoire limitée au corps exact

Reçu /Users/moi/.cache/bridget-delivery163-20261013/decision-tokenizer163-preparation.json, SHA-256 0a797b7c37f213bfc0f2fa27fec6f2ef13336701993f077d7f64bf340de27fe0 vérifié localement. Corps corrigé /Users/moi/.cache/bridget-delivery163-20261013/decision-request-max8-corrected.json : 709 octets, SHA-256 d5ea2390a4474e148ae919e1d33a4fcf3ee79a9b97d816351ef0c8a67033c667 vérifié. Il remplace le constructeur historique questions liste pour ce contrôle précis ; l’ancienne empreinte a27dabc5… reste historique.

Le préflight rapporte RC0 offline sur le tokenizer existant du snapshot 7b928d828b7b0e022f929d9bd2e44165aa270148, modèle typed-decisions. Séquence attendue et construite : 169 tokens ; huit marqueurs exacts, options dix tokens chacune sous le plafond 48, budget 424 rapporté. Comparaisons head/options/state/séquence complète exactes. Ce résultat porte sur ce corps précis : il ne garantit pas toute description arbitraire de 32 octets ni une entrée maximale différente. Aucun Agent/Router ou poids chargé, aucun POST de modèle et T040 non complète. Le rédacteur n’a chargé aucun tokenizer ni rejoué ces comparaisons.

## Estimation restante actualisée

ROOT recommande 210–330 minutes restantes, fin prévue le 14 octobre 2026 entre 02:57 et 04:57 CEST, hors quota ou panne externe. L’estimation initiale 360–600 minutes reste historique ; ce recalage ne réécrit pas son origine. Les corrections de convergence, leur attestation R2, les contrôles affectés, la recette puis la livraison/nettoyage restent à terminer ; l’attestation R1 est désormais acquise. Aucun nouveau pipeline ni synchronisation de documents vers le worktree Linux pendant le développement.

## Mise à jour du 14 octobre 2026 — ronde 1 attestée, ronde 2 distincte

Total contrôlé dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/tasks.md : 51 tâches, quatre cochées (T001–T004), In Progress. La tâche T051 santé HTTP est ajoutée aux corrections T046–T050 déjà consignées ; elle couvre le schéma sanitaire, statut ok, santé négative/malformée/expirée, délai de reprise et permis, selon les oracles de T021 déjà prévus. T046–T049 et T051 sont en correction Cartae. T050 a été préparée dans deux fichiers T3, mais ceux-ci n’ont PAS été testés depuis cette modification ; les anciens 191 PASS ne prouvent pas les nouveaux hash de ces fichiers.

Reçu indépendant Linux R1 : /Users/moi/.cache/bridget-delivery163-20261013/linux-validation163-final.json, SHA-256 7dc90d5eb8f824bf4d6a6a746010cdd6521ebb42419f8f5951883682581f4791 vérifié localement par le rédacteur. À 21:30:35 UTC le 13 octobre 2026, avant R2, la comparaison est concordante : 21 sources du WT Mac, 23 surfaces du bundle et 23 surfaces distantes. La réception porte observed_at_utc 21:30:35.836416 ; l’observation de compilateur est datée 21:30:35.890266 UTC. Les deux preuves du bundle ne doivent pas obligatoirement être présentes dans le WT Mac. Manifeste R1 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163.json, SHA-256 54b4b59d39b45ec39ed549d55e55b1b959c31b8068ad45374ba23a032e3fddf7 vérifié.

Les 27 PASS Linux R1 sont attestés indépendamment depuis les sorties complètes et les commandes liées, et non extrapolés des seules fins de journal. Le premier filtre worker --exact incomplet avait exécuté zéro test ; le replay avec namespace complet apporte le PASS worker compté. L’autre filtre E2E spec163_ exécutant zéro test ne compte pas comme validation. Les échecs historiques restent conservés. Aucun hash par test au début d’exécution n’avait été relevé : la concordance du snapshot final ne prouve pas que chaque essai antérieur utilisait exactement ces octets. Les temps de compilation et de harness restent séparés ; aucune durée totale ou comparaison Mac/Linux n’est inventée.

Le contrôle indépendant est une attestation en lecture seule, pas un replay par le rédacteur. Les modifications R2 en cours et les nouvelles raisons T3 de T050 sont hors de cette preuve. Aucun POST, inférence, T040, G8/G9, Converged ou pipeline terminé déduit. ETA de fin 02:57–04:57 CEST le 14 octobre 2026 inchangée, hors quota/panne externe. Aucun document synchronisé sur le worktree distant ; ownership rendu à ROOT après ce complément.

## Checkpoint P2 et contre-revue de code R1 — 14 octobre 2026

Inventaire réel relu sans modification de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/tasks.md : 55 tâches, quatre cochées, In Progress. Le header de ce fichier annonce encore 4/45 ; il est une métadonnée historique non corrigée par le rédacteur, qui n’en possède pas l’édition. ROOT a ajouté T052–T055 en convergence append-only : DTO de santé réelle/garde de démarrage ; compléments de preuve probe ; durabilité/concurrence des reçus ; gardes de l’entrée auto CLI/MCP. Les défauts déjà consignés servent le contrat existant ; aucune nouvelle fonctionnalité.

P2 conclut « quatre tâches ajoutées », pas Converged. Reçu /Users/moi/.cache/bridget-delivery163-20261013/converge163-p2.json, SHA-256 5242b7ab5ec8e2aa7113dacb6f03bb5977ce82e913a5cafe7302c4ba003db879 vérifié. Sources R2 Mac figées : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r2.json, SHA-256 1901f295c99b8fec80e70d569e6ce9c8e57ef6527f6a3cdd27c21562f553bbae. Le développeur déclare 17 PASS R2 : huit HTTP, six contrôles bibliothèque d’un test et trois CLI. Commandes RC0, copies et empreintes sont conservées ; cette campagne n’a pas encore une attestation indépendante fraîche de ses traces complètes. Elle ne devient pas un GREEN global. Les27PASS distincts R1 restent attestés sur leur ancienne provenance, sans attribution aux sources R3 en modification.

T050 dispose maintenant de 30 PASS sur le banc de contrats existant : /Users/moi/.cache/bridget-delivery163-20261013/t3-t050163-r3.json et son log /Users/moi/.cache/bridget-delivery163-20261013/t3-t050-contracts-tests163-r3.log. Lint/typecheck RC0, deux warnings de regex intentionnels ; format initial RC1 conservé. Après correction de format uniquement, /Users/moi/.cache/bridget-delivery163-20261013/t3-t050163-r4.json, SHA-256 70b55b393a098bbe245b2c81caa439e70fd5c80c9b3e2c8690df33ebb3c3e4bb, atteste format RC0 et égalité exacte des tokens TypeScript : 2041/5814. Aucun replay des 30 tests, du Reader ou du groupe 191. Cette preuve plus récente précise le checkpoint précédent où les deux fichiers n’avaient pas encore été testés après la modification T050.

Sources après format : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/packages/contracts/src/bridgetLineage.ts, SHA-256 093c288b6e6a2ddb0473719231966d2bffadd61dcba6c727b63de208a62f5fce ; /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/packages/contracts/src/bridgetLineage149.test.ts, SHA-256 724d3d178909088d31644dbd5feecdaf9515abfae1a6bfb8f3b870c50eb1d274. Les empreintes sont attestées par le reçu, sans nouvelle exécution ou modification de source par le rédacteur.

## Reprise R3 et santé nominale encore à adapter

R3 admise : tâche 94799f7a-d849-43cf-aab4-6be90eb318bc, enfant 336d8ef8-795f-450f-950b-eeaa971aa3b5, seul propriétaire code/tests Rust SPEC163. Reçu /Users/moi/.cache/bridget-delivery163-20261013/delegation-converge163-p2-mapped.json, SHA-256 f2494e6cf4813960b54623e44c994f840b9f2975a123f3fa764714e85a0a94c7. La première entrée avec cwd /home a été refusée cwd_outside_parent_project ; la demande corrigée porte le chemin source Mac et le mapping autorisé détermine /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission. Aucun bypass, garde ou droit modifié. L’admission ne prouve aucune correction R3 achevée. L’autre développement 164 reste isolé dans /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/.connector164 ; aucun Cargo parallèle autorisé.

Le préflight rapporte GET santé réel HTTP 200, 271 octets, SHA-256 9e0a04a6e2b3a19664d304215ad9ab7dee2282ccb4ed6c225d6c37fedc3fc1a9, observé le 13 octobre 2026 à 22:09:06Z (14 octobre à 00:09:06 CEST). Champs status=ok, loaded, revisions et device. Le DTO R2 fermé sur status seul rejette ce nominal ; cette incompatibilité est retenue dans T052, pas déclarée corrigée par un test de santé synthétique {status:ok}. Aucun POST ni inférence, T040 non réalisée.

La contre-revue de code R1 Claude Sonnet 5.5, sur sources R2, est APPROVE_WITH_CHANGES, avec une objection moyenne et quatre mineures externes non bloquantes. Cela ne solde ni la convergence ni les gates de livraison. O1 owner hors ligne et O4 duplication de chemin tests-only sont retenus par ROOT pour R3 ; autres arbitrages et limites sont conservés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/adversarial-review-claude.md. Reçu de revue /Users/moi/.cache/bridget-delivery163-20261013/code-review163-r1-final.json, SHA-256 09b9227924e182aee635637d90f4a80af45e5fe7bc64e8a39c2df8ba438c0537 ; steering /Users/moi/.cache/bridget-delivery163-20261013/review163-r1-steer.json, SHA-256 7fabcc91bfdf21eb692d1d2e79323e2e75b987745c0234e06c06d9f5fec03b7d. Ce steering était remis en vol sans confirmation dans ce reçu ; aucun accusé de lecture futur n’est inventé.

ETA publiée par ROOT à 00:13 CEST : 190–300 minutes restantes, fin 03:23–05:13 CEST le 14 octobre 2026, hors quota/panne externe. La précédente ETA reste historique. P3, Converged, T040, G8/G9 et achèvement du pipeline ne sont pas présumés. Aucun cochage, source ou test modifié ; aucun document recopié vers Linux pendant le développement.


## Checkpoint R3/R4 — préparation de Converge P3, 14 octobre 2026

Checkpoint inventorié avant T056 : In Progress, 4/55 tâches cochées, T001–T004 seulement. Le header de tasks.md avait été corrigé à 4/55 par ROOT. Les anciennes mentions 4/45 ci-dessus décrivent leurs checkpoints historiques. Aucune tâche nouvelle ni case modifiée par ce complément.

R3 : ROOT et attestation_cartae163 rapportent 30 PASS indépendamment attestés, soit 28 bibliothèque, un CLI et un worker. La trace primaire distante est /home/moi/.codex/sessions/2026/10/13/rollout-2026-10-13T22-14-41-01a13755-06e5-7033-a00b-1b4e1ecd4262.jsonl : lignes 579/580, 583/584 et 606/607 ; format RC0 aux lignes 610/611 et check RC0 aux lignes 616/617. Le snapshot de 651 lignes relu par l’agent avait SHA-256 f58fb6c41573f95c2ec70b2c475f536e0f8211697b45160172eb98a636647651 ; la trace courante a depuis augmenté avec R4. Aucun reçu local d’attestation n’a été créé par cet agent en lecture seule. Le reçu producteur /Users/moi/.cache/bridget-delivery163-20261013/retour-cartae163-r3.json, SHA-256 dfe6bec24f2fa9660fa24eefd8c925fb8b7de010cb1d1e3f5c54185ca20f0448, reste distinct de cette attestation.

Sources R3 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r3.json, SHA-256 61356696baec516dd789b31b118fb696d0b11cc20369574dd1a43705ce22353c. Sources R4 transférées : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r4.json, SHA-256 0a9b6eb57f9cc2ad0948dca0ea1eb4b861c0dd7a47ec098d1ddf30c24255f015. Le transfert rapporte 21 sources Mac identiques au snapshot et au distant après copie, cinq changements et seize sources R3 inchangées. Quatorze logs/manifestes et le journal privé sont copiés byte-identiques. Native round 3 et révision source R4 sont deux informations distinctes. Le transfert n’est ni un nouveau test ni Converge.

R4 : les trois PASS ciblés et le format RC0, d’abord déclarés par le producteur, sont maintenant attestés indépendamment par attestation_cartae163 et transmis par ROOT. Les logs complets et les RC d’exécution concordent : concurrence 7,93 s, réouverture 5,83 s, worker 4,73 s ; format final RC0 en 1,44 s. Deux FAIL historiques RC101, 13,79 s et 6,94 s, restent conservés. Références de trace transmises : lignes 878, 882, 924 et 930 du même déroulé distant ; aucun reçu local neuf créé par l’agent en lecture seule. Reçu /Users/moi/.cache/bridget-delivery163-20261013/retour-cartae163-r4.json, SHA-256 3b69f02faa0719fd6269143d25723094901319b12862c4bce9baf79573690ec1 ; journal /Users/moi/.cache/bridget-delivery163-20261013/remote-r4-developpement-cartae163.md, SHA-256 89e9eb5a68d06eb31374a25a9cdafc3a6130d3982c2f79c9e6ffb678f414a308. Les 30 PASS R3 n’ont pas été rejoués et ne constituent pas 30 PASS nouveaux sur les cinq sources R4 modifiées. Les 27 PASS R1, 17 PASS déclarés R2 et preuves T3 restent sur leurs provenances respectives.

Le rapport en lecture seule t3_consumer163, rendu le 14 octobre à 01:03:56 CEST, ne relève aucune objection nouvelle dans neuf fichiers correspondant aux empreintes finales. Il rapproche les preuves 191 PASS préparatoires, 30 PASS T050 et 446 PASS des fixtures affectées, sans nouveau replay. La consommation des trois modes et de decision:null est couverte côté T3 ; la projection Rust finale et l’interopérabilité restent à rapprocher par ROOT. Ce rapport contribue à T035, sans valider T054 ni l’audit final.

La lecture ROOT de R4 et la clôture Converge P3 restent attendues. Aucun CONVERGED, audit final, build de livraison, recette T040, G8/G9, commit/push, installation ou nettoyage 163 n’est déduit. Aucun POST sur le modèle de production n’a été exécuté par le rédacteur ; les GET/POST R4 décrits ci-dessous visent le serveur HTTP contrôlé du test.

### Oracles R4 ciblés — résultats indépendamment attestés et logs relus

Toutes les commandes utilisent CARGO_BUILD_JOBS=2, RUST_TEST_THREADS=2, TMPDIR=/tmp/b163.uqd3443p, CARGO_TARGET_DIR=/home/moi/.local/lib/bridget-communication/162-611bfa7557b9/source/target et nice -n 10.

| Commande cargo test | Résultat final du log | Preuve et limite |
|---|---|---|
| -p bridget-daemon --lib spec163_t054_concurrent_auto_configured_ne_decide_ni_lance_qu_une_fois | 1 PASS, RC0, 7,93 s | /Users/moi/.cache/bridget-delivery163-20261013/t053-t054-r4-concurrence-auto-replay.log ; un GET santé, un POST décision, une Task et un ManagedSupervisorCommand::Start réellement émis. Aucun processus fournisseur observé. |
| -p bridget-daemon --lib spec163_t027_reouverture_disque_conserve_recu_choix_et_lien_task | 1 PASS, RC0, 5,83 s | /Users/moi/.cache/bridget-delivery163-20261013/t053-t054-r4-reouverture-sqlite.log ; fermeture/réouverture du store sur disque et rollback de Task/projection provoqué par trigger. |
| -p bridget-daemon --test native_delegation_e2e worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement | 1 PASS, RC0, 4,73 s | /Users/moi/.cache/bridget-delivery163-20261013/t053-t054-r4-worker-basefuture-replay.log ; huit profils Worktree, reconnexion, révocation sans effet puis Prepare autorisé sur base_ref antérieure à HEAD. |

Deux échecs initiaux sont conservés : rejeu comparé après tick, qui transforme normalement queued en starting ; identifiant task_id non UUID de la fixture Worktree, refusé StoreUnavailable. Les correctifs restent limités au séquencement et aux identifiants de fixture ; les replays affectés passent. Le journal R4 rectifie aussi la description R3 « Shared » : l’oracle utilisait Worktree. Cette correction de provenance n’efface aucun résultat R3.

La compilation des trois tests et le format ciblé ne remplacent ni la campagne complète R3, ni une FULL, ni les tests Mac/T3, ni une recette du service réel. Aucun décompte global vert ni nouvelle exécution par le rédacteur.

### Nuance de provenance des empreintes R4

Le delta entre les révisions R3 et R4 finales porte sur cinq sources. Les manifestes de la campagne R4 avant/après test portent chacun sur 21 sources et diffèrent sur quatre : native_delegation.rs, delegation.rs, remote_worker.rs et la fixture remote_worker_162.rs. decision_service.rs possède déjà son empreinte R4 finale dans le manifeste avant tests. Le libellé HASHES_BEFORE_AFTER_IDENTIQUES=1 du log encode le RC1 de cmp, donc une différence, pas une identité. Aucun succès de test n’est attribué à des sources inchangées par cette seule comparaison.

Manifestes relus : /Users/moi/.cache/bridget-delivery163-20261013/t053-t054-r4-hashes-before.sha256, SHA-256 ae4f494883384a9da9b3ed944164d0fcb55f70f4b6a21f62a7c47d8023131076 ; /Users/moi/.cache/bridget-delivery163-20261013/t053-t054-r4-hashes-after.sha256, SHA-256 f88aa1bd193141a6bae84bcd0b965b0bc8a3489a701972aa9bc3376664b390d1 ; /Users/moi/.cache/bridget-delivery163-20261013/t053-t054-r4-hashes-compare.log. Les 21 empreintes après tests correspondent aux sources finales selon l’attestation et le transfert, sans écart. Le rédacteur a comparé ces manifestes locaux, sans exécuter de tests ou lire le runtime distant.

### Références finales de l’attestation indépendante R4

attestation_cartae163 rapporte un snapshot de trace de 960 lignes, SHA-256 70efddb58d4fe144a5e6eb29f7d1143dfb99b2ef6aff5bcaa6197fb4177c0a83. RC d’exécution : lignes 856 FAIL, 878 PASS, 882 PASS, 886 FAIL, 924 PASS et 930 format PASS. À 01:18:24 CEST, les 21 sources distantes et le journal R4 concordent avec leurs empreintes finales. Aucun reçu indépendant n’a été écrit, aucun test rejoué ni transfert effectué par cet agent.

Logs complets privés, mode 0600, et copies locales exactes :

- /Users/moi/.cache/bridget-delivery163-20261013/t053-t054-r4-concurrence-auto-replay.log : SHA-256 be00486fa038336d3eb648d390f442c91379432bc5f417fd11b5fa9ce6360dbf. Origine /home/moi/.local/state/bridget-delivery163/t053-t054-r4-concurrence-auto-replay.log.
- /Users/moi/.cache/bridget-delivery163-20261013/t053-t054-r4-reouverture-sqlite.log : SHA-256 50cb9c3a0df1d3dd76fe32e502596ef0144f9bcf7a0047ca89523798a22bbeb0. Origine /home/moi/.local/state/bridget-delivery163/t053-t054-r4-reouverture-sqlite.log.
- /Users/moi/.cache/bridget-delivery163-20261013/t053-t054-r4-worker-basefuture-replay.log : SHA-256 03fb40b5479ea77bf7d2da59588655b192ccbe3a71d4aee88d9f843f35b186d3. Origine /home/moi/.local/state/bridget-delivery163/t053-t054-r4-worker-basefuture-replay.log.

Lecture source rapportée par l’agent, sur R4 : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs, lignes 9000/9008/9019/9037 pour GET/POST/Task/Start ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs, lignes 4069/4090/4093/4095 pour rollback ; /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs, lignes 1161/1190/1262/1314/1331 pour Worktree/base future/reconnexion/révocation. Le rédacteur n’a pas relu ces sources ; il distingue cette attestation du futur rapprochement final ROOT.

### Limite spécifique T055 — oracle MCP non exécuté dans la campagne R3

attestation_cartae163 a recherché la trace complète de 960 lignes : aucun PASS visible pour delegation_mcp::tests::closed_arguments_reject_forged_parent_without_connecting. Les seules mentions sont des lectures de code. Ce test ne porte pas le préfixe spec163_ et ne fait donc pas partie des 28 PASS de bibliothèque R3. Ses nouvelles assertions sur l’entrée auto, les bornes et le parent forgé ne doivent pas recevoir un PASS par extrapolation. Le PASS CLI R3 est distinct. Cette preuve MCP prévue par T029/T055 reste à fournir avant clôture ; aucun test nouveau ajouté ni replay effectué par le rédacteur. Le code concerné est /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_mcp.rs.

### Écart réel P3 et poursuite autorisée — T056, 14 octobre 2026

ROOT a confirmé que preferences est configuré et validé mais n’est pas consommé dans la demande de décision R4. Ce manque concerne l’acceptation US2 et le choix de préférences prévu à T039. ROOT a ajouté T056 dans tasks.md : raccord des préférences figées aux seuls IDs opaques admissibles, dans les champs déjà bornés, et oracle HTTP existant. Inventaire relu : 56 tâches, quatre cochées seulement. Aucun nouveau banc, protocole ou case cochée par le rédacteur.

La couverture documentaire demeure 31/31 exigences associées à des tâches ; elle ne signifie pas 31/31 comportements prouvés. T056 et la preuve MCP de T055 restent ouvertes. P3 ne peut pas être déclaré CONVERGED avec ces manques. ROOT trace la poursuite de correction en phase d’implémentation sous le mandat utilisateur de terminer autonomement, avant contre-revue et audit finaux ; aucun quatrième passage de convergence déguisé ni gate de livraison contourné. Aucun succès futur, audit, build, recette T040 ou G8/G9 déduit.

### P3 NONCONVERGED — phase 5 corrective autorisée, sans P4 formel

ROOT a consigné P3 NONCONVERGED : préférences non consommées (T056) et preuve MCP insuffisante (T055). ROOT autorise la poursuite en phase 5 pour terminer le mandat utilisateur : « ne pose pas de question, finis les développements ... fais tout le nécessaire ». Cette autorisation explicite prime sur l’arrêt à trois passages de la skill qui laisserait l’objectif inachevé. Elle ne crée ni CONVERGED ni quatrième passage formel.

Continuation Cartae : tâche 94799f7a-d849-43cf-aab4-6be90eb318bc, enfant 336d8ef8-795f-450f-950b-eeaa971aa3b5 ; request root163-t056-preferences-frozen-opaque-r5-20261014. Reçu local /Users/moi/.cache/bridget-delivery163-20261013/continuation-preferences163-r5.json, SHA-256 b1de8fd7877ab1cb823e79967976ec3d98ed69919ecdc5350dc193362d06c9a2, relu : native round 4, mission_pending/working. Le libellé correctif r5 n’est ni un verdict ni un nouveau passage Converge. Cette admission n’atteste aucune correction ou réussite future.

Checkpoint source, contre-revue fraîche, audit et livraison attendent la clôture prouvée de T056 et des critères de T055. Le compteur reste 4/56 tant que ROOT n’a pas rapproché les preuves et autorisé leur cochage. Aucun build, recette T040, G8/G9 ou livraison anticipé.

## Checkpoint de validation R5 — 14 octobre 2026

Attestation indépendante attestation_cartae163 : dix PASS finaux, soit T054 un test, RC0/12,20 s, et service neuf tests, RC0/16,80 s ; format des deux sources RC0/1,38 s. Les deux campagnes initiales échouées totalisent trois FAIL conservés : fixture T054 auto_placement_not_fixed, ancienne comparaison GET et PoisonError consécutif. Le seam HTTP a été corrigé pour lire le corps complet selon Content-Length borné avec timeout, puis seuls les contrôles affectés ont été rejoués.

Le test exact delegation_mcp::tests::closed_arguments_reject_forged_parent_without_connecting reste FAIL, RC101/5,83 s : tools(brief_schema).len() donne neuf outils contre huit attendus. Les trois nouvelles assertions de gardes auto ont été exécutées avant l’assertion finale ; aucun PASS global T055 n’est attribué. R6 est autorisée pour cette seule attente 8→9 et un replay exact, sans rejouer les dix succès R5. Aucun PASS R6 anticipé.

Provenance finale : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r5.json, SHA-256 048f38bd7fcdbbec0893f0756886a65fef8d036e06c1d10f1d6ebc85dd3c32d5 ; archive 21 sources SHA-256 d1a398bf79b8b28eae95a59b0a07f6bbc021912d15b13bdeb32bd0544ce05770, vérifiée en lecture seule. Deux sources changées depuis R4, dix-neuf inchangées. Les dix-neuf artefacts t056 R5 et le journal copiés par ROOT sont byte-identiques au manifeste, mode 0600. t056-r5-hashes-after à 23:28:35Z est intermédiaire ; t056-r5-validation-hashes-after à 23:37:16Z est final, SHA-256 f6bc148334f5495c79472782d8c53953ab2b3d96a644b5b2008bd8db01dc641c. Ces preuves ne portent pas sur le MCP R6 courant.

Le rédacteur n’a exécuté aucun test ni commande. 4/56 tâches cochées seulement ; acceptation en lecture seule de T056 en cours. Aucun GREEN global, clôture P3, audit, build de livraison, recette T040 ou G8/G9 déduit.

## Validation exacte R6 — 14 octobre 2026

attestation_cartae163 confirme delegation_mcp::tests::closed_arguments_reject_forged_parent_without_connecting : un PASS, zéro FAIL, 2036 filtrés, PID 1100274, RC0 en 6,38 s. Le contrôle de format est RC0 en 0,04 s. La substitution unique tools(brief_schema).len() attendu huit→neuf concerne le nombre d’outils MCP, pas les champs handoff. Les trois gardes auto restent inchangées. Le FAIL MCP R5 reste conservé ; aucun replay des dix succès R5 ni succès global déduit.

Source R6 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r6.json, SHA-256 09d25daa87620ce6aada5d1132babeee1f0d7510800c908a76133551396ceefe. Log exact /Users/moi/.cache/bridget-delivery163-20261013/t055-r6-mcp-closed-parent.log, SHA-256 02f70366e4b3b46e06f85c18c0bf2eca191b3f418db29e4012a23c111ab5b043 ; origine privée mode 0600 /home/moi/.local/state/bridget-delivery163/t055-r6-mcp-closed-parent.log. La trace aux lignes 1318/1319 corrobore RC0 selon l’agent. L’empreinte de delegation_mcp.rs après la substitution unique est 62336683d8551465ce8e2587d09bfd3b603eec0539bd0bf4b77e062e7af671e1 ; vingt autres sources R5 inchangées. Le rédacteur archive ces attestations sans nouveau test ni commande.

T055 dispose maintenant de sa preuve exacte ; son cochage reste sous autorité ROOT. La contre-revue CODE Claude R2 demandée à 01:51 CEST est encore en attente. Progression 4/56 ; aucun audit, build, recette T040, G8/G9 ou clôture P3 future attesté.

## Contre-revue CODE R2 reçue — F1 retenue, R7 admise

Verdict externe APPROVE_WITH_CHANGES, Claude Sonnet 5.5 en lecture seule sur 21 sources R6 concordantes. Reçu /Users/moi/.cache/bridget-delivery163-20261013/code-review163-r2-final.json, SHA-256 8477c4e88a67f0ce5d3d65680d5475c6b4a1aff38247de14870889c0f3c71c3c, relu localement. ROOT confirme F1 moyenne : un refus local avant la clôture, notamment reply_timeout=10 ou base_ref inconnue en Worktree, peut laisser chosen actif ; le rejeu retourne admission_pending et le slot reste pris. T057 réemploie validation et refus CAS locaux Shared/Worktree/reprise, sans nouveau banc. Les preuves R6 antérieures restent distinctes ; aucune correction R7 encore attestée.

R7 admise selon ROOT : tâche 94799f7a-d849-43cf-aab4-6be90eb318bc, enfant 336d8ef8-795f-450f-950b-eeaa971aa3b5, native round 7 ; ownership exclusif native_delegation.rs et ses oracles affectés. Permis Cargo ciblé jobs2/nice10/tests2, sans replay des 28 bibliothèque, neuf HTTP, worker, T3 ou FULL. Aucun test lancé par le rédacteur ; admission ne vaut pas résultat.

F2, faible à moyenne, reste une réserve de cadence : reprise jusqu’à 128 reçus, Prepare synchrone jusqu’à trente secondes et erreurs techniques réentretenues. ROOT précise que les pending/started/chosen sont compris dans le plafond global 128 ; l’affamement SQL permanent allégué par le reviewer n’est pas prouvé par LIMIT 128 seul. F3 faible reste une réserve sur le résultat de probe après cinq secondes, rejet CommandMismatch/WorkerRejected et reconnexion possible. Aucun chrono ni correctif de scheduler/connexion prétendu ; limites détaillées dans spec/plan et reçu de revue.

Les O1 absence de route, O3/O4/O5 et T056 sont jugés résolus par le reviewer sur son périmètre ; F1 reste la suite locale d’O1. Audit R6 readonly en cours sur sept modules, delta R7 frais requis avant grade ; archive R6 prête mais source finale non clôturée. 57 tâches/quatre cochées ; P3 NONCONVERGED historique et phase 5 corrective autorisée conservés. Aucun P4, audit final, build, recette T040, Git, G8/G9 ou livraison annoncé.

## Checkpoint préparatoire T057 R8 — 14 octobre 2026

ROOT et attestation_cartae163 ont confirmé que R7 traite les refus métier locaux, mais peut faussement clore un reçu après une erreur SQL INSERT Task provoquée par le trigger fixture_task_sql lorsque l’UPDATE de refus réussit. Cette panne n’est pas un refus métier. Les résultats déclarés R7 et ses échecs restent historiques ; ils ne prouvent pas la préservation de chosen dans ce cas. T057 reste ouverte.

R8 est en cours sur Cartae selon ROOT depuis 00:25Z le 14 octobre. Ownership courant exclusif : /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs et /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/workspace_git.rs, avec les oracles existants affectés. Le périmètre de provenance à capturer passe de 21 à 22 sources pour inclure workspace_git.rs ; aucun manifeste R8 final concordant encore attesté. Un petit discriminant Git interne doit distinguer une absence de base attestée d’une erreur E/S, timeout ou sortie tronquée. Aux trois fermetures concernées, seuls les codes métier d’une liste fermée peuvent terminer le reçu ; aucune analyse libre de chaînes d’erreurs arbitraires. SQL et erreurs techniques restent visibles, non résolues, avec leur dernière révision confirmée et sans faux refus durable.

Source R7 historique : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r7.json, SHA-256 d661b9685c61b68cf02337e522d84243d05dbdca79c2b31ea25e7a47821519bb. Ce manifeste porte sur 21 sources, native_delegation.rs seul changé ; il ne prouve ni les 22 sources futures ni R8. Journal copié /Users/moi/.cache/bridget-delivery163-20261013/remote-r7-developpement-cartae163.md, SHA-256 a5ee0ca201ee59624c9943dcedf09703d63f1fc7d15dd3a4e88d872f2cd35d96. Aucun résultat R8 attesté à ce checkpoint.

Audit R6 en cours en lecture seule : ses drafts doivent relire le delta R8 sur ces deux sources avant scoring final. Aucun grade final, correction SQL réussie, PASS R8, nouveau Converge, recette T040, G8/G9 ou livraison déduit. Le checkpoint P3 NONCONVERGED et la poursuite autorisée en phase 5 restent conservés ; 57 tâches/quatre cochées seulement. Aucun source, test, build, Git ou service modifié/exécuté par le rédacteur.

## Table préparatoire d’acceptation T005–T037 et T046–T057 — R8/R9 ciblées attestées, revue/audit final ouverts

Chaque ligne donne la source, l’oracle existant et la preuve historique disponible. Une ligne n’est pas un PASS de tâche : ROOT doit confronter le correctif R9 testé, la contre-revue finale, les limites et le delta d’audit avant acceptation. Aucun test nouveau, scénario hors plan, replay de succès inchangé ou cochage effectué. Les noms Rust R3 ont été rapprochés de l’archive sources163-r3-remote-snapshot.tgz : 28 noms spec163_ dans les sources de bibliothèque, cohérents avec la campagne R3 attestée. Les numéros de source ci-dessous viennent du checkout Mac R7 et des neuf fichiers T3 relus ; ils restent des repères historiques, pas des lignes R9 futures.

### Repères de sources — chemins absolus

- REG : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/registry.rs
- STORE : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation.rs
- NAT : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs
- GIT : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/workspace_git.rs
- HTTP : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/decision_service.rs
- WORKER : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/remote_worker.rs
- FIX : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs
- MCP : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_mcp.rs
- CLI : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/cli.rs
- CLI-L : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/cli_lineage.rs
- CLI-T : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/tests/lineage_cli_test.rs
- LINEAGE : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/delegation_lineage_tests.rs
- NATIVE-L : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_lineage_tests.rs
- WIRE : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/protocol.rs
- WIRE-L : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-transport/src/lineage_protocol.rs
- TS-C : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/packages/contracts/src/bridgetLineage149.test.ts
- TS-R : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/server/src/bridget/BridgetReader149.test.ts
- TS-U : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx
- TS-O : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts

### Repères de preuves — exécutions distinctes

- R1 Linux : /Users/moi/.cache/bridget-delivery163-20261013/linux-validation163-final.json, SHA-256 7dc90d5eb8f824bf4d6a6a746010cdd6521ebb42419f8f5951883682581f4791. Groupe27 attesté, 21lib/3CLI/1worker/2transport ; commandes seq1457–1458. Il ne sélectionne pas les tests legacy sans spec163_.
- R3 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r3.json et attestation de trace décrite plus haut : 28lib/1CLI/1worker. Les noms ne prouvent pas que leurs corps modifiés ultérieurement ont été rejoués.
- R4 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r4.json et trois logs exacts déjà archivés plus haut : concurrence, disque et worker.
- R5 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r5.json et dix PASS finaux attestés, T0541/service9 ; MCP FAIL distinct conservé.
- R6 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r6.json et /Users/moi/.cache/bridget-delivery163-20261013/t055-r6-mcp-closed-parent.log : MCP exact1 PASS.
- T3 : /Users/moi/.cache/bridget-delivery163-20261013/t3-validation163-final.json et /Users/moi/.cache/bridget-delivery163-20261013/t3-t050163-r4.json ; self-review neuf hashes. Groupe191 préparatoire, contrats30, fixtures446 et format/tokenisation ont des provenances séparées.

| Tâche | Source | Oracle exact existant | Preuve historique disponible | Limite / reste avant acceptation |
|---|---|---|---|---|
| T005 | STORE:3499 | spec163_recu_auto_additif_borne_et_idempotent | R3 lib28 attestée ; R1 lib21 ancienne. | Canon/métadonnées/idempotence ciblés ; ne valide pas tout le pipeline natif. |
| T006 | STORE:3544/4135 | spec163_choix_et_task_sont_durables_et_atomiques ; spec163_projection_ne_publie_qu_apres_commit_externe | R3 lib28 ; rollback sur disque R4 attesté. | Publication/transaction du store ; faux refus SQL corrigé R8 ; classement E/S R9 testé, revue/audit final ouverts. |
| T007 | REG:2021 | spec163_configuration_admission_est_stricte_et_desactivee_par_defaut | R3 lib28 attestée. | Configuration stricte et valeurs par défaut ; aucune politique de production activée. |
| T008 | NAT:8718 | spec163_axes_independants_figent_les_profils_sans_effet_reseau | R3 lib28 attestée. | Axes et lecture fraîche ; corps NAT modifié R7/R8, rapprochement final ROOT requis. |
| T009 | REG/NAT | spec163_configuration_admission_est_stricte_et_desactivee_par_defaut ; spec163_axes_independants_figent_les_profils_sans_effet_reseau | R3 lib28 attestée. | Campagne regroupée ; ne représente pas de nouveaux PASS sur R8. |
| T010 | REG + documents | spec163_configuration_admission_est_stricte_et_desactivee_par_defaut ; spec163_axes_independants_figent_les_profils_sans_effet_reseau | Tests R3 ; documents et choix utilisateur disponibles. | Acceptation de relecture configuration ROOT à consigner ; profils de production T039 futurs. |
| T011 | NAT | spec163_axes_independants_figent_les_profils_sans_effet_reseau ; spec163_admission_locale_recue_atomique_et_rejeu_sans_delegue_externe | R3 lib28 ancienne. | Candidat unique/zéro et filtre local ; refus local/SQL prouvés R8 ; classement E/S R9 ciblé attesté ; revue/audit final ouverts. |
| T012 | WORKER/GIT/FIX:1003 | worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement | R4 worker1 attesté. | Worktree futur/base antérieure à HEAD ; Git10 R8 attestés ; transport de classe E/S R9 testé ; revue/audit final ouverts. |
| T013 | WORKER/WIRE | spec163_auto_delegate_et_probe_restent_des_contrats_fermes ; worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement | Transport2 R1 attestés ; worker1 R4 attesté ; lecture CODE R2. | Wire fermé et époque ; ce n’est pas une recette d’ancien worker réel. |
| T014 | NAT/FIX | spec163_axes_independants_figent_les_profils_sans_effet_reseau ; spec163_admission_locale_recue_atomique_et_rejeu_sans_delegue_externe ; worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement | R3/R4 ciblés ; lecture R2. | Préfiltre/élu revalidé ; SQL locale prouvée R8 ; classe E/S R9 testée ; revue/audit final ouverts. |
| T015 | FIX/WORKER:1003 | worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement | R4 worker1 attesté, RC0/4,73s ; échec initial conservé. | Huit Worktree, reconnexion/révocation/base future ; pas de processus fournisseur observé. |
| T016 | WORKER + contrat | worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement ; spec163_auto_delegate_et_probe_restent_des_contrats_fermes | R4/R1 ciblés ; routes autorisées dans docs162. | Relecture ROOT des trois routes et transmission wire à consigner ; aucune route nouvelle activée. |
| T017 | HTTP:935/955 | spec163_endpoint_et_runtime_sont_refuses_avant_reseau ; spec163_minreq_timeout_present_vide_ou_invalide_refuse_le_service | R3 lib28 ; service9 R5 attestés. | Garde MINREQ_TIMEOUT et limites dans module ; pas de recette du daemon installé. |
| T018 | HTTP:1053/1117 | spec163_sante_et_decision_partagent_un_budget_unique ; spec163_corps_excessif_et_permis_occupe_restent_visibles | R3 lib28 ; service9 R5 attestés. | Budget/permis sur serveur contrôlé ; aucune garantie de terminaison d’un réseau bloqué. |
| T019 | HTTP:974/1084 | spec163_sante_laya_fermee_cache_les_refus_sans_post ; spec163_sante_cachee_et_decision_bornee_utilisent_le_vrai_transport | R3 lib28 ; service9 R5 attestés. | Santé/cache/refus sans POST ; endpoint réel non activé. |
| T020 | HTTP:823 | spec163_laya_normalise_sans_prompt_ni_candidat_hors_snapshot | R3 lib28 ; service9 R5 attestés ; lecture T056 R2. | Normalisation et requête contrôlée ; protocole/checkpoint réel T040 futur. |
| T021 | HTTP:823–1117/NAT | Neuf oracles spec163_ de decision_service::tests:: ; spec163_t054_concurrent_auto_configured_ne_decide_ni_lance_qu_une_fois étendu T021 | Service9 R5 RC0/16,80s ; oracle T054 T021 final1 PASS/RC0/7,718489s, échecs historiques conservés. | Réponse tardive après vraie reprise via serveur contrôlé ; aucun résultat service de production, aucune FULL. |
| T022 | HTTP + recherche/contrat | Oracles T018/T019 et lecture CODE R2 des limites | Résultats R3/R5 ; limites réseau écrites. | Relecture/acceptation ROOT et maintien off avant recette à consigner ; pas une recette réelle. |
| T023 | STORE/NAT | spec163_recu_auto_additif_borne_et_idempotent ; spec163_choix_et_task_sont_durables_et_atomiques ; spec163_t054_concurrent_auto_configured_ne_decide_ni_lance_qu_une_fois | R3/R4, concurrence1 R5 attestés. | Snapshot/CAS/choix ; conservation SQL locale attestée R8 ; E/S R9 testée ; revue/audit final ouverts. |
| T024 | NAT | spec163_admission_locale_recue_atomique_et_rejeu_sans_delegue_externe ; closed_arguments_reject_forged_parent_without_connecting | R3 locale ; MCP exact R6 ; local R8 et R9 attestés ; R7 historique insuffisante pour SQL. | Local/reprise R8 attestés, y compris SQL INSERT Task puis UPDATE refus ; E/S R9 testée ; revue/audit final ouverts. |
| T025 | NAT | spec163_t025_entretien_reprend_une_epoque_ancienne_sans_redecision | R3 lib28 ; reprise R8/R9 exactes attestées ; R7 déclarée historique. | Absence owner et entretien ; F2 cadence reste réserve, SQL locale prouvée R8 ; classe E/S R9 testée ; revue/audit final ouverts. |
| T026 | STORE:3630/3698/3758 | spec163_t026_compteurs_union_places_et_collision_native ; spec163_t026_purge_refus_auto_tombstone_et_retention_trente_jours ; spec163_t026_ne_purge_ni_vivants_ni_etat_non_committe | R3 lib28 attestée ; lecture CODE R2. | Rétention/pression/tombstone ciblées ; ne prouve pas des erreurs SQL arbitraires du puits NAT. |
| T027 | STORE/NAT | spec163_t027_reouverture_disque_conserve_recu_choix_et_lien_task ; spec163_t054_concurrent_auto_configured_ne_decide_ni_lance_qu_une_fois ; spec163_t049_repli_absent_refuse_durablement_et_l_erreur_sql_reste_visible ; spec163_t025_entretien_reprend_une_epoque_ancienne_sans_redecision | Disque/concurrence R4 et T054 R5 attestés ; R3 autres ciblés. | SQL INSERT Task corrigée/prouvée R8 ; classement E/S R9 testé, revue/audit final ouverts avant garantie complète. |
| T028 | CLI:8080/MCP:467/CLI-T:459 | spec163_auto_delegate_cli_conserve_le_payload_ferme_du_handler_mcp ; spec163_mcp_auto_delegate_negocie_et_preserve_le_wire_ferme ; spec163_auto_delegate_cli_negocie_le_feature_et_ne_masque_pas_les_erreurs | R3 lib28 + CLI1 attestés ; CODE R2. | Wire/capacité ; aucun modèle ou serveur de production lancé. |
| T029 | MCP:849/CLI-T:459 | closed_arguments_reject_forged_parent_without_connecting ; spec163_auto_delegate_cli_negocie_le_feature_et_ne_masque_pas_les_erreurs | MCP1 exact R6 RC0/6,38s ; CLI1 R3 attesté. | Gardes avant connexion et wire ; Task CLI simulée n’est pas preuve d’admission réelle ; classe E/S du puits local R9 testée, sans syscall EIO forcée. |
| T030 | LINEAGE:2937/CLI-L/WIRE-L | spec163_lineage_v3_projette_seulement_le_resume_ferme_de_decision ; spec163_lineage_v3_decision_est_ferme_et_v1_v2_restent_distincts ; spec163_lineage_cli_negocie_v3_seulement_pour_list_et_show | R3 lib28 ; transport2 R1 ; lecture CODE R2. | Projection/codec ; runtime T3 chargé non renouvelé, interopérabilité finale à rapprocher. |
| T031 | TS-C:582/657/TS-R:853/1028/1062 | requires nullable decision only in v3 and preserves every supported mode ; rejects decision enrichment through the frozen v1 and v2 decoders ; discards v3 staging before reading a homogeneous v2 snapshot and detail | T3 groupe191, contrats30 T050, self-review neuf hashes. | Fixtures du consommateur et versions ; pas de runtime T3 nouveau ni de lancement fournisseur. |
| T032 | TS-U:401 | met à jour la décision dans l’infobulle sans inventer un mode ni interpréter le texte | T3 groupe191 et self-review neuf hashes. | Trois modes/null/texte inerte ; rendu de fixture, pas recette application active. |
| T033 | WIRE-L/LINEAGE/CLI-T/NATIVE-L | spec163_lineage_v3_decision_est_ferme_et_v1_v2_restent_distincts ; spec163_lineage_v3_projette_seulement_le_resume_ferme_de_decision ; spec163_lineage_cli_version_explicite_est_stricte_et_v3_garde_placement ; native149_gardes_client_et_authorite ; delegation149_page_bornes_ordre_pagination_budget_et_curseur ; native149_cancel_recu_rejeu_et_mismatch_via_flux | Transport2 R1 ; R3 lib28 ; CLI R2 exact2 ; quatre anciens contrôles R8 attestés, références ci-dessous. | Le store v3 n’exerce pas garde humaine daemon. Quatre anciens contrôles R8 nominativement attestés ; pas d’attribution R1/R3 par extrapolation ni de runtime T3 neuf. |
| T034 | TS-C/TS-R/TS-U/TS-O:1759 | Oracles T031/T032 ; persists remote metadata independently of master availability without launching a provider ; does not negotiate or replay journal reads and cancellation | Groupe191, contrats30, self-review neuf hashes ; fixtures446/typecheck web distincts. | Persistance et absence run/provider/outbox ; pas de runtime interop actif. T050 format relié par tokens, pas replay191. |
| T035 | Documents + sources finales | Table31/57 et self-review T3 neuf hashes ; contre-revue CODE R2 | Analyse documentaire et revue T3 reçues. | Acceptation transversale ROOT après R9/audit encore attendue ; tableau n’est pas un cochage. |
| T036 | Documents + campagnes | Oracles historiques ci-dessus ; aucun nouvel oracle de confort | P3 NONCONVERGED historique, phase5 corrective autorisée. | Clôture fonctionnelle après R9 et rapprochement des preuves prévues ; pas de P4 formel ni FULL nouvelle déduite. |
| T037 | Revue/audit | CODE R2/R3/R4 APPROVE_WITH_CHANGES ; audit32 en rafraîchissement | Revues R2/R3/R4 reçues ; audit en cours readonly. | SQL R8 prouvée, perte de classe E/S R9 testée ; revue/audit final ouverts ; delta R9 requis avant scoring final. Aucun grade ou APPROVE final. |
| T046 | LINEAGE:2937/CLI-T:410 | spec163_lineage_v3_projette_seulement_le_resume_ferme_de_decision ; spec163_lineage_cli_version_explicite_est_stricte_et_v3_garde_placement | Store R3 attesté ; CLI R2 exact2 désormais attesté par trace/journal, lecture CODE R2. | Pas d’empreinte avant test ; snapshot final CLI R2 qualifié séparément. Observation runtime final à rapprocher, pas de PASS R8 déduit. |
| T047 | CLI-L/CLI-T/TS-R | spec163_lineage_cli_negocie_v3_seulement_pour_list_et_show ; spec163_lineage_cli_v3_invalid_request_ne_declenche_pas_de_repli ; refuses mixed success versions without downgrade or partial publication | Négociation lib R3 ; invalid_request CLI R1 historique puis R2 exact2 nominatif ; T3 groupe191/self-review. | CLI R2 attesté sans empreinte avant test ; snapshot final distinct. Versions/staging, pas de nouveau runtime T3. |
| T048 | STORE:3817 | spec163_t048_reservation_precede_les_plafonds_et_rejoue_sans_double_compte ; spec163_t026_compteurs_union_places_et_collision_native | R3 lib28 attestée. | UNION/réservation/rejeu/place propre ; refus SQL local NAT découvert R7 distinct. |
| T049 | NAT/HTTP | spec163_t049_repli_absent_refuse_durablement_et_l_erreur_sql_reste_visible ; spec163_sante_et_decision_partagent_un_budget_unique ; spec163_corps_excessif_et_permis_occupe_restent_visibles | R3 ciblé et service9 R5 attestés. | Repli/SQL de transition et permis ; ne couvre pas par analogie SQL INSERT Task R8. |
| T050 | TS-C:597 | bounds reason in UTF-8 bytes and rejects unknown decision fields | Contrats30 PASS T050 ; format final et égalité tokens2041/5814 ; self-review. | Pas de replay191 ni runtime infobulle nouveau. |
| T051 | HTTP:974/1053 | spec163_sante_laya_fermee_cache_les_refus_sans_post ; spec163_sante_et_decision_partagent_un_budget_unique | R3 lib28 et service9 R5 attestés. | Serveur contrôlé, corps borné/TTL/cooldown ; aucun POST réel du décideur de production. |
| T052 | HTTP:913/955 | spec163_sante_laya_accepte_les_metadonnees_connues_sans_leur_donner_autorite ; spec163_minreq_timeout_present_vide_ou_invalide_refuse_le_service | R3/R5 ciblés ; GET santé réel271octets préalable. | GET préalable et parsing fixture distincts ; configuration/recette T039/T040 futures. |
| T053 | FIX/WORKER/GIT | worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement | R4 worker1 attesté. | Git base future/permissions réemployés ; Git10 R8 attestés ; classe E/S corrigée/testée R9, Git nested1 ; revue/audit final ouverts. |
| T054 | NAT/STORE | spec163_t054_concurrent_auto_configured_ne_decide_ni_lance_qu_une_fois ; spec163_t027_reouverture_disque_conserve_recu_choix_et_lien_task | R4 concurrence/disque ; T0541 R5 ; oracle étendu T021 final1 attesté. | 1GET/1POST/1Task/Start émis, pas processus fournisseur. T021 couvre reprise epoch+1 réelle puis réponse tardive déjà résolue/rejeu Starting sans duplication ; SQL R8 et E/S R9 restent preuves distinctes. |
| T055 | MCP:849/CLI-T:459 | closed_arguments_reject_forged_parent_without_connecting ; spec163_auto_delegate_cli_negocie_le_feature_et_ne_masque_pas_les_erreurs | MCP1 exact R6 et CLI1 R3 attestés. | Mission/canon/parent forgé avant connexion ; faux maître CLI ne prouve pas l’admission métier. |
| T056 | NAT/HTTP | spec163_t054_concurrent_auto_configured_ne_decide_ni_lance_qu_une_fois ; spec163_laya_normalise_sans_prompt_ni_candidat_hors_snapshot | T0541 et service9 R5 attestés ; acceptation RO decision_preflight et CODE R2. | POST réel sept critères ; max huit/instructions/state construction pure. Ce n’est pas recette maximale T040. |
| T057 | NAT/GIT | spec163_admission_locale_recue_atomique_et_rejeu_sans_delegue_externe ; spec163_t025_entretien_reprend_une_epoque_ancienne_sans_redecision ; cas SQL fixture_task_sql dans les oracles affectés | Deux natifs + Git10 R8 ; T024/T025/Git nested3 R9 attestés ; CODE R3 historique et R4 APPROVE_WITH_CHANGES qualifiée. | SQL et base inconnue R8 prouvées ; EIO typé via mapper/puits R9 conserve chosen/noTask/rx0. Pas de syscall OS forcée ; qualification R4 acceptée ROOT ; audit final/cochage encore ouverts. |

### T033 — état historique avant récupération des quatre preuves R8

Les oracles v3 de store/codec/CLI sont identifiés. Le reçu R1 nomme invalid_request. La preuve nominative R2 de spec163_lineage_cli_version_explicite_est_stricte_et_v3_garde_placement et spec163_lineage_cli_v3_invalid_request_ne_declenche_pas_de_repli a depuis été retrouvée et attestée, sans replay ; détails ci-dessous. Aucun succès n’est déduit du seul renommage. Le succès du store v3 ne prouve pas une garde humaine du daemon simplement parce que le propriétaire de fixture s’appelle humain. Les oracles legacy existants couvrent effectivement les gardes, pagination et annulation : NATIVE-L:292, LINEAGE:625, NATIVE-L:541 et CLI-T:864. Le reçu /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/149-sous-agents-lineage/validation/native-lineage-tests-wire-r2.md décrit préparation complète et exécution zéro, pas un PASS. /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/149-sous-agents-lineage/validation/final.md rapporte1840PASS agrégés historiques, sans preuve nominative retrouvée dans cette lecture. Ni le groupe27 R1 ni le filtre spec163_ R3 n’attribue un PASS à ces quatre noms non sélectionnés.

Si ROOT ne dispose pas d’une preuve historique nominative réutilisable et juge leur exécution nécessaire au gate T033 prévu, les filtres existants suivants suffisent pour le créneau Linux final ; aucune commande lancée ici, aucun scénario ajouté :

```text
cargo test -p bridget-daemon --lib daemon::native_lineage::native_lineage_tests::native149_gardes_client_et_authorite -- --exact --test-threads=2
cargo test -p bridget-daemon --lib delegation::lineage::delegation_lineage_tests::delegation149_page_bornes_ordre_pagination_budget_et_curseur -- --exact --test-threads=2
cargo test -p bridget-daemon --lib daemon::native_lineage::native_lineage_tests::native149_cancel_recu_rejeu_et_mismatch_via_flux -- --exact --test-threads=2
cargo test -p bridget-daemon --test lineage_cli_test lineage_cli_149_bout_en_bout_liste_show_journal_et_cancel -- --exact --test-threads=2
```

Exécuter seulement sous permis ROOT sur le worktree Cartae isolé, avec CARGO_BUILD_JOBS=2/nice10/TMPDIR privé et cache déjà attribué. Ces noms ne sont pas une campagne21/28 nouvelle. Leur version legacy ne vaut pas, à elle seule, recette v3 maximale ou runtime T3. Le filtre journal/cancel v3 CLI et le test T3 does not negotiate or replay journal reads and cancellation restent leurs preuves ciblées séparées.

Restent réellement ouverts dans ce dossier : SQL locale R8/T057 et son discriminant Git ; preuve nominative des anciens contrôles T033 si exigée par ROOT ; acceptations documentaires/manuelles T010/T016/T022/T035 ; clôture T036 et audit T037. Les gates de livraison T038–T045 sont hors de cette table et restent futurs. Aucune absence de preuve transformée en réussite.

### Complément T033 — CLI R2 nominativement attesté, sans replay

attestation_cartae163 a retrouvé la commande cargo test -p bridget-daemon --test lineage_cli_test spec163_lineage_cli -- --nocapture dans /home/moi/.codex/sessions/2026/10/13/rollout-2026-10-13T20-13-22-01a136e5-f325-7f72-95ef-c10f27c7ef13.jsonl : appel call_4ccafa63ba504fd49f98fa154efcf147, ligne 1828 à 2026-10-13T22:00:56.198Z ; sortie complète ligne 1829 à 22:01:00.658Z. Les deux noms exacts sont spec163_lineage_cli_version_explicite_est_stricte_et_v3_garde_placement et spec163_lineage_cli_v3_invalid_request_ne_declenche_pas_de_repli : 2 PASS/0 FAIL/8 filtrés, temps du banc 0,04 s. SHA-256 stdout : 21641ab510f67350e4d1230803b61d7553255c503e0fc7b5978345092e8ff8a7.

RC0 corroboré dans /home/moi/.local/share/bridget-worker/cartae/journals/77d51437-317d-4562-ad42-3c78b417e63c-g472/26a3400e-8f93-4811-a251-de84dd17e6b2/2026-10-13.jsonl, seq3124, item exec-b86249ba-f773-4ecc-9e74-fcc4a7db34c8, même session/commande et 22:01:00Z. Le snapshot final R2 de lineage_cli_test.rs porte SHA-256 3193ed15f20ca467d74e5dcba5f39bcf1a036b9fe72b69bd9ec56bcf1aabfdf7 ; aucune empreinte collectée avant test, donc cette provenance finale est qualifiée séparément du PASS exact. Ces succès R2 ne sont ni ceux de l’ancien nom R1, ni des PASS sur R8.

La limite d’absence de preuve CLI R2 est levée seulement pour ces deux noms. Les quatre anciens filtres gardes humaines/pagination/annulation restent en attente ; aucune tâche cochée, aucune commande exécutée ici, compteur 4/57 conservé.

## Checkpoint R8 attesté / R9 préparatoire — 14 octobre 2026

R8 : 16 PASS ciblés indépendamment attestés (deux natifs T024/T025, dix Git et quatre T033) ; SQL INSERT Task laisse chosen sans Task/Start et la reprise conserve le choix. Périmètre final 23 sources, et non les 22 prévues : workspace_git_tests.rs est inclus explicitement, trois changements et vingt sources inchangées par rapport à R7 augmentée. Preuves détaillées : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md. Échec de compilation initial RC101/E0308 et filtre initial zéro test restent conservés/non comptés.

CODE R3 APPROVE_WITH_CHANGES confirme la perte de classe Technical en String : workspace_path_unsafe peut encore terminer un reçu après une erreur E/S. ROOT retient la correction R9 dans T057, sans nouvelle tâche. R9 code-only préparé sur native_delegation.rs et workspace_git.rs ; trois replays affectés attendus, aucun résultat R9 encore attesté. L’oracle EIO typé prévu vérifiera le passage par la méthode de production et la fermeture, chosen/noTask/rx0 ; il ne forcera pas une syscall OS réellement en erreur.

31/31 couverture documentaire et 4/57 inchangés, aucun cochage. T033 quatre contrôles nominativement prouvés, sans extrapolation vers nouveau runtime T3. P3 NONCONVERGED historique/phase 5 corrective autorisée conservés, aucun P4 formel. Audit final/delta R9, builds, recette T040, Git, G8/G9 et livraison restent ouverts. T039 prépare quatre profils off/on moteur/placement sans appliquer la configuration ; quota CLI à 100 % et nouveaux children ne prouvent pas l’usage opérationnel de ces profils en production.

### Preuves exactes R8 — 16 PASS ciblés

Attestation indépendante attestation_cartae163 ; logs complets lus ici, RC0 corroborés dans le journal natif selon l’attestation. Manifeste /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r8.json, SHA-256 14ad4325319dcede604f72422ea22157a1ecfdbbc25ee8b233882c5354814167 : 23 sources Mac/archive/distant concordantes ; trois modifiées native_delegation/workspace_git/workspace_git_tests, vingt inchangées R7 augmentée. Les 26 artefacts de logs/manifestes sont byte-identiques et privés600 selon l’attestation.

| Oracle ou groupe existant | PASS | Durée commande ; durée banc | Log complet et SHA-256 |
|---|---:|---|---|
| daemon::native_delegation::tests::spec163_admission_locale_recue_atomique_et_rejeu_sans_delegue_externe | 1 | 10,6 s executor arrondi ; 1,06 s | /Users/moi/.cache/bridget-delivery163-20261013/t057-r8-t024-local-final.log ; 5beb5355a8d97376af0797c3c1839a13f5e4e97449415560ca1d8ed98258faf0 |
| daemon::native_delegation::tests::spec163_t025_entretien_reprend_une_epoque_ancienne_sans_redecision | 1 | 6,090 s ; 0,23 s | /Users/moi/.cache/bridget-delivery163-20261013/t057-r8-t025-recovery-final.log ; 8ca976543935e9c9702bd51f2182786d37387e16abbc8f14960a570bc5ee06c0 |
| workspace_git::tests:: | 10 | 7,114 s ; 1,25 s | /Users/moi/.cache/bridget-delivery163-20261013/t057-r8-workspace-git.log ; f3e734ee16daa00a6afd37b61dc2ba2075f54eb6da6181e4941b12f2fc916739 |
| daemon::native_lineage::native_lineage_tests::native149_gardes_client_et_authorite | 1 | 5,910 s ; 0,06 s | /Users/moi/.cache/bridget-delivery163-20261013/t033-r8-native149-authority.log ; faa7aa0c9478e6b12836580ad63dfa3909abccb7897892e229233abe9b24c4e1 |
| delegation::lineage::delegation_lineage_tests::delegation149_page_bornes_ordre_pagination_budget_et_curseur | 1 | 5,955 s ; 0,04 s | /Users/moi/.cache/bridget-delivery163-20261013/t033-r8-delegation-lineage-page-corrected-path.log ; 1c95301405cf89e358732916452534587786a641a1b69e43aa5d9c07cd9aca9c |
| daemon::native_lineage::native_lineage_tests::native149_cancel_recu_rejeu_et_mismatch_via_flux | 1 | 5,976 s ; 0,07 s | /Users/moi/.cache/bridget-delivery163-20261013/t033-r8-native149-cancel.log ; 007bf5388bcfccb7d363bdad1410dfb56f5ec5ede7ed9f9159de6423a5d2b3f0 |
| lineage_cli_149_bout_en_bout_liste_show_journal_et_cancel | 1 | 11,393 s ; 0,66 s | /Users/moi/.cache/bridget-delivery163-20261013/t033-r8-lineage-cli.log ; eca58881899b21c483b7277c9d2809813014856efd035f33b4c16e189bcd27dd |

Premier T024 : compilation RC101/E0308, 9,366 s, conservée. Premier filtre store : RC0 mais zéro test, 5,777 s, non compté ; filtre au namespace corrigé fournit le PASS. Format final RC0/1,387 s et diff final RC0. Les 16 PASS ne sont ni un rejeu des groupes R3/R5 ni un GREEN global. Les quatre filtres T033 auparavant en attente sont maintenant satisfaits par ces exécutions nominatives ; les paragraphes précédents décrivent leur état avant récupération de la preuve. Aucun cochage effectué.

## Checkpoint R9 final ciblé — 14 octobre 2026

ROOT atteste le transfert final R9 : 23 sources concordantes Mac/Cartae/snapshot, native_delegation.rs et workspace_git.rs changées, 21 inchangées R8. Trois oracles existants exacts passent avec RC0 : T024 local 6,918 s, T025 reprise 5,859 s, Git worktree emboîté 5,909 s. Format canonique des deux sources et diff-check RC0. Preuves exactes et erreurs historiques : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md.

Le correctif préserve la classe technique jusqu’au puits auto. Les refus déterministes de chemin au préflight restent métier ; après réservation, toute erreur de create en mode auto garde chosen avec diagnostic original et éventuels résidus de branche/chemin. Les erreurs E/S de la passe Commit restent non terminales ; ses gardes pures conservent leur traitement métier. L’oracle EIO typé traverse le mapper de production et la fermeture/store : chosen/noTask/rx0. Il ne force pas une syscall système EIO. Les résultats R8/R9 ne sont pas des suites globales ; aucun replay des succès inchangés.

Compteur 4/57 sans cochage, 31/31 couverture documentaire. ROOT relit le delta R9 et demande CODE R4 ; audit32 refresh en cours, aucun verdict/grade final anticipé. P3 NONCONVERGED historique et phase 5 corrective autorisée conservés, aucun P4 formel. T039 configuration off/on non appliquée, T040 service réel, builds/paquet/signatures, Git/livraison/G8/G9 encore ouverts. Quota CLI100 % et children ne prouvent pas des profils opérationnels en production.

### Preuves exactes R9 — trois replays affectés

Manifeste /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r9.json, SHA-256 91e01b7560889a13e713dc8f7812c34c2dd7194e17ff9ef321e21f861829aaad. Delta /Users/moi/.cache/bridget-delivery163-20261013/source-delta163-r8-r9.patch, SHA-256 57c16512ffac2846febc531f1a9e237c1225314271bc844bbc0e0fc4c9d45b93. Les 18 preuves sont privées600 et byte-identiques dans le cache selon ROOT/manifeste ; 23 sources concordent, 21 inchangées. R9 désigne la révision de sources, pas une ronde native9 inventée.

| Oracle exact | Résultat ; durée commande / banc | Log complet ; SHA-256 |
|---|---|---|
| daemon::native_delegation::tests::spec163_admission_locale_recue_atomique_et_rejeu_sans_delegue_externe | 1 PASS/0 FAIL, RC0 ; 6,918 s / 1,08 s | /Users/moi/.cache/bridget-delivery163-20261013/t057-r9-t024-git256-replay.log ; a2accceb08302cfc74cb65c5c15e048e331fd26bf1e2a0cb4dc6a553410b3ce9 |
| daemon::native_delegation::tests::spec163_t025_entretien_reprend_une_epoque_ancienne_sans_redecision | 1 PASS/0 FAIL, RC0 ; 5,859 s / 0,08 s | /Users/moi/.cache/bridget-delivery163-20261013/t057-r9-t025-git256-final.log ; b93f4fdfe6a0d1356b279a139f69ba21a375969d6f6888bdfe7b2d47aeec3af5 |
| workspace_git::tests::native154_git_creation_depuis_sous_dossier_et_worktree_emboite | 1 PASS/0 FAIL, RC0 ; 5,909 s / 0,14 s | /Users/moi/.cache/bridget-delivery163-20261013/t057-r9-git-nested-git256-final.log ; 3f04569a53b31c35fa58e460d8ede1849815318e0e3098e7e02bed596b1dc695 |

Empreintes finales des deux sources changées :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs : f5c40118dd062807b46795d0363ece590bf26e63aa69506932b3880371dc9631
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/workspace_git.rs : dfb08b8606c5c6fd9c1379aca1c2df6d179426721ca3e1810407e5701044c4dc

Échecs conservés : T024 initial RC101/14,038 s avec Git système ancien2.43 ; replay Git2.56 déjà installé, aucune modification de code entre ce FAIL de prérequis et le PASS. Formatter initial absent du PATH : exception du wrapper, formatter/compiler non lancés et source inchangée ; retry stable refuse skip-children RC1 ; format stdin canonique puis contrôle d’égalité RC0 sur les deux sources. Incident de wrapper conservé dans les reçus ROOT, pas masqué comme un PASS. Aucun outil installé, nouvelle suite ou scénario ajouté par ce checkpoint.

Les trois replays ferment le manque d’exécution du correctif R9 ; ils ne valent pas APPROVE CODE R4 ou grade d’audit. T033 quatre contrôles R8 et CLI2 R2 restent leurs propres preuves. Aucune tâche cochée ni gate de livraison déduit.

## Qualification CODE R4 — 14 octobre 2026

CODE R4 conclut APPROVE_WITH_CHANGES, sans changement de code ni test supplémentaire demandé. F1 faible est traité documentairement : la garde déterministe au préflight ferme le reçu ; le wrapper auto de create garde chosen pour toute erreur, même une garde pure. Une modification entre préflight et create, par exemple .worktrees devenu symlink, peut conserver la place jusqu’à la reprise sous une autre époque du coordinateur. Cette reprise refait le préflight. Un nouvel identifiant de Task et un worktree résiduel supplémentaire restent possibles après échec de création ; diagnostics et résidus de branche/chemin sont conservés. ROOT accepte cette limite, sans correctif R10 ni campagne nouvelle.

F2 faible de couverture : T024 exerce l’EIO typée via into_admission puis fermeture/store et la vraie garde .worktrees fichier vers Refused. Les map_err de create et les erreurs E/S de la passe Commit sont vérifiés par compilation et lecture, sans oracle direct. Une NotFound de canonicalize en Commit est non terminale ; aucun PASS direct de ce cas ni syscall EIO forcée revendiqué. Les trois PASS R9 restent les trois exécutions nommées, pas une preuve supplémentaire de ces chemins.

Reçu /Users/moi/.cache/bridget-delivery163-20261013/code-review163-r4-final.json, SHA-256 5cb0c98f361e7079eb59085ec2988b39636c4378a1fc5f4cab8f1544b7f8a5f0. Reviewer Claude Sonnet 5.5/claudeAgent ; taskId node:delegated-task:command%3Amcp%3Aa3163845-67c8-4e7f-bea0-f779216e5c09%3Adelegate-task%3A163-code-review-20261014-r4. Deux hashes source, manifeste et delta R9 vérifiés ; 21 sources inchangées, T3, logs et résultats de tests non relus par le reviewer. R1/R2 non relus exactement ; aucun test exécuté par cette revue. Audit32 encore en cours, aucun grade/cochage/livraison déduit ; 4/57, P3 NONCONVERGED historique et exception phase5 conservés, aucun P4 formel.

## Checkpoint T021 final — delta de tests seulement, 14 octobre 2026

Le manifeste final R9+T021 porte sur 23 sources : deux corps de tests changés dans native_delegation.rs/decision_service.rs, 21 sources inchangées ; les deux préfixes de production restent identiques à R9. L’oracle T054 existant, étendu à la réponse tardive après reprise réelle, donne 1 PASS/RC0, 7,718489 s commande/0,07 s banc. Premier FAIL de fixture conservé ; aucun test supplémentaire ni replay des succès antérieurs. Preuves exactes : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/tests163.md.

L’entrée de build existante decision-request-max8.json, 709 octets/objet questions choice, a été copiée byte-identique du serveur au miroir Mac. Elle doit être versionnée avant le build Mac T038 ; ce n’est pas une preuve de recette de production. L’ancien artefact privé861 octets/questions tableau reste historique et obsolète. T040 doit utiliser la requête réellement capturée et ses tokens ; aucun fixture ne vaut réussite d’un POST de production. Audit32 et validation finale encore en cours, aucune acceptation49/57 anticipée : 4/57 inchangé. P3 NONCONVERGED historique/exception phase5 intactes, aucun R10 de production ou P4 formel ; builds/G8/G9 encore ouverts.

### Preuves exactes T021 / TEST-001

Manifeste /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r9-t021-final.json, SHA-256 c8876bbdad9d8ab6a89c2b6226c19d3974ef282648adbcaa7a68702050f4d604. Delta /Users/moi/.cache/bridget-delivery163-20261013/source-delta163-r9-t021.patch, SHA-256 7906115390844fe4b9b6b193152a0ee3a3bd8e1c64e30f06e02bbc35ac33e63d. Concordance Mac/snapshot/Cartae attestée ROOT ; deux corps de tests modifiés, préfixes production identiques R9, 21 autres sources inchangées.

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/daemon/native_delegation.rs : d7239915926c0f50647fa5a5a58917f8048ab7a9cde4199ec92d6aa21c9a48f5
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/decision_service.rs : 488478ab43a491cda1b4f4a14204aa3d989c5d434f67fdd97f90076a31882e93

Oracle existant exact daemon::native_delegation::tests::spec163_t054_concurrent_auto_configured_ne_decide_ni_lance_qu_une_fois : 1 PASS/0 FAIL/2036 filtrés, RC0, commande7,718488636 s/banc0,07 s. Log complet /Users/moi/.cache/bridget-delivery163-20261013/t021-test001-t054-final-v3.log, SHA-256 c7d0cf3e4ea621ecfde51e55b448ff5dca12802c42582fed3a3bb8991832f0a3. Aucun it/nouveau test ajouté ; comptes de fonctions191 native et9 service inchangés.

Le POST complet reste suspendu, puis la vraie reprise epoch+1 résout le reçu par CAS vers c07/Fallback/révision3 et émet une Task/un Start intent avant réponse. La réponse tardive c01 reçoit déjà-résolu ; le rejeu retrouve exactement Starting, sans second POST/Task/Start. Premier FAIL RC101 conservé : la fixture attendait encore Queued/révision2 alors que la méthode mappée réelle avait produit Starting ; correction du constat de fixture, sans code de production modifié. ROOT a lu le delta complet puis les neuf lignes du dernier correctif ; attestation fraîche03/06 ferme TEST-001 sur ces preuves, audit32/validator final toujours en cours.

Entrée de build séparée des23 sources : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/validation/decision-request-max8.json ;709 octets, SHA-256 d5ea2390a4474e148ae919e1d33a4fcf3ee79a9b97d816351ef0c8a67033c667. Copie Mac auparavant absente, maintenant byte-identique à celle du serveur ; objet questions/choice/criteria. Versionner avant build Mac T038. L’ancienne construction privée861 octets/a27dabc5 avec questions tableau reste historique, jamais entrée finale. La future recette réelle T040/tokenizer utilisera la requête capturée et ne déduira aucune réussite de production de ce fixture. Aucun PASS Mac, GREEN global, audit final ou recette POST réelle déduit.

## Acceptation ROOT du développement — 49/57, 14 octobre 2026

GO ROOT après lecture du delta T021 final et du reçu FINAL_T021_READONLY_CONFORMANT : T005–T037 et T046–T057 acceptées/cochées, ajoutées aux quatre préparatoires déjà prouvées. T038–T045 restent ouvertes ; aucune recette, build Mac, source poussée/installée, activation ou G8/G9 anticipée. P3 NONCONVERGED reste historique ; l’exception utilisateur de poursuite corrective en phase5 est conservée, aucun P4 formel ou faux CONVERGED ajouté.

Audit final readonly : grade A/99,047619 ; 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW ; TEST-001 fermé par l’oracle T054 existant exécuté. Couverture4,655325 % du snapshot et0,355837 % du contexte global, donc audit partiel et aucun GREEN global. Reçu /Users/moi/.cache/bridget-delivery163-20261013/audit-preparation/audit163-final-t021-receipt.json, SHA-256 057c0b4349388fa46b3cc1c843a7c352c738260d01434161ade6eb5f4810fe87, relu et hash vérifié par le rédacteur. Aucun audit, source, test, Git, build ou service modifié/exécuté par ce cochage documentaire.

## Checkpoint installation et provenance — 53/57, 14 octobre 2026

ROOT accepte T038/T040/T041/T042 sur leurs preuves réelles. Bridget source 5105378bf4f03376e9bc93e52065cb022f81dee5 installé : Mac SHA-256 6c8f89203c85c76e7add2a4d06bb86eedd6cdb3f74470eac11920cc0f6068063, Linux SHA-256 6e5d9717d6054511326f84c684d65c3f3bc1a5086cb514b2d6b64a03ad9b97e0. Maître 44553 né 04:55:03 CEST et pont 44667 né 04:55:05 utilisent /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget ; worker 1411115, naissance 146166371 utilise /home/moi/.local/lib/bridget-communication/163-5105378bf4f0/bin/bridget.

T3 /Applications/T3 Code (Local).app remplacée atomiquement en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18, signature stricte/deep RC0 et entitlements préservés. PID 704/969 et naissances 17:31:27/33 du 13 octobre restent inchangés : aucun redémarrage, ancien code encore chargé. Aucun réveil différé programmé.

Sauvegarde retry : phase_complete/activate, projection173→174 de même génération et 33 autres tables exactes, schéma/count inchangés ; halt humain complet préservé. Migration additive reçusauto, quick_check ok/FK0, schémas 7/12/Fleet5. Sauvegarde jointe vérifiée ; après activation, toute rétrogradation exige une nouvelle sauvegarde jointe sans perte. La sauvegarde préactivation ne couvre pas les nouveaux reçus. L’incident171→172 et les anciens PID 86807/86830/1403968 restent uniquement historiques.

Le reçu d’installation conserve handshake_and_cancel_recipe_not_proven=true : poignée de main et annulation opérationnelles restent à vérifier pour G8.

T039 : /Users/moi/.cache/bridget-core/agents.json est appliquée au SHA 5dead7198c0220477d88229d1b0ab0f4b3fe5858a0391bde1cfb687c3265c93b ; fixed/fallback local-codex-pro, agents/machines inchangés. auto_engine=false, auto_placement=false, service_enabled=false. L’application off est réelle, mais l’activation des axes/service et les preuves G8 restent ouvertes : T039 non cochée, comme T043/T044/T045.

Ordre réel : sources committées/intégrées/poussées avant laboratoire T040, pour obtenir un checkpoint déterministe compilé identique sur Mac/Linux ; validation T040 exécutée avant activation. Ce commit de source préalable n’est pas une clôture de livraison ni un hash futur inventé. ROOT accepte T040 sur observations R4 et supplément corrigé 54056f…8d49 ; RC1/course F historique conservé, aucun nouveau RC0 ou POST ajouté. P3 NONCONVERGED historique/exception utilisateur phase5 conservés, aucun P4 formel.

Reçus :

- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-retry-projection/progress.json ; SHA-256 b6da23d355fbb396b50fab7c946dd5693dd6ced27aafe5ae3ac2aaffaf7cef16
- /Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json ; SHA-256 b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0
- /Users/moi/.cache/bridget-delivery163-20261013/g8-registry-off-applied.json ; SHA-256 ed358ea42dc927adf19d4c4e7cca2ecdd0de76e2aa2a213116f6b15049b26aae

## Annexe T038/T040 acceptée ROOT — provenance et échecs conservés

T038 : Bridget source 5105378bf4f03376e9bc93e52065cb022f81dee5 compilée sur Mac et Linux. Build Mac RC0 en 146,948735 s, signature ad hoc vérifiée strict RC0 ; binaire SHA-256 6c8f89203c85c76e7add2a4d06bb86eedd6cdb3f74470eac11920cc0f6068063. Build Linux RC0 en 86,502369 s, ELF x86-64 SHA-256 6e5d9717d6054511326f84c684d65c3f3bc1a5086cb514b2d6b64a03ad9b97e0 ; aucune signature codesign Linux revendiquée.

14 tests Mac passent dans cinq commandes : 9+2+1+1+1, zéro FAIL, aucune FULL. La politique existante a retiré 46,46 Gio logiques de caches ; aucun gain physique attesté. Premier filtre non rejoué, dépendances recompilées une fois au second filtre. Les builds et leurs observations de production inchangée précèdent la maintenance ; ils ne prouvent pas l’état actuel.

Paquet T3 source 511190beb307e0952066fe1a4b511cb9dd7adb18, version 0.0.45-local.164 : build RC0/176,676 s, signature stricte/deep RC0 et entitlements préservés selon le reçu. ASAR 6eb2fbdfed44a603d0a66d81d92fadd6d3486ff1ff43a880fe788c7d39fc923f. Les 2574 PASS R1 et deux FAIL ZIP restent historiques ; les deux seuls contrôles ZIP affectés passent R2. Aucune nouvelle campagne globale ni installation déduite.

T040 : R4 utilise les binaires réels source 5105378, un maître/worker isolés, le service de décision réel et un fournisseur synthétique. Le cas D refuse auto_candidate_unavailable avant Prepare, zéro commande/slot/processus enfant. N reçoit c01 via un POST réel HTTP200, routing.model=typed-decisions, avec huit critères opaques. Le tokenizer du corps capturé donne 245/245 tokens, huit marqueurs identiques, aucune coupe, sans charger les poids. R3 et R4 comptent deux POST réels au total, un chacun ; pas de nouvel appel dans le supplément.

R3 reste un échec historique full_access_profile_not_denied ; son POST ne vaut pas validation de la recette. La commande R4 conserve RC1/durable_cancel_missing sur une course d’observation du cas froid F. La recette n’est pas réécrite RC0. F est ensuite observée cancelled/cleanup_done=true. Le supplément N corrigé confirme result_available/cleanup_done=true, résultat fixture162-remote-answer inchangé, commande terminale, slot terminal fermé cancelled, drainage et processus de laboratoire absents. N est Shared : guarded=false, aucune garde acquise ; guard_released=false n’est pas présenté comme une libération manquée. L’annulation de clôture conserve le résultat déjà terminal.

Le supplément corrigé SHA-256 54056f844075544f45ae83a8072f7a6c1421ea16bf391d32458a05726d4c8d49 retire l’affirmation finale production_unchanged. Les anciens PID de production sont importés de l’opération close-N, intervalle 02:47:58,740909–02:48:16,106909Z, sans heure individuelle. Ils ne prouvent pas l’état après les opérations ROOT suivantes. Aucun R5, nouveau modèle, succès global ou état courant de production inventé.

Reçus lus, avec empreintes :

- /Users/moi/.cache/bridget-delivery163-20261013/build163-mac-5105378bf4f0.json ; SHA-256 4c634d8ef74dfebf2578081475edfb5dea34a2726150870f0d235e2d02cafb7a
- /Users/moi/.cache/bridget-delivery163-20261013/build163-linux-5105378bf4f0.json ; SHA-256 93edb25e689ed834d231fb806a8f90ade016132748d38b7f80b7c1f6af08a741
- /Users/moi/.cache/bridget-delivery163-20261013/tests163-mac-5105378bf4f0.json ; SHA-256 a5642e75c57a83117eff51a92784157dab383dc3bc998251567eae6a157f9554
- /Users/moi/.cache/bridget-delivery163-20261013/t040-execution-r4/recipe163.json ; SHA-256 67f5268bd03fdae806649da3a795c9122c60469bab4be1f89e16aafc4059e9d1
- /Users/moi/.cache/bridget-delivery163-20261013/t040-execution-r4/native-tokenizer.json ; SHA-256 774e82c383928309524cec1ac2abdea57b22cf7f1942b23689fc41c2540ccd60
- /Users/moi/.cache/bridget-delivery163-20261013/lab163-preparation/t040-r4-cleanup-conclusions-supplement.json ; SHA-256 54056f844075544f45ae83a8072f7a6c1421ea16bf391d32458a05726d4c8d49
- /Users/moi/.cache/bridget-delivery164-20261013/t3-package164-result.json ; SHA-256 2fb1b8f95140f09676d107f2429be07322c7ff77ff094993f1a82f95dff18984

## Correctif G8 — façade MCP T058, 14 octobre 2026

Le MCP propre à la nouvelle commande annonce bridget_auto_delegate dans tools/list, mais tools/call renvoie JSON-RPC -32602, message outil inconnu, request_id g8-spec163-a73f9432-53d5-46b5-94b9-e8f0ee3cfe7e. Les attestations officielles immédiatement avant appel portent version2 et permissions_present=true ; provider codex-pro déclaré disponible et capacité maître contrôlée. Aucun reçu auto/Task sous cette clé selon la vérification durable ROOT. La cause confirmée ROOT est le dispatcher de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:753–760 : routage auto omis malgré l’annonce. Scope/hook/wrapper doivent être raccordés à la voie native existante. Ce refus ne démontre pas un problème de quota ou d’authentification et ne nécessite pas de redémarrage T3.

T058 reste dans la session163 autorisée : correctif minimal sur Cartae, oracles MCP existants affectés, contre-revue/delta d’audit ciblés, build et livraison du correctif avant reprise G8. Aucun nouveau framework, banc de confort ou FULL. Le prochain commit source sera différent de5105378bf4f03376e9bc93e52065cb022f81dee5 ; aucun hash futur fixé. T038/T042 demeurent cochées pour les builds/installations réellement prouvés du510, pas pour le futur correctif. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18/version164 est déjà installée et inchangée. Aucun résultat antérieur effacé, aucun livré final ou G8/G9 déduit.

Compteur courant53/58 : T039/T043/T044/T045/T058 seules ouvertes. La preuve brute est conservée : /Users/moi/.cache/bridget-delivery163-20261013/g8-mcp-off-error-observation.log ; SHA-256 03e823bb6a993a7f1b8b546edc53f6aeded6e163bcd9ed507b22a68dc28f2f47. P3 NONCONVERGED historique et poursuite corrective sous mandat utilisateur restent explicites ; aucun P4 formel inventé.


## Checkpoint source T058 — tests et contre-revue, 14 octobre 2026

Les douze oracles ciblés réussissent : onze tests de bibliothèque et un test d’intégration exact. Le reçu /Users/moi/.cache/bridget-delivery163-20261013/auto-mcp163-g8-validation-final.json porte SHA-256 bcbe4d808ea71c47c11690ef9570c282ed8ba62e324c186de1176e41cf60aa78. Les six empreintes source sont identiques avant et après la campagne. Le filtre initial qui avait exécuté zéro test reste archivé et ne compte pas parmi les PASS. Son nom a été corrigé ; les six premiers succès n’ont pas été rejoués.

La contre-revue indépendante R5 reste APPROVE_WITH_CHANGES. Elle valide les cinq empreintes du premier périmètre par lecture, sans exécuter de test. F1 a été traité ensuite : une seule ligne ajoute bridget_auto_delegate à l’attendu du test d’intégration existant. spec094_claude_recoit_la_liste_fermee_des_outils_bridget_autorises donne 1 PASS/0 FAIL, RC0, 9,584249 s de commande. Les cinq sources déjà revues sont inchangées. Reçu R5 : /Users/moi/.cache/bridget-delivery163-20261013/t058-review-r5.json ; SHA-256 51c128704ecad1610eae22c50038e9c35f7a6deff8a7e8943279d9b22cc36c4d. Aucun nouveau verdict APPROVE final n’est inventé.

L’audit T058 readonly conclut A/100 sur le delta lu, avec 0C/0H/0M/0L actif. Sa couverture est partielle : 1 561/23 127 lignes physiques des six fichiers, soit 6,749687 %. Le validateur canonique donne RC0, zéro erreur et zéro avertissement. L’audit163 précédent conserve ses 3 MEDIUM et 2 LOW ; aucun finding historique n’est effacé ou réputé corrigé. Artefacts : /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-10-14/session-2026-10-14-spec-163-t058-01. read_manifest SHA-256 58538c71eeab71f5fcebd97b4fc19bb68f43b137c6e02025359ec6791c17014a.

Limite conservée : le refus du hook AutoDelegate sans request_id est vérifié par lecture de la garde, sans oracle négatif auto spécifique. L’oracle public tools/call prouve le raccord du wire sur un daemon simulé ; il ne prouve pas une Task durable ni un processus fournisseur réel.

T058 reste ouverte. Reconstruction, réinstallation du correctif et recette G8 réelle restent à faire. Compteurs inchangés : SPEC163 53/58, SPEC164 19/22. La source5105378bf4f03376e9bc93e52065cb022f81dee5 reste chargée au dernier checkpoint de runtime ; aucun nouveau commit, binaire ou succès G8/G9 n’est anticipé. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18 reste inchangée. P3 NONCONVERGED historique et poursuite corrective autorisée sont conservés.

### Empreintes finales de la campagne T058

Les chemins ci-dessous désignent le worktree Linux isolé. Leurs six SHA sont ceux du reçu final ; les copies privées de l’audit ont été comparées aux mêmes valeurs.

| Source absolue | SHA-256 |
|---|---|
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/mcp.rs | ae0881ee92864a23ca30552b132621d5e5a96093c01c0b8128d3c975b867885c |
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/mcp_scopes.rs | 23289c4d220d00ecc471587534f2d3836485f4d439aa8f72b63621c2e07afb11 |
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/wrapper.rs | be18e6f827fa56dc73554f719377e3acabca86d42f6b25f3ea41bdf50f9b2149 |
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/native_permission_observer.rs | 3cdf9c6dbf4ab69ff330a20cca15575834fb4ca76c09cab38eade68b4acec579 |
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/src/native_permission_observer149_tests.rs | db29eb33882a3b42ff42eb99ee57392667eea37c7ed48e5e3c696e49130c1219 |
| /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission/crates/bridget-daemon/tests/claude_native_permissions_test.rs | 2bb6910a4f0a1eb3ef95e38c0c946c0e6391cb0334de611de85915b53ba77cf0 |

Snapshot complet : /Users/moi/.cache/bridget-delivery163-20261013/auto-mcp163-g8-source6-final.tgz ; SHA-256 25ac409e7810222d7bb401877b1bbf9e6c01dc1fbbb62f705f3b5ffb404d0ebf. Patch six fichiers : /Users/moi/.cache/bridget-delivery163-20261013/auto-mcp163-g8-scope6-final.patch ; SHA-256 cfcbfbbfcfc8cbbc5e1db53febeb98d11202732bdb98b3928404f1286458523f.

Les noms exacts, logs complets, commandes et SHA des preuves relues sont consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-10-14/session-2026-10-14-spec-163-t058-01/test_evidence.json. Cet ajout documentaire ne lance aucun test, build, Git ou service et ne coche aucune tâche.


## Livraison opérationnelle T058 et recette G8 — 14 octobre 2026

La source Bridget 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 est intégrée et poussée sur github/main. Le build Mac donne RC0 en 148,075920 s ; signature ad hoc vérifiée strict RC0, binaire SHA-256 cfb1ff4f91e1b59fbbfd403607d4f6dd08f24229724822f3d8eee9b6eb1a381c. Le build Linux donne RC0 en 73,638388 s, ELF x86-64 SHA-256 dfc9c3c78f7c8e16a394bc57f77d06488acde6495be45093674025be4209af97. Les deux builds portent le même commit de source. Le maître Mac 31298 et le pont 31420 sont nés à 05:32:24/26 CEST ; le worker Cartae 1454489 a pour naissance 146390445 et utilise /home/moi/.local/lib/bridget-communication/163-3efa3ef5a88a/bin/bridget. Ces identités remplacent les anciens checkpoints510.

G8 est prouvée par une admission native réelle, avec les automatismes désactivés. Le fournisseur codex-pro/model gpt-6.1-sol retourne exactement RECETTE163_G8_OK pour Task 8aaba836-5451-4497-bfcc-a90d9614ac59. Le résultat est disponible et transmis (result_sent=true), cleanup_done=true. Le workspace Shared/discovery n’a acquis aucune garde : guarded=false et guard_released=false ne sont pas présentés comme une libération de garde.

La configuration effective /Users/moi/.cache/bridget-core/agents.json porte SHA-256 36c7d13c571f97e49f4caaa59c272597c87735fbfd44667068da77f883399cf4 : auto_engine=true, auto_placement=true et serveur de décision activé, protocole laya_systemone_v1/model typed-decisions. L’endpoint est http://127.0.0.1:8792/v1/systemone ; machine de décision laya-lan, santé/reprise 30 s. Les quatre profils restent configurés ; fixed/fallback local-codex-pro. Seul codex-pro est nouvellement prouvé disponible par G8 ; aucune nouvelle validation de quota/auth des autres profils n’est déduite.

Le rejeu de la même clé après activation ON conserve le même Task, le résultat et le reçu auto entier, avec un seul reçu. Cet état durable prouve l’absence de nouvelle décision et de nouvelle Task sur ce rejeu. Aucune capture réseau n’a été effectuée. La recette HTTP réelle reste T040 R4, avec ses qualifications et son RC1 historique conservés : deux POST réels au total dans R3/R4, aucun nouvel appel de décision ajouté pour G8.

T3 /Applications/T3 Code (Local).app est installée en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18. Ses processus 704/969 restent en version 162, sans redémarrage. Le catalogue d’outils de l’hôte conserve sa version précédente jusqu’à rechargement autorisé ; G8 utilise le MCP propre à la commande. Aucune nouvelle recette runtime T3 n’est revendiquée.

ROOT accepte T039/T043/T044/T058 ; progression 57/58 In Progress, T045 seule ouverte pour la clôture documentaire/Git. Le commit source poussé ne prouve pas encore le push de ces métadonnées finales. P3 NONCONVERGED historique et la poursuite corrective autorisée par l’utilisateur en phase 5 restent tracés ; aucun P4 formel inventé.

Reçu G8 : /Users/moi/.cache/bridget-delivery163-20261013/g8-t058-operational-final.json ; SHA-256 5b1c00923f2f307136b5831cc8d3328227b7284deacf44c793d71542e3ceb52c. Archive opérationnelle : /Users/moi/.cache/bridget-delivery163-20261013/g8-t058-operational-evidence.tgz ; SHA-256 9c1e6c33f936fab753f2fbfaa7fe6196a504c87b44a59d9f77e938cea5521eb8.


## Nettoyage G9 accepté ROOT — 14 octobre 2026

Les onze ressources éligibles retirées comprennent trois worktrees Mac, leurs trois branches fusionnées, deux stages de build Mac et six namespaces TMP du lot. Aucun retrait forcé, gain physique de disque ni mutation des bases de production n’est revendiqué. Les premiers refus de nettoyage restent historiques.

Le dernier scan du worktree T3 163 a été complet : 282 139 fichiers dans la fenêtre 180 s. L’inventaire a ensuite changé après l’archive, donc la garde de stabilité a échoué. Le worktree /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission et sa branche sont conservés. Cette cause remplace le seul constat historique du scan 60 s incomplet ; aucune cause d’inode ou mutation de contenu n’est inventée. Reçu : /Users/moi/.cache/bridget-delivery163-20261013/g9-t3wt163-complete-retirement.json ; SHA-256 7e2388b90e22f0b33bb3f6140851c7dedc1a398a5b5d10957a490153d4e13d29.

Sur Cartae, sudo lsof retourne RC0 mais signale fuse.portal et une visibilité incomplète. Le worktree /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission, sa branche, son .connector164, le cache Cargo partagé et les TMP de tests164 restent conservés. Les sources suivies du worktree/main sont alignées sur 3efa et propres. Le refus de la garde d’usage n’est pas contourné.

Les stages de paquet /Users/moi/.cache/t3-final162.2nosz6zd et /Users/moi/.cache/t3-final163.zsb7fmfu contiennent chacun une base persistante de cache SQLite de 20 480 octets ; ils restent protégés, sans prétexte de futur build163. L’ancienne application /Applications/.T3 Code (Local).app.new-163 reste chargée par les processus T3 protégés. Sauvegardes, preuves, retours arrière, bases et caches partagés sont conservés.

G9 est accepté sur ces retraits et conservations nommées. Reçu final : /Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json ; SHA-256 08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3. Les preuves d’exécution e397 et de retrait T3 164 b066 restent rattachées au reçu final. Aucun fichier de production ou service n’est touché par cet ajout documentaire.


## Limites conservées à la clôture

L’audit 163 reste A/99,047619 partiel : 0C/0H/3M/2L, couverture 4,655325 % du snapshot et 0,355837 % du contexte global. Le delta T058 reste A/100 partiel 6,749687 %, 0 finding actif, validateur RC0/0 erreur/0 avertissement. Le delta T3 R6 conserve 0 finding actif sur son seul périmètre. Ces audits ne deviennent pas des audits globaux du produit.

Les suites FULL antérieures avec échecs restent historiques et ne sont pas réécrites GREEN. Les campagnes ciblées réussies restent rattachées à leurs sources. Aucun FULL répété, benchmark nouveau, nouveau test du hook auto sans request_id ou nouveau processus fournisseur issu du test de dispatch n’est inventé. Les douze exacts T058 restent 11 lib + 1 intégration ; le zéro-filtre initial reste exclu.

L’ETA initiale du plan 300–540 min et celle après Tasks 360–600 min sont historiques. La dernière fenêtre recalibrée 05:30–06:00 CEST n’est pas une mesure CPU. ROOT situe approximativement le début du planning au 13 octobre 2026 à 19:52:55 CEST. Le reçu G8 du 14 octobre vers 05:38:41 CEST intervient près de 9 h 46 plus tard ; ce temps écoulé approximatif inclut attente, incidents, revues, tests, builds et maintenance, sans attribution de temps processeur.


## Clôture T045 — 58/58, 14 octobre 2026

ROOT a intégré et poussé les documents de livraison G8/G9 au commit c57cfd35efa1338cda5256a02c57d71258d8b8d4 sur github/main. Le reçu vérifie HEAD égal à la référence distante et dépôt propre à ce checkpoint. T045 est acceptée sur cette preuve : 58/58, Implemented ; Pipeline : Livré avec conservations justifiées. Le checkpoint 57/58 précédent, intégré dans c57, reste historique.

Preuve : /Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json ; SHA-256 56316fdcb4202865fbb57e7979a11d302bb968e2170453a9523070a490a30390. Le build actif reste 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5. Le commit documentaire c57 n’est pas un nouveau binaire. Les marqueurs Implemented/58/58 ajoutés par ce GO attendent leur commit/push final ROOT ; aucun hash ultérieur n’est inventé.

Les conservations G9 sont acceptées et nommées, sans suppression forcée. Anciennes FULL non GREEN, audits partiels et leurs findings, P3 NONCONVERGED historique et poursuite corrective autorisée par l’utilisateur en phase 5 sont maintenus. Aucun nouveau test, audit global, service ou P4 formel n’est déduit. L’ETA initiale 300–540 min et l’ETA après Tasks 360–600 min restent historiques ; dernière fenêtre 05:30–06:00 CEST. Aucun temps écoulé exact de développement ou CPU n’est ajouté.
