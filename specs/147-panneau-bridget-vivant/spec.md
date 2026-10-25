# SPEC147 — Panneau Bridget vivant et sélection mémorisée

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-1).

## Besoin et périmètre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-2).

L'utilisateur a approuvé ces deux besoins dans la session147. La session146 reste le socle. Le panneau conserve son apparence, son ordre récent d'abord, sa pagination, sa copie exacte et ses contrôles de dépliage.

Ajout utilisateur approuvé à la même session : montrer les destinataires sollicités dans la ligne auteur/date existante. Cet ajout de lisibilité ne change aucun message, destinataire effectif, notification ou droit.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-3).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-4).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-5).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-6).

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

### US147-10 — Conserver le pont lors du passage à adaptateur de bureau V2 (P1)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-7).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-8).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-9).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-10).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-11).

Test indépendant : sur un aperçu isolé, sélectionner deux fils, naviguer A→B→A, chercher un autre titre et restaurer un fil hors première page. Vérifier la sélection visible, l'accès clavier aux détails et l'absence d'en-tête répété avant les messages.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-12).

Plan court Web et gate réutilisation natif5/5 validés par le principal avant T051–T054. Aucun nouveau contrat/API/store/composant/helper ni refonte graphique ; tests/recette ouverts.

## Functional Requirements

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-13).

Les exigences FR147-01 à18, dont FR147-15, décrivent le panneau et son suivi humain. Les nouvelles écritures CLI/MCP interagents sont testées séparément sur des fixtures privées avec des identités attestées. Elles ne sont pas des émissions produites par la consultation humaine. Les corps, missions, verdicts et relances de cette consultation restent inchangés.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-14).

- FR147-24 : permettre l'ajout idempotent de membres à un fil partagé existant via CLI et vrai outil MCP. Seul le créateur initial d'un fil ouvert peut ajouter ; maximum16 membres uniques après normalisation, doublons sans place supplémentaire. Aucun retrait ni transfert.
- FR147-25 : vérifier avant toute écriture les droits de l'acteur et les contrôles projet existants sur l'audience complète, anciens membres et candidats. Un refus, un dépassement ou un replay n'altère pas les appartenances, corps, curseurs ou verdicts existants.
- FR147-26 : autoriser explicitement le nouveau membre à lire tout l'historique du fil, sans borne liée à sa date d'adhésion, par read/history paginés et les contrôles existants. Préserver corps et relations de remplacement ; accès à une preuve historique n'est jamais ordre de rejouer une mission.
- FR147-27 : créer le curseur du nouveau membre à0, sans notification automatique à l'adhésion, sans retransmission d'anciens posts ou notifications, sans appel modèle ni mission reprise. Un wakepending préexistant reste inchangé et n'est pas dispatché par l'ajout. Les futurs notify=all incluent le nouveau membre ; les anciens targets effectifs et les kinds existants ne sont pas recalculés.
- FR147-28 : signaler aux consultations humaines autorisées une appartenance réellement ajoutée seulement après commit, afin de revalider liste/détail/membres. Pas de signal pour refus, replay ou doublon sans changement ; pas de corps dans le flux. La mutation d'appartenance reste un outil interagents, pas une fonction de composition UI.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-15).

- FR147-32 : détecter V1/V2 par une lecture authentifiée, sans repli sur une panne
  ou un refus. Réutiliser les structures internes du pont derrière l'adaptateur.
- FR147-33 : lire les messages/runs V2 et envoyer message.dispatch par le RPC
  officiel ; conserver idempotence, corrélation et permissions du fil.
- FR147-34 : attester les identifiants natifs depuis la base de la version active,
  en lecture seule et avec les contrôles PID/naissance/lignée existants.
- FR147-35 : valider V1/V2, erreurs, file, rejeu, identité et intégration réelle
  isolée sans modèle ; construire sans installation ni redémarrage automatique.

## Success Criteria

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-16).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-17).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-18).

- **Adhésion ajoutée** : appartenance accordant l'accès à l'historique autorisé, sans mission, notification ou rejeu automatique. Les anciennes séquences et targets restent inchangés.

## Assumptions

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-19).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/spec.md#extrait-20).

Un fil fermé peut rester consultable et sélectionné si ses droits le permettent. Clôture n'est pas synonyme de suppression ou de refus.

Le suivi humain est propre au panneau. Il n'utilise pas les abonnements qui sollicitent les agents et ne modifie pas les relances de leurs missions.

Le socle offre création, publication, clôture et remplacement d'une consigne. Il ne possède pas encore de commande de renommage ou de modification des membres. Le premier lot humain US1–US5 ne les ajoute pas ; l'extension US8 autorise désormais l'ajout seul via CLI/MCP, pas le renommage, retrait, transfert ou composeur UI. Les tests de révocation humaine utilisent un retrait de rattachement ou un refus simulé ; ils valident les gardes, pas une fonction de retrait de membre.

Les deux secondes sont mesurées en connexion saine, sans coupure ni reprise en cours. La reprise expose son indisponibilité puis converge vers l'état autorisé ; elle ne prétend pas conserver ce délai pendant une panne.

Les détails du transport, de la pagination et du stockage de références sont fixés dans le plan après examen de l'existant et validés sur les preuves147. Aucun nouveau framework, aucune nouvelle interface de composition de message et aucun changement graphique large ne sont requis.

Les ajouts US6/US7/US8 sont approuvés dans147, pas dans une nouvelle session148. Les34 tâches US1–US6 sont prouvées et cochées. Les tâches US7 T035–T041 suivent leur gate de conception validé ; US8 suit son plan accepté et sa revue détaillée. Les résultats d'audit existants couvrent le premier lot uniquement ; ils ne qualifient pas encore les extensions.
