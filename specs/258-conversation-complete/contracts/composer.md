# Contrat de raccord 258

Étendre HumanLineage par un parcours de boîte humaine. Le nom exact des types
reste natif ; aucune écriture dans les projections T3 ni commande de fournisseur
T3 ne peut remplacer ce parcours.

Contexte obligatoire : conversation racine attestée, projet, task_id, ronde
attendue. Mutation : request_id UUID conservé au rejeu, opération fermée et
charge bornée. Entrée CLI par JSON stdin, sans texte libre dans les arguments.

Opérations : lecture de capacités/sélection/catalogue de la cible ; intervention
textuelle normale ou guidage en cours ; sélection modèle/effort ; interruption
du tour. Le bouton d'annulation de mission garde son parcours séparé existant.
Une tâche libérée ne se relance jamais par ces opérations.

Réponse : identité/ronde, capacités attestées, sélection constatée et choix
disponibles ; pour une mutation, effet confirmé, refus précis ou issue inconnue.
Le transport T3 décode strictement et vérifie la tâche/ronde de la réponse.
Un protocole ancien refuse les mutations, sans remplacement par un envoi libre.

Les reçus empêchent un double envoi ou double interruption. Un changement de
ronde/instance invalide les effets anciens. Les contrôles avant et après E/S
préservent droits, propriétaire, révocations et périmètre. Aucun secret dans le
journal. Les références de fichiers gardent leur hôte et leur portée réelle.

## Forme réalisée

`lineage compose --json --t3-thread ROOT --project-root ABS --task UUID
--input-stdin` reçoit `{request_id, expected_round, operation}`. La lecture
emploie la ronde0 ; les mutations exigent la ronde positive constatée.
Les opérations fermées sont read, send(body,intentqueue|steer),
select_runtime(model,effortnullable) et interrupt. Un effort null demande
le défaut réellement annoncé par la session ; l'ordre de la liste ne vaut
jamais défaut. Le corps est borné à65536octets UTF8.

La réponsev1 fermée contient status,request_id,task_id,round,state,outcome,
code,message,selection,models et capabilities. Les issues sont observed,accepted,
selected,refused,outcome_unknown. Les valeurs nullables restent présentes.
La sélection confirmée concerne les prochains tours. L'interruption n'annule
pas la tâche. Les gardes ordinaires d'écriture T3 restent fermées sur la vue.

ReadRuntimeCatalogue interroge le wrapper exact et son pilote existant. Le
parseur provider_catalogue.model_page est réutilisé. Aucun catalogue du parent
ou du serveur ne remplace ce résultat. Les wrappers ou pilotes sans cette
lecture gardent messages/journal selon leurs capacités, avec un code de limite
explicite pour modèle/effort. Le contrôle garde4secondes et la lecture3secondes ;
le client CLI conserve un budget absolu9secondes pour compose (6secondes pour les lectures historiques), et le Reader accorde10secondes à compose.
