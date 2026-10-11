# Déléguer avec Bridget

Bridget possède son moteur de délégation. T3 peut être absent. Le connecteur T3
atteste seulement la session qui appelle les outils. Il ne crée pas les tâches.

Ce moteur n'est pas l'outil `delegate_task` de T3. L'exécution ne dépend pas de T3.
T3 sert seulement à attester la session MCP.

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
  "cwd": "/Users/moi/Nextcloud/10.Scripts/64.bridget"
}
```

`posture` est facultatif. Son absence demande l'héritage de la politique prouvée
du parent (voir plus bas).

`glm` est le nom du profil Bridget observé dans le registre de cette session.
Le nom d'instance T3 `claude_glm` n'est pas un type du registre Bridget.
Cet exemple ne prouve pas que ce profil existe dans tous les registres.
Un profil absent, un modèle différent ou un effort non déclaré produit un refus.
Bridget ne remplace jamais la cible demandée par une autre.

Le reçu conserve `task_id`, les références de l'enfant et de la mission.
Si la réponse se perd, reprendre les mêmes arguments avec le même `request_id`.
Le rejeu retrouve la tâche existante. Modifier son enveloppe produit
`envelope_mismatch`, sans nouvelle mission. Une clé neuve crée une nouvelle mission.
Ne pas changer de clé après une issue inconnue.

Le parent reçoit automatiquement un résultat corrélé à la mission.
`bridget_task_status` permet aussi de lire l'état durable avec `task_id`.
Un enfant actif retarde la publication du résultat de son parent.
Le résultat publié ne change plus. Une fin de tour ne suffit pas à le publier.
`bridget_task_cancel` annule sa tâche et ses descendants actifs.
Le catalogue annonce `mission_reply_limit_secs` (3600 secondes, le défaut),
`mission_reply_min_secs` (60) et `mission_reply_max_secs` (86 400).
`reply_timeout` est facultatif : il fixe le délai de réponse de cette mission,
en secondes entières. Sans lui, le délai reste 3600 secondes.
Le reçu donne `mission_deadline_at` après admission de la remise. Cette échéance
reste identique après rejeu ou reprise. Son expiration produit
`mission_reply_timeout`, puis le nettoyage de la mission. Un délai de réponse
HTTP ou une attente du parent ne change pas cette échéance et n'annule rien.

Si le daemon s'arrête pendant une mission engagée sans réponse capturée, la tâche
passe en `failed` avec `unreachable`. Un résultat déjà capturé reste durable et
attend la fin des descendants avant sa remise. Bridget ne relance aucune mission
engagée. Un rejeu renvoie cette même tâche.

## Droits du parent et de l'enfant

Le parent doit être inscrit dans Bridget avec une identité et un projet attestés.
Il ne choisit pas son identité dans les arguments des outils. L'enfant reçoit
la mission demandée et les faits natifs d'identité et de Git utiles à sa reprise.
Ces faits sont joints au premier tour. Il ne reçoit ni la conversation du parent
ni ses credentials T3.

Le catalogue indique le périmètre du dossier avec `cwd_scope` :

- `project` : le parent a une preuve de droits. La mission doit rester dans
  `cwd_root`, c'est-à-dire le dossier effectif du parent. Un dossier hors de cette
  racine est refusé (`cwd_outside_parent_project`).
- `root` : un grant humain explicite fixe la racine.
- `same_project` : parent externe avec projet attesté, sans grant. `cwd_root` vaut
  `null`. Ce parent peut inspecter ce dépôt et ses worktrees.

Un dépôt étranger reste refusé. Le dossier `.git` ne sert jamais de racine de droits.

**Posture (facultative) :**

- **Absente (cas normal) :** l'enfant hérite de la politique que le parent a prouvée.
  Bridget n'ajoute aucun droit et ne demande aucun grant. Si la cible ne peut pas
  garder cette politique, le motif figure dans `inherit_refusal` du catalogue.
- **`discovery` :** avec un parent attesté, la mission reste en lecture seule. Sans
  preuve, cette valeur suit le chemin 148 : un grant humain existant est requis,
  sinon `delegation_grant_required`.
- **`development` :** avec un parent attesté, l'écriture n'est accordée que si la
  politique du parent et la cible le permettent. Sans preuve, la demande est refusée
  avec `permission_attestation_unavailable`. Le motif éventuel figure dans
  `development_refusal`.

**Ce qui passe d'un parent à une cible (149) :**

- **Codex vers Codex :** Bridget garde la politique d'approbation, le bac à sable et le
  réseau du parent. Un parent en lecture seule ne donne pas `development`
  (`permission_not_inherited`). Un bac à sable externe est refusé
  (`provider_confinement_unavailable`).
- **Codex vers Claude ou GLM avec écriture :** seulement si le parent est en `dangerFullAccess` avec
  approbation `never`. L'enfant passe en `bypassPermissions`, dans la seule définition
  de cette mission. Un parent en `workspaceWrite` est refusé
  (`provider_confinement_unavailable`). Un parent en lecture seule donne une mission en
  mode plan, avec `Read`, `Glob` et `Grep`.
- **Claude ou GLM vers Claude ou GLM :** la mission réutilise les mêmes entrées : même
  lanceur, même profil, mêmes sources et mêmes empreintes. Le dossier de l'enfant doit
  être celui du parent. Les règles `allow` et `deny` du parent restent en vigueur.
  Bridget n'ajoute aucun bypass universel. Si une entrée change, la demande est refusée
  avec `settings_revision_changed`. Une source opaque, par exemple une politique gérée
  non observable, donne `permission_source_unavailable`.
- **Claude ou GLM vers Codex :** seulement si la politique est un bypass complet sans
  règle, ou une lecture seule limitée à `Read`, `Glob` et `Grep`.

Dans tous les autres cas, la réponse est `permission_mapping_unavailable`. Tous ces
refus arrivent avant la création de la tâche : aucun enfant n'est lancé.

Un humain peut accorder ou retirer des droits natifs à une instance avec
`bridget delegate-grant`, dans un terminal humain. Le grant porte sur l'instance,
une racine absolue et une posture maximale. Le MCP ne crée jamais ce grant.

Un grant n'est requis que pour la voie 148 : `discovery` sans preuve de droits du
parent. Une délégation héritée d'un parent attesté n'en demande aucun.

La révocation reste prioritaire. Elle ferme les nouvelles demandes du même agent,
même après changement d'instance, héritage compris (`delegation_grant_required`).
Seul un nouveau grant humain explicite lève ce refus durable. Elle ne transforme pas
le rejeu d'une demande déjà acceptée en une nouvelle mission.

## Identité dans T3

Deux conversations peuvent partager un processus fournisseur. Chaque montage
MCP Bridget reçoit une preuve privée distincte. Bridget la vérifie auprès du
serveur T3 à chaque opération, puis vérifie son inscription vivante au daemon.
Une preuve partielle, périmée ou révoquée produit un refus explicite.
Bridget n'emprunte jamais l'identité d'une conversation voisine.

Un credential retiré, révoqué ou tourné ferme tous les outils Bridget de cette
session avec `t3_session_unavailable`. Ce refus n'a ni repli par PID ni repli en
lecture seule. Une session qui n'a jamais eu de fait de droits garde l'identité seule (version 1)
pour les lectures et le rejeu d'une demande déjà admise. Une admission héritée ou
`development` exige la version 2 : sans elle, la réponse est
`permission_attestation_unavailable`, sans grant ni repli vers `discovery`.

Les options MCP humaines existantes et les désactivations restent prioritaires.
Les outils148 deviennent disponibles au chargement des nouveaux processus.
Une installation de fichiers seule ne recharge pas une session déjà ouverte.
Les comportements 149 exigent le daemon et le binaire 149 chargés. Ce guide ne
prouve pas leur activation.

Au redémarrage, une identité durable contradictoire bloque le démarrage du daemon
avant la reprise des missions. Les erreurs `native_restart_identity_mismatch`,
`parent_task_unavailable`, `native_owner_identity_mismatch`,
`native_execution_changed` et `native_execution_mismatch` exigent un diagnostic
des données conservées. Ne supprimez pas leurs marqueurs pour forcer la reprise.
Si un wrapper natif reste vivant après le délai commun de 8 secondes, le daemon
refuse aussi de démarrer. Il conserve le marqueur et ne force pas cet arrêt.

## Suivi humain et panneau Lineage

Le terminal humain lit une tâche avec `bridget lineage inspect`, `watch` et `cancel`.
Ces commandes demandent `--t3-thread` (fil T3 du parent) et `--project-root` (chemin
absolu). Elles n'utilisent pas le MCP. Le fil T3 sert à la lecture : il ne crée pas la
tâche.

Dans T3, le panneau Lineage affiche l'arbre des tâches du fil parent : la tâche
racine, puis les enfants imbriqués. Une tâche se lit dans son journal et son résultat.
Ce panneau n'est pas un fil de conversation. Il n'a pas de zone de saisie, et l'enfant
ne reçoit aucun tour. Le bouton Arrêter demande l'annulation native de la tâche et de
ses descendants.

Statuts natifs : `queued`, `starting`, `mission_pending`, `working`,
`waiting_for_children`, `cancelling`, `result_available`, `failed`, `cancelled`. Un
statut inconnu ferme le contrat.

Preuve : l'interface web a été testée avec un daemon simulé. Aucune preuve ne couvre
le daemon réel, l'application de bureau ou une installation. Ne pas annoncer cette
fonction comme livrée.

## Délai de réponse choisi

`reply_timeout` fait partie de l'enveloppe de la demande. Rejouer la même clé avec un
autre délai produit `envelope_mismatch`. Omettre le champ et envoyer `3600` sont deux
enveloppes différentes.

Une valeur hors de 60 à 86 400 est refusée. Bridget ne la corrige pas.

Un enfant qui fixe un `reply_timeout` ne peut pas dépasser le temps restant de sa tâche
parente. Sans `reply_timeout`, rien n'est contrôlé : le comportement reste celui de la
session 149. La même valeur règle l'échéance de la mission et le suivi de la demande.
Avec un délai explicite, le suivi de la demande compte depuis l'admission de la mission,
même si le lancement de l'enfant a pris du retard.
Les tâches déjà actives gardent leur délai de 3600 secondes.

## Refus

Le refus revient dans `native_delegation_refused`, avec le code ci-dessous.
Aucun de ces refus ne crée de tâche. Aucun ne consomme la clé de rejeu.

| Code | Cause |
|---|---|
| `referent_paused` | Le référent est en pause et la clé est nouvelle. |
| `control_state_unavailable` | L'état de pause est illisible : Bridget refuse plutôt que de déléguer pendant une pause possible. |
| `request_expired` | La tâche de cette clé a été purgée depuis moins de 30 jours. |
| `retention_saturated` | 4096 tâches sont conservées et aucun arbre n'est purgeable. |
| `reply_timeout_out_of_range` | `reply_timeout` est hors de 60 à 86 400. |
| `reply_timeout_exceeds_parent` | Le délai explicite d'un enfant dépasse le temps restant du parent. Sans délai explicite, aucun contrôle (comportement 149). |

Ordre des contrôles : identité, forme de la demande, bornes du délai, rejeu
(`envelope_mismatch`, `request_expired`), pause (`referent_paused`, ou
`control_state_unavailable` si l'état de pause est illisible), droits, `task_limit`,
`retention_saturated`, lignée du parent, délai de l'enfant.

`task_limit` reste pour 128 tâches actives, ou 16 enfants actifs par instance.
L'historique cumulé ne le provoque plus.

## Pause du référent

Pendant la pause, une nouvelle clé reçoit `referent_paused`. Rejouer une clé déjà
admise rend la tâche existante. La lecture du catalogue, la lecture de statut et
l'annulation restent possibles. Après la levée de la pause, la même clé réussit.

Une pause n'arrête pas encore les missions en cours. La session 152 traite ce point.

## Rétention des tâches

Bridget garde au plus 4096 tâches. Il purge par arbre : une tâche sans parent et tous ses
descendants. L'entretien horaire purge les arbres finis depuis plus de 7 jours, 128 au
plus par passage. Si 4096 tâches sont conservées, les arbres purgeables les plus anciens
partent d'abord, même avant 7 jours. L'entretien le fait dans le même quota de 128 arbres
par passage, partagé avec la purge d'âge. Une nouvelle délégation le fait aussi : elle
retire seulement les arbres nécessaires pour repasser sous 4096, sans limite de 128.

Un arbre n'est jamais purgé dans ces cas :

- une de ses tâches est active ou son nettoyage n'est pas fini ;
- un résultat ou un avis d'échec n'a pas été accusé par le destinataire ;
- un de ses sous-agents est connecté, routé, en cours de lancement, en cours de
  décommission ou désiré `Running`.

Avant la purge, chaque sous-agent est retiré de la flotte. Une remise indéterminée n'a
pas d'accusé : elle bloque donc son arbre. Si ces arbres remplissent la borne, la
délégation est refusée avec `retention_saturated`.

Après une purge, `bridget_task_status` sur cette tâche rend le refus existant de tâche
inconnue (`task_unavailable`). La vue Lineage fait une lecture complète sans tâche
fantôme. Le rejeu de la clé rend `request_expired` pendant 30 jours. La clé redevient
libre au passage d'entretien qui suit ces 30 jours (1 h au plus de décalage).

## Ce qui ne change pas

- Le lancement ordinaire (`bridget spawn`), ses postures et ses protections du
  terminal humain ne changent pas. Ce guide ne remplace pas leur tutoriel.
- `bridget delegate-grant` reste une commande humaine de terminal.
