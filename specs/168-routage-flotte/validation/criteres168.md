# Critères SPEC168 — matrice de preuves

## État courant — 15/10/2026 à 17:24 CEST

**Implemented — 56/56 tâches cochées. Pipeline : Livré, avec conservations explicites T048.** ROOT a validé le contrôle final des 35 exigences, 11 critères et 7 histoires. Aucune tâche ne reste ouverte.

La récupération documentaire T047 est prouvée : C1 `db46c7816777199da7c41bb9bc10063cd361dae7` intégré et poussé, main Mac/Cartae alignés et propres aux reçus. Source binaire B et release v0.4.0 inchangées ; aucun rebuild. Preuves : /home/moi/.local/state/bridget-delivery168/git/integration-pushC1-cartae.json, SHA-256 `e32d1529d986fff8af4441928353d8b1a5fd163e4aefaf7e3b90651d9ce90454` ; /Users/moi/.cache/bridget-delivery168/git/integration-pullC1-mac.json, SHA-256 `4cf769d0707b2b28535a6f91052d786ff5c5b1c861738e645f6a20df84111766`.

L’examen T048 est terminé avec retraits et conservations motivées. Le worktree source Cartae et sa branche sont conservés : C1 fusionné/poussé, arbre propre et zéro ignoré ; garde lsof RC0 mais 4 337 octets stderr, donc observation incomplète. Aucun match observé ne vaut preuve d’absence d’usage. Aucun contournement ni suppression inventée. Les 18 anciens candidats conservent leurs motifs. Reçu : /home/moi/.local/state/bridget-delivery168/cleanup/t048-source-final-guard.json, SHA-256 `e2410c3dc3d838b1fb49cdb065f0a9c067cda717a105cce4d0458b94ea25cfc3`. Bilan : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/nettoyage168.md. Le contrôle final T049 est validé. Le SHA du commit documentaire final est vérifié hors de son propre contenu dans les reçus Git privés.

La source binaire B `431b78f72bdd73eac4772af17bf36879d7a629d2`, version 0.4.0, est construite et activée sur Mac/Cartae. Les trois voies réelles ont résultats et rejeux vérifiés : Laya `decision_service` sur 2 candidats, Haiku `lightweight_model` sur 21 candidats, dernier repli `deterministic` sur 21 candidats. Détail : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/recette168.md.

**Release publiée et vérifiée :** https://github.com/guthubrx/bridget/releases/tag/v0.4.0 . Tag annoté `43202c9de66ce466b76ba05bd3077699ac88ca3d`, cible exacte B ; quatre actifs retéléchargés et hashes conformes. Reçu : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/release168.md, SHA-256 `15a7b44610ebe060c47164e6c511606e67c37c4d2e4e85a71c2db5be879b9f5d`. Preuve privée : /home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication168.json, SHA-256 `d815ca75ca354ec4941577cb39a29aab62cf641ac71895f2ba69e3f0c8ea9666`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/criteres168.md#extrait-1).

Preuves existantes :

- /Users/moi/.cache/bridget-delivery168/final-health/mac-health.json — SHA-256 `be3ec722c86f8fcdd6de90353e1748e318b0de3beb086690cc66fdd276d114ba` ;
- /Users/moi/.cache/bridget-delivery168/final-health/database-health.json — SHA-256 `98feb24442412b8e2ce12d5c6a92e75605c500bd6f67b4e7fc349fc6cebab147` ;
- /home/moi/.local/state/bridget-delivery168/final-health/cartae-health.json — SHA-256 `a409e7698a2f7ff7bdede77a2a2126eeda0d835052765473f0ef60850818f32a`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/criteres168.md#extrait-2).

Cet état daté prévaut sur les checkpoints historiques ci-dessous, dont les anciennes mentions de publication en préparation. Les DB, preuves et sauvegardes sont protégées.

---

Publication documentaire du 15 octobre 2026. **56/56 tâches validées par ROOT ; Implemented, pipeline Livré.** La release v0.4.0 est publiée et vérifiée. La récupération et l’examen du nettoyage sont prouvés ; le contrôle final T049 est validé. Ce document réemploie le brouillon privé relu et ne modifie aucun statut.

Source produit B active : 431b78f72bdd73eac4772af17bf36879d7a629d2, version 0.4.0. Le checkpoint source A269de64e reste historique. Les preuves contrôlées, les recettes réelles et les opérations encore ouvertes sont séparées ci-dessous.

## Références

- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md — SHA-256 49810c8b5351fb63f0a72902bbd7cd4e2640109391f51ed51f067547d3bf7e9b.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/recette168.md — SHA-256 705c63d30f9a72d6500d40befc02fb10849d481fb23d0910dd88bc5f2493c5c6.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/audit168.md — SHA-256 49eccb84e91d926d1294aaf5ec7cf52bd9f2d5ef73a2f6e2ed002a3b6b46effd.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/preflight-activation168.md — SHA-256 71d47398109468e87dfdb4f21ac585d41a912e272fbf5fffc5bd6f253f149d8e.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/activation168.md — SHA-256 99b053c32645e52e4bd76bb52309137f86f81c283d4fd3f26ec1f6289fd2362f.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/paquets168.md — SHA-256 2d9ee483f53155b627088dea6ab4129c5793c82fde4ad14f704e841c9d328939.

Le reçu de tests donne les commandes, logs, empreintes et chronologies. Les identifiants P01 à P16 ci-dessous renvoient aux noms exacts de tests qui y sont publiés. Ils ne comptent pas comme de nouvelles exécutions.

## Index des contrôles

- P01 : spec168_catalogue_daemon_reel_12_128_sans_agent_t3_task_ni_modele.
- P02 : registry::tests::routing168_catalogue_12_preserve_connexions_efforts_et_defaut_natif.
- P03 : daemon::native_delegation::tests::spec168_cinq_axes_effort_impose_et_defaut_natif_sont_fermes.
- P04 : provider_limits::tests::spec168_quota_connexion_instance_definition_epoque_ttl_et_credits_100.
- P05 : daemon::native_delegation::tests::spec168_score_saturation_fraiche_inconnus_stabilite_et_compilation.
- P06 : daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete.
- P07 : daemon::native_delegation::tests::spec168_modele_unique_et_deux_niveaux_off_ne_consultent_pas_et_rejouent_avant_politique.
- P08 : daemon::native_delegation::tests::spec168_niveaux_inaccessibles_zero_essai_et_effort_role_non_qualifie_explicite.
- P09 : claude_stream_json::tests::spec168_formateur_refuse_outils_capacites_et_reponses_contradictoires.
- P10 : claude_stream_json::tests::spec168_stdin_non_lu_expire_depuis_le_lancement_et_arrete_enfant.
- P11 : daemon::native_delegation::tests::spec168_service_post_reel_concurrence_enveloppe_retour_tardif_reprise_et_reouverture.
- P12 : daemon::native_delegation::tests::spec168_crash_pending_et_started_reprennent_le_repli_fige_sans_nouvelle_consultation.
- P13 : daemon::native_delegation::tests::spec168_git_local_sha_exact_missing_mismatch_worktree_et_hors_git.
- P14 : remote_worker_162::worker168_probe128_connexion_revoquee_prepare_et_base_epinglee.
- P15 : delegation::tests::spec168_migre_163_sans_alterer_les_blobs_et_borne_v2.
- P16 : delegation::lineage::delegation_lineage_tests::spec168_lineage_v4_projette_seulement_decision_figee_et_v3_reste_null.

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/criteres168.md#extrait-3).

## Critères de succès

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/criteres168.md#extrait-4).

## Histoires utilisateur

| Histoire | Rapprochement | État de la preuve |
|---|---|---|
| US1 | FR-002 à FR-004 ; SC-002/SC-011 | Catalogue complet et sélection sur 21 candidats réels ; droit fournisseur inconnu. |
| US2 | FR-005/FR-006 ; SC-003 | Axes contrôlés, contraintes réelles et refus conservés. |
| US3 | FR-011 à FR-015 ; SC-005/SC-010 | Ressources et Worktrees contrôlés ; distribution réelle Darwin/Cartae. Examen du nettoyage source terminé avec conservation motivée. |
| US4 | FR-016 à FR-024 ; SC-006/SC-007/SC-009 | Trois décideurs et rejeux réels ; isolation et délais contrôlés. |
| US5 | FR-007 à FR-010 ; SC-004/SC-011 | Connexions et observations séparées ; OAuth Cartae distinct du profil Mac déconnecté. |
| US6 | FR-025 à FR-028/FR-033 ; SC-008 | Concurrence/reprise contrôlées ; canons et résultats stables dans les trois rejeux réels. |
| US7 | FR-029 à FR-032/FR-034/FR-035 ; SC-001/SC-010 | Base exacte et résultats vérifiés ; publication et récupération prouvées ; examen nettoyage terminé, conservations motivées. |

## Preuves réelles et état actif

Les trois voies et leurs reçus complets sont détaillés dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/recette168.md. Laya choisit sur deux candidats ; Haiku et le score sur 21. Les rejeux après redémarrage conservent les trois BLOBs canoniques et le résultat métier. Le payload binaire Task diffère. Pour Haiku, les compteurs globaux de deux reçus/deux Tasks/deux exécutions incluent le premier essai à sept ; la seule clé matricielle a un reçu et une Task. Aucun zéro HTTP ou zéro nouveau processus n’est déduit.

Tous les runtimes privés de recettes sont soldés, avec preuves et DB conservées. Cela ne vaut pas nettoyage des branches, Worktrees, TMP et stages de livraison. Les refus initiaux resource_saturated et lightweight_decision_unavailable, ainsi que le repli Laya A sous défaut 2 s, restent historiques. Les anciens cas Mac à télémétrie privée OFF sont explicitement séparés de la garde de production ON.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/criteres168.md#extrait-5).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/criteres168.md#extrait-6).

## Limites et étapes restantes

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/168-routage-flotte/validation/criteres168.md#extrait-7).
