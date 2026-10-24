# Spécification 177 — Bridget Mobile et pilotage fédéré

Date : 2026-10-24
Statut : In Progress
Branche : session-177-bridget-mobile
Responsable : propriétaire de la session 177

## Demande initiale

L'utilisateur demande une application Android complète, packagée, simple à installer, pour commander ses agents Bridget. Une association par QR code doit donner accès aux agents du Mac ou de toute la fédération. Un ou plusieurs serveurs doivent rester pilotables quand le Mac est éteint. Le domaine cartae.app est disponible ; un sous-domaine ou un routage est autorisé. L'implémentation, l'identité et la documentation sont propres à Bridget, sans filiation produit revendiquée.

L'invocation my-specify-all autorise la spécification, l'implémentation, les vérifications, l'intégration Git, le push, la livraison et le nettoyage dans ce périmètre. Elle n'autorise aucune dépense nouvelle.

## Demande reformulée

Livrer une application Android Bridget associée par QR code à un accès humain dédié sur bridget.cartae.app. L'utilisateur peut choisir une machine ou toute sa fédération, retrouver un projet et un agent, lire son historique, lui donner une instruction et suivre son résultat. Il peut agir sur les tâches selon leurs capacités et ses droits. Les commandes restent identifiables après une coupure, sans lancement en double. Le serveur permanent conserve l'autorité de la fédération ; le Mac devient une machine participante. L'installation comprend l'accès serveur, le raccordement et les mises à jour.

## Apport du cadrage

Le domaine est intégré à l'installation. L'identité humaine du téléphone reste distincte des identités des agents et des fournisseurs. Une réponse perdue est un état à réconcilier. Le produit distingue commande locale, commande reçue, travail en cours et résultat. L'indépendance du Mac est une condition de livraison, pas une propriété supposée d'un relais Internet.

## Périmètre et exclusions

Compris : Android, accès distant, une fédération, plusieurs machines et projets, messages, tâches, pièces jointes bornées, notifications, gestion des appareils, migration de l'autorité vers un serveur permanent, installation et mise à jour.
Exclus de cette première livraison, sur décision utilisateur du 24 octobre 2026 : création directe de nouveaux agents depuis le téléphone. Les missions sont adressées aux agents enregistrés, qui gardent leurs droits de délégation. L'extension ultérieure réutilisera le socle ; aucun droit de lancement direct n'est accordé par anticipation.
Exclus : IDE complet, bureau à distance, iOS, basculement automatique entre plusieurs maîtres, publication obligatoire sur une boutique, nouvelle dépense de service tiers.
La possibilité d'une notification repose sur l'autorisation du téléphone et sur la configuration d'un service de livraison. Son absence doit être visible et ne bloque pas l'accès interactif.

## Dépendances

- 089 : noyau de communication et identités opaques.
- 099 : reçus d'envoi idempotent.
- 138 : portée de projet.
- 145–147 : consultation humaine, historique paginé et suivi.
- 148–150, 155 : droits hérités, tâches et demandes de l'enfant.
- 162, 168 : workers et admission distante.
- 173 : documentation anglaise par défaut, français conservé et confidentialité.
- 174–176 : diagnostics, worktrees et entretien borné des builds.

## Scénarios utilisateur

### US1 — Associer et révoquer un téléphone (P1)

L'utilisateur installe l'application et scanne un QR délivré depuis son installation Bridget de confiance.
Acceptation :
1. Un QR valide permet l'association sans saisir de secret dans une conversation.
2. Un QR expiré, consommé ou révoqué est refusé sans nouvel appareil.
3. Un téléphone révoqué ne peut plus consulter ni commander, y compris avec une session déjà ouverte.
4. Le remplacement d'un téléphone n'élargit pas les projets ou machines autorisés.

### US2 — Retrouver les agents de la fédération (P1)

L'utilisateur choisit Tous, Mac, Cartae ou un autre serveur, puis un projet.
Acceptation :
1. Un agent présent sur une machine apparaît une fois avec son identité stable.
2. Les filtres sont mémorisés et ne déclenchent aucune commande.
3. Une machine hors ligne reste identifiée comme indisponible ; ses anciennes données ne deviennent pas des données actuelles.
4. Un projet non autorisé n'apparaît pas dans les listes, recherches ou résultats indirects.

### US3 — Dialoguer et consulter les résultats (P1)

L'utilisateur lit un historique puis envoie une instruction à un destinataire explicite.
Acceptation :
1. Les événements récents, messages et résultats sont accessibles par pages bornées.
2. L'application affiche la différence entre enregistré localement, reçu par Bridget, remis à l'agent et terminé.
3. Une même commande réconciliée ou rejouée avec la même enveloppe n'est exécutée qu'une fois.
4. Une pièce jointe ne donne aucun accès arbitraire au système de fichiers ; les limites et les refus sont explicites.
5. Les brouillons survivent à la fermeture de l'application.

### US4 — Commander les tâches (P1)

L'utilisateur peut demander du travail, suivre, répondre, reprendre, suspendre et annuler selon les capacités disponibles.
Acceptation :
1. Une action indisponible n'est pas proposée comme acquise.
2. Les politiques de pause, quotas, héritage et sélection de machine restent appliquées.
3. Une annulation demandée, une annulation confirmée et un résultat déjà arrivé sont distingués.
4. Un refus précise la cause et l'action utile, sans changer silencieusement modèle, base ou machine.
5. Les interactions demandées par un agent n'accordent pas de nouveaux droits au téléphone ou à cet agent.

### US5 — Continuer après une coupure (P1)

Le téléphone change de réseau, ferme l'application ou perd la réponse d'une commande.
Acceptation :
1. La clé et l'enveloppe de la commande sont conservées avant son premier envoi.
2. Après une réponse perdue, l'application cherche le reçu avant de proposer un nouvel essai.
3. Une commande dont la protection contre les doublons a expiré exige une décision explicite.
4. Une reprise du flux indique une lacune ou une resynchronisation sans prétendre posséder un historique complet.
5. La reprise ne diffuse pas automatiquement de commande métier sous une nouvelle clé.

### US6 — Piloter les serveurs Mac éteint (P1)

La fédération fonctionne avec une autorité permanente et les machines participantes reconnectées.
Acceptation :
1. Le téléphone lit et commande un agent du serveur après extinction du Mac.
2. Le Mac hors ligne est signalé, sans bloquer les autres machines.
3. La migration conserve identités, reçus, tâches et possibilités de reprise vérifiables.
4. Aucun second maître concurrent n'est mis en service dans la même fédération.
5. Une attestation liée au poste d'origine ne devient pas valide par simple déplacement d'un fichier.

### US7 — Être averti et revenir au bon résultat (P2)

L'utilisateur reçoit une notification de résultat, de question ou de blocage.
Acceptation :
1. La notification ouvre l'objet autorisé et relit son état actuel.
2. Aucun contenu de conversation ni secret fournisseur n'est envoyé au service de notification.
3. L'utilisateur règle les notifications et les périodes silencieuses.
4. Le refus des notifications ne bloque pas l'application ; leur état est explicite.

### US8 — Installer, mettre à jour et récupérer (P1)

L'utilisateur installe un seul APK et conserve une fédération maintenable.
Acceptation :
1. Un APK signé et vérifié est disponible ; son installation ne demande aucun terminal ni VPN sur le téléphone.
2. L'accès serveur et le raccordement des machines sont couverts par l'installation Bridget.
3. Une version incompatible est refusée avec une consigne claire.
4. Une sauvegarde et un retour arrière sont vérifiés avant migration.
5. La documentation publique a une version anglaise par défaut et une version française.
6. Les caches, fichiers temporaires et preuves de ce chantier sont traités sans perdre les travaux ou données de production.

## Exigences fonctionnelles

- FR-001 : association humaine temporaire, à usage unique et révocable.
- FR-002 : identité de téléphone distincte de toute identité d'agent ; secrets protégés au repos.
- FR-003 : contrôle des droits sur chaque lecture, commande et abonnement.
- FR-004 : sélection globale ou par machine, puis projet, sans diffusion implicite.
- FR-005 : identité d'agent stable et absence de doublon dans l'annuaire.
- FR-006 : indication de disponibilité et de fraîcheur des observations.
- FR-007 : historique paginé et événements progressifs bornés.
- FR-008 : envoi à un destinataire explicite avec reçu durable.
- FR-009 : clé et enveloppe persistées avant envoi ; réconciliation et rejeu strict.
- FR-010 : distinctions visibles entre attente locale, reçu, remise, exécution, résultat et refus.
- FR-011 : brouillons persistants et pièces jointes autorisées, bornées et non exécutables.
- FR-012 : demande de travail adressée à un agent enregistré et pilotage des tâches réutilisant les décisions et droits natifs ; pas de création directe d'agent par le téléphone.
- FR-013 : refus actionnables, sans diagnostic technique brut contenant des secrets.
- FR-014 : pas de changement silencieux des axes, de la base ou de la clé.
- FR-015 : reprise réseau et flux avec lacunes visibles.
- FR-016 : indépendance du Mac pour les agents des serveurs.
- FR-017 : migration d'un maître unique, sans copie de base ouverte ni relance métier implicite.
- FR-018 : préservation des attestations et autorités d'origine.
- FR-019 : notifications minimales et réglables ; ouverture avec authentification.
- FR-020 : gestion et révocation des appareils, expiration et renouvellement.
- FR-021 : installation serveur intégrée et HTTPS sous bridget.cartae.app.
- FR-022 : APK signé, mise à jour vérifiée, compatibilité et retour arrière.
- FR-023 : interface accessible en français et anglais ; messages simples.
- FR-024 : documentation anglaise par défaut et française conservée.
- FR-025 : journaux et preuves sans secret ; limites de mémoire, débit et stockage.
- FR-026 : code et présentation propres à Bridget, réemploi du moteur local.
- FR-027 : entretien des builds et nettoyage du périmètre selon les outils existants.

## Critères de réussite

- SC-001 : l'association d'un téléphone préparé prend moins de 2 minutes, hors téléchargement.
- SC-002 : l'annuaire de 500 agents et 50 projets reste filtrable en moins de 2 secondes sur le téléphone de recette, après réception d'une page.
- SC-003 : une coupure après réception serveur suivie de 3 reprises produit une seule commande durable et une seule remise métier.
- SC-004 : la révocation bloque la prochaine action et ferme le flux dans les 30 secondes.
- SC-005 : une recette Wi-Fi puis réseau mobile réalise instruction et lecture du résultat.
- SC-006 : la même recette vers un serveur réussit Mac éteint, sans identité perdue.
- SC-007 : chaque refus testé affiche un état, une cause et une action ; aucun succès n'est déduit d'un timeout.
- SC-008 : une mise à jour conserve association, brouillons et reçus sans perte.
- SC-009 : toutes les étapes de livraison requises sont attestées avant le statut Livré.
- SC-010 : la recette et les tests de perte de réponse n'accèdent à aucun projet non autorisé.

## Entités

Appareil autorisé, autorité de fédération, machine, projet, agent, commande, reçu, tâche, événement, pièce jointe et préférence de notification.

## Cas limites

QR intercepté ou déjà consommé ; réponse d'association perdue ; téléphone perdu ; maître indisponible ; Mac éteint ; résultat reçu après annulation ; processus Android arrêté ; horloge incohérente ; doublon d'identifiant avec enveloppe différente ; protection expirée ; attestation liée à une autre machine ; projet supprimé ; fichier trop grand ; notifications refusées ; mises à jour incompatibles.

## Hypothèses

Le serveur Cartae et cartae.app sont disponibles, sans coût nouveau. L'accès DNS a été confirmé en lecture. Le téléphone de recette et la configuration des notifications seront requis pour la validation physique. Une absence de ces accès bloque la seule preuve ou livraison qui en dépend, pas le travail indépendant.
