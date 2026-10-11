# Validation des tests — session152

Capture documentaire du 2026-10-11. Les commandes sont exécutées par le
principal. Le propriétaire documentaire lit les sources, les JSON et les
compteurs des journaux. Il ne réexécute aucun test.

## T032 — comparaison R5 à la référence

Référence avant code152 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/baseline.md

Résultats R5 :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r5-results.json

SHA-256 du JSON R5 :
423fd5036659fe9a68826ad60b9899bca930ca2a8457a2c006cf76ec99a0fb63.

Environnement privé : /tmp/b.cpge7gc2. La capture atteste des sources stables
pendant les commandes. Les six filtres de référence gardent leurs compteurs.
Les 78 PASS149 comprennent 72 unitaires et six intégrations réellement exécutées.

| Filtre | Référence PASS/FAIL/ignored | R5 PASS/FAIL/ignored | RC R5 |
|---|---|---|---:|
| native148 | 27/0/1 | 27/0/1 | 0 |
| native149 | 78/0/0 | 78/0/0 | 0 |
| native150 | 51/0/0 | 51/0/0 | 0 |
| delegation149 | 10/0/0 | 10/0/0 | 0 |
| spec_087 | 11/0/0 | 11/0/0 | 0 |
| native151 | 62/0/0 | 62/0/0 | 0 |
| **Six filtres compatibles** | **239/0/1** | **239/0/1** | **0 chaque commande** |
| native152, complément | — | 56/0/0 | 0 |
| **Tranche R5, sept filtres** | — | **295/0/1** | **0 chaque commande** |

Le complément152 contient 51 unitaires et cinq E2E. Ce total de tranche
n'est pas un résultat de suite complète. Les E2E emploient un fournisseur
de test privé. Ils exécutent les vrais chemins de daemon, flotte et MCP.
Ils ne valident ni un modèle réel ni le runtime de production.

Commande R5 de départ :

```sh
python3 scripts/build.py cargo test --locked --offline -p bridget-daemon --features test-support native148 -- --test-threads=4 --nocapture
```

Les six autres commandes emploient les mêmes arguments avec leur filtre.
Le JSON conserve toutes les commandes exactes et leurs durées.

### Changements d'oracle explicitement autorisés

Source149 relue :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs

Cinq oracles149 sont réellement adaptés au contrat152. Les noms historiques
des tests restent inchangés. Aucun scénario, test autonome ou waiver n'est ajouté.

| Cas149 existant | FR | Avant | Après152 |
|---|---|---|---|
| native149_redemarrage_echoue_les_missions_engagees_sans_relancer_ni_toucher_aux_durables, ligne531 | FR-013/014/015/017/029 | Tous les engagés failed/unreachable ; seul l'ordinaire repris. | Pending/working attestés repris avec compteur1 et mission conservée ; Starting sans checkpoint failed/native_restart_identity_mismatch, compteur0 ; queued, résultat, attente et ordinaire protégés. |
| native149_bail_actif_d_un_enfant_natif_n_est_pas_un_candidat_de_reprise, ligne623 | FR-029 | Starting non candidat failed/unreachable. | Absence d'entrée desired vérifiée ; failed/native_restart_identity_mismatch, compteur0 ; l'ordinaire reste l'unique reprise. |
| native149_apres_redemarrage_le_tick_admet_le_queue_et_ne_relance_aucune_mission_echouee, ligne645 | FR-013/014/015/027/029 | Aucune reprise native ; seul queued lancé ; engagés failed/unreachable. | Deux reprises attestées, aucun Start pendant réservation ; vraies connexions et baux enregistrés ; seul queued lancé ensuite ; une remise pending, corps/ID exacts ; zéro remise brute du working ; Starting invalide refusé. |
| native149_f2_mission_engagee_est_fermee_avec_son_execution_et_l_ordinaire_reste_intact, ligne1898 | FR-013/015/027 | Mission et exécution unreachable ; descendant ne bloque plus. | Working conservé, compteur1, mission/instance et état/révision/génération d'exécution inchangés ; descendant encore actif bloque ; exécution ordinaire et contrôle négatif conservés. |
| native149_o4_restart_identities_refuse_toute_incoherence_saga_flotte_marqueur, ligne3014 après fixture R7 | FR-027/029 | Incohérence native : Err global native_restart_identity_mismatch. | Ok avec identités vides ; tâche failed/native_restart_identity_mismatch, instance/compteur préservés ; six variantes et témoin positif conservés. |

Le témoin supplémentaire initialement cité dans le diagnostic reste
**byte-identique** au ROOT actuel :
`native149_reprise_d_execution_ignore_la_mission_native_et_garde_l_ordinaire`,
ligne880 dans les sources finales (ligne866 dans R5). Il exige toujours zéro continuation ordinaire pour la tâche native
sans identité attestée, et une continuation ordinaire pour son témoin.
Son PASS R5 n'est pas une adaptation d'oracle.

Source148/150 relue :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs

| Cas existant | FR / avant | Après152 |
|---|---|---|
| native148_cut_after_admission_retains_exact_ids_and_frozen_model | FR-024/T021 ; comparaison à task_view pur. | Comparaison stricte à current_task_view avant/après admission et réouverture ; IDs, modèle gelé et droits conservés. |
| native150_delai_suivi_aligne_sur_admission_apres_lancement_retarde | FR-010/T045 ; le retard de lancement était compté. | Origine mission_issued_at, délai explicite ou omis. |
| native150_delai_suivi_realigne_apres_echec_d_ecriture | FR-010/T045 ; le suivi déjà aligné n'exerçait plus le trigger. | Ancien suivi divergent préparé avant le trigger ; vrai UPDATE refusé, puis retry réussi conservés. |
| native150_pause_refus_par_le_gestionnaire_complet_ne_fait_rien_avancer | FR-008 ; Start attendu pendant pause. | Contrôle positif Start après levée seulement ; refus sans progression conservé. |
| native150_pause_rejeu_cle_admise_rend_la_tache | FR-024 ; vue sans suspension attendue. | Même égalité stricte, seul suspended attendu true. |

Les cinq autres F2 sont conservés stricts. Le défaut152 de descendant
failed/unreachable avec exécution encore active est corrigé en production :

- native149_f2_descendant_perdu_en_running_est_ferme_puis_la_racine_livre_son_vrai_resultat_une_fois.
- native149_f2_execution_d_un_autre_agent_est_refusee_sans_etre_modifiee.
- native149_f2_execution_terminale_n_est_jamais_rouverte_au_redemarrage.
- native149_f2_proprietaire_hors_ligne_garde_le_resultat_en_attente_puis_livre_une_seule_fois.
- native149_f2_reponse_reelle_capturee_survit_au_redemarrage_et_au_descendant_perdu.

Deux autres défauts152 de garde sont corrigés en conservant leurs contrôles :
`native149_o4_restart_identities_refuse_toute_incoherence_saga_flotte_marqueur`
et `native149_e2_filiation_contradictoire_ou_incomplete_est_refusee_sans_fermer_l_enfant`.
L'adaptation FR-029 du retour global O4 ci-dessus reste distincte de la garde
d'admission/reprise corrigée ; elle ne supprime aucun contrôle d'identité.
Les droits révoqués, sources modifiées, liens non attestés, cibles étrangères
et CAS restent refusés. Les 16 FAIL de R1 ne sont pas classés hérités.

Les captures R1/R2/R3/R4 et leurs diagnostics restent conservés dans :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/implementation.md

### Journaux R5

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r5-native148.log — SHA-256 0892e55f385a40578271ba3ee1e7ac80b26c30c492630009d2f76c8fb4bfaabb.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r5-native149.log — SHA-256 34f56e1351ffe0fa065bb7d5b11a0dcdcf8a9fae854ab59d0b2223b7eb4dcffd.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r5-native150.log — SHA-256 f591c5e5ab7198de58d642493300117b21736837dc22eaf083fe16b4ae8ded9c.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r5-delegation149.log — SHA-256 6a699ff7f7c87f4d85ac656438aaf4e9de4ba5e81fa416b01c26bfc075abf5a8.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r5-spec_087.log — SHA-256 31b56d90d972db1bbaaeb69f7bfbfdeca78048bf881ed142aee7e4bb8df317c5.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r5-native151.log — SHA-256 a077403b2507c34b2370cbb282a655e1a81f61c36f1c88bbfe52df106aa87d3e.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r5-native152.log — SHA-256 a43f9107ea490c7f412170efd09559e88c7b0409a3787ca1cb0e71bd718b1335.

## T036 — suite complète R1, avant le dernier patch

Commande réelle :

```sh
python3 scripts/build.py cargo test --locked --offline --workspace --features bridget-daemon/test-support --no-fail-fast -- --test-threads=4 --nocapture
```

Racine privée : /tmp/b.w1jnlgm5. Durée : 434,66 s. RC101.
Le compteur extérieur trouve **94 cibles, 2103 PASS, 8 FAIL, 72 ignored**.
La comparaison trouve exactement les cinq mêmes cibles et huit mêmes noms
d'échec que W004151 : 94 cibles, 2046 PASS, 8 FAIL, 72 ignored.
Les cinq fichiers de tests concernés sont byte-identiques au socle selon la
contre-lecture qualité rapportée au principal. Aucun succès global n'est déduit.

Preuves distinctes de R5 :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/workspace-152-r1-results.json — SHA-256 6dfe9af7704b34f35f6b5432163b44e4bec8f874c2a73ca3819e60e9ca852730.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/workspace-152-r1-comparison.json — SHA-256 ef806aa33540aaee73eaa8d6d75814a59d0e556e53bbc9918b9e4de02effdc83.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/workspace-152-r1-workspace.log — SHA-256 0e6b4255a2cf1ad4ccca1c92a5b495fd71b8d8e99693c17dfbc3922026358706.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/151-attente-fin-de-tache/validation/T022-workspace151-final.log — SHA-256 8af70bd02343f7820e8bbad7b39c70d0bb4fb63b05da591394957fbf1e24ff3b.

## État des gates et limites

T032 est clôturée après comparaison R5. T034 est clôturée après lecture des
commentaires de complexité et du minimalisme. Le compteur est **48/50**.
T036 et T037 restent ouvertes.

R5 et la suite complète R1 précèdent le correctif de demande absente.
Les deux branches maintenant posées réutilisent `close_lost_execution`
avant de sauvegarder failed/request_unavailable. Une erreur SQL de lecture,
de cible étrangère ou de CAS reporte sans mutation de tâche dans ces branches.
Le chemin avant admission reste inchangé. Le résultat R6 ci-dessous ne
valide pas encore ce patch.

FR-029 demeure limitée aux commandes, instances et liens natifs dans un
marqueur lisible. Les refus sûrs de cible étrangère, CAS et stockage restent
bloquants. La causalité L5 proposée par la revue est réfutée : les clôtures
métier suivent tous les arrêts confirmés. Ces clôtures peuvent rester
partielles sur erreur de CAS ; aucune atomicité des processus n'est promise.

La revue R2 reste REQUEST_CHANGES. La suite finale, T037, Converge, audit,
modèle réel, performance et activation restent des preuves distinctes.
La couverture et la mutation ne sont pas mesurées.

## R6 après patch de demande absente — validation finale en cours

Le filtre native152 compile et exécute ses unitaires : **48 PASS/3 FAIL/0 ignored**,
RC101. Les cinq E2E ne sont pas exécutés après cet échec du lib. La capture
atteste des sources stables, avec N194be156a6873feb4761c332fe03548a72feed033a10d854edc3e5f13f00f0a5.

Les trois FAIL concernent les cas existants d'annulation partielle,
multi_ticks et descendant non racine : l'oracle attend working, la tâche
observée est failed. Le principal fait vérifier si la fixture working sans
suivi durable ou le produit est en cause. Aucune adaptation d'oracle ni
correction supplémentaire n'est déduite de ce reçu. Aucun GREEN final.

Preuves :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r6-results.json — SHA-256 4c36e274e0752e7f0d2cb2f6a5e61bfc51511d14cd5513c03e9c3604870612b2.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r6-native152.log — SHA-256 1dfaf2499da2b80a742338988bb60d52cc9156b82ec6ec6e6978e31c229a69b5.

T036/T037 restent ouvertes. T032/T034 sont closes sur les preuves R5 et
la relecture ciblée ; aucun total final de workspace n'est recalculé.

### Diagnostic R6 confirmé et contrôles finaux bornés

Le principal et la qualité ont lu les trois bases privées des FAIL R6.
Elles contiennent zéro tracked_request, zéro exécution et des échéances NULL.
Ces fixtures forçaient Working sans admission durable. Aucun défaut
FR-018/FR-019 n'est démontré : les arrêts observés restent [false,false]
et [false,true]. R6 atteint bien le nouveau refus request_unavailable.

La correction autorisée porte seulement sur les données des trois tests :
create_request existant, vraies mission/owner/child/délai. Aucun oracle ou
préfixe produit n'est modifié, aucune échéance artificielle. R7 n'est pas
encore GREEN à cette écriture.

La suite complète prévue est déjà exécutée en R1 sur son propre hash :
2103 PASS/8 FAIL hérités/72 ignored. Après l'unique correctif produit des
deux absences de suivi, les contrôles prévus sont R7 (six filtres compatibles
et native152 toutes cibles), puis098 seul, sensible à la charge, prévu T036.
Une nouvelle répétition globale n'est pas décidée si ces preuves suffisent.
Les runs et empreintes restent séparés ; R1 n'est pas présenté comme exécuté
sur le hash final. T036 se clôture seulement après R7/098 ; T037 après le
diff complet et les Articles XIX/XX. Compteur actuel :48/50.

## R7 et correctif de pause — avant R8

| Filtre R7 | PASS | FAIL | ignored | RC |
|---|---:|---:|---:|---:|
| native152 | 56 | 0 | 0 | 0 |
| native148 | 22 | 1 | 0 | 101 |
| native149 | 71 | 1 | 0 | 101 |
| native150 | 51 | 0 | 0 | 0 |
| delegation149 | 10 | 0 | 0 | 0 |
| spec_087 | 11 | 0 | 0 | 0 |
| native151 | 62 | 0 | 0 | 0 |

Les intégrations148/149 sont non exécutées après leurs FAIL lib. Les deux
DB privées confirment zéro demande durable dans les Working synthétiques
148TurnCompleted et149tick ; executionCompleted148 reste intacte et Pending149
est correctement admis/suivi. Les owners réutilisent create_request avec les
vrais IDs/délai, sans deadline Task artificielle ni changement d'oracle.
R7 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r7-results.json — SHA-256 688194f03e932a673758ad00883092e904952f0c4b3895e348663bce22ceb49d.

La revue R3 trouve une pause encore ouverte après request_unavailable.
Le correctif source ajoute cette raison au booléen existant après les gardes
cible/CAS, puis réutilise abandon_pause_interruption. Aucun nouvel état ou
continuation. R8 est en cours sur N c19496f3eeba93e1234f04e23a0fc4ff5735cbf9efcfe578fff0520d5a740cf3
et P1494e10d22f5f52a1b9719a52089c202a44d6c6c60b60c760f9be564d14af6fb2ca.
T036/T037 restent ouvertes avant R8/098 et la revue complète.

## T036 exécutée — R8 final et098 seul, preuves séparées

Le principal autorise T036 après la suite prévue et son rejeu098. **49/50**.
T037 reste ouverte avant le verdict final du reviewer et son GO de cochage.

| Preuve | PASS/FAIL/ignored | RC | Portée |
|---|---|---|---|
| Suite complète R1 | 2103/8/72 | 101 | 94 cibles, avant les correctifs finaux ; mêmes cinq cibles/huit noms que151, pas GREEN global. |
| R8 final | 295/0/1 | 0 pour sept commandes | Six filtres compatibles239/0/1, plus native15256PASS ; sources finales stables. |
| 098 seul prévu | 0/1/0 | 101 | Même test/assertion HTTP404 que151 et R1 ; aucune causalité de charge démontrée. |

R8 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/regressions-152-r8-results.json.
SHA-256 : 162ee39e5d29668b25178a6839540b0f6e35d4fbfbdb2d4eadd29d8b4b795038.
098 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/t036-098-r1-results.json.
SHA-256 : 05167a7366bcbc219c04505d2cc3b1077193581b290baf10221ebe3f72d2fae4.
Log098 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/t036-098-r1-spec098-standalone.log.
SHA-256 : 229e31fb20ca04f1d5d9493962c60d14a77d8c94290dce238425746e79d0badf.

R8 termine native14827/0/1, native14978/0/0, native15051/0/0,
delegation14910/0/0, spec_08711/0/0, native15162/0/0, native15256/0/0.
Les intégrations148/149/151/152 sont effectivement exécutées ; la capture
atteste les SHA stables des dix sources. Le replay098 conserve les HTTP404
répétés et son échec `spec098_pont_expose_remet_repond_renouvelle_et_retire`.

La suite R1 n'a pas été réexécutée sur les hashes R8. Aucun total global
final n'est fabriqué. Les huit échecs hérités ont leurs noms, sources
byte-identiques et diagnostics indépendants conservés. Les préoccupations
natives finales sont couvertes par R8 ; aucun nouveau scénario ni répétition
globale supplémentaire demandé. E2E avec fournisseur privé, pas modèle réel.

## Limite explicite après la revue R4

Aucun test automatisé ciblé n'affirme ensemble `request_unavailable` et
l'abandon de pause dans les deux branches du tick. Cette consommation de
pause est vérifiée par lecture causale indépendante du code. R6 conserve
l'observation empirique des DB : tâche failed/request_unavailable. R8
vérifie les régressions existantes ; il ne remplace pas cet oracle ciblé.
Les passages précédents décrivent le code et le diagnostic R6, sans
revendiquer une assertion automatisée de consommation de pause.
