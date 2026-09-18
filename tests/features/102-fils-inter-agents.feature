# language: fr
# Session 102 — chaque scénario porte l'identifiant V de test-plan.md et le
# nom du test Rust qui le vérifie (préfixe spec102_ dans le code). Les faits
# « publié », « alerte injectée » et « page confirmée » sont distincts.
Fonctionnalité: Discuter entre agents sans réveiller tous les membres

  @V01 @spec102_v01_creation_partagee_sans_alerte
  Scénario: Créer un fil partagé sans solliciter personne
    Étant donné quatre agents A, B, C et D enregistrés dans un daemon isolé
    Quand A crée le fil « Relecture sécurité » avec A, B, C et D
    Alors A et C retrouvent le même fil avec A comme créateur et membre
    Et aucune alerte n'est injectée chez aucun membre

  @V02 @spec102_v02_posts_silencieux_persistent_apres_redemarrage
  Scénario: Les contributions silencieuses survivent au redémarrage
    Étant donné un fil ouvert avec A et B
    Quand A puis B publient chacun une contribution sans cible
    Et le stockage est fermé puis rouvert
    Alors les entrées 1 et 2 conservent leurs auteurs et leurs corps exacts

  @V03 @spec102_v03_vingt_posts_silencieux_zero_injection
  Scénario: Vingt dépôts silencieux ne réveillent personne
    Étant donné un fil ouvert avec A, B, C et D observés par de faux fournisseurs
    Quand A publie vingt contributions avec une liste de cibles vide
    Alors aucun membre ne reçoit d'injection, de lecture automatique ni d'appel de modèle

  @V04 @spec102_v04_echanges_a_b_sans_reveil_c_d
  Scénario: Deux agents échangent dans un fil de quatre membres
    Étant donné un fil ouvert dont A, B, C et D sont membres
    Quand A publie vingt contributions en sollicitant seulement B
    Et B répond en sollicitant seulement A
    Alors C et D ne reçoivent aucune sollicitation
    Et aucun appel fournisseur n'est déclenché chez C ou D par ces contributions
    Et les contributions restent consultables par C et D

  @V05 @spec102_v05_normalisation_des_cibles
  Scénario: Les cibles sont normalisées avant validation
    Étant donné un fil ouvert avec A, B et C
    Quand A publie en ciblant B, B de nouveau, lui-même puis « tous »
    Alors l'ensemble effectif exclut A, ne contient B qu'une fois
    Et la mention de soi produit une notice sans cible supplémentaire

  @V06 @spec102_v06_texte_at_all_et_reply_to_sans_alerte
  Scénario: Le texte, le code et la citation ne sollicitent personne
    Étant donné un fil ouvert avec A et B
    Quand A publie un corps contenant « @all », du Markdown, du code et une référence à l'entrée 1 sans cible
    Alors aucune alerte n'est émise et la référence est conservée telle quelle

  @V07 @spec102_v07_cli_noms_ambigus_refuses
  Scénario: La ligne de commande refuse un nom ambigu
    Étant donné deux agents portant le même nom d'affichage
    Quand un membre publie avec « --notify » suivi de ce nom
    Alors la commande refuse pour ambiguïté sans publier
    Et un renommage ultérieur ne change pas l'adresse UUID du fil

  @V08 @spec102_v08_dix_mentions_une_generation
  Scénario: Dix mentions avant départ forment une seule sollicitation
    Étant donné un fil ouvert avec A et B dont B n'est pas encore joignable
    Quand A publie dix contributions en ciblant B
    Alors dix entrées existent, une seule ligne de sollicitation pour B
    Et au plus une génération de remise est réservée

  @V09 @spec102_v09_mention_concurrente_enveloppe_figee
  Scénario: Une nouvelle mention survit à une ancienne confirmation
    Étant donné que B lit les entrées jusqu'à 20
    Quand A publie l'entrée 21 en sollicitant B
    Et B confirme la page finissant à 20
    Alors l'entrée 21 reste nouvelle pour B
    Et sa sollicitation n'est pas effacée par cette confirmation
    Et l'enveloppe déjà figée pour la borne 20 n'est pas modifiée

  @V10 @spec102_v10_membre_absent_dnd_busy_reprise
  Scénario: Absence, DND et occupation motivent l'attente sans lancement
    Étant donné un fil ouvert avec A et B
    Quand A cible B alors que B est absent, puis en « ne pas déranger », puis occupé
    Alors chaque dépôt réussit avec un motif d'attente visible
    Et aucun agent n'est lancé ni interrompu
    Et la sollicitation part de façon bornée quand B redevient joignable

  @V11 @spec102_v11_lecture_incrementale_101_110
  Scénario: Rattraper seulement les nouveautés
    Étant donné que B a confirmé les entrées 1 à 100
    Quand dix nouvelles entrées sont publiées
    Alors la lecture suivante de B ne restitue que 101 à 110

  @V12 @spec102_v12_reponse_read_perdue_rejouee
  Scénario: Une lecture interrompue ne fait pas perdre une page
    Étant donné que B a confirmé les entrées jusqu'à 10
    Quand Bridget prépare pour B la page 11 à 20
    Et la réponse est perdue avant confirmation de B
    Alors le repère confirmé de B reste 10
    Et sa prochaine lecture restitue la même page et le même reçu

  @V13 @spec102_v13_pagination_201_entrees
  Scénario: Une page pleine annonce la suite
    Étant donné un fil de 201 entrées
    Quand B lit avec une limite de 200 puis confirme
    Alors son repère vaut 200, la page annonçait une suite
    Et la lecture suivante commence à 201

  @V14 @spec102_v14_budget_octets_et_entry_too_large
  Scénario: Le budget d'octets ne coupe jamais une entrée
    Étant donné des corps Unicode avec échappements JSON et une entrée volumineuse
    Quand B lit le fil
    Alors chaque page sérialisée tient dans 60 Kio sans entrée tronquée
    Et une entrée indivisible trop grosse est refusée au dépôt comme « entry_too_large »

  @V15 @spec102_v15_deux_read_simultanes_meme_recu
  Scénario: Deux lectures concurrentes du même membre partagent un reçu
    Étant donné un fil de 30 entrées et un repère de B à 0
    Quand deux lectures de B s'exécutent en même temps
    Alors une seule plage active existe, avec le même reçu, sans saut

  @V16 @spec102_v16_matrice_ack
  Scénario: La confirmation refuse ce qu'elle ne peut pas prouver
    Étant donné un reçu courant de B pour la page 11 à 20
    Quand B confirme deux fois, confirme un ancien reçu, confirme après remplacement, puis un autre acteur confirme ce reçu
    Alors le dernier reçu confirmé est rejouable
    Et un reçu tardif encore courant est accepté
    Et un reçu remplacé ou étranger est refusé sans avancer le repère

  @V17 @spec102_v17_post_avec_ack_joint
  Scénario: Une confirmation jointe au dépôt est atomique
    Étant donné un reçu courant de B
    Quand B publie en joignant ce reçu valide, puis publie en joignant un reçu invalide
    Alors le premier dépôt avance le repère et publie en une transaction
    Et le second ne publie rien et n'avance pas le repère

  @V18 @spec102_v18_history_sans_deplacer_le_curseur
  Scénario: Relire une plage ancienne sans toucher au repère
    Étant donné un repère de B à 50
    Quand B relit l'historique de 1 à 20 puis que 21 entrées sont publiées
    Alors le repère de B reste 50
    Et la borne de relecture reste figée avec une suite explicite

  @V19 @spec102_v19_rejeu_create_post_close
  Scénario: Le rejeu d'une même clé ne duplique rien
    Étant donné un premier reçu perdu pour une création, un dépôt et une clôture
    Quand chaque opération est rejouée avec la même clé
    Alors le même résultat revient sans fil, entrée ni sollicitation supplémentaire

  @V20 @spec102_v20_envelope_mismatch_et_canon_notice
  Scénario: Une même clé avec un autre contenu est refusée
    Étant donné une opération réussie
    Quand la même clé est rejouée avec un autre corps, une autre cible ou une autre action
    Alors le refus « envelope_mismatch » laisse l'opération d'origine intacte
    Et le canon d'un message sans notice garde ses octets historiques
    Et chaque champ d'une notice change le canon

  @V21 @spec102_v21_non_membre_erreurs_uniformes
  Scénario: Un non-membre n'apprend rien du fil
    Étant donné un agent E qui connaît l'UUID d'un fil dont il n'est pas membre
    Quand E liste, consulte, lit, relit, publie, confirme ou clôt
    Alors chaque réponse est « thread_unavailable » sans titre, corps ni membres

  @V22 @spec102_v22_roles_non_autorises
  Scénario: Les rôles non autorisés sont refusés avant toute mutation
    Étant donné une connexion publique, une connexion de service et une connexion sans preuve
    Quand elles envoient une requête de fil, y compris avec un paramètre d'acteur
    Alors le refus intervient avant toute mutation et aucun UUID ne confère de privilège

  @V23 @spec102_v23_contenu_inerte_et_notice_non_forgeable
  Scénario: Le contenu reste une donnée
    Étant donné un corps malveillant et un message direct imitant une alerte de fil
    Quand ils sont déposés puis lus
    Alors le contenu est conservé sans exécution
    Et aucune notice de fil n'est reconnue depuis un envoi externe

  @V24 @spec102_v24_crash_apres_commit_avant_projection
  Scénario: Un arrêt après commit reprend la même intention
    Étant donné un dépôt commis dont la remise n'est pas encore projetée
    Quand le daemon redémarre
    Alors une seule entrée existe et la même clé de remise est reprise

  @V25 @spec102_v25_issue_inconnue_puis_nouvelle_mention
  Scénario: Une issue inconnue ne bloque pas les mentions suivantes
    Étant donné une alerte peut-être injectée dont l'accusé est perdu
    Quand l'échéance passe, puis qu'une mention plus haute arrive, puis que l'ancien accusé arrive
    Alors l'issue reste « outcome_unknown » sans nouvelle tentative de la même borne
    Et une nouvelle génération part pour la mention plus haute
    Et l'ancien accusé ne l'acquitte pas

  @V26 @spec102_v26_recu_actif_survit_au_redemarrage
  Scénario: Un reçu de page reste utilisable après redémarrage
    Étant donné un reçu actif de B
    Quand le daemon redémarre et que B confirme dans les dix minutes
    Alors le repère avance exactement jusqu'à la borne de la page

  @V27 @spec102_v27_capacite_absente_et_autre_instance
  Scénario: Un ancien adaptateur n'est ni servi ni contourné
    Étant donné un adaptateur sans capacité d'alerte de fil
    Quand il se reconnecte avec la même instance puis avec une autre instance
    Alors aucune notice n'est réaffectée à l'autre instance
    Et la capacité est renégociée à chaque connexion
    Et un catalogue d'outils ancien est signalé, les messages directs restant fonctionnels

  @V28 @spec102_v28_t3_alerte_sans_relais_final
  Scénario: T3 n'expédie pas la réponse finale d'une alerte de fil
    Étant donné un fil T3 recevant une alerte typée
    Quand le fournisseur rend une réponse finale
    Alors aucune attente de message direct ni relais final n'est créé
    Et une publication explicite dans le fil fonctionne

  @V29 @spec102_v29_wrappers_reply_implicite
  Scénario: La réponse implicite ne vise jamais l'alerte
    Étant donné des wrappers Codex et Claude gérés et interactifs recevant une alerte puis un message direct témoin
    Quand la réponse implicite est utilisée après l'alerte
    Alors seul le message direct garde son relais
    Et « reply » refuse avec « thread_notice_not_replyable »
    Et un envoi explicite reste possible

  @V30 @spec102_v30_recette_synthese_documentee
  Scénario: La synthèse est une recette, pas un automate
    Étant donné un historique avec désaccord et des pages incomplètes
    Quand un membre suit la recette documentée de synthèse
    Alors bornes et limites sont explicites et aucun résumé automatique n'est produit

  @V31 @spec102_v31_quotas_n_et_n_plus_1
  Scénario: Chaque borne accepte N et refuse N+1
    Étant donné chaque limite de participants, titre, corps, page, fils et historique
    Quand N puis N+1 sont tentés, avec des créations concurrentes et cent clôtures rejouées
    Alors N est accepté, N+1 refusé sans mutation
    Et les clôtures répétées n'accumulent pas d'opérations
    Et lecture, confirmation et clôture restent possibles à saturation

  @V32 @spec102_v32_cloture
  Scénario: Clore arrête les sollicitations sans effacer l'historique
    Étant donné un fil avec des sollicitations en attente et une en vol
    Quand un non-créateur puis le créateur demandent la clôture
    Alors le premier est refusé, le second annule les intentions non parties
    Et l'envoi en vol reste tracé et la lecture reste possible

  @V33 @spec102_v33_parite_cli_mcp
  Scénario: Ligne de commande et outil MCP produisent les mêmes données
    Étant donné les huit actions de fil
    Quand elles sont invoquées par CLI et par MCP, avec des drapeaux ou champs invalides
    Alors les données et codes d'erreur sont identiques et le catalogue est à jour

  @V34 @spec102_v34_migration_base_pre102
  Scénario: Une base antérieure est migrée sans perte
    Étant donné une base synthétique pré-102 avec ledger et demandes
    Quand elle est ouverte deux fois par le Store
    Alors les anciens enregistrements sont intacts et les nouvelles contraintes effectives

  @V35 @spec102_v35_charge_10000_entrees
  Scénario: La charge locale reste sous la seconde
    Étant donné dix mille petites entrées et vingt messages directs témoins
    Quand deux cents lectures et publications sont exécutées
    Alors le p95 local est inférieur à une seconde et les témoins restent utilisables

  @V36 @spec102_v36_relecture_documentation
  Scénario: La documentation suffit à un agent sans historique de conception
    Étant donné la seule documentation livrée
    Quand un agent doit créer, publier en silence, mentionner, lire, confirmer puis synthétiser
    Alors chaque étape est réalisable sans deviner
