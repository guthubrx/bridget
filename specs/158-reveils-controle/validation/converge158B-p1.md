# Converge158B P1 — contrôle ciblé frais
Date : 2026-10-12T12:04:57.412155+00:00
Verdict : **NOT_CONVERGED**. Un manque FR103 confirmé. Aucun succès global de tests.

Le dépôt reste intact. Les 22 tâches B ne sont pas cochées par ce lecteur. La tâche T123 ci-dessous est une proposition cache pour ROOT.

| Exigence | Conclusion du code | Tâches / oracles existants |
|---|---|---|
| FR101 | STATIC_PRESENT | T102,T103,T104,T105,T106,T110 ; wake158B_groupe1_domaine_ferme_et_bornes, wake158B_groupe1_sql_limites_physiques_et_annulees, wake158B_groupe2_calendrier_trou_fold_tz_et_date, wake158B_protocol_objets_fermes |
| FR102 | STATIC_PRESENT | T107,T108,T109 ; wake158b_daemon_g4_g5_pause_et_grace_avant_depart |
| FR103 | GAP_REPRISE_102 | T104,T106,T108,T109,T112,T117 ; wake158B_groupe3_intervalle_gel_restart_et_frontieres, wake158b_daemon_g6_fil_commit_uuid_et_revalidation, wake158B_groupe7_cli_quota_rejeu_collision_et_reprise |
| FR104 | STATIC_PRESENT | T107,T108,T109,T110,T113 ; wake158b_mcp_surface_wake_et_arguments_fermes, wake158b_daemon_g4_g5_pause_et_grace_avant_depart, wake158b_daemon_g6_fil_commit_uuid_et_revalidation, mcp158_scopes_couvrent_tous_les_outils |
| FR105 | STATIC_PRESENT_WITH_FR103_DEPENDENCY | T111,T112,T113 ; wake158b_cli_g7_formes_fermees_et_identifiant_retry, wake158B_groupe7_cli_quota_rejeu_collision_et_reprise |
| FR106 | STATIC_PRESENT | T114,T115,T116,T117,T118 ; observation158b_g8_provenance_once_writer_et_transaction, observation158b_g9_bornes_bytes_ttl_proprietaires_et_lacunes, observation158b_g10_retour_cutoff_resume_uuid_et_ack_cas, wake158b_daemon_g8_g10_once_worker_et_retour_ack |
| FR107 | STATIC_PRESENT | T114,T115,T116,T117,T118 ; observation158b_g9_bornes_bytes_ttl_proprietaires_et_lacunes, wake158b_mcp_events_missed_passe_par_identite_attestee |

## Manque unique : reprise102
Post102 committé puis perte de finalize099 CLI ou settle_wake_occurrence. Reprise après grâce600s, pause ou retrait de droits : résultat ancien committé masqué par garde de création. Aucun nouvel effet nécessaire pour lire sa preuve.

Preuve normative : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/spec.md:227 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/specs/158-reveils-controle/contracts/wake-158B.md:99 et :104.
Code : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/daemon.rs:2129,2144,2179.
Réemploi : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/threads.rs:369,988,1020–1052 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/store/threads.rs:971.

Rendre pub(crate) le digest existant, reconstruire ce seul canon fermé inline, relire thread_operation_replay avec owner/UUID/hash avant les gardes d’un nouvel effet. Replayed(posted,thread_id exact) clôture CommittedThread et099 CLI sans repost. None applique les gardes existantes. Mismatch, reçu invalide ou erreurSQL ferment. Même preuve à utiliser avant annulation/lifecycle qui solderaient l’occurrence comme absence.
Canon fermé exact : digest([b"post", b"v1", thread_id.as_bytes(), wake.body.as_bytes(), b"all", b"reply_to", b"ack"]). All a zéro cible canonique ; aucun membre courant dans ce hash.

- [ ] T123 Réparer la reprise du post102 déjà committé avant grâce/gardes de création pour FR103/US5/US6, par digest et thread_operation_replay existants. Étendre seulement l’oracle G6 existant aux frontières de perte de clôture ; préserver membre/projet avant tout post neuf. ROOT recompile après arrêt de compilation et poursuit l’unique FULL T121 réelle ; la tentative14.37s de compilation/0tests n’est pas une FULL de régression. Consigner preuves et relectureT122.

## Résumés OwnState
NativeTask/instance figées FR104 concernent les programmations. Les résumés sont des lectures/état propre OwnState ; aucun ajout de politique de programmation à ces résumés.
Résumé persiste owner/UUID/body/issued_at/cutoffs/génération. Son FrozenWake reconstruit owner_project au départ et owner_native=None. Le départ recontrôle le projet capturé et ferme une origine native aujourd’hui terminale via wake_native_origin ; aucune preuve native figée historique du résumé revendiquée. Pas de gain d’autorité client ni mutation du cutoff sansACK. Immutabilité du canon099 après réservation protégée par comparaison099 ; pas de garantie additionnelle sur le snapshot projet avant première réservation revendiquée.

## Relecture ArticlesXIX/XX bornée

necessity:
- Domaine calendrier et occurrence figée justifiés FR101/103 ; deux modulesSQL justifiés par les trois tables bornées exigées et Store existant.
- Worker unique étendu avec provenance et retour64, sans nouvelle boucle/service ni journal universel.
- deliver_frozen_wake a trois sources réelles scheduled/cli_once/missed_summary ; wake_target_guard a admission/gel/départ ; wake_native_origin a admission/gel/départ.

simplicity:
- SocketUnix,099,102,Store,garde087,tableScopes existants réemployés.
- Calendrier libc borné2881x8 horsmutex ; CAS timezone/next/date paire.
- Bornes16/128,60CLI/min,64/sub,8192facts/owners,16KiBfacts/résumé,64KiBpage,5sworker/1sIO/600sgrâce/7jours rétention visibles dans code.

hypotheses:
- ADR011 borne mêmeUID ; aucune isolationOS forte revendiquée.
- Fuseaux minutes entières dans +/-24h seulement ; pas un inverse universel libc.
- ACKwrapper ne prouve pas lecturemodèle ; résultat inconnu n’est pas absence.
- Perte connue après crash reste honnête ; aucune exactement-unefois fournisseur.

avoided:
- Pas scheduler/worker/outbox/quota universels nouveaux.
- Pas autostart fournisseur,HTTP,secretUS8,modèle ou outil supplémentaire.
- Pas nouveau test/scanner/benchmark ni optimisation de confort.

Relecture ciblée des exigences et trois modules nouveaux de production ; le diff entier/tests complets/30oracles n’ont pas été tous relus ici. T122 et son implementation-158B.md/diff-158B.patch restent une livraison ROOT distincte. Aucun crédit de revue complète inventé.
Lignes de production suppressibles prouvées :0 dans cet échantillon seulement.

## Limites et empreintes
- Tests non exécutés par ce lecteur. Noms source et reçus ROOT séparés.
- T107-T117-affected-GREEN-r2 historique :3PASS/1FAIL/RC101, source propre à ce reçu. Ne représente pas le dernier code formaté.
- ROOT annonce G3/G6/CLI ciblés puis uniqueFULL ; résultats finaux non reçus et non déduits.
- TentativeFULL14.37s compilation/0tests selon ROOT ; aucune régression exécutée dans cette tentative.
- Les hashes ci-dessous sont une capture de clôture, pas une capture avant la toute première lecture.
- Les recherches simples pour maintenance docs/testnoms ne deviennent pas une couverture sémantique complète.
- Pas Analyze supplémentaire/auditglobal/grade/GO livraison.

Manifest précis SHA/plages dans /Users/moi/.cache/bridget-code158B-daemon-20261012/converge158B-p1.json.
Tasks SHA à la capture de clôture : 6af7dd12f1f594fab0d10ea1076d99e414c89898988c9918e5d13a4a026691fa.
Aucune preuve runtime du manque ; contre-lecture ROOT confirme son ordre dans le code.
