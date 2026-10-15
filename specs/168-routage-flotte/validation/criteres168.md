# Critères SPEC168 — matrice de preuves

## État courant — 15/10/2026 à 17:13 CEST

**In Progress, 53/56 tâches cochées par ROOT.** T045 et T046 sont closes. T047, commit documentaire C1, est en cours ; T048 et T049 restent ouvertes. Le nettoyage final de la source appartient à ROOT. Aucun statut Livré ou Implemented n’est annoncé.

La source binaire B `431b78f72bdd73eac4772af17bf36879d7a629d2`, version 0.4.0, est construite et activée sur Mac/Cartae. Les trois voies réelles ont résultats et rejeux vérifiés : Laya `decision_service` sur 2 candidats, Haiku `lightweight_model` sur 21 candidats, dernier repli `deterministic` sur 21 candidats. Détail : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/recette168.md.

**Release publiée et vérifiée :** https://github.com/guthubrx/bridget/releases/tag/v0.4.0 . Tag annoté `43202c9de66ce466b76ba05bd3077699ac88ca3d`, cible exacte B ; quatre actifs retéléchargés et hashes conformes. Reçu : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/release168.md, SHA-256 `15a7b44610ebe060c47164e6c511606e67c37c4d2e4e85a71c2db5be879b9f5d`. Preuve privée : /home/moi/.local/state/bridget-delivery168/release-0.4.0-431b78f72bdd/publication168.json, SHA-256 `d815ca75ca354ec4941577cb39a29aab62cf641ac71895f2ba69e3f0c8ea9666`.

Santé finale reçue : B en ligne, catalogue 27, pont 32 fils vers 97328 ; DB readonly quick_check ok/FK0, versions 7/12 ; worker B 3767260 actif, configuration inchangée. Application T3 65544 et tunnel 60206 préservés ; le serveur 65608 a été remplacé extérieurement par 97328 né à 16:51:02 CEST, sans signal T3 de notre part. La restriction temporaire de 16:56 CEST est levée.

Preuves existantes :

- /Users/moi/.cache/bridget-delivery168/final-health/mac-health.json — SHA-256 `be3ec722c86f8fcdd6de90353e1748e318b0de3beb086690cc66fdd276d114ba` ;
- /Users/moi/.cache/bridget-delivery168/final-health/database-health.json — SHA-256 `98feb24442412b8e2ce12d5c6a92e75605c500bd6f67b4e7fc349fc6cebab147` ;
- /home/moi/.local/state/bridget-delivery168/final-health/cartae-health.json — SHA-256 `a409e7698a2f7ff7bdede77a2a2126eeda0d835052765473f0ef60850818f32a`.

Limites : FULL unique 2820 PASS/32 FAIL/71 ignorés ; 29 échecs initiaux PASS après reprises, trois annulations 159 non validables sur Linux, cause initiale T3 inconnue malgré replay PASS. Deux LOW rustls-webpki et une INFO unsafe-libyaml restent documentés ; aucun GREEN global. Aucun nouvel audit/FULL/P4/build/service/test/modèle par cette rédaction. Aucun retour direct au binaire 0.3.0 après reçu admission v2. Les commits documentaires descendants conservent la source binaire B et ne nécessitent aucune reconstruction.

Cet état daté prévaut sur les checkpoints historiques ci-dessous, dont les anciennes mentions de publication en préparation. Les DB, preuves et sauvegardes sont protégées.

---

Publication documentaire du 15 octobre 2026. **T045 et T046 validées par ROOT ; état 53/56 In Progress.** La release v0.4.0 est publiée et vérifiée. Le nettoyage final et la clôture de livraison restent à prouver. Ce document réemploie le brouillon privé relu et ne modifie aucun statut.

Source produit B active : 431b78f72bdd73eac4772af17bf36879d7a629d2, version 0.4.0. Le checkpoint source A269de64e reste historique. Les preuves contrôlées, les recettes réelles et les opérations encore ouvertes sont séparées ci-dessous.

## Références

- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md — SHA-256 49810c8b5351fb63f0a72902bbd7cd4e2640109391f51ed51f067547d3bf7e9b.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/recette168.md — SHA-256 705c63d30f9a72d6500d40befc02fb10849d481fb23d0910dd88bc5f2493c5c6.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/audit168.md — SHA-256 49eccb84e91d926d1294aaf5ec7cf52bd9f2d5ef73a2f6e2ed002a3b6b46effd.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/preflight-activation168.md — SHA-256 31999cc9ec97ac0fb18f094f1000810d960b2d6be86f2ecd5dceaf5481ee6b99.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/activation168.md — SHA-256 8eab428fe0d80b40d1c031c628279f9993df510bc1706bf32ce51c2dd321b336.
- /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/paquets168.md — SHA-256 768523236ef17d770c95583876e38dad548a9c3e26d0edf23dfd8f05b6d42f9c.

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

| Exigence | Preuve disponible | Portée ou limite |
|---|---|---|
| FR-001 | P01, P06 ; trois recettes natives et rejeux hors T3. | Aucun signal T3 envoyé ; son serveur a été remplacé extérieurement. |
| FR-002 | P02 ; catalogue actif de 27 options. | Modèle, moteur, machine, connexion et effort séparés ; droit fournisseur inconnu. |
| FR-003 | P02 et préflight : sept IDs réels par connexion Codex ; déclarations Claude par machine. | Une déclaration ou model/list ne prouve pas l’accès à chaque modèle. |
| FR-004 | P01/P02 : 12 et 128 options ; registre 129 refusé ; score et Haiku réels sur 21 candidats. | Catalogue 27 distinct des ensembles admissibles ; aucune troncature à huit. |
| FR-005 | P03 ; arguments explicites des recettes conservés. | Chaque axe est filtré indépendamment ; aucune bascule silencieuse. |
| FR-006 | P03/P05/P14 : droits, contraintes et recontrôles avant effets. | Le score et les décideurs ne créent aucun droit. |
| FR-007 | Préflight : environnements déjà authentifiés ; aucun login ni copie fournisseur automatique. | La sauvegarde privée du credential worker sert au retour arrière. |
| FR-008 | P04 et preuves de crédits/fraîcheur : connexion, instance, définition et époque séparées. | Les faits d’un compte ne comblent pas les inconnues d’un autre. |
| FR-009 | P04 : quota inclus à 100 % et crédits utilisables ; faits conservés dans le POST et Lineage. | Aucun solde opaque ni autorisation de dépense déduit. |
| FR-010 | P02 et préflight : connexions Claude représentées ; OAuth Cartae attestée. | Pas de gestion de facturation ; le profil Mac déconnecté reste un fait distinct. |
| FR-011 | P05 et collecteur : âge, provenance, absence, péremption et horloge future vérifiés. | Les faits périmés deviennent inconnus ; aucune mesure de débit revendiquée. |
| FR-012 | P03/P05 : workload general/compilation fermé ; capacité déclarée et poids bornés. | Aucun mot-clé de mission ni framework de benchmark. |
| FR-013 | P06 : saturation avant décideur ; refus réel Mac resource_saturated avant reçu et Task. | Les anciens namespaces Mac à télémétrie OFF sont explicitement isolés. |
| FR-014 | P13 : deux Worktrees simultanés et rapports ; recettes réelles Darwin et Linux/Cartae. | Les recettes réelles sont Shared ; la séparation Worktree est prouvée par contrôle. |
| FR-015 | P12 : reprise depuis le choix figé. | Aucune migration à chaud ; pas de promesse de toutes les pannes possibles. |
| FR-016 | P06/P07/P08 ; réel Laya 2, Haiku 21 et score 21, avec résultats et rejeux. | Les anciennes tentatives en repli restent conservées. |
| FR-017 | P07 : candidat unique et axes fixes sans consultation. | Le rejeu consulte le reçu avant une politique nouvelle. |
| FR-018 | P02/P08 : politique et niveaux par machine ; activation B ON vérifiée. | Les scénarios OFF sont prouvés par les contrôles instrumentés. |
| FR-019 | P08 : niveau désactivé ou inaccessible sans HTTP ni enfant. | Le compteur réseau réel n’a pas été instrumenté dans les recettes. |
| FR-020 | P06 et adaptateur de service ; Laya réel sur deux candidats entiers. | Contrat indépendant du nom Laya ; aucune promesse Laya 128. |
| FR-021 | P06 et charges contrôlées : modèle, effort, ressources, quotas et crédits ; Haiku réel sur 21. | ID opaque seul autoritaire ; ensemble complet ou refus de capacité explicite. |
| FR-022 | P09 et tests du service : ID, effort, capacité et sortie tardive invalides rejetés. | Aucun effet métier depuis ces réponses invalides. |
| FR-023 | P09/P10 : outils métier interdits, isolation et bornes ; Haiku réel sur Cartae. | Seul StructuredOutput formate le JSON ; brief utilisateur borné, sans détecteur parfait de secrets. |
| FR-024 | P10/P11 ; correctif de délai T045B : un test exact PASS et trois sous-cas. | Le service respecte sa configuration dans le budget global ; défaut 2 s conservé. |
| FR-025 | P11/P15 et store : réservation durable, CAS de niveau et choix avant admission. | Les écritures Task et metadata sont atomiques dans les contrôles SQLite. |
| FR-026 | P11 ; trois rejeux réels après redémarrage avec mêmes canons, décision et résultat. | Payload Task brut différent ; aucun compteur HTTP/processus réel inventé. |
| FR-027 | P12 et rejeux réels cohérents. | Zéro redécision prouvé par les oracles instrumentés ; pas mesuré par HTTP réel. |
| FR-028 | P14/P15 et régressions : plafonds, permissions, rétention, annulation et legacy conservés. | Trois anciens cas annulation 159 non validables sur Linux restent signalés. |
| FR-029 | P13/P14 ; résultats réels au SHA B exact. | Shared hors Git général reste distinct ; aucune base distante implicite. |
| FR-030 | P13 et Lineage : base, branche, tête, commits et dirty contrôlables. | Un résultat disponible ne signifie pas intégration ; reçus finaux Git à venir. |
| FR-031 | Audit de réemploi et procédure Git : aucune synchronisation par le daemon. | Le coordinateur autorisé récupère et intègre ; cette synthèse n’exécute aucun Git. |
| FR-032 | Gardes et procédure contrôlées ; runtimes privés des recettes soldés, preuves et DB conservées. | PARTIEL : récupération et nettoyage final des branches/Worktrees/stages T048 à documenter. |
| FR-033 | P16 : mode, connexion et faits figés dans Lineage v4 ; v3 conserve null pour 168. | Faits historiques ; pas de nouvelle mesure au moment de la lecture. |
| FR-034 | P15, tests CLI/MCP et worker legacy : V1 préservé ; V2 refusé sans capacité, sans downgrade. | La nouvelle façade V2 reste stable lors d’un changement ON/OFF. |
| FR-035 | Préflight, activation B active_verified, health Mac/Cartae et trois voies réelles. | Catalogue et possibilités déclarées distincts des accès réellement observés. |

## Critères de succès

| Critère | Preuve disponible | Portée ou limite |
|---|---|---|
| SC-001 | P01/P06 ; missions et rejeux natifs privés sans T3, résultats corrélés. | Aucun signal T3 ; remplacement externe du serveur consigné. |
| SC-002 | Catalogue 27 ; score et Haiku réels sur 21 ; P01/P02 pour bornes 12/128/129. | Le catalogue ne prouve pas le droit fournisseur à chaque entrée. |
| SC-003 | P03 et contraintes des requêtes réelles. | Aucune sélection des cas vérifiés ne viole l’axe imposé. |
| SC-004 | P04 et observations séparées normal/pro, crédits et quotas isolés. | Aucun regroupement financier ni crédit transféré. |
| SC-005 | P05/P06 : cible saturée exclue devant alternative fraîche ; refus réel Mac conservé. | Les recettes privées à télémétrie OFF ne servent pas de preuve de saturation en production. |
| SC-006 | Laya : decision_service sur 2 ; Haiku : lightweight_model sur 21 ; score : deterministic sur 21. | Niveaux OFF sans consultation prouvés par contrôle ; pas de compteur réseau réel universel. |
| SC-007 | P10/P11 et les trois sous-cas de délai configuré T045B. | created_at 1792075807 et updated_at 1792075812 sont des timestamps durables, pas un chronomètre monotone. |
| SC-008 | P11/P12 ; trois rejeux réels après redémarrage, mêmes canons et résultat. | Haiku : global 2/2/2 inclut l’essai à sept ; par clé matricielle un reçu et une Task. HTTP/processus non comptés. |
| SC-009 | P09/P10 : isolation, sortie fermée et StructuredOutput seul ; rôle Haiku réel distinct de l’exécutant. | Pas de détecteur universel des secrets du brief ni d’inventaire fournisseur non exécuté. |
| SC-010 | P13/P14 et SHA B exact dans les résultats réels ; runtimes privés arrêtés. | PARTIEL : preuve de récupération et nettoyage final T048/T049 attendue. |
| SC-011 | Préflight par machine/connexion, catalogue public et health ; modèles exécutants et décideur identifiés. | Les déclarations et l’activation observée restent distinguées ; pas d’accès universel annoncé. |

## Histoires utilisateur

| Histoire | Rapprochement | État de la preuve |
|---|---|---|
| US1 | FR-002 à FR-004 ; SC-002/SC-011 | Catalogue complet et sélection sur 21 candidats réels ; droit fournisseur inconnu. |
| US2 | FR-005/FR-006 ; SC-003 | Axes contrôlés, contraintes réelles et refus conservés. |
| US3 | FR-011 à FR-015 ; SC-005/SC-010 | Ressources et Worktrees contrôlés ; distribution réelle Darwin/Cartae. Nettoyage source final ouvert. |
| US4 | FR-016 à FR-024 ; SC-006/SC-007/SC-009 | Trois décideurs et rejeux réels ; isolation et délais contrôlés. |
| US5 | FR-007 à FR-010 ; SC-004/SC-011 | Connexions et observations séparées ; OAuth Cartae distinct du profil Mac déconnecté. |
| US6 | FR-025 à FR-028/FR-033 ; SC-008 | Concurrence/reprise contrôlées ; canons et résultats stables dans les trois rejeux réels. |
| US7 | FR-029 à FR-032/FR-034/FR-035 ; SC-001/SC-010 | Base exacte et résultats vérifiés ; publication, récupération et nettoyage final encore ouverts. |

## Preuves réelles et état actif

Les trois voies et leurs reçus complets sont détaillés dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/recette168.md. Laya choisit sur deux candidats ; Haiku et le score sur 21. Les rejeux après redémarrage conservent les trois BLOBs canoniques et le résultat métier. Le payload binaire Task diffère. Pour Haiku, les compteurs globaux de deux reçus/deux Tasks/deux exécutions incluent le premier essai à sept ; la seule clé matricielle a un reçu et une Task. Aucun zéro HTTP ou zéro nouveau processus n’est déduit.

Tous les runtimes privés de recettes sont soldés, avec preuves et DB conservées. Cela ne vaut pas nettoyage des branches, Worktrees, TMP et stages de livraison. Les refus initiaux resource_saturated et lightweight_decision_unavailable, ainsi que le repli Laya A sous défaut 2 s, restent historiques. Les anciens cas Mac à télémétrie privée OFF sont explicitement séparés de la garde de production ON.

- /Users/moi/.cache/bridget-delivery168/activation/B-runtime-followup168.json — SHA-256 6d50c3181012c13b2334722f3e3ae13468800c0f8c75d988cfeca27be2c8f86e : B active_verified, routage ON, catalogue 27, quick_check=ok et FK0.
- /Users/moi/.cache/bridget-delivery168/final-health/mac-health.json — SHA-256 be3ec722c86f8fcdd6de90353e1748e318b0de3beb086690cc66fdd276d114ba : daemon B en ligne, catalogue 27 ; pont running avec 32 fils vers serveur T3 97328.
- /home/moi/.local/state/bridget-delivery168/final-health/cartae-health.json — SHA-256 a409e7698a2f7ff7bdede77a2a2126eeda0d835052765473f0ef60850818f32a : worker 3767260 actif sur B, binaire et registre inchangés ; PID privés inventoriés absents.

Le serveur T3 65608 a été remplacé extérieurement par 97328, né à 16:51:02 CEST. Aucun signal T3 n’a été envoyé par nos opérations. L’application 65544 et le tunnel de production 60206 restent ceux des observations protégées. Le checkpoint initial ne prouve pas que 65608 est encore vivant.

## Limites et étapes restantes

- FR-032 et SC-010 restent partiels pour le nettoyage final. ROOT doit joindre les preuves de récupération/intégration, les gardes sur les fichiers/processus/handles et les conservations justifiées. Aucun nettoyage futur n’est déclaré terminé.
- T046 est close : tag annoté, cible B et quatre actifs retéléchargés conformes au reçu de release. T047, commit documentaire C1, est en cours ; T048/T049 portent le nettoyage et la vérification finale.
- SC-007 est soutenu par les tests contrôlés de deadline. L’écart de cinq secondes entre created_at et updated_at du reçu Haiku ne mesure pas toute la chaîne avec une horloge monotone.
- FULL unique : 2 820 PASS / 32 FAIL / 71 ignorés. Après reprises, 29 échecs initiaux passent ; trois anciens cas annulation 159 restent non validables sur Linux. La cause initiale T3 reste inconnue malgré son replay PASS. Aucun GREEN global ni seconde FULL.
- Converge P3 CONVERGED ; audit V15 pass A sur son manifeste, deux LOW TLS et un INFO ouverts. Les axes qualité restent N/A. Aucun nouveau scanner, score ou audit n’est déduit de cette synthèse.
- Aucun modèle, test, build, service ni commande Git exécuté pour publier ce document. Les champs inconnus de quotas, crédits et accès fournisseur restent inconnus.
