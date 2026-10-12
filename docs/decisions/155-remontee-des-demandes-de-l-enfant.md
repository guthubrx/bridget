# Décision155 — demandes de l’enfant et décision dans les droits hérités

Date : 2026-10-12. Statut : **Accepté**.

Cette décision fixe l’architecture de la session155. Son acceptation ne prouve
pas que le code, les tests ou le runtime du parcours155 sont terminés.

## Contexte

Un enfant natif hérite des droits attestés de son parent. La session149 permet
cet héritage sans nouveau grant humain Bridget et sans dialogue de validation.
FR008 de cette session interdit ce dialogue. L’ADR149 précise aussi que les
permissions non couvertes restent refusées sans dialogue humain Bridget.

Le fournisseur peut pourtant poser une question ou demander une approbation
pendant le travail. Une réponse automatique ne permet ni au parent de fournir
une information, ni à l’humain de décider d’une commande dans ses droits.
La demande doit rester liée à la tâche, à l’instance, à la session et au tour.

Le contrat T002 repose sur les schémas primaires Codex app-server 0.162.1.
Ils distinguent les formes de réponse et leurs limites. Ils ne prouvent pas
l’exécution runtime des variantes. Les effets non établis doivent rester refusés.

## Décision

### Changement limité de149

Cette ADR modifie explicitement la clause « sans dialogue de validation de
permission » de FR008 et la règle de dialogue de l’ADR149. Pour une tâche
qui choisit forward, une approbation de commande admissible peut attendre
une décision humaine. Cette décision porte sur une opération courante.
Elle ne crée aucun grant par mission, permission supplémentaire ou droit durable.

Les autres garanties149 restent valables : héritage attesté et figé, réduction
par la posture de l’enfant, absence de gain de droits, révocations et choix
humains prioritaires. Une permission hors politique reste refusée. La limite
absolue s’applique à l’humain autant qu’au parent.

child_requests est une option par tâche. Son défaut est deny. forward est
réservé au fournisseur Codex app-server pris en charge. Le mode est figé dans
la tâche et appartient à son enveloppe de rejeu. La politique du fournisseur
reste inchangée : approval_policy=never ne devient pas on-request et ne crée
aucune route humaine pour une demande que le fournisseur ne produit pas.
Le repli fermé indépendant T000 reste la seule exception explicite au
comportement historique deny.

### Une route par demande

| Demande | Décideur et limite |
|---|---|
| item/tool/requestUserInput complète, corrélée, isSecret=false (défaut normatif) | Parent propriétaire attesté, vivant et non révoqué ; réponse d’information seulement |
| item/commandExecution/requestApproval, kind=command, effets établis et décision explicitement offerte | Humain attesté, dans les droits figés du parent et de l’enfant ; accept pour la commande courante seulement |
| fileChange | Refus v1 approval_method_unsupported ; grantRoot non nul donne duration_unsupported |
| permissions, writeStdin, méthode ou effet inconnu | Refus fermé ; aucune réponse ni permission inventée |
| Tout formulaire MCP tiers, même typé, complet, booléen ou enum | Refus v1 approval_method_unsupported : aucune preuve structurée de non-secret ou de non-approbation |

Le parent ne peut jamais approuver une action. L’humain ne répond pas aux
questions du parent. Aucun heuristique textuel ou marqueur _meta inventé ne
rend un formulaire MCP tiers admissible. L’exception historique du formulaire
vide Bridget reste distincte ; elle ne s’étend pas à un serveur tiers.
Les modes URL restent refusés.

Une question monte d’un seul niveau, vers le propriétaire de la tâche enfant.
Elle n’est pas transférée au parent racine. Un agent ne répond pas à sa propre
demande. Le plafond d’un petit-enfant dépend de son propre parent.

La commande exige des faits complets et les droits effectifs nécessaires.
Un enfant en lecture seule reste refusé. Une politique incomplète ou inconnue
reste refusée. Les commandes non bornées exigent le plein accès attesté du
parent ; l’accord humain ne remplace aucune de ces gardes.

fileChange est fermée en amont dans le transport réel v1. Ses Params ne
portent ni paths ni availableDecisions. Les notifications changes/path/diff/kind
restent des faits cités et une piste future, sans cache de corrélation neuf.
Le classifier pur peut classer Human des faits synthétiques complets once/plein
accès ; cela ne rend pas la méthode fournisseur réelle supportée. Les chemins
ne prouvent jamais un confinement et ne sont pas résolus dans le filesystem.

### Réponse, remise et fermeture

Bridget réutilise la boîte humaine intervention_required, les lectures par
identifiant exact, la garde de pause, le routage et les transitions d’exécution.
Les faits structurés viennent avant le texte enfant, cité, borné et non fiable.
La décision historique reste distincte de sa remise : delivery_state vaut
none, pending, sent, delivered, void ou unknown.

La décision exige une demande open, une échéance encore valide et un contrôle
conditionnel du nombre de lignes modifiées. Le claim de remise exige pending
et la validité de FR027. Le daemon écrit sent avant l’envoi. Chaque renvoi
recontrôle la tâche, l’instance, la pause, la révocation et l’échéance.
Expiration et fermeture sont traitées avant toute redelivery.

Le wrapper sérialise réponse et fermeture sous un même verrou. Toute fermeture
non confirmée utilise ChildRequestCancel puis ChildRequestClosed, même pour
none ou pending. closed_before_apply interdit un accord ultérieur ; already_applied
conserve le fait qu’une réponse a déjà été appliquée. void signifie accord
invalidé, pas fermeture fournisseur prouvée. Sans accusé fiable, l’issue reste
unknown ; elle n’est pas annoncée comme une annulation confirmée.
Un replay terminal rend la réponse persistée sans rouvrir une demande ni
réappliquer un accord invalidé.

Les réponses suivent la forme de chaque méthode. user_input adapte le mapping
interne answer[qid] vers answers[qid].answers. Son schéma ne définit aucun
decline/cancel natif : refus et expiration utilisent l’erreur JSON-RPC fermée.
Une erreur JSON-RPC seule ne prouve pas une fermeture runtime du fournisseur.

Pendant une pause, Respond est refusé. Une décision humaine reçue pendant la
pause est acquittée sans exécution ni accord différé. Une reprise réussie porte
sur une nouvelle demande, jamais sur l’accord invalidé.

### Attente et bornes

L’attente151 reste fondée sur l’état, sans mémoire persistante de waiter ni
consommation. ready contient uniquement les tâches terminales. pending garde
les identifiants nonterminaux. pending_requests, à la racine du résultat wait,
contient les questions Parent ouvertes, non expirées et actionnables avec task_id.
Human reste visible dans status mais ne déclenche pas le retour wait.
Une question Parent déjà ouverte peut rendre immédiatement la main.
timed_out est faux si ready ou pending_requests est non vide ; il est vrai
seulement à l’échéance wait avec ces deux listes vides. WaitEpoch réutilise
la notification après commit et la lecture du compteur avant l’état.

Les budgets et fenêtres restent ceux de la spec155 : quatre demandes ouvertes
par enfant, 32 au total par tâche ; 600 s pour le parent, 3600 s pour l’humain,
plafonnées par le délai de mission moins 30 s. Les lignes de demandes sont
supprimées avec leur arbre dans la purge150. Aucun nouveau service ou moteur
d’attente, helper de sélection, framework de droits ou API supplémentaire au périmètre155.

## Alternatives rejetées

- Défaut forward : un parent qui ne sait pas répondre ferait attendre son enfant.
- Approbation par le parent : un texte enfant peut induire un modèle en erreur ; ce parent ne peut pas créer de droits supplémentaires.
- Escalade automatique : elle contournerait la route unique et la limite du propriétaire direct.
- Décisions durables ou amendements : leur portée dépasse l’opération courante autorisée.
- Relève humaine par balayage : des décisions anciennes peuvent masquer un item utile ; les lectures exactes existantes sont réutilisées.
- Inclusion des enfants Claude/GLM et ACP : elle rouvrirait leurs parcours fournisseur et leur lancement ; ils restent hors v1.

## Conséquences

Le parent peut fournir une information sans approuver une opération. L’humain
peut décider d’une commande admissible sans élargir les droits hérités.
Le choix deny préserve les tâches qui n’attendent aucune réponse externe.

La décision ajoute une attente corrélée et une trace de remise. Les courses,
replays, pauses et pertes de connexion doivent être vérifiés par les tâches
et scénarios déjà prévus. Les refus fileChange, permissions et MCP tiers
limitent le service v1. Ils ne déclarent pas ces fournisseurs définitivement
incapables de fournir une preuve dans une évolution future.

Cette ADR ne coche aucune tâche. Elle ne vaut ni validation des parcours155,
ni installation, activation, livraison ou recette utilisateur.

## Références lues

- ADR149 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/docs/decisions/149-permissions-heritees-et-lineage-native.md — section Décision, héritage et refus sans dialogue humain.
- SPEC149 FR008 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/149-sous-agents-lineage/spec.md — ligne164 : héritage sans nouveau grant ni dialogue.
- T036 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/tasks.md — ligne96.
- Contrat T002 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/contracts/codex-requests.md.
- SPEC155 : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/spec.md.

[ADR149 — droits hérités](/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/docs/decisions/149-permissions-heritees-et-lineage-native.md)
