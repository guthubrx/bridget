# Candidat154 — passageR3 et cochage proposé

R3 :51,04s,RC101,sources stables (207fichiers). Finales : daemon32PASS/9FAIL,
transport3PASS/0FAIL. Total des finales :35PASS/9FAIL. Les six résuméshelpers
0PASS/1FAIL sont imbriqués ; ils ne sont jamais additionnés.
Le grand résultat et les deuxfixtures corrigées aprèsR2 sont PASS.

Reçu : /Users/moi/.cache/bridget-delivery154.l50umk90/candidat154-green-r3.json
Journal : /Users/moi/.cache/bridget-delivery154.l50umk90/candidat154-green-r3.log
SHA256journal : `9edab37f0d8ae6d639c99cdbe7798f1b716e671cf0fd069ba21f225c98a65383`.

## Neuf échecs à traiter

- Sixfixturesisolées : chemin de socket trop long ; le log demande BRIDGET_HOME
  absolu plus court. Les helpers ne sont pas des échecs supplémentaires.
- Rétention : purge renvoie0 au lieu de1 attendu par la fixture.
- Worktree emboîté : la fixture ne confirme pas le Stop de tous les membres.
- Vrai défaut de production dans les40appels : erreurGit brute pendant lecture
  d’un commondir temporaire de worktree en création, au lieu du refus fermé.
  Leprincipal rapporte un correctif de mapping fermé de l’ownerGit, sansretry.
  Correctionsfixture et mapping attendentR4. Aucun GREENglobal proclamé.

## Tâches proposées — aucun cochage avant GORoot aprèsR4

| Tâches | PreuvesR3 | Réserve |
|---|---|---|
| T005/T006 | Dix testsGit PASS (environnement/formes/noms/temps/sorties/version/creation/rapport/hookGPG/ancre) | ConfirmerR4 après correctifmapping ; pas de suiteconfort ajoutée |
| T010 | Catalogueparpaire et refusClaudeClaude PASS | AttendreGO |
| T013 | Worktreeabsent lancement PASS | Fixturecorrigée ; attendreGO |
| T014/T015 | Rapport0/2/60/têtedétachée, unicité, budgetUTF8, autorité, CAS, indisponibilité/rondes PASS | Changementdebranche obligatoire ajouté au testdirectexistant APRÈSR3, donc pasprouvéR3 ; attendreR4/GO |
| T016 | Rapportstatutautoritaire etcompatibilitépartage PASS | AttendreGO |
| T020 | Autresécrivains etpartagediscovery/terminal PASS | AttendreGO |

Noms exacts des oracles associés :

T005/T006 :
- `workspace_git::tests::native154_git_version_ancienne_refusee_a_l_avant_vol`
- `workspace_git::tests::native154_git_environnement_et_options_effectives`
- `workspace_git::tests::native154_git_sorties_cumulees_bornees`
- `workspace_git::tests::native154_git_delai_termine_son_enfant`
- `workspace_git::tests::native154_git_formes_interdites_ne_lancent_rien`
- `workspace_git::tests::native154_git_noms_canoniques_sans_option`
- `workspace_git::tests::native154_git_hook_post_checkout_et_gpg_ne_sont_jamais_lances`
- `workspace_git::tests::native154_git_creation_depuis_sous_dossier_et_worktree_emboite`
- `workspace_git::tests::native154_git_rapport_zero_deux_soixante_commits_et_tete_detachee`
- `workspace_git::tests::native154_git_rapport_ignore_gitfile_enfant_et_parent_emboite`

T010 :
- `daemon::native_delegation::native_workspace154_tests::native154_creation_catalogue_par_paire_et_refus_claude_claude`

T013 :
- `daemon::native_delegation::native_workspace154_tests::native154_isolation_worktree_absent_au_lancement`

T014/T015 :
- `workspace_git::tests::native154_git_rapport_zero_deux_soixante_commits_et_tete_detachee`
- `daemon::native_delegation::native_workspace154_tests::native154_rapport_faits_git_et_ligne_unique`
- `daemon::native_delegation::native_workspace154_tests::native154_rapport_budget_utf8_et_statut_authoritative`
- `daemon::native_delegation::native_workspace154_tests::native154_rapport_cas_et_premier_rapport_gagnant`
- `daemon::native_delegation::native_workspace154_tests::native154_rapport_erreur_indisponible_marqueur_durable_et_ronde_suivante`

T016 :
- `daemon::native_delegation::native_workspace154_tests::native154_rapport_budget_utf8_et_statut_authoritative`
- `daemon::native_delegation::native_workspace154_tests::native154_partage_discovery_termine_et_statut_compatibles150`

T020 :
- `daemon::native_delegation::native_workspace154_tests::native154_partage_autres_ecrivains_seulement`
- `daemon::native_delegation::native_workspace154_tests::native154_partage_discovery_termine_et_statut_compatibles150`

## Revues et risques encore ouverts

Leprincipal et l’ownerNatif rapportent une relecture complète de cancel_tree,
admission/helpers, rapport/exécuteur, libération de garde153 avantStop et
cohérence marqueur/CAS. Cette relecture ne remplace niR4 ni T026final.
Les SHA de production transmis par l’owner concernent son gel àcetinstant ;
ils ne sont pas présentés comme un hashfraîchementrecalculé ici.

FR014 dit « toute commandeGit deBridget ». LeplanH04 exclut communication.rs ;
wrapper derive_domain_at utilise encoreGitdirect historique (:1056lorslecture).
Cette contradiction de portée doit être tranchée par la revuefinale du principal.
Aucun correctif horspérimètre, aucune baisse d’exigence appliquée dans cette passe.

Risque transitoire borné : les métadonnéesGit d’une créationconcurrente peuvent
être incomplètes pendant la lecture d’avantvol. Lechoixrapporté ferme l’erreur
avec un refusnommé, sansretryautomatique ni moteur enarrièreplan. Réutiliser les
mêmesclé/arguments si l’issue est inconnue selon lecontrat de rejeu. La correction
et la recette40appels restent àprouverR4. Aucun gain de droits ni élargissementGit.

Self-review documentaire : hashlogrecalculé, sourcebefore/afteridentique,
compteursfinaux et sixhelpers confirmés, chaqueoracle proposé localisé dans
lelog avec issueok ; testchangementbranche observé dans source actuelle mais
nonattribué àR3. Tasks.md reste inchangé :7/28acquises,21ouvertes.
