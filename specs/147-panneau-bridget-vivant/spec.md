# SPEC147 — Panneau Bridget vivant et sélection mémorisée

Date : 2026-10-08 ; extensions approuvées le 2026-10-09. Développement : Implemented, Validé,54/54 tâches,31FR/20SC. Sources committées/fusionnées/poussées. Livraison EN COURS : Bridget installé et actif, T3 compilé/signé prêt mais activation encore en attente. Branche : `session-147-panneau-bridget-vivant`. Ne pas confondre package T3 vérifié et application active.

## Besoin et périmètre

L'utilisateur veut deux améliorations du panneau Bridget. Les fils partagés et les messages doivent suivre leurs changements sans clic sur le bouton de rafraîchissement. Le fil choisi dans une conversation T3 doit rester sélectionné quand l'utilisateur change d'agent, puis revient.

L'utilisateur a approuvé ces deux besoins dans la session147. La session146 reste le socle. Le panneau conserve son apparence, son ordre récent d'abord, sa pagination, sa copie exacte et ses contrôles de dépliage.

Ajout utilisateur approuvé à la même session : montrer les destinataires sollicités dans la ligne auteur/date existante. Cet ajout de lisibilité ne change aucun message, destinataire effectif, notification ou droit.

Ajout approuvé à147 : compatibilité des UUID dans les commandes et l'outil CLI/MCP de fils partagés, et des outils Bridget dans les lancements Claude/GLM attestés. Le plan de montage MCP concret doit être validé avant de créer ses nouvelles tâches. La cause locale est vérifiée : configuration MCP effective sans Bridget et injection T3 limitée à t3-code pour regional-wrkr-1/claude_glm. Aucune exclusion volontaire GLM déduite.

Cette consultation humaine reste indépendante du travail des agents. Elle ne publie aucun message. Elle ne prend aucune mission en charge. Elle ne réveille aucun agent et ne consomme aucun appel de modèle.

L'approbation couvre la préparation et le développement isolé. Elle ne vaut pas autorisation nouvelle d'installation ou de redémarrage. Les travaux non committés des sessions142 et145 restent intacts.

## User Scenarios & Testing

### US147-01 — Voir les changements sans rafraîchir (P1)

Comme utilisateur, je veux que le panneau ouvert affiche les nouveautés et les changements des fils partagés sans devoir cliquer.

Test indépendant : ouvrir un panneau avec des fils de test. Créer un fil accessible. Ajouter des messages dans un fil non sélectionné puis dans le fil sélectionné, remplacer une consigne et fermer un fil. Mettre à jour un nom d'agent dans les données d'annuaire de test. Vérifier le contenu et le tri sans interaction de l'utilisateur.

1. Étant donné un panneau ouvert et une connexion saine, quand un fil accessible reçoit un message, alors la liste reflète son activité et son ordre sous deux secondes.
2. Étant donné un fil sélectionné, quand il reçoit un message, alors ce message devient visible sous deux secondes, sans doublon et sans perdre les messages déjà chargés.
3. Étant donné un nouveau fil accessible, un nom de participant actualisé ou une clôture, quand ce changement est confirmé, alors la liste et les détails ouverts reflètent cet état sous deux secondes. Le panneau relit les métadonnées autorisées ; il ne crée pas une fonction de renommage du fil ou d'édition de ses membres.
4. Étant donné une consigne remplacée, quand le remplacement est consultable, alors ses relations de remplacement sont à jour. L'ancienne consigne reste une preuve historique, jamais une consigne courante restaurée par une actualisation tardive.
5. Étant donné un panneau ouvert sans changement, quand une minute passe, alors aucune relecture périodique de la liste ou des messages n'a lieu.
6. Étant donné une rafale de changements, quand le panneau les reçoit, alors il les regroupe. Il rejoint le dernier état sans démarrer une lecture de contenu pour chaque événement.

### US147-02 — Retrouver le fil choisi en revenant (P1)

Comme utilisateur, je veux retrouver le fil partagé que j'avais choisi dans chaque conversation T3. Je ne veux pas le sélectionner à nouveau après chaque changement d'agent.

Test indépendant : choisir un fil A dans une conversation T3 A. Choisir un autre fil B dans une conversation T3 B. Revenir sur A, puis sur B. Répéter dans deux projets et deux environnements portant des identifiants de conversation identiques dans les données de test.

1. Étant donné un fil choisi dans la conversation A, quand je vais sur B puis reviens sur A, alors le fil A reste sélectionné. Ses messages autorisés sont affichés sans clic supplémentaire.
2. Étant donné des fils choisis dans A et B, quand je passe de l'une à l'autre, alors chacune retrouve son propre choix. La sélection de B n'écrase pas celle de A.
3. Étant donné le fil mémorisé hors de la première page de liste, quand je reviens dans sa conversation, alors ce fil est retrouvé et sélectionné. Le premier fil de la liste ne le remplace pas.
4. Étant donné des projets ou environnements distincts, quand une conversation a le même identifiant dans deux contextes, alors son choix et ses messages ne passent pas d'un contexte à l'autre.
5. Étant donné une interruption réseau, quand je reviens dans une conversation, alors son choix reste mémorisé. Le panneau indique l'indisponibilité et ne présente pas un contenu ancien comme un contenu dont l'accès vient d'être confirmé.
6. Étant donné un refus d'accès au fil choisi confirmé ou ce fil supprimé confirmé, quand je reviens dans sa conversation, alors le choix devenu inaccessible est retiré et aucun de ses messages n'apparaît. Un refus de contexte ne prouve pas la suppression de ce fil ; il masque ses données sans lui accorder un droit.

### US147-03 — Reprendre après une interruption sans fuite (P1)

Comme utilisateur, je veux une vue fiable après une interruption. Je veux aussi que les données dont l'accès a été retiré disparaissent.

Test indépendant : interrompre le lien, produire des changements, puis rétablir le lien. Simuler une réponse tardive de l'ancien contexte et une révocation d'appartenance. Vérifier les données visibles et les permissions de chaque lecture.

1. Étant donné une interruption, quand la connexion revient, alors le panneau rattrape les changements manqués et retrouve le dernier état autorisé, sans trou ni doublon.
2. Étant donné une notification de changement, quand le panneau la traite, alors la notification ne contient aucun corps de message, prompt, raisonnement ou secret.
3. Étant donné une modification d'appartenance, quand le panneau reprend la lecture, alors l'accès est vérifié à nouveau. Un accès révoqué efface immédiatement le contenu concerné après constat du refus.
4. Étant donné une réponse encore en transit pour A, quand je passe sur B, alors cette réponse ne remplit jamais la vue B. Un retour ultérieur sur A ne rend pas cette réponse ancienne valide à nouveau.
5. Étant donné un panneau fermé, quand de nouveaux changements interviennent, alors aucun flux propre à ce panneau ni lecture automatique de contenu ne reste actif.
6. Étant donné une fermeture puis une réouverture, quand le panneau retrouve une connexion saine, alors il recharge l'état autorisé du fil mémorisé et rétablit son suivi.

### US147-06 — Utiliser un UUID sans astuce de casse (P1)

Comme utilisateur, je veux qu'un UUID valide copié en majuscules ou en minuscules désigne le même fil, membre ou destinataire dans les actions de fils partagés. Je ne veux pas que l'agent doive bricoler cet identifiant pour utiliser Bridget.

Pourquoi P1 : une variation de casse ne doit pas empêcher une communication autorisée ni créer un doublon lors d'une reprise.

Test indépendant : utiliser les commandes CLI et l'outil MCP de fils partagés existants sur une base privée. Comparer UUID hyphéné de36 caractères en minuscules, majuscules et casse mixte. Répéter une écriture avec le même identifiant de requête. Tester membres, targets et reçus, puis des UUID malformés.

1. Étant donné un UUID hyphéné valide, quand une commande CLI ou l'outil MCP de fils partagés reçoit sa forme majuscule, minuscule ou mixte, alors elle le valide et utilise la même forme canonique avant l'idempotence et les accès SQL.
2. Étant donné une écriture déjà reçue, quand le même appel revient avec une autre casse des références UUID, alors il reste le même appel idempotent. Aucun second événement, membre ou destinataire n'est créé.
3. Étant donné des membres, targets ou reçus identifiés par UUID, quand l'entrée varie seulement la casse, alors leurs relations pointent vers les mêmes identités et fils. Les résultats suivent le format canonique existant.
4. Étant donné un UUID invalide, quand l'entrée le reçoit, alors elle refuse l'opération avant toute mutation dans les champs déclarés UUID. Les noms et préfixes déjà acceptés par le CLI conservent leur résolution existante ; la correction ne transforme pas un UUID malformé en identifiant accepté.
5. Étant donné un corps contenant du texte qui ressemble à un UUID, quand le message est traité, alors le corps reste strictement inchangé. Seuls les champs d'identifiant déclarés sont normalisés.
6. Étant donné les gardes humaines145/146/147, quand les tests de compatibilité s'exécutent, alors leurs refus d'identité, de projet ou de rattachement restent inchangés. La normalisation n'accorde aucune autorité.

### US147-07 — Retrouver les outils Bridget dans Claude/GLM (P1)

Comme utilisateur, je veux que les agents lancés par les chemins Claude/GLM pris en charge aient les vrais outils Bridget, dont bridget_thread. Je veux une identité attestée et des permissions respectées, pas un outil qui prétendrait parler au nom d'un autre agent.

Pourquoi P1 : le canal interagents doit fonctionner dans les environnements de modèles pris en charge. L'agent ne doit pas remplacer un outil absent par un contournement d'autorité.

Test indépendant : construire les configurations et montages MCP des chemins de lancement réellement identifiés avec des programmes de test. Inspecter le catalogue et le contexte d'identité sans lancer de modèle ni modifier une configuration de production.

1. Étant donné un lancement Claude/GLM pris en charge avec Bridget activé, quand le montage MCP est construit, alors le serveur Bridget et le catalogue attendu, dont bridget_thread, sont disponibles avec l'identité attestée de ce lancement.
2. Étant donné une désactivation explicite de Bridget ou une permission refusée, quand le lancement est préparé, alors cette décision reste effective. La correction ne force pas l'activation ni n'élargit les permissions.
3. Étant donné un outil T3 ordinaire ou une identité fournie par le navigateur ou le modèle, quand une communication Bridget est demandée, alors ce raccourci ne crée aucune fausse autorité. Seul le montage Bridget attesté est accepté.
4. Étant donné une configuration ou un catalogue invalide, quand la fixture prépare le lancement, alors l'échec est observable et fermé. Aucun secret n'est exposé, aucune communication n'est lancée sous une autre identité.
5. Étant donné le problème signalé, quand son diagnostic est consigné, alors seule une cause vérifiée par code ou configuration est déclarée comme certaine. La cible regional-wrkr-1, provider claude_glm et home .claude-glm, possède une configuration MCP effective sans Bridget vérifiée en lecture seule. Le diagnostic ne prouve pas une réussite après correction.
6. Étant donné cette phase de développement, quand les tests s'exécutent, alors seuls des montages et programmes privés sont utilisés. Aucun modèle, agent actif, configuration de production ou service n'est démarré ou redémarré.

### US147-08 — Ajouter un membre et lui donner accès à l'historique (P1)

Comme utilisateur, je veux qu'un agent puisse rejoindre un fil partagé existant et lire tout son historique autorisé. Je ne veux pas créer un nouveau fil ni perdre le contexte des échanges passés.

Pourquoi P1 : un changement d'équipe doit conserver l'histoire du chantier, sans réveiller inutilement un agent ni lui faire rejouer d'anciennes missions.

Test indépendant : créer un fil privé de fixture avec plusieurs pages de messages et des remplacements. Ajouter un membre via le vrai outil ou la CLI de fil partagé. Vérifier appartenance, accès à tout l'historique, idempotence, permissions et absence d'alerte automatique. La mutation ne passe pas par un nouveau composeur de l'interface.

1. Étant donné un fil ouvert, quand son créateur initial ajoute un ou plusieurs membres via une opération idempotente autorisée, alors les nouvelles appartenances sont ajoutées une seule fois. Un membre déjà présent ne consomme pas de place supplémentaire.
2. Étant donné la limite existante de16 membres, quand l'ajout la dépasserait, alors il est refusé sans mutation. Le même principe vaut pour un fil fermé, un auteur non autorisé ou un contrôle de projet refusé.
3. Étant donné un nouvel agent membre, quand il utilise les lectures read/history existantes, alors il peut consulter tout l'historique autorisé, y compris les pages antérieures à son adhésion et les relations de remplacement ; history conserve les corps historiques exacts. Aucun ancien message ni le sens de read n'est réécrit.
4. Étant donné l'adhésion seule, quand elle est validée, alors aucune alerte ni mission n'est envoyée au nouveau membre. Son curseur initial est0 ; l'accès à l'histoire n'exécute ni ne rejoue les anciennes publications.
5. Étant donné une demande d'alerte ultérieure, quand un auteur publie une consigne, alors les règles de notify restent explicites et ciblées. L'adhésion ne transforme pas une ancienne notification en notification courante.
6. Étant donné un panneau humain autorisé déjà ouvert, quand une nouvelle appartenance est effectivement committée, alors la liste ou le détail pertinents sont revalidés par le suivi humain. Le signal ne transporte aucun corps.
7. Étant donné l'audience existante et les membres proposés, quand l'ajout est préparé, alors les contrôles de projet existants couvrent toute cette audience avant l'écriture. Un refus conserve les données et curseurs précédents.

Périmètre accepté par le principal : créateur initial seul, fil ouvert, maximum16 après normalisation/dédoublonnage, ajout seulement par add_members. Aucun retrait, transfert de propriété ou nouveau composeur UI. Le contrat et les tâches suivent le plan accepté et sa revue ; aucune réussite produit anticipée.

### US147-10 — Conserver le pont lors du passage à T3 V2 (P1)

Extension approuvée le 2026-10-09, ajoutée à147 à la demande de l'utilisateur.
Comme utilisateur, je veux mettre T3 à jour sans perdre les messages interagents,
leurs réponses ni l'identité attestée des agents. Le même Bridget reste utilisable
avec mon ancien T3 pendant la préparation et en cas de retour arrière.

Pourquoi P1 : les anciennes routes HTTP et le stockage des sessions ont disparu
dans T3 V2. Une application compilée ne suffit donc pas à préserver le pont.

Test indépendant : serveurs locaux et bases synthétiques V1/V2, puis API du vrai
T3 isolé sans fournisseur. Aucune donnée réelle modifiée, aucun modèle appelé.

1. Étant donné T3 V1 ou V2, quand le pont découvre les fils, alors il sélectionne
   explicitement le contrat correspondant et conserve racine, fournisseur et état.
2. Étant donné une remise, quand T3 V2 reçoit la commande native, alors les mêmes
   identifiants de commande et de message assurent le rejeu sans doublon. Une panne
   après envoi n'autorise jamais un second envoi par un autre protocole.
3. Étant donné une réponse en cours ou finale, quand le pont lit son historique,
   alors il la rattache au bon message et au bon tour. File, interruption, échec,
   sous-agent et réponse partielle ne deviennent pas une fausse réponse finale.
4. Étant donné un processus fournisseur, quand son identité est vérifiée, alors
   seules les sessions natives attestées du T3 actif sont utilisées. Une ancienne
   base conservée sur disque ne peut pas attester une nouvelle session.
5. Étant donné une forme inconnue, un refus d'accès ou une lecture incomplète,
   quand le pont la reçoit, alors il refuse explicitement sans identité inventée,
   secret dans les diagnostics ni action sur un autre fil.
6. Étant donné l'application en service, quand la version est préparée, alors ni
   T3 ni Bridget ne sont relancés. Installation et activation restent distinctes.

### US147-04 — Garder une lecture confortable (P2)

Comme utilisateur, je veux que les nouveautés n'interrompent pas ma lecture et ne défassent pas mes gestes.

Test indépendant : charger plusieurs pages, déplier un message et ses détails, placer le focus sur un contrôle et sélectionner du texte. Produire un nouveau message et un remplacement. Comparer l'état de lecture avant et après.

1. Étant donné des messages déjà chargés, quand de nouvelles données arrivent, alors les pages utiles restent consultables. Les dépliages et les détails des messages inchangés restent ouverts.
2. Étant donné un contrôle au clavier et une sélection de texte, quand une mise à jour n'affecte pas leurs éléments, alors le focus et le texte sélectionné sont conservés.
3. Étant donné un corps copié, quand la vue se met à jour, alors la copie reste celle du corps original complet. Aucun résumé ou texte technique ne le remplace.
4. Étant donné un message supprimé de la vue après refus d'accès, quand il était déplié ou sélectionné, alors la sécurité prime : son contenu est retiré. Le focus revient à un contrôle accessible du panneau.

### US147-05 — Comprendre qui est sollicité (P2)

Comme utilisateur, je veux voir à qui l'auteur demande une notification. Je veux le comprendre dans la ligne auteur/date, sans ouvrir les détails techniques ni ajouter de hauteur au message.

Pourquoi P2 : l'abonnement et la sélection restent les besoins P1. Cette indication rend les échanges lisibles sans changer leur fonctionnement.

Test indépendant : afficher des messages de test avec notify none, targets et all. Utiliser un ou plusieurs destinataires, dont un sans nom autorisé disponible. Vérifier la ligne existante, le détail exact et la copie du corps.

1. Étant donné notify none, quand le message est affiché, alors la ligne montre « Auteur · Sans sollicitation ». Elle ne laisse pas croire que l'auteur a demandé un réveil.
2. Étant donné notify targets, quand le message est affiché, alors la ligne montre « Auteur → Noms » pour les destinataires effectifs déjà fournis par la lecture autorisée, auteur exclu selon le périmètre approuvé. La date reste dans cette même ligne.
3. Étant donné notify all, quand le message est affiché, alors les noms correspondent aux targets effectifs conservés avec ce message, pas à une nouvelle déduction de tous les membres actuels du fil.
4. Étant donné un destinataire sans nom disponible, quand le message est affiché, alors un libellé honnête « Nom indisponible » avec UUID court reste visible. Le détail permet l'identifiant exact si nécessaire ; aucun nom n'est inventé et aucune nouvelle recherche d'annuaire n'est déclenchée.
5. Étant donné plusieurs destinataires, quand le message est affiché, alors ils restent compréhensibles dans la ligne auteur/date existante. Aucun bloc ou ligne supplémentaire n'est créé pour cette indication.
6. Étant donné cette nouvelle indication, quand je copie le message, alors je reçois exactement son corps original. La flèche indique la sollicitation demandée, jamais une confirmation de livraison ou de lecture.

### US147-09 — Repérer le fil choisi et accéder directement aux messages (P2)

Comme utilisateur, je veux voir clairement quel fil partagé est sélectionné dans la liste du haut. Je veux que la partie du bas commence directement par les messages, sans répéter le titre et les membres du fil. Je veux toujours accéder aux détails du fil quand ils sont utiles.

Pourquoi P2 : la sélection mémorisée et le suivi restent les besoins P1. Cet ajustement réduit les répétitions et utilise la charte native T3, sans changer les échanges ou les autorisations.

Test indépendant : sur un aperçu isolé, sélectionner deux fils, naviguer A→B→A, chercher un autre titre et restaurer un fil hors première page. Vérifier la sélection visible, l'accès clavier aux détails et l'absence d'en-tête répété avant les messages.

1. Étant donné un fil sélectionné dans la liste du haut, quand la vue est affichée, alors sa ligne utilise une mise en valeur de sélection cohérente avec la barre des agents T3. La sélection reste compréhensible sans couleur seule et accessible au clavier.
2. Étant donné un fil choisi mémorisé, quand je change d'agent puis reviens, alors le même choix et son indication visible sont restaurés sans clic supplémentaire.
3. Étant donné un fil choisi hors première page ou exclu par ma recherche, quand ses messages autorisés restent visibles, alors je peux encore identifier le fil sélectionné dans la partie du haut. Aucun autre fil n'est sélectionné automatiquement et aucun en-tête répété n'est réintroduit en bas pour compenser.
4. Étant donné des messages chargés, quand je consulte la partie du bas, alors elle commence par les échanges. Elle ne répète pas un bloc titre/membres/état de fil déjà identifiable en haut ; les états chargement, refus et absence de contenu restent compréhensibles.
5. Étant donné ce bloc supprimé, quand je demande les détails du fil, alors titre, membres et état autorisés restent accessibles par un contrôle identifiable au clavier. Messages, destinataires, copie exacte, dépliage, focus et sélection de texte gardent leurs règles précédentes.

Plan court Web et gate réutilisation natif5/5 validés par le principal avant T051–T054. Aucun nouveau contrat/API/store/composant/helper ni refonte graphique ; tests/recette ouverts.

## Functional Requirements

- FR147-01 : suivre automatiquement les changements pertinents de la liste et du fil sélectionné tant que le panneau est ouvert. Conserver le rafraîchissement manuel.
- FR147-02 : prendre en compte les créations, publications, remplacements et clôtures disponibles. Relire le titre, les membres, leurs noms d'annuaire, l'activité et les messages autorisés. Appliquer le tri récent d'abord du socle146. Le panneau humain n'ajoute aucune commande de renommage, édition de membres ou composeur ; l'ajout CLI/MCP de membres US8 reste permis.
- FR147-03 : utiliser une notification de changement pour déclencher la relecture nécessaire. Ne pas remplacer ce suivi par une relecture périodique de contenu quand rien n'a changé.
- FR147-04 : regrouper les changements rapprochés. Ne conserver qu'une invalidation en attente par contexte de liste et par fil sélectionné. Ne pas multiplier les flux lors des réouvertures ou des navigations rapides.
- FR147-05 : après reconnexion ou reprise d'une vue, revalider les droits et rattraper tout intervalle de changements manqués. Une interruption ne doit pas produire une vue silencieusement incomplète.
- FR147-06 : arrêter le suivi et annuler les lectures devenues inutiles à la fermeture ou au changement de contexte. Rejeter toute réponse issue d'un contexte ou d'une consultation devenus obsolètes.
- FR147-07 : mémoriser le fil choisi séparément pour chaque combinaison d'environnement, de projet et de conversation T3. Restaurer ce choix lors du retour dans cette conversation sans clic supplémentaire.
- FR147-08 : retrouver explicitement un fil mémorisé absent de la première page. Ne pas choisir automatiquement un autre fil à sa place.
- FR147-09 : distinguer une indisponibilité temporaire d'un refus d'accès au fil choisi confirmé. La première conserve le choix, mais suspend l'affichage d'un contenu non revalidé lors d'une reprise. Le second retire le choix et efface les données refusées. Un refus du contexte arrête le suivi et masque les corps sans déduire que le fil a disparu.
- FR147-10 : vérifier l'autorisation avant toute nouvelle lecture et tout abonnement. Révoquer le suivi et purger le contenu lors d'un refus avéré. Un identifiant mémorisé ne constitue jamais une autorisation.
- FR147-11 : limiter les notifications de changement aux références et informations de contrôle nécessaires. Aucun corps de message, prompt, raisonnement ou secret n'y figure.
- FR147-12 : limiter la mémoire de sélection aux références de contexte et de fil. Ne pas persister une copie de la base Bridget ou les corps des messages pour satisfaire ce besoin.
- FR147-13 : conserver les pages déjà chargées utiles et la stabilité de leur parcours lors d'une mise à jour. Fusionner les nouveautés sans doublon et sans rendre un remplacement ancien courant. Réconcilier explicitement un instantané devenu incompatible plutôt que mélanger des pages incohérentes.
- FR147-14 : préserver dépliage, détails, focus et sélection de texte des éléments inchangés. Conserver la copie exacte des corps et les contrôles accessibles. Un retrait de données non autorisées prime sur cette conservation.
- FR147-15 : ne produire aucun message agent, ACK, notification de mission, réveil, appel de modèle, avance de curseur agent ou modification de verdict. Les boucles de relance gardent leur comportement.
- FR147-16 : afficher les états de connexion et de refus sans révéler de contenu sensible. Ne pas modifier largement l'apparence du panneau ni le comportement des surfaces T3 ordinaires.
- FR147-17 : afficher les destinataires sollicités dans la ligne auteur/date existante : « Auteur → Noms » pour targets/all et « Auteur · Sans sollicitation » pour none. Ne pas ajouter de ligne dédiée ni changer le corps ou sa copie.
- FR147-18 : utiliser les targets effectifs déjà présents, auteur exclu, et les noms de detail.members autorisés. Si un nom manque, afficher un fallback explicite avec UUID court et accès au détail exact. Ne pas recalculer all à partir des membres actuels, inventer un nom, ajouter une API/recherche d'annuaire/mutation/réveil, ni présenter la sollicitation comme livraison confirmée. L'exclusion de l'auteur est une règle de présentation approuvée, pas une déduction de livraison.

Les exigences FR147-01 à18, dont FR147-15, décrivent le panneau et son suivi humain. Les nouvelles écritures CLI/MCP interagents sont testées séparément sur des fixtures privées avec des identités attestées. Elles ne sont pas des émissions produites par la consultation humaine. Les corps, missions, verdicts et relances de cette consultation restent inchangés.

- FR147-19 : accepter dans les commandes CLI et l'outil MCP de fils partagés les UUID hyphénés valides de36 caractères en casse majuscule, minuscule ou mixte. Valider puis normaliser leurs seuls champs d'identifiant avant toute clé d'idempotence et tout accès SQL concerné.
- FR147-20 : appliquer cette représentation cohérente aux UUID thread/operation/membres/targets/reçus des actions create/post/read/ack/history/show/close existantes. Un UUID d'agent référencé comme membre ou target est une référence, pas un acteur d'autorité. Conserver les relations et les mêmes résultats idempotents ; ne modifier aucun corps, identifiant opaque de send ou d'autre outil, acteur d'autorité, nom ou préfixe UUID partiel. Refuser les UUID malformés sans mutation et conserver les gardes humaines145/146/147 strictes.
- FR147-21 : monter le vrai serveur MCP Bridget dans les chemins de lancement Claude/GLM réellement identifiés et pris en charge, avec identité attestée et catalogue contenant bridget_thread. Réutiliser le montage existant ; aucun outil t3-code ne reçoit une autorité Bridget inventée.
- FR147-22 : préserver désactivations explicites, permissions et contrôles d'identité/projet. Un montage indisponible ou invalide doit échouer de façon observable et fermée ; aucun repli silencieux sous une autre identité ni secret dans les diagnostics.
- FR147-23 : prouver ces changements par tests CLI/MCP/replay/UUID invalides et fixtures de catalogue/configuration/lancement sans modèle. Ne pas modifier une configuration de production, relancer T3/Bridget ou un agent actif dans cette phase. Ne déclarer la cause du cas exact qu'après observation vérifiable.

- FR147-24 : permettre l'ajout idempotent de membres à un fil partagé existant via CLI et vrai outil MCP. Seul le créateur initial d'un fil ouvert peut ajouter ; maximum16 membres uniques après normalisation, doublons sans place supplémentaire. Aucun retrait ni transfert.
- FR147-25 : vérifier avant toute écriture les droits de l'acteur et les contrôles projet existants sur l'audience complète, anciens membres et candidats. Un refus, un dépassement ou un replay n'altère pas les appartenances, corps, curseurs ou verdicts existants.
- FR147-26 : autoriser explicitement le nouveau membre à lire tout l'historique du fil, sans borne liée à sa date d'adhésion, par read/history paginés et les contrôles existants. Préserver corps et relations de remplacement ; accès à une preuve historique n'est jamais ordre de rejouer une mission.
- FR147-27 : créer le curseur du nouveau membre à0, sans notification automatique à l'adhésion, sans retransmission d'anciens posts ou notifications, sans appel modèle ni mission reprise. Un wakepending préexistant reste inchangé et n'est pas dispatché par l'ajout. Les futurs notify=all incluent le nouveau membre ; les anciens targets effectifs et les kinds existants ne sont pas recalculés.
- FR147-28 : signaler aux consultations humaines autorisées une appartenance réellement ajoutée seulement après commit, afin de revalider liste/détail/membres. Pas de signal pour refus, replay ou doublon sans changement ; pas de corps dans le flux. La mutation d'appartenance reste un outil interagents, pas une fonction de composition UI.

- FR147-29 : mettre en valeur la ligne du fil choisi dans la liste supérieure avec les états de sélection natifs T3. Préserver lisibilité, navigation clavier et indication accessible ; aucune refonte, couleur de marque nouvelle ou sélection différente de la mémoire US2.
- FR147-30 : retirer le bloc répété titre/membres/état avant les messages de la partie inférieure. Le fil choisi reste identifiable en haut même s'il est hors première page ou masqué par la recherche, sans réassignation automatique ni nouveau header en bas.
- FR147-31 : garder les détails autorisés du fil accessibles par un contrôle visible/identifiable au clavier après retrait du bloc inférieur. Ne pas ajouter API/store ni modifier messages, copie, autorisations, suivi ou règles de silence ; conserver les états chargement/refus/absence.

- FR147-32 : détecter V1/V2 par une lecture authentifiée, sans repli sur une panne
  ou un refus. Réutiliser les structures internes du pont derrière l'adaptateur.
- FR147-33 : lire les messages/runs V2 et envoyer message.dispatch par le RPC
  officiel ; conserver idempotence, corrélation et permissions du fil.
- FR147-34 : attester les identifiants natifs depuis la base de la version active,
  en lecture seule et avec les contrôles PID/naissance/lignée existants.
- FR147-35 : valider V1/V2, erreurs, file, rejeu, identité et intégration réelle
  isolée sans modèle ; construire sans installation ni redémarrage automatique.

## Success Criteria

- SC147-21 : les mêmes scénarios de remise et de réponse passent sur V1 et V2 ;
  zéro double remise après reprise et zéro réponse attribuée au mauvais tour.
- SC147-22 : une base V1 résiduelle, une session étrangère ou un état inconnu ne
  donnent aucune identité ; les tests d'autorité existants restent verts.
- SC147-23 : la recette utilise un vrai transport T3 isolé avec données synthétiques,
  sans appel de modèle ni changement des données et processus de production.

- SC147-01 : dans une connexion saine, chaque scénario de création, de publication, de remplacement, de clôture ou de changement de nom d'annuaire devient visible dans le panneau ouvert sous deux secondes après sa validation.
- SC147-02 : pendant 60 secondes sans changement, après la lecture initiale, le nombre de relectures de contenu est zéro. Une rafale de 100 changements produit au plus une relecture en cours et une relecture complémentaire en attente par contexte concerné ; l'état final reste exact.
- SC147-03 : le parcours A → B → A → B restitue les deux sélections sans clic supplémentaire. Le cas où le fil choisi se situe au-delà de la première page donne le même résultat.
- SC147-04 : les scénarios multi-projets et multi-environnements montrent zéro sélection ou message provenant d'un autre contexte. Les réponses tardives, y compris A → B → A, ne changent pas le résultat actuel.
- SC147-05 : après une interruption et des changements manqués, la reprise restitue chaque message autorisé une fois. Un accès au fil choisi révoqué montre zéro corps ancien après le refus confirmé et retire la sélection concernée. Un refus du contexte masque les corps et arrête le suivi, sans retirer automatiquement le UUID du fil.
- SC147-06 : dix ouvertures, fermetures ou changements successifs ne font pas croître le nombre de suivis actifs. Après fermeture, zéro suivi propre au panneau et zéro relecture automatique de contenu restent actifs.
- SC147-07 : les tests avec prompts et secrets reconnaissables montrent zéro contenu de message dans les notifications de changement ou leurs diagnostics. La mémoire durable de sélection ne contient que des références.
- SC147-08 : une recette isolée conserve les messages des pages déjà chargées, les dépliages, les détails, le focus, une sélection de texte et la copie originale après l'arrivée d'une nouveauté. Les messages remplacés ne retrouvent jamais le statut courant à cause d'une réponse tardive.
- SC147-09 : pour l'ouverture, la navigation, le suivi, la reprise et la fermeture du panneau, le nombre d'émissions agent, d'ACK, de réveils et d'appels de modèle est zéro. Les données métier, verdicts et curseurs des agents restent identiques.
- SC147-10 : les cas none, targets, all, nom absent et multidestinations affichent les seuls destinataires effectifs attendus, ou Sans sollicitation, dans la même ligne auteur/date. Aucun appel supplémentaire n'est produit ; la copie reste strictement égale au corps original et aucun libellé ne promet une livraison.

- SC147-11 : les matrices des commandes CLI et de l'outil MCP de fils partagés majuscule/minuscule/mixte des UUID hyphénés valides donnent les mêmes références canoniques et résultats autorisés. Les matrices de formats invalides donnent zéro mutation.
- SC147-12 : une requête reprise avec casse différente conserve un seul effet métier, les mêmes membres/targets et les mêmes reçus idempotents. Les corps restent identiques octet par octet ; les tests de refus humains145/146/147 restent verts.
- SC147-13 : chaque chemin réel Claude/GLM retenu dans le plan possède une fixture de lancement qui expose le vrai catalogue Bridget, dont bridget_thread, sous l'identité attendue. Les configurations opt-out, refusées ou fausses identités ne donnent aucune autorité supplémentaire.
- SC147-14 : les tests de configuration/catalogue/montage s'exécutent avec zéro appel de modèle, zéro restart, zéro mutation de configuration de production et zéro agent actif modifié. Le diagnostic exact sépare constat, hypothèse et information encore manquante.

- SC147-15 : les tests d'ajout montrent une seule nouvelle appartenance pour un replay, aucune place supplémentaire pour un doublon et zéro mutation pour acteur refusé, fil fermé ou dépassement16. Aucun retrait ou transfert disponible.
- SC147-16 : un membre ajouté à un fil de fixture de plus d'une page peut lire l'ensemble des séquences antérieures et postérieures autorisées, une fois chacune, avec leurs relations de remplacement et corps exacts. Son ajout ne rejoue aucun ancien post ou mission.
- SC147-17 : l'adhésion seule produit zéro alerte agent, réveil, appel modèle ou ancien post retransmis. Le curseur du nouveau membre vaut0 ; le suivi humain reçoit seulement l'invalidation autorisée du changement réel.
- SC147-18 : les refus de projet testés sur anciens membres et candidats surviennent avant mutation ; aucun contenu ou sélection étrangère n'est accordé. Le panneau ouvert revalide les appartenances committées sans changer les règles notify des messages anciens.

- SC147-19 : sélection A/B, retour A→B→A, fil mémorisé hors première page et recherche excluant le choix montrent tous le fil sélectionné correctement en haut, sans clic de resélection ni choix automatique ; la ligne native et son état accessible sont contrôlés dans les tests et l'aperçu isolé.
- SC147-20 : la vue chargée ne montre aucun bloc titre/membres/état répété avant les échanges ; les mêmes détails restent consultables au clavier. Les tests copie/dépliage/focus et états loading/refused/empty restent verts, avec zéro appel/modification d'agent ou nouveau contrat/API/store.

## Key Entities

- **Contexte de consultation** : environnement, projet et conversation T3 auxquels la lecture humaine est liée.
- **Choix mémorisé** : référence du fil partagé sélectionné dans un contexte. Il n'accorde aucun droit d'accès.
- **Fil partagé** : titre, membres, activité récente et état de clôture déjà gérés par Bridget.
- **Notification de changement** : signal limité aux références utiles pour revalider la vue. Il ne transporte pas le corps des échanges.
- **État de lecture** : pages autorisées déjà chargées et gestes de lecture encore valides. Il est distinct des missions et curseurs des agents.
- **Reprise** : revalidation de l'accès et rattrapage après une interruption ou un retour dans la conversation.
- **Sollicitation affichée** : mode notify et targets effectifs déjà lus, associés aux noms autorisés disponibles. Cette information exprime une demande, pas un reçu de livraison.

- **UUID canonique d'action de fil partagé** : identifiant déclaré validé puis normalisé ; il ne modifie pas le corps et ne constitue pas une autorisation.
- **Montage MCP attesté** : configuration du vrai serveur Bridget liée au contexte de lancement vérifié, distincte des outils T3 ordinaires.

- **Adhésion ajoutée** : appartenance accordant l'accès à l'historique autorisé, sans mission, notification ou rejeu automatique. Les anciennes séquences et targets restent inchangés.

## Assumptions

Le panneau déjà ouvert reste ouvert lors de la navigation T3. La session147 ajoute la conservation du fil choisi ; elle ne refait pas ce comportement existant.

La mémoire de choix couvre au minimum les changements de conversation au sein de la session T3. Une conservation après redémarrage peut réutiliser le stockage de références existant, sans ajout de corps de message. Elle ne conditionne pas la recette principale demandée par l'utilisateur.

Un fil fermé peut rester consultable et sélectionné si ses droits le permettent. Clôture n'est pas synonyme de suppression ou de refus.

Le suivi humain est propre au panneau. Il n'utilise pas les abonnements qui sollicitent les agents et ne modifie pas les relances de leurs missions.

Le socle offre création, publication, clôture et remplacement d'une consigne. Il ne possède pas encore de commande de renommage ou de modification des membres. Le premier lot humain US1–US5 ne les ajoute pas ; l'extension US8 autorise désormais l'ajout seul via CLI/MCP, pas le renommage, retrait, transfert ou composeur UI. Les tests de révocation humaine utilisent un retrait de rattachement ou un refus simulé ; ils valident les gardes, pas une fonction de retrait de membre.

Les deux secondes sont mesurées en connexion saine, sans coupure ni reprise en cours. La reprise expose son indisponibilité puis converge vers l'état autorisé ; elle ne prétend pas conserver ce délai pendant une panne.

Les détails du transport, de la pagination et du stockage de références sont fixés dans le plan après examen de l'existant et validés sur les preuves147. Aucun nouveau framework, aucune nouvelle interface de composition de message et aucun changement graphique large ne sont requis.

Les ajouts US6/US7/US8 sont approuvés dans147, pas dans une nouvelle session148. Les34 tâches US1–US6 sont prouvées et cochées. Les tâches US7 T035–T041 suivent leur gate de conception validé ; US8 suit son plan accepté et sa revue détaillée. Les résultats d'audit existants couvrent le premier lot uniquement ; ils ne qualifient pas encore les extensions.
