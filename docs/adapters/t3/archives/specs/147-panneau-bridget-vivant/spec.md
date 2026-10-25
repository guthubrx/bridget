# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/spec.md](../../../../../../specs/147-panneau-bridget-vivant/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-08 ; extensions approuvées le 2026-10-09. Développement : Implemented, Validé,54/54 tâches,31FR/20SC. Sources committées/fusionnées/poussées. Livraison EN COURS : Bridget installé et actif, T3 compilé/signé prêt mais activation encore en attente. Branche : `session-147-panneau-bridget-vivant`. Ne pas confondre package T3 vérifié et application active.

<a id="extrait-2"></a>

## Extrait 2

L'utilisateur veut deux améliorations du panneau Bridget. Les fils partagés et les messages doivent suivre leurs changements sans clic sur le bouton de rafraîchissement. Le fil choisi dans une conversation T3 doit rester sélectionné quand l'utilisateur change d'agent, puis revient.

<a id="extrait-3"></a>

## Extrait 3

Ajout approuvé à147 : compatibilité des UUID dans les commandes et l'outil CLI/MCP de fils partagés, et des outils Bridget dans les lancements Claude/GLM attestés. Le plan de montage MCP concret doit être validé avant de créer ses nouvelles tâches. La cause locale est vérifiée : configuration MCP effective sans Bridget et injection T3 limitée à t3-code pour regional-wrkr-1/claude_glm. Aucune exclusion volontaire GLM déduite.

<a id="extrait-4"></a>

## Extrait 4

Comme utilisateur, je veux retrouver le fil partagé que j'avais choisi dans chaque conversation T3. Je ne veux pas le sélectionner à nouveau après chaque changement d'agent.

<a id="extrait-5"></a>

## Extrait 5

Test indépendant : choisir un fil A dans une conversation T3 A. Choisir un autre fil B dans une conversation T3 B. Revenir sur A, puis sur B. Répéter dans deux projets et deux environnements portant des identifiants de conversation identiques dans les données de test.

<a id="extrait-6"></a>

## Extrait 6

1. Étant donné un lancement Claude/GLM pris en charge avec Bridget activé, quand le montage MCP est construit, alors le serveur Bridget et le catalogue attendu, dont bridget_thread, sont disponibles avec l'identité attestée de ce lancement.
2. Étant donné une désactivation explicite de Bridget ou une permission refusée, quand le lancement est préparé, alors cette décision reste effective. La correction ne force pas l'activation ni n'élargit les permissions.
3. Étant donné un outil T3 ordinaire ou une identité fournie par le navigateur ou le modèle, quand une communication Bridget est demandée, alors ce raccourci ne crée aucune fausse autorité. Seul le montage Bridget attesté est accepté.
4. Étant donné une configuration ou un catalogue invalide, quand la fixture prépare le lancement, alors l'échec est observable et fermé. Aucun secret n'est exposé, aucune communication n'est lancée sous une autre identité.
5. Étant donné le problème signalé, quand son diagnostic est consigné, alors seule une cause vérifiée par code ou configuration est déclarée comme certaine. La cible regional-wrkr-1, provider claude_glm et home .claude-glm, possède une configuration MCP effective sans Bridget vérifiée en lecture seule. Le diagnostic ne prouve pas une réussite après correction.
6. Étant donné cette phase de développement, quand les tests s'exécutent, alors seuls des montages et programmes privés sont utilisés. Aucun modèle, agent actif, configuration de production ou service n'est démarré ou redémarré.

<a id="extrait-7"></a>

## Extrait 7

Extension approuvée le 2026-10-09, ajoutée à147 à la demande de l'utilisateur.
Comme utilisateur, je veux mettre T3 à jour sans perdre les messages interagents,
leurs réponses ni l'identité attestée des agents. Le même Bridget reste utilisable
avec mon ancien T3 pendant la préparation et en cas de retour arrière.

<a id="extrait-8"></a>

## Extrait 8

Pourquoi P1 : les anciennes routes HTTP et le stockage des sessions ont disparu
dans T3 V2. Une application compilée ne suffit donc pas à préserver le pont.

<a id="extrait-9"></a>

## Extrait 9

Test indépendant : serveurs locaux et bases synthétiques V1/V2, puis API du vrai
T3 isolé sans fournisseur. Aucune donnée réelle modifiée, aucun modèle appelé.

<a id="extrait-10"></a>

## Extrait 10

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

<a id="extrait-11"></a>

## Extrait 11

Pourquoi P2 : la sélection mémorisée et le suivi restent les besoins P1. Cet ajustement réduit les répétitions et utilise la charte native T3, sans changer les échanges ou les autorisations.

<a id="extrait-12"></a>

## Extrait 12

1. Étant donné un fil sélectionné dans la liste du haut, quand la vue est affichée, alors sa ligne utilise une mise en valeur de sélection cohérente avec la barre des agents T3. La sélection reste compréhensible sans couleur seule et accessible au clavier.
2. Étant donné un fil choisi mémorisé, quand je change d'agent puis reviens, alors le même choix et son indication visible sont restaurés sans clic supplémentaire.
3. Étant donné un fil choisi hors première page ou exclu par ma recherche, quand ses messages autorisés restent visibles, alors je peux encore identifier le fil sélectionné dans la partie du haut. Aucun autre fil n'est sélectionné automatiquement et aucun en-tête répété n'est réintroduit en bas pour compenser.
4. Étant donné des messages chargés, quand je consulte la partie du bas, alors elle commence par les échanges. Elle ne répète pas un bloc titre/membres/état de fil déjà identifiable en haut ; les états chargement, refus et absence de contenu restent compréhensibles.
5. Étant donné ce bloc supprimé, quand je demande les détails du fil, alors titre, membres et état autorisés restent accessibles par un contrôle identifiable au clavier. Messages, destinataires, copie exacte, dépliage, focus et sélection de texte gardent leurs règles précédentes.

<a id="extrait-13"></a>

## Extrait 13

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

<a id="extrait-14"></a>

## Extrait 14

- FR147-19 : accepter dans les commandes CLI et l'outil MCP de fils partagés les UUID hyphénés valides de36 caractères en casse majuscule, minuscule ou mixte. Valider puis normaliser leurs seuls champs d'identifiant avant toute clé d'idempotence et tout accès SQL concerné.
- FR147-20 : appliquer cette représentation cohérente aux UUID thread/operation/membres/targets/reçus des actions create/post/read/ack/history/show/close existantes. Un UUID d'agent référencé comme membre ou target est une référence, pas un acteur d'autorité. Conserver les relations et les mêmes résultats idempotents ; ne modifier aucun corps, identifiant opaque de send ou d'autre outil, acteur d'autorité, nom ou préfixe UUID partiel. Refuser les UUID malformés sans mutation et conserver les gardes humaines145/146/147 strictes.
- FR147-21 : monter le vrai serveur MCP Bridget dans les chemins de lancement Claude/GLM réellement identifiés et pris en charge, avec identité attestée et catalogue contenant bridget_thread. Réutiliser le montage existant ; aucun outil t3-code ne reçoit une autorité Bridget inventée.
- FR147-22 : préserver désactivations explicites, permissions et contrôles d'identité/projet. Un montage indisponible ou invalide doit échouer de façon observable et fermée ; aucun repli silencieux sous une autre identité ni secret dans les diagnostics.
- FR147-23 : prouver ces changements par tests CLI/MCP/replay/UUID invalides et fixtures de catalogue/configuration/lancement sans modèle. Ne pas modifier une configuration de production, relancer T3/Bridget ou un agent actif dans cette phase. Ne déclarer la cause du cas exact qu'après observation vérifiable.

<a id="extrait-15"></a>

## Extrait 15

- FR147-29 : mettre en valeur la ligne du fil choisi dans la liste supérieure avec les états de sélection natifs T3. Préserver lisibilité, navigation clavier et indication accessible ; aucune refonte, couleur de marque nouvelle ou sélection différente de la mémoire US2.
- FR147-30 : retirer le bloc répété titre/membres/état avant les messages de la partie inférieure. Le fil choisi reste identifiable en haut même s'il est hors première page ou masqué par la recherche, sans réassignation automatique ni nouveau header en bas.
- FR147-31 : garder les détails autorisés du fil accessibles par un contrôle visible/identifiable au clavier après retrait du bloc inférieur. Ne pas ajouter API/store ni modifier messages, copie, autorisations, suivi ou règles de silence ; conserver les états chargement/refus/absence.

<a id="extrait-16"></a>

## Extrait 16

- SC147-21 : les mêmes scénarios de remise et de réponse passent sur V1 et V2 ;
  zéro double remise après reprise et zéro réponse attribuée au mauvais tour.
- SC147-22 : une base V1 résiduelle, une session étrangère ou un état inconnu ne
  donnent aucune identité ; les tests d'autorité existants restent verts.
- SC147-23 : la recette utilise un vrai transport T3 isolé avec données synthétiques,
  sans appel de modèle ni changement des données et processus de production.

<a id="extrait-17"></a>

## Extrait 17

- **Contexte de consultation** : environnement, projet et conversation T3 auxquels la lecture humaine est liée.
- **Choix mémorisé** : référence du fil partagé sélectionné dans un contexte. Il n'accorde aucun droit d'accès.
- **Fil partagé** : titre, membres, activité récente et état de clôture déjà gérés par Bridget.
- **Notification de changement** : signal limité aux références utiles pour revalider la vue. Il ne transporte pas le corps des échanges.
- **État de lecture** : pages autorisées déjà chargées et gestes de lecture encore valides. Il est distinct des missions et curseurs des agents.
- **Reprise** : revalidation de l'accès et rattrapage après une interruption ou un retour dans la conversation.
- **Sollicitation affichée** : mode notify et targets effectifs déjà lus, associés aux noms autorisés disponibles. Cette information exprime une demande, pas un reçu de livraison.

<a id="extrait-18"></a>

## Extrait 18

- **UUID canonique d'action de fil partagé** : identifiant déclaré validé puis normalisé ; il ne modifie pas le corps et ne constitue pas une autorisation.
- **Montage MCP attesté** : configuration du vrai serveur Bridget liée au contexte de lancement vérifié, distincte des outils T3 ordinaires.

<a id="extrait-19"></a>

## Extrait 19

Le panneau déjà ouvert reste ouvert lors de la navigation T3. La session147 ajoute la conservation du fil choisi ; elle ne refait pas ce comportement existant.

<a id="extrait-20"></a>

## Extrait 20

La mémoire de choix couvre au minimum les changements de conversation au sein de la session T3. Une conservation après redémarrage peut réutiliser le stockage de références existant, sans ajout de corps de message. Elle ne conditionne pas la recette principale demandée par l'utilisateur.
