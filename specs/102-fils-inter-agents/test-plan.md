# Recette détaillée 102 — tests à implémenter, non exécutés

Oracle principal : compter les entrées persistées, destinataires d'injection,
pages transmises, appels de faux fournisseur et curseurs. Ne pas confondre un
mock « send appelé » avec une injection réelle, ni absence de log avec silence.

## Organisation prévue

- BDD lisible : tests/features/102-fils-inter-agents.feature.
- Tests purs, noms spec102_* : module threads.rs et contrat protocol.rs/message.rs.
- Intégration réelle DB/socket/CLI/MCP : crates/bridget-daemon/tests/spec102_threads_test.rs.
- Adaptateurs : tests locaux wrapper.rs/t3code.rs et fixtures existantes, complétés
  si nécessaire dans spec102_threads_test.rs ; aucune API fournisseur réelle.
- Les chemins ci-dessus sont relatifs au worktree102 documenté dans quickstart.md.

Une même propriété centrale peut être testée au niveau unitaire puis une fois
aux frontières ; ne pas copier toute la matrice sur chaque couche. Les tests
autorisés manipulent seulement données synthétiques, répertoires temporaires,
horloge injectable et processus qu'ils ont créés.

## Matrice obligatoire

| ID | Niveau | Préparation et action | Oracle observable | Exigences |
|---|---|---|---|---|
| V01 | DB/MCP | A crée avec A/B/C/D, A et C list/show | même fil, créateur membre, aucune alerte | FR-001, SC-001 |
| V02 | DB | post silencieux A puis B, redémarrer Store | seq1,2 et auteurs exacts, corps intacts | FR-002, FR-014 |
| V03 | E2E simulé | 20 posts notify:[] | zéro injection/lecture/appel modèle tous membres | FR-003, SC-001 |
| V04 | E2E simulé | 20 échanges A→B / B→A | cibles exactes, zéro injection C/D | FR-004, SC-001 |
| V05 | contrat | notify multiples, doublons, soi, all | ensemble normalisé ; auteur exclu, notice soi | FR-004 |
| V06 | contrat | corps @all, Markdown/code/citation, reply_to sans notify | zéro alerte implicite | FR-005, FR-020 |
| V07 | CLI | deux noms identiques puis renommage d'un UUID | ambiguïté refusée ; adresse stable inchangée | FR-005, FR-012 |
| V08 | DB | dix mentions B avant réservation | dix entrées, une ligne wake, une seule génération réservée | FR-006, SC-004 |
| V09 | course | figer alerte20, publier21, confirmer20 | enveloppe20 stable, pending21 non perdu | FR-006, FR-007, SC-004 |
| V10 | DB/socket | membre absent/DND/busy puis redevient joignable | dépôt réussi, état motivé, aucun spawn/interruption ; reprise bornée | FR-007, FR-013 |
| V11 | DB | read1–100/ACK puis dix nouvelles entrées | seulement101–110, aucun corps ancien retransmis | FR-008, SC-002 |
| V12 | perte | perdre réponse read avant ACK | curseur inchangé, même page/receipt au rejeu | FR-009, SC-003 |
| V13 | pages | 201 entrées et limite200 puis ACK | curseur200, has_more=true, page suivante201 | FR-008, FR-009 |
| V14 | octets | Unicode/échappements JSON/grosse entrée | JSON≤60Kio, aucune entrée coupée ; entry_too_large si indivisible | FR-008, FR-017 |
| V15 | course | deux read simultanés même membre | une seule plage active, même reçu, aucun saut | FR-009 |
| V16 | ACK | double ACK, ACK ancien après autrepage, tardif non remplacé/remplacé, autre acteur/fil | dernierACK rejouable ; tardif courant accepté ; remplacé/étranger refusé sans avance | FR-009, FR-012, SC-006 |
| V17 | post+ACK | ACK valide joint à post puis ACK invalide | premier atomique, second sans post ni avance | FR-009, FR-011 |
| V18 | historique | history1–20 après curseur50, puis publication21+ | curseur50 intact, borne snapshot stable et suite explicite | FR-010 |
| V19 | idempotence | perdre premier reçu create/post/close puis mêmeclé | même résultat, aucun double objet/entrée/wake | FR-011, SC-003 |
| V20 | idempotence | mêmeclé avec autre corps/cible/action ; chaque champ notice changé au canon | envelope_mismatch, ancienne opération inchangée ; golden historique sans notice identique octet pour octet | FR-011, SC-006 |
| V21 | sécurité | non-membre list/show/read/history/post/ack/close + UUID connu | pas de titre/corps/membres ; erreurs uniformes thread_unavailable | FR-012, SC-006 |
| V22 | sécurité | rôle public/service non autorisé, preuve absente, paramactor | refus avant mutation, aucun privilège par UUID | FR-012, FR-015 |
| V23 | injection | corps malveillant et faux préfixe d'alerte via send | contenu inerte ; aucune thread_notice forgée ni autorité système | FR-012, FR-020 |
| V24 | crash | post commis avant projection099 puis redémarrage | une entrée, même clé de remise, intention reprise | FR-011, FR-014, SC-003 |
| V25 | crash | reçu wrapper préparé/injection possible/ACK perdu puis échéance | outcome_unknown ; pas de retry même borne ; nouvelle mention supérieure permise ; ancien ACK n'acquitte pas nouvelle génération | FR-007, FR-014 |
| V26 | reprise | reçu page actif puis redémarrage, ACK dans10min | reçu utilisable, curseur avancé exactement àthrough_seq | FR-009, FR-014 |
| V27 | capacité | ancien wrapper sans ThreadNoticeV1, reconnexion même instance puis AUTRE instance ; catalogue outil absent | aucune notice réaffectée àautreinstance, capacité renégociée ; catalogue ancien signalé ; DM fonctionnels | FR-014, FR-015, FR-020 |
| V28 | T3 | alerte typée injectée, provider rend réponsefinale | aucun pendingDM ni relaisfinal ; post explicite marche | FR-020 |
| V29 | wrappers | Codex/Claude gérés et interactifs, alerte vs DM témoin ; reply implicite après alerte | seul DM garde relais ; reply refuse thread_notice_not_replyable ; send explicit reste possible ; ACK injection distinctread | FR-015, FR-020 |
| V30 | synthèse | historique avec désaccord et pages incomplètes | recette explicite bornes/limites ; zéro résuméLLM automatique | FR-016, SC-007 |
| V31 | quota | chaque borne N puis N+1 ; créations concurrentes ;100close avec clésnouvelles aprèsclôture | N accepté,N+1 refusé sans mutation ; close répétée n'accumule pas d'opérations ; read/ack/close restent utilisables | FR-017, SC-006 |
| V32 | clôture | noncréateurclose puis créateurclose pendant pending/inflight | refus puis arrêt intentions ; envoienvol tracé ; lecture conservée | FR-018 |
| V33 | parité | chaque action CLI et MCP, flags invalides/champs inconnus | mêmes données/codes, catalogue et docs à jour | FR-015, FR-019 |
| V34 | migration | base synthétique pré102 avec ledger/demandes ; deuxopen | anciens enregistrements intacts, nouvellescontraintes effectives | FR-014, FR-020 |
| V35 | charge | 10000 petites entrées,200read/post et DM témoins | p95<1s local, pagesbornées, pas de scan des corps, DM utilisables | FR-017, SC-005 |
| V36 | relectureagent | suivre documentation sans historique de conception | create/silent/mention/read/ack/synthèse réalisables sans deviner | FR-019, SC-007 |

## Scénarios Gherkin de référence

À transcrire dans le fichier .feature avant les tests Rust ; ils ne sont pas
exécutables dans l'état documentaire actuel.

```gherkin
# language: fr
Fonctionnalité: Discuter entre agents sans réveiller tous les membres
  Scénario: Deux agents échangent dans un fil de quatre membres
    Étant donné un fil ouvert dont A, B, C et D sont membres
    Quand A publie vingt contributions en sollicitant seulement B
    Et B répond en sollicitant seulement A
    Alors C et D ne reçoivent aucune sollicitation
    Et aucun appel fournisseur n'est déclenché chez C ou D par ces contributions
    Et les contributions restent consultables par C et D

  Scénario: Une lecture interrompue ne fait pas perdre une page
    Étant donné que B a confirmé les entrées jusqu'à 10
    Quand Bridget prépare pour B la page 11 à 20
    Et la réponse est perdue avant confirmation de B
    Alors le repère confirmé de B reste 10
    Et sa prochaine lecture restitue la page non confirmée

  Scénario: Une nouvelle mention survit à une ancienne confirmation
    Étant donné que B lit les entrées jusqu'à 20
    Quand A publie l'entrée 21 en sollicitant B
    Et B confirme la page finissant à 20
    Alors l'entrée 21 reste nouvelle pour B
    Et sa sollicitation n'est pas effacée par cette confirmation
```

## Mesures, traces et non-régression

SC-005 : 10000 entrées synthétiques de256octets,200 opérations séquentielles
lecture/publication de1Kio, participants16 max, daemon local sans fournisseur.
Mesurer durée avec horloge monotone ; publier machine/modebuild et p50/p95/max,
nombre de requêtes/entrées renvoyées. Le seuil est un critère de recette sur le
poste déclaré, pas une promesse universelle de performance. Préserver un flux
de20 DM témoins et constater leurs reçus pendant le test, pas seulement après.

Crash tests : points de coupure contrôlés, fermeture/reprise de composants de
test ou erreur injectée, pas SIGKILL d'un processus partagé. Cas de concurrence
avec barrières/canaux, délais d'attente bornés ; pas sleep arbitraire comme preuve.

Les tests de synthèse vérifient l'absence de génération automatique et la recette
de provenance, pas la qualité universelle d'un modèle. Aucun « coûttokens réduit
deX% » déclaré sans mesure fournisseur spécifique. Zéro appel induit chez les
non-ciblés est en revanche vérifiable exactement avec le faux fournisseur.

Régressions minimales : DM send/reply idempotents099 ; preuve auxiliaire et
annuaire public ; observations100/101 ; ledger ; journaux attach ; T3 finalrelay
pour les seuls DM ; inventaire CLI/MCP094 ; skill et allowlists wrappers.
Le rapport futur nomme tests exécutés, refusés par isolation, ignorés et échecs.

## Liaison scénario → test Rust (T003)

Fichier Gherkin : tests/features/102-fils-inter-agents.feature ; chaque scénario
porte les tags `@Vnn` et `@spec102_vnn_<slug>`. Sauf mention contraire, le test
vit dans crates/bridget-daemon/tests/spec102_threads_test.rs.

| Scénario | Test Rust | Emplacement |
|---|---|---|
| V01 | spec102_v01_creation_partagee_sans_alerte | spec102_threads_test.rs |
| V02 | spec102_v02_posts_silencieux_persistent_apres_redemarrage | spec102_threads_test.rs |
| V03 | spec102_v03_vingt_posts_silencieux_zero_injection | spec102_threads_test.rs |
| V04 | spec102_v04_echanges_a_b_sans_reveil_c_d | spec102_threads_test.rs |
| V05 | spec102_v05_normalisation_des_cibles | protocol.rs / threads.rs (unitaire) |
| V06 | spec102_v06_texte_at_all_et_reply_to_sans_alerte | threads.rs (unitaire) + intégration |
| V07 | spec102_v07_cli_noms_ambigus_refuses | cli.rs (unitaire) + CLI réelle |
| V08 | spec102_v08_dix_mentions_une_generation | spec102_threads_test.rs |
| V09 | spec102_v09_mention_concurrente_enveloppe_figee | spec102_threads_test.rs |
| V10 | spec102_v10_membre_absent_dnd_busy_reprise | spec102_threads_test.rs |
| V11 | spec102_v11_lecture_incrementale_101_110 | spec102_threads_test.rs |
| V12 | spec102_v12_reponse_read_perdue_rejouee | spec102_threads_test.rs |
| V13 | spec102_v13_pagination_201_entrees | spec102_threads_test.rs |
| V14 | spec102_v14_budget_octets_et_entry_too_large | spec102_threads_test.rs |
| V15 | spec102_v15_deux_read_simultanes_meme_recu | spec102_threads_test.rs |
| V16 | spec102_v16_matrice_ack | spec102_threads_test.rs |
| V17 | spec102_v17_post_avec_ack_joint | spec102_threads_test.rs |
| V18 | spec102_v18_history_sans_deplacer_le_curseur | spec102_threads_test.rs |
| V19 | spec102_v19_rejeu_create_post_close | spec102_threads_test.rs |
| V20 | spec102_v20_envelope_mismatch_et_canon_notice | communication.rs (canon) + intégration |
| V21 | spec102_v21_non_membre_erreurs_uniformes | spec102_threads_test.rs |
| V22 | spec102_v22_roles_non_autorises | spec102_threads_test.rs |
| V23 | spec102_v23_contenu_inerte_et_notice_non_forgeable | spec102_threads_test.rs |
| V24 | spec102_v24_crash_apres_commit_avant_projection | spec102_threads_test.rs |
| V25 | spec102_v25_issue_inconnue_puis_nouvelle_mention | spec102_threads_test.rs |
| V26 | spec102_v26_recu_actif_survit_au_redemarrage | spec102_threads_test.rs |
| V27 | spec102_v27_capacite_absente_et_autre_instance | spec102_threads_test.rs |
| V28 | spec102_v28_t3_alerte_sans_relais_final | t3code.rs / t3code_098_test.rs |
| V29 | spec102_v29_wrappers_reply_implicite | wrapper.rs + intégration |
| V30 | spec102_v30_recette_synthese_documentee | core_089_skill_test / docs |
| V31 | spec102_v31_quotas_n_et_n_plus_1 | spec102_threads_test.rs |
| V32 | spec102_v32_cloture | spec102_threads_test.rs |
| V33 | spec102_v33_parite_cli_mcp | mcp.rs, cli.rs + intégration |
| V34 | spec102_v34_migration_base_pre102 | spec102_threads_test.rs |
| V35 | spec102_v35_charge_10000_entrees | spec102_threads_test.rs |
| V36 | spec102_v36_relecture_documentation | core_089_skill_test / docs |
