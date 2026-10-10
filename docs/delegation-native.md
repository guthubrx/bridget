# Déléguer avec Bridget

Bridget possède son moteur de délégation. T3 peut être absent. Le connecteur T3
atteste seulement la session qui appelle les outils. Il ne crée pas les tâches.

Le parent consulte `bridget_capabilities`. Il choisit une entrée réellement
disponible. Le catalogue provient du registre natif Bridget. Les noms de modèle
et d'effort doivent correspondre exactement aux valeurs déclarées.

Un seul appel `bridget_delegate` crée l'enfant et remet sa mission. Exemple
de forme d'appel, à adapter au catalogue de la session :

```json
{
  "request_id": "revue-contrat-round1",
  "agent_type": "glm",
  "model": "glm-5.3",
  "task": "Lis le contrat indiqué et rends les défauts avec leur preuve.",
  "cwd": "/Users/moi/Nextcloud/10.Scripts/64.bridget",
  "posture": "discovery"
}
```

`glm` est le nom du profil Bridget observé dans le registre de cette session.
Le nom d'instance T3 `claude_glm` n'est pas un type du registre Bridget.
Cet exemple ne prouve pas que ce profil existe dans tous les registres.
Un profil absent, un modèle différent ou un effort non déclaré produit un refus.
Bridget ne remplace jamais la cible demandée par une autre.

Le reçu conserve `task_id`, les références de l'enfant et de la mission.
Si la réponse se perd, reprendre les mêmes arguments avec le même `request_id`.
Le rejeu retrouve la tâche existante. Modifier son enveloppe est refusé.

Le parent reçoit automatiquement un résultat corrélé à la mission.
`bridget_task_status` permet aussi de lire l'état durable avec `task_id`.
Un enfant actif retarde la publication du résultat de son parent.
Le résultat publié ne change plus. Une fin de tour ne suffit pas à le publier.
`bridget_task_cancel` annule sa tâche et ses descendants actifs.
Le catalogue annonce `mission_reply_limit_secs`, actuellement 3600 secondes.
Le reçu donne `mission_deadline_at` après admission de la remise. Cette échéance
reste identique après rejeu ou reprise. Son expiration produit
`mission_reply_timeout`, puis le nettoyage de la mission. Un délai de réponse
HTTP ou une attente du parent ne change pas cette échéance et n'annule rien.

## Droits du parent et de l'enfant

Le parent doit être inscrit dans Bridget avec une identité et un projet attestés.
Il ne choisit pas son identité dans les arguments des outils. L'enfant reçoit
la mission demandée et les faits natifs d'identité et de Git utiles à sa reprise.
Ces faits sont joints au premier tour. Il ne reçoit ni la conversation du parent
ni ses credentials T3.

Un parent externe avec un projet attesté peut inspecter ce dépôt et ses worktrees.
Le catalogue indique alors `cwd_scope="same_project"` et `cwd_root=null`.
Un grant ou un parent géré conserve une racine explicite : `cwd_scope="root"`.
Un dépôt étranger reste refusé. Le dossier `.git` ne sert jamais de racine de droits.

`discovery` sert aux missions d'inspection. `development` permet le travail
d'écriture uniquement si les droits du parent et le protocole du fournisseur
le permettent. Le catalogue expose les motifs de refus. Dans le socle actuel,
Claude/GLM n'ont pas de protocole natif de développement confiné. Bridget refuse
cette posture. Le catalogue l'annonce par `development_refusal` avec la valeur
`development_protocol_unavailable`. L'appel de délégation lui-même peut rendre
le message du registre « posture développement réservée à Codex app-server ».

Un humain peut accorder ou retirer des droits natifs à un parent externe avec
`bridget delegate-grant`, dans un terminal humain. Le grant porte sur l'instance,
une racine absolue et une posture maximale. Le MCP ne peut pas créer ce grant.
Sa révocation ferme les nouvelles demandes du même agent, même après changement
d'instance. Seul un nouveau grant humain explicite lève ce refus durable.
Elle ne transforme pas le rejeu
d'une demande déjà acceptée en une nouvelle mission.

## Identité dans T3

Deux conversations peuvent partager un processus fournisseur. Chaque montage
MCP Bridget reçoit une preuve privée distincte. Bridget la vérifie auprès du
serveur T3 à chaque opération, puis vérifie son inscription vivante au daemon.
Une preuve partielle, périmée ou révoquée produit un refus explicite.
Bridget n'emprunte jamais l'identité d'une conversation voisine.

Les options MCP humaines existantes et les désactivations restent prioritaires.
Les outils148 deviennent disponibles au chargement des nouveaux processus.
Une installation de fichiers seule ne recharge pas une session déjà ouverte.
