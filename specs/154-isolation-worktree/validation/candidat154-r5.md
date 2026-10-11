# Candidat154 — passage R5, preuves ciblées finales

R5 : 36,25 s, RC0, sources stables avant/après. Finale : 6 PASS, 0 FAIL,
0 ignoré. Seulement les six tests de fixture concernés ont été rejoués.
Reçu : /Users/moi/.cache/bridget-delivery154.l50umk90/candidat154-green-r5.json
Journal : /Users/moi/.cache/bridget-delivery154.l50umk90/candidat154-green-r5.log
SHA256 : `34ec151be6c5585cf81928c09e8d401364924122d47405e1e7fba2b1c587f901`.

Noms exacts :
- `daemon::native_delegation::native_workspace154_tests::native154_conservation_trace_git_sans_suppression`
- `daemon::native_delegation::native_workspace154_tests::native154_creation_delai_git_refus_borne_sans_tache`
- `daemon::native_delegation::native_workspace154_tests::native154_creation_deux_appels_meme_cle_un_worktree`
- `daemon::native_delegation::native_workspace154_tests::native154_creation_git_ancien_aucune_ecriture`
- `daemon::native_delegation::native_workspace154_tests::native154_creation_hors_verrou_et_task_limit_apres_reprise`
- `daemon::native_delegation::native_workspace154_tests::native154_rapport_hors_verrou_et_tick_concurrent_sans_second_git`

La recette privée est corrigée : BRIDGET_HOME est conservé après env_clear.
Les chemins des sockets restent privés et courts. Aucun scénario nouveau.

## Couverture native acquise par passages distincts

Vingt-sept noms natifs existent dans le fichier de tests actuel. Chaque nom
possède une preuve PASS dans R3, R4 ou R5. Ce rapprochement n’est pas un
passage complet sur une empreinte unique. Il ne vaut pas GREEN global.

| Oracle natif | Dernière preuve PASS retenue |
|---|---|
| `native154_creation_refus_et_rejeu_enveloppe` | R3 |
| `native154_isolation_creation_cwd_et_parent_intact` | R3 |
| `native154_rapport_faits_git_et_ligne_unique` | R3 |
| `native154_conservation_apres_arret_confirme` | R3 |
| `native154_partage_autres_ecrivains_seulement` | R3 |
| `native154_creation_refus_avant_ecriture_par_cas` | R3 |
| `native154_creation_configurations_hostiles_et_objet_absent` | R3 |
| `native154_creation_catalogue_par_paire_et_refus_claude_claude` | R3 |
| `native154_isolation_sous_dossier_deux_branches_et_lancement_par_paire` | R3 |
| `native154_isolation_worktree_absent_au_lancement` | R3 |
| `native154_creation_rejeu_admis_precede_git_et_change_enveloppe` | R3 |
| `native154_rapport_budget_utf8_et_statut_authoritative` | R3 |
| `native154_partage_discovery_termine_et_statut_compatibles150` | R3 |
| `native154_creation_limite_union_et_propre_trente_deuxieme_place` | R3 |
| `native154_creation_reservation_liberee_sur_panique` | R3 |
| `native154_creation_quarante_appels_trois_proprietaires` | R4 |
| `native154_rapport_cas_et_premier_rapport_gagnant` | R3 |
| `native154_rapport_erreur_indisponible_marqueur_durable_et_ronde_suivante` | R3 |
| `native154_conservation_rapport_apres_arret_trois_etats` | R3 |
| `native154_isolation_petit_enfant_et_conservation_emboitee` | R4 |
| `native154_conservation_retention_150_active_kept_et_rejeu_expire` | R4 |
| `native154_creation_hors_verrou_et_task_limit_apres_reprise` | R5 |
| `native154_creation_deux_appels_meme_cle_un_worktree` | R5 |
| `native154_rapport_hors_verrou_et_tick_concurrent_sans_second_git` | R5 |
| `native154_creation_delai_git_refus_borne_sans_tache` | R5 |
| `native154_creation_git_ancien_aucune_ecriture` | R5 |
| `native154_conservation_trace_git_sans_suppression` | R5 |

## Cochage autorisé par le principal

T007/T008/T009 : refus, droits, admission/rejeu, délais/version, commandeGit
hors verrou et revalidationtask_limit ; preuves R3/R4/R5 et relecture root.
T017/T018 : rapports après arrêt, conservation/trace sans suppression et arbre
emboîté ; preuves R3/R4/R5 et relecture complète cancel_tree par root.
T027 : union/plafond, réservationpanique,40appels et mêmeclé ; R3/R4/R5.
T028 : rapport/CAS, tickconcurrent sans secondGit lent ; R3 et R5.
Seules ces sept tâches ajoutées au cochage. Total26/28 ; T025/T026 ouvertes.

Preuves candidates séparées : protocole3, stockage2, MCP1, observation1,
Git10, natif27 et wrapper4. Ces nombres décrivent les groupes ciblés validés.
Ils ne sont pas additionnés pour annoncer une suite globale sur un même hash.
T025 exécute la seule suite workspace finale. T026 attend cette suite et la
revue finale du diff/exportpatch. Le correctif unused_mut de fixture avantgel
relève de l’owner des tests ; aucune nouvelle validation déduite ici.

Self-review : hashR5recalculé, sourcesbefore/aftercomparées, sixnomsPASS vérifiés,
27oracles natifs rapprochés des logs R3/R4/R5 ; mêmes limites de preuve conservées.
Aucun test, Git, Cargo ni service exécuté par l’owner documentaire.
