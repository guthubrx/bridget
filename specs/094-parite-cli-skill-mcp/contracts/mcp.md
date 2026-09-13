# Contrat MCP 094

Tous les schémas sont fermés : rejeter tout champ inconnu, notamment from,
agent, agent_id, instance_id, source, socket, chemin. L'identité est résolue à
chaque appel. Toutes les commandes utilisent le namespace déjà configuré.
Rejets métier réutilisent les reçus typés daemon ; erreurs techniques isError
avec code stable. Ne pas convertir un ACK étranger en succès.
Un renommage Applied doit porter l'UUID propre et le nom demandé après
normalisation canonique, avec sa révision attestée. Une divergence du nom ou
de l'UUID est une erreur de protocole, jamais un succès. Les refus typés restent
des refus ; ne pas leur appliquer le contrat d'un résultat Applied.

| Outil | Arguments | Résultat / autorité |
|---|---|---|
| bridget_rename | display_name : chaîne obligatoire | DisplayNameOutcome existant, UUID/révision attestés ; propre |
| bridget_dnd | mode : on/off obligatoire ; duration : chaîne facultative uniquement on | ACK availability corrélé ; défaut 60m, entier=min, s/m/h, 1–604800s ; checked_mul/checked_add ; propre |
| bridget_domain | exactement domain : chaîne OU reset : true | ACK domain + sauvegarde existante confirmée ; propre |
| bridget_runtime | model obligatoire, effort facultatif, chaînes | ACK runtime, source Declared imposée ; propre ; aucune sélection |
| bridget_status | aucun | running, agents_inventory_available, agent_count nullable, build_id et daemon_host ; aucune base/socket/instance |
| bridget_control_status | history_limit facultatif entier 0–50, défaut 0 | état de contrôle et inbox_open_count ; historique seulement si >0, strictement borné ; aucune mutation |

## Identité et compatibilité

Domain, Availability, Runtime/Declared : la frontière daemon atteste l'agent
et l'instance courants sous verrou à chaque action, pas seulement à Register.
Nack sans mutation si cible étrangère, non inscrit, ancienne instance. CLI
domain/dnd/runtime utilise cette même couture. Pas de durcissement arbitraire
des hooks Claude observés dans ce lot, mais aucune source hook libre au MCP.

Control_status : vérification identité auxiliaire puis connexion Client
négociée pour lecture avec portée issue de l'instance. Utiliser Lookup (suffit
aux deux lectures) plutôt que capacité permettant les mutations ; aucune
portée humaine réservée. Une tentative ControlStateSet doit rester refusée.

## Domaine et durabilité

Même fichier agent-domains/UUID que le wrapper existant, pas de chemin fourni
par le modèle. ACK daemon puis écriture atomique/removal pour reset. Si
sauvegarde échoue : erreur technique domain_persistence_failed explicitant
application mémoire attestée mais persistance non confirmée ; aucune promesse
de rollback ni de durabilité. Fichier préexistant préservé en cas d'échec de
remplacement. Pas de lecture de la base depuis le client.

Convergence de revue : validation identique au canon `validate_technical_label`
du CLI, également au daemon. Un verrou interprocessus stable par UUID, voisin
du socket client, couvre mutation puis persistance. Le wrapper prend le même
verrou avant lecture du domaine et jusqu'à la fin de l'inscription/réapplication.
Ne jamais supprimer le fichier de verrou ; attente bornée, libération par RAII.
L'ordre unique est verrou local puis daemon. Les writers anciens/non coopératifs
ne bénéficient pas de cette garantie : ne pas annoncer leur mise à jour sans
rechargement. Conserver le stockage local pour les sockets fédérées SSH.

Le Register principal transmet le vrai domaine dérivé du projet. S'il existe une
surcharge persistée, le wrapper la réapplique ensuite par connexion auxiliaire
attestée, sans repersister ni reprendre le verrou, avant de déclarer l'inscription
prête. Ne pas consommer l'ACK sur le reader principal : celui-ci peut déjà porter
des messages différés. Ainsi reset revient immédiatement au vrai domaine dérivé,
même après redémarrage du daemon.

Disponibilité : `None` seul lève DND. Une échéance `Some(until <= now)` est un
refus expiré sans mutation, jamais un ACK mode:on transformé silencieusement en
off et jamais une seconde supplémentaire inventée.

## Permissions fournisseur

Les six outils et les deux outils artefacts déjà exposés rejoignent les quatre
outils de communication déjà autorisés : douze noms Bridget exacts, pas Maicie.
Ils utilisent les listes fermées déjà utilisées par Codex
approval_mode=approve et Claude allowedTools. Aucune permission globale,
aucune nouvelle possibilité de choisir une cible ou de lancer un processus.
Un ancien serveur MCP vivant conserve son binaire/catalogue ; l'installation
ne prouve pas son rechargement. La recette documente exactement le client testé.

## Équivalences et exclusions

reply = bridget_send(in_reply_to), requests = bridget_ledger(view=requests),
agents = bridget_who ; aucun doublon. Les outils artefacts existants restent.
Pas de spawn/stop/relaunch/decommission/adopt-stopped, mutation control/inbox,
reaper/reprise, hooks, migration, daemon, terminal dans le nouveau catalogue.
Chacun figure toutefois dans l'inventaire avec ses usages CLI autorisés.
