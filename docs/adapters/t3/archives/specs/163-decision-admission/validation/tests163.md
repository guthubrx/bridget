# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/validation/tests163.md](../../../../../../../specs/163-decision-admission/validation/tests163.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Résultats transmis par ROOT et rapprochés du reçu local /Users/moi/.cache/bridget-delivery163-20261013/t3-validation163-final.json, SHA-256 9076676edfd08d77d0e489d8ab740ad9e6fb5fe758a7237c7e5b9a9c557e13e2. Empreinte locale vérifiée par le rédacteur ; aucun test, build ou service lancé par ce rédacteur.

<a id="extrait-2"></a>

## Extrait 2

Le premier contrôle, non globalement réussi, reste archivé dans /Users/moi/.cache/bridget-delivery163-20261013/t3-validation163-before-fixture-correction.json, SHA-256 e5c765649cb9e2f1444b0eb3628ee1d967c3171ed6740bc703c7cf46d1c005ed, également vérifié. Aucun échec historique supprimé.

<a id="extrait-3"></a>

## Extrait 3

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

<a id="extrait-4"></a>

## Extrait 4

La correction explicitement autorisée par ROOT ajoute providerThreadId:null dans deux lignes de fixtures uniquement : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/web/src/components/chat/MessagesTimeline.logic.test.ts et /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/apps/web/src/components/chat/MessagesTimeline.test.tsx. Le champ optionnel de l’union fork ne satisfait pas le champ nullable requis de dynamic_tool ; la valeur concrète attendue est null. Aucun code de production, cast ou scénario nouveau. Les sources initiales ont été qualifiées identiques à HEAD174a ; le reçu conserve leurs empreintes et les diagnostics.

<a id="extrait-5"></a>

## Extrait 5

Reçu /Users/moi/.cache/bridget-delivery163-20261013/t3-dependencies163-preparation.json, SHA-256 5201168f1eba80280f87f9b3cc6ba1922f7d2757cab57a38dd1f552ee287c585 vérifié localement. ROOT rapporte 24copies indépendantes, 6452 liens internes et 676 rebases ; le reçu détaille comparaison des fichiers rebases et invariants du cache.

<a id="extrait-6"></a>

## Extrait 6

Conserver l’échec web initial et ses diagnostics, les warnings historiques et les replays exacts. Aucun test supplémentaire de confort, répétition des 191tests ou banc autonome ajouté. Aucun source Rust en cours considéré validé par ces résultats T3. G8/G9 et validation T030 non déduits. Les résultats sont exécutés par les propriétaires indiqués dans les reçus ; le rédacteur a seulement lu les métadonnées et vérifié les empreintes locales.

<a id="extrait-7"></a>

## Extrait 7

Le développeur rapporte en dernière campagne ciblée : cargo test -p bridget-daemon spec163 --lib -- --nocapture, RC0, 12 PASS/0 FAIL, 2010 filtrés ; cargo test -p bridget-transport spec163 --lib -- --nocapture, RC0, 2 PASS/0 FAIL, 351 filtrés. Son git diff --check est rapporté RC0. Warnings de style/interfaces privées/noms hérités signalés. Aucune FULL workspace, essai Mac, T3, service réel ou production par cette campagne.

<a id="extrait-8"></a>

## Extrait 8

ROOT a ajouté uniquement T046–T051 dans la section Convergence de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/tasks.md. Le rédacteur ne modifie ni ce fichier, ni spec/plan, ni les sources. Périmètre des corrections : placement nullable manquant dans list/show v3 ; cohérence CLI/Reader des versions ; plafonds avant réservation et absence de double compte du reçu propre ; refus de fallback filtré et ordre du permis HTTP ; texte affichable T3 ; validation sanitaire HTTP et délai de reprise T051. Ces corrections servent les contrats existants, sans nouvelle fonctionnalité.

<a id="extrait-9"></a>

## Extrait 9

Total contrôlé dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/163-decision-admission/specs/163-decision-admission/tasks.md : 51 tâches, quatre cochées (T001–T004), In Progress. La tâche T051 santé HTTP est ajoutée aux corrections T046–T050 déjà consignées ; elle couvre le schéma sanitaire, statut ok, santé négative/malformée/expirée, délai de reprise et permis, selon les oracles de T021 déjà prévus. T046–T049 et T051 sont en correction Cartae. T050 a été préparée dans deux fichiers T3, mais ceux-ci n’ont PAS été testés depuis cette modification ; les anciens 191 PASS ne prouvent pas les nouveaux hash de ces fichiers.

<a id="extrait-10"></a>

## Extrait 10

Le contrôle indépendant est une attestation en lecture seule, pas un replay par le rédacteur. Les modifications R2 en cours et les nouvelles raisons T3 de T050 sont hors de cette preuve. Aucun POST, inférence, T040, G8/G9, Converged ou pipeline terminé déduit. ETA de fin 02:57–04:57 CEST le 14 octobre 2026 inchangée, hors quota/panne externe. Aucun document synchronisé sur le worktree distant ; ownership rendu à ROOT après ce complément.

<a id="extrait-11"></a>

## Extrait 11

T050 dispose maintenant de 30 PASS sur le banc de contrats existant : /Users/moi/.cache/bridget-delivery163-20261013/t3-t050163-r3.json et son log /Users/moi/.cache/bridget-delivery163-20261013/t3-t050-contracts-tests163-r3.log. Lint/typecheck RC0, deux warnings de regex intentionnels ; format initial RC1 conservé. Après correction de format uniquement, /Users/moi/.cache/bridget-delivery163-20261013/t3-t050163-r4.json, SHA-256 70b55b393a098bbe245b2c81caa439e70fd5c80c9b3e2c8690df33ebb3c3e4bb, atteste format RC0 et égalité exacte des tokens TypeScript : 2041/5814. Aucun replay des 30 tests, du Reader ou du groupe 191. Cette preuve plus récente précise le checkpoint précédent où les deux fichiers n’avaient pas encore été testés après la modification T050.

<a id="extrait-12"></a>

## Extrait 12

Sources après format : /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/packages/contracts/src/bridgetLineage.ts, SHA-256 093c288b6e6a2ddb0473719231966d2bffadd61dcba6c727b63de208a62f5fce ; /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission/packages/contracts/src/bridgetLineage149.test.ts, SHA-256 724d3d178909088d31644dbd5feecdaf9515abfae1a6bfb8f3b870c50eb1d274. Les empreintes sont attestées par le reçu, sans nouvelle exécution ou modification de source par le rédacteur.

<a id="extrait-13"></a>

## Extrait 13

R4 : les trois PASS ciblés et le format RC0, d’abord déclarés par le producteur, sont maintenant attestés indépendamment par attestation_cartae163 et transmis par ROOT. Les logs complets et les RC d’exécution concordent : concurrence 7,93 s, réouverture 5,83 s, worker 4,73 s ; format final RC0 en 1,44 s. Deux FAIL historiques RC101, 13,79 s et 6,94 s, restent conservés. Références de trace transmises : lignes 878, 882, 924 et 930 du même déroulé distant ; aucun reçu local neuf créé par l’agent en lecture seule. Reçu /Users/moi/.cache/bridget-delivery163-20261013/retour-cartae163-r4.json, SHA-256 3b69f02faa0719fd6269143d25723094901319b12862c4bce9baf79573690ec1 ; journal /Users/moi/.cache/bridget-delivery163-20261013/remote-r4-developpement-cartae163.md, SHA-256 89e9eb5a68d06eb31374a25a9cdafc3a6130d3982c2f79c9e6ffb678f414a308. Les 30 PASS R3 n’ont pas été rejoués et ne constituent pas 30 PASS nouveaux sur les cinq sources R4 modifiées. Les 27 PASS R1, 17 PASS déclarés R2 et preuves T3 restent sur leurs provenances respectives.

<a id="extrait-14"></a>

## Extrait 14

Le rapport en lecture seule t3_consumer163, rendu le 14 octobre à 01:03:56 CEST, ne relève aucune objection nouvelle dans neuf fichiers correspondant aux empreintes finales. Il rapproche les preuves 191 PASS préparatoires, 30 PASS T050 et 446 PASS des fixtures affectées, sans nouveau replay. La consommation des trois modes et de decision:null est couverte côté T3 ; la projection Rust finale et l’interopérabilité restent à rapprocher par ROOT. Ce rapport contribue à T035, sans valider T054 ni l’audit final.

<a id="extrait-15"></a>

## Extrait 15

La compilation des trois tests et le format ciblé ne remplacent ni la campagne complète R3, ni une FULL, ni les tests Mac/T3, ni une recette du service réel. Aucun décompte global vert ni nouvelle exécution par le rédacteur.

<a id="extrait-16"></a>

## Extrait 16

R7 admise selon ROOT : tâche 94799f7a-d849-43cf-aab4-6be90eb318bc, enfant 336d8ef8-795f-450f-950b-eeaa971aa3b5, native round 7 ; ownership exclusif native_delegation.rs et ses oracles affectés. Permis Cargo ciblé jobs2/nice10/tests2, sans replay des 28 bibliothèque, neuf HTTP, worker, T3 ou FULL. Aucun test lancé par le rédacteur ; admission ne vaut pas résultat.

<a id="extrait-17"></a>

## Extrait 17

Chaque ligne donne la source, l’oracle existant et la preuve historique disponible. Une ligne n’est pas un PASS de tâche : ROOT doit confronter le correctif R9 testé, la contre-revue finale, les limites et le delta d’audit avant acceptation. Aucun test nouveau, scénario hors plan, replay de succès inchangé ou cochage effectué. Les noms Rust R3 ont été rapprochés de l’archive sources163-r3-remote-snapshot.tgz : 28 noms spec163_ dans les sources de bibliothèque, cohérents avec la campagne R3 attestée. Les numéros de source ci-dessous viennent du checkout Mac R7 et des neuf fichiers T3 relus ; ils restent des repères historiques, pas des lignes R9 futures.

<a id="extrait-18"></a>

## Extrait 18

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

<a id="extrait-19"></a>

## Extrait 19

- R1 Linux : /Users/moi/.cache/bridget-delivery163-20261013/linux-validation163-final.json, SHA-256 7dc90d5eb8f824bf4d6a6a746010cdd6521ebb42419f8f5951883682581f4791. Groupe27 attesté, 21lib/3CLI/1worker/2transport ; commandes seq1457–1458. Il ne sélectionne pas les tests legacy sans spec163_.
- R3 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r3.json et attestation de trace décrite plus haut : 28lib/1CLI/1worker. Les noms ne prouvent pas que leurs corps modifiés ultérieurement ont été rejoués.
- R4 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r4.json et trois logs exacts déjà archivés plus haut : concurrence, disque et worker.
- R5 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r5.json et dix PASS finaux attestés, T0541/service9 ; MCP FAIL distinct conservé.
- R6 : /Users/moi/.cache/bridget-delivery163-20261013/source-manifest163-r6.json et /Users/moi/.cache/bridget-delivery163-20261013/t055-r6-mcp-closed-parent.log : MCP exact1 PASS.
- T3 : /Users/moi/.cache/bridget-delivery163-20261013/t3-validation163-final.json et /Users/moi/.cache/bridget-delivery163-20261013/t3-t050163-r4.json ; self-review neuf hashes. Groupe191 préparatoire, contrats30, fixtures446 et format/tokenisation ont des provenances séparées.

<a id="extrait-20"></a>

## Extrait 20

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

<a id="extrait-21"></a>

## Extrait 21

Exécuter seulement sous permis ROOT sur le worktree Cartae isolé, avec CARGO_BUILD_JOBS=2/nice10/TMPDIR privé et cache déjà attribué. Ces noms ne sont pas une campagne21/28 nouvelle. Leur version legacy ne vaut pas, à elle seule, recette v3 maximale ou runtime T3. Le filtre journal/cancel v3 CLI et le test T3 does not negotiate or replay journal reads and cancellation restent leurs preuves ciblées séparées.

<a id="extrait-22"></a>

## Extrait 22

31/31 couverture documentaire et 4/57 inchangés, aucun cochage. T033 quatre contrôles nominativement prouvés, sans extrapolation vers nouveau runtime T3. P3 NONCONVERGED historique/phase 5 corrective autorisée conservés, aucun P4 formel. Audit final/delta R9, builds, recette T040, Git, G8/G9 et livraison restent ouverts. T039 prépare quatre profils off/on moteur/placement sans appliquer la configuration ; quota CLI à 100 % et nouveaux children ne prouvent pas l’usage opérationnel de ces profils en production.

<a id="extrait-23"></a>

## Extrait 23

Reçu /Users/moi/.cache/bridget-delivery163-20261013/code-review163-r4-final.json, SHA-256 5cb0c98f361e7079eb59085ec2988b39636c4378a1fc5f4cab8f1544b7f8a5f0. Reviewer Claude Sonnet 5.5/claudeAgent ; taskId node:delegated-task:command%3Amcp%3Aa3163845-67c8-4e7f-bea0-f779216e5c09%3Adelegate-task%3A163-code-review-20261014-r4. Deux hashes source, manifeste et delta R9 vérifiés ; 21 sources inchangées, T3, logs et résultats de tests non relus par le reviewer. R1/R2 non relus exactement ; aucun test exécuté par cette revue. Audit32 encore en cours, aucun grade/cochage/livraison déduit ; 4/57, P3 NONCONVERGED historique et exception phase5 conservés, aucun P4 formel.

<a id="extrait-24"></a>

## Extrait 24

T3 /Applications/T3 Code (Local).app remplacée atomiquement en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18, signature stricte/deep RC0 et entitlements préservés. PID 704/969 et naissances 17:31:27/33 du 13 octobre restent inchangés : aucun redémarrage, ancien code encore chargé. Aucun réveil différé programmé.

<a id="extrait-25"></a>

## Extrait 25

- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-retry-projection/progress.json ; SHA-256 b6da23d355fbb396b50fab7c946dd5693dd6ced27aafe5ae3ac2aaffaf7cef16
- /Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json ; SHA-256 b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0
- /Users/moi/.cache/bridget-delivery163-20261013/g8-registry-off-applied.json ; SHA-256 ed358ea42dc927adf19d4c4e7cca2ecdd0de76e2aa2a213116f6b15049b26aae

<a id="extrait-26"></a>

## Extrait 26

Paquet T3 source 511190beb307e0952066fe1a4b511cb9dd7adb18, version 0.0.45-local.164 : build RC0/176,676 s, signature stricte/deep RC0 et entitlements préservés selon le reçu. ASAR 6eb2fbdfed44a603d0a66d81d92fadd6d3486ff1ff43a880fe788c7d39fc923f. Les 2574 PASS R1 et deux FAIL ZIP restent historiques ; les deux seuls contrôles ZIP affectés passent R2. Aucune nouvelle campagne globale ni installation déduite.

<a id="extrait-27"></a>

## Extrait 27

- /Users/moi/.cache/bridget-delivery163-20261013/build163-mac-5105378bf4f0.json ; SHA-256 4c634d8ef74dfebf2578081475edfb5dea34a2726150870f0d235e2d02cafb7a
- /Users/moi/.cache/bridget-delivery163-20261013/build163-linux-5105378bf4f0.json ; SHA-256 93edb25e689ed834d231fb806a8f90ade016132748d38b7f80b7c1f6af08a741
- /Users/moi/.cache/bridget-delivery163-20261013/tests163-mac-5105378bf4f0.json ; SHA-256 a5642e75c57a83117eff51a92784157dab383dc3bc998251567eae6a157f9554
- /Users/moi/.cache/bridget-delivery163-20261013/t040-execution-r4/recipe163.json ; SHA-256 67f5268bd03fdae806649da3a795c9122c60469bab4be1f89e16aafc4059e9d1
- /Users/moi/.cache/bridget-delivery163-20261013/t040-execution-r4/native-tokenizer.json ; SHA-256 774e82c383928309524cec1ac2abdea57b22cf7f1942b23689fc41c2540ccd60
- /Users/moi/.cache/bridget-delivery163-20261013/lab163-preparation/t040-r4-cleanup-conclusions-supplement.json ; SHA-256 54056f844075544f45ae83a8072f7a6c1421ea16bf391d32458a05726d4c8d49
- /Users/moi/.cache/bridget-delivery164-20261013/t3-package164-result.json ; SHA-256 2fb1b8f95140f09676d107f2429be07322c7ff77ff094993f1a82f95dff18984

<a id="extrait-28"></a>

## Extrait 28

Le MCP propre à la nouvelle commande annonce bridget_auto_delegate dans tools/list, mais tools/call renvoie JSON-RPC -32602, message outil inconnu, request_id g8-spec163-a73f9432-53d5-46b5-94b9-e8f0ee3cfe7e. Les attestations officielles immédiatement avant appel portent version2 et permissions_present=true ; provider codex-pro déclaré disponible et capacité maître contrôlée. Aucun reçu auto/Task sous cette clé selon la vérification durable ROOT. La cause confirmée ROOT est le dispatcher de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:753–760 : routage auto omis malgré l’annonce. Scope/hook/wrapper doivent être raccordés à la voie native existante. Ce refus ne démontre pas un problème de quota ou d’authentification et ne nécessite pas de redémarrage T3.

<a id="extrait-29"></a>

## Extrait 29

T058 reste dans la session163 autorisée : correctif minimal sur Cartae, oracles MCP existants affectés, contre-revue/delta d’audit ciblés, build et livraison du correctif avant reprise G8. Aucun nouveau framework, banc de confort ou FULL. Le prochain commit source sera différent de5105378bf4f03376e9bc93e52065cb022f81dee5 ; aucun hash futur fixé. T038/T042 demeurent cochées pour les builds/installations réellement prouvés du510, pas pour le futur correctif. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18/version164 est déjà installée et inchangée. Aucun résultat antérieur effacé, aucun livré final ou G8/G9 déduit.

<a id="extrait-30"></a>

## Extrait 30

T058 reste ouverte. Reconstruction, réinstallation du correctif et recette G8 réelle restent à faire. Compteurs inchangés : SPEC163 53/58, SPEC164 19/22. La source5105378bf4f03376e9bc93e52065cb022f81dee5 reste chargée au dernier checkpoint de runtime ; aucun nouveau commit, binaire ou succès G8/G9 n’est anticipé. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18 reste inchangée. P3 NONCONVERGED historique et poursuite corrective autorisée sont conservés.

<a id="extrait-31"></a>

## Extrait 31

T3 /Applications/T3 Code (Local).app est installée en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18. Ses processus 704/969 restent en version 162, sans redémarrage. Le catalogue d’outils de l’hôte conserve sa version précédente jusqu’à rechargement autorisé ; G8 utilise le MCP propre à la commande. Aucune nouvelle recette runtime T3 n’est revendiquée.

<a id="extrait-32"></a>

## Extrait 32

Le dernier scan du worktree T3 163 a été complet : 282 139 fichiers dans la fenêtre 180 s. L’inventaire a ensuite changé après l’archive, donc la garde de stabilité a échoué. Le worktree /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission et sa branche sont conservés. Cette cause remplace le seul constat historique du scan 60 s incomplet ; aucune cause d’inode ou mutation de contenu n’est inventée. Reçu : /Users/moi/.cache/bridget-delivery163-20261013/g9-t3wt163-complete-retirement.json ; SHA-256 7e2388b90e22f0b33bb3f6140851c7dedc1a398a5b5d10957a490153d4e13d29.

<a id="extrait-33"></a>

## Extrait 33

Les stages de paquet /Users/moi/.cache/t3-final162.2nosz6zd et /Users/moi/.cache/t3-final163.zsb7fmfu contiennent chacun une base persistante de cache SQLite de 20 480 octets ; ils restent protégés, sans prétexte de futur build163. L’ancienne application /Applications/.T3 Code (Local).app.new-163 reste chargée par les processus T3 protégés. Sauvegardes, preuves, retours arrière, bases et caches partagés sont conservés.

<a id="extrait-34"></a>

## Extrait 34

G9 est accepté sur ces retraits et conservations nommées. Reçu final : /Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json ; SHA-256 08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3. Les preuves d’exécution e397 et de retrait T3 164 b066 restent rattachées au reçu final. Aucun fichier de production ou service n’est touché par cet ajout documentaire.

<a id="extrait-35"></a>

## Extrait 35

L’audit 163 reste A/99,047619 partiel : 0C/0H/3M/2L, couverture 4,655325 % du snapshot et 0,355837 % du contexte global. Le delta T058 reste A/100 partiel 6,749687 %, 0 finding actif, validateur RC0/0 erreur/0 avertissement. Le delta T3 R6 conserve 0 finding actif sur son seul périmètre. Ces audits ne deviennent pas des audits globaux du produit.
