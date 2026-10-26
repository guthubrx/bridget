# Spécification183 — Interface mobile et suivi d’activité

Statut : Implemented. Pipeline : Livré le26octobre2026 (activation et nettoyage vérifiés ; voir validation/livraison.md). Date : 25 octobre 2026. Branche : session-183-interface-mobile.

## Demande initiale et cadrage

L’utilisateur demande de réaliser la proposition d’interface présentée, par le pipeline complet. Il confirme les aperçus HTML interactifs isolés. Il ajoute un volet gauche regroupant agents/échanges, tous ou favoris, avec icônes et marqueurs d’attention, travail en cours et questions.

Demande reformulée : améliorer les parcours natifs Bridget sur téléphone : accès rapide aux agents, recherche dans les échanges, dictée avec relecture, pièces jointes visibles avant envoi, code copiable, aperçus en grand et état réel de compactage. Conserver la flèche bleue, la file dépliable et ses actions, les questions et la réponse libre. Réutiliser les règles de portée, reçus et reprises existantes.

Apport : retrouver le bon échange et agir sans confondre enregistrement local, remise à l’agent et résultat. Aucun état de succès n’est inféré d’une simple présence réseau.

Hors périmètre : client Mac/Linux complet, migration du socle Android, synchronisation des brouillons entre appareils, accès réseau des pages générées, voix conversationnelle permanente, modifications des comptes d’applications tierces.

## Scénarios et tests

### US1 — Volet de navigation et attention (P1)

Ouvrir le volet depuis une conversation. Retrouver les agents autorisés avec icône, nom, projet/machine, filtre Tous/Favoris et recherche. Passer d’un agent à l’autre sans perdre brouillon, file ni position de lecture. Les entrées existantes de l’annuaire et les favoris restent la source.

Acceptation : deux agents de même nom restent distinguables ; un favori hors ligne est signalé comme ancien ; une question active a un marqueur explicite ; un travail en cours a son état ; une fin ou nouveauté observée demeure à consulter jusqu’à ouverture effective du bon échange. L’état inconnu ou ancien est identifié, sans faux « terminé ». Pagination et liste vide restent utilisables.

### US2 — Conversation, recherche et saisie (P1)

Lire les réponses et activités sans bruit technique. Rechercher un texte dans l’historique chargé, atteindre le résultat et comprendre si des messages plus anciens restent à charger. Dicter une instruction puis la relire/modifier. La flèche bleue garde l’envoi normal dans la file. Modifier, annuler ou demander une interruption depuis la file existante.

Acceptation : une dictée annulée ou indisponible ne perd pas le brouillon ; elle n’envoie rien automatiquement. Un changement d’agent pendant une dictée n’adresse jamais le résultat au mauvais agent. Une recherche vide ou sans résultat n’efface pas les messages. La recherche annonce sa couverture, elle ne prétend pas rechercher un historique non chargé.

### US3 — Fichiers et lecture enrichie (P1)

Choisir photo ou fichier depuis « + », voir la pièce jointe et la retirer avant son envoi. Les références déjà téléversées restent réutilisées. Lire du code avec langage et Copier ; ouvrir une page générée et revenir à la conversation à la même position. Les activités restent dépliables.

Acceptation : aucune donnée de fichier n’est envoyée au simple choix ; le destinataire et l’association sont figés avant réseau. Une annulation ou un refus garde le texte ; un résultat réseau inconnu conserve l’identité de commande. Les blocs de code ne sont ni dupliqués ni tronqués par le rendu. Un HTML trop grand ou un moteur non corrigé présente un refus explicite et le code reste lisible.

### US4 — Aperçu interactif isolé (P1)

Ouvrir un HTML autonome avec scripts locaux pour boutons, graphiques et formulaires locaux. L’aperçu n’accède ni à la session Bridget, ni aux fichiers, ni au réseau ; il ne déclenche aucun envoi ni permission. Le mode statique demeure disponible.

Acceptation : compteur manipulable ; essais de lecture de fichiers, stockage de session, navigation, ouverture externe, téléchargements, réseau et accès au parent refusés. Une page défaillante peut être fermée. Retour au fil sans perte du brouillon ni de sa position. Les ressources externes ne sont pas chargées silencieusement.

### US5 — État réel de travail (P1)

Voir « Compactage du contexte en cours » lorsque le producteur l’atteste. Revenir à l’état normal lorsque la fin est attestée ; interruption, changement de session et perte du suivi n’entretiennent pas un compactage fictif. Aucun pourcentage inventé.

Acceptation : début/fin du même item et même tour ; doublon sans effet ; événement d’un autre agent ignoré. Le contrat est natif Bridget et documentable pour chaque adaptateur. Un producteur muet ne donne aucune fausse information.

## Exigences fonctionnelles

- FR-001 : volet gauche accessible depuis le fil, tous/favoris, recherche, icônes, identité explicite, pagination et fermeture par Retour.
- FR-002 : marqueurs avec texte accessible, question prioritaire, état en cours, nouveauté/fin observée et état ancien ; aucune preuve de lecture créée hors du bon fil visible.
- FR-003 : brouillon, file et position propres à l’agent conservés lors de navigation et aperçu.
- FR-004 : recherche bornée aux messages disponibles avec compte de résultats et accès au résultat ; couverture explicite.
- FR-005 : dictée éditable, bornée à la taille actuelle, sans envoi automatique et liée à l’agent/association d’origine.
- FR-006 : pièces jointes préparées localement et retirables, envoyées uniquement sur action explicite ; reçus et reprises existants conservés.
- FR-007 : code coloré avec langage et copie exacte ; aperçu en grand et retour au fil.
- FR-008 : HTML interactif sans pont privilégié, accès fichier, session, permission, navigation externe ou réseau ; mode statique disponible.
- FR-009 : compactage attesté avec identité et cycle ; absence, trou d’observation et déconnexion explicites.
- FR-010 : conserver les parcours de questions, notifications et commandes idempotentes ; interface et erreurs FR/EN, cibles tactiles au moins48dp.
- FR-011 : aucune dépendance à un client externe ; contrats réutilisables par un futur client, interface actuelle Android assumée.
- FR-012 : recette émulateur, essais d’isolation, preuves visibles, source intégrée/poussée, installation vérifiée et nettoyage tracé.

## Critères de réussite

- SC-001 : passage entre deux échanges en deux actions au plus après ouverture du volet ; aucun brouillon perdu lors de dix alternances.
- SC-002 : une question active et une nouveauté attestée se distinguent d’un travail en cours, sans se fier seulement à une couleur ; état ancien visible en cas de perte de connexion.
- SC-003 : copie d’un code identique aux octets textuels présentés ; recherche sans perte de l’historique ; pièce jointe retirée avant envoi n’atteint pas le serveur.
- SC-004 : compteur local utilisable ; toutes les tentatives hostiles décrites en US4 échouent sans effet privilégié observé.
- SC-005 : début/fin de compactage propagés par le même parcours que les autres activités ; aucun résultat d’un autre tour appliqué.
- SC-006 : parcours réels et refus illustrés dans le diaporama de cette livraison ; aucune case restante pour déclarer Implemented.

## Entités et dépendances

Agent autorisé, favori local, repère de lecture local, événement de travail natif, demande humaine existante, brouillon, préparation de fichier, commande et reçu existants. Dépendances :177mobile,178sécurité,180file,181questions et182autorité produit/documentation.

## Hypothèses et limites à vérifier

Un volet est modal sur téléphone et peut rester visible sur grand écran. Les préférences et repères sont propres à l’association. Une absence de signal n’est pas une preuve de fin. La voix réelle dépend du moteur système ; son indisponibilité garde la saisie. Les aperçus interactifs concernent du contenu autonome, sans bibliothèques distantes.
