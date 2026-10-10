# Contrat natif148

Les quatre outils MCP utilisent l'identité attestée de la connexion auxiliaire.
Aucun argument ne désigne le parent. Tous les objets d'arguments sont fermés.

| Outil | Arguments | Effet |
|---|---|---|
| `bridget_capabilities` | aucun | Lit le registre et les droits du parent. |
| `bridget_delegate` | request_id, agent_type, model, task, cwd, posture ; effort facultatif | Accepte une tâche durable et pilote son lancement. |
| `bridget_task_status` | task_id | Lit seulement sa tâche. |
| `bridget_task_cancel` | task_id | Annule sa tâche et sa descendance active. |

`request_id` contient de 1 à 128 octets, sans contrôle interdit. `task` contient
au plus 65536 octets et doit être non vide après trim. Le répertoire doit exister
et respecter le périmètre natif autorisé. Le catalogue expose `cwd_scope` :
`root` avec `cwd_root` pour une racine explicite, ou `same_project` avec
`cwd_root=null` pour la découverte dans le projet attesté. Ce second périmètre
accepte le dépôt et ses worktrees après résolution Git canonique. Le dossier
`.git` n'est jamais une racine de permission. `posture` vaut `discovery` ou
`development`. Le registre fixe les valeurs exactes de modèle et d'effort.

Le reçu version1 conserve `task_id`, `child_agent_id`, `message_id`, `agent_type`,
`model`, `effort`, `cwd`, `posture`, `status`, `result`, `error` et
`mission_deadline_at`. Le catalogue annonce `mission_reply_limit_secs=3600`.
L'échéance est nulle avant remise. Elle est ensuite figée en secondes Unix depuis
l'admission de l'exécution native. Le retry et la reprise ne la renouvellent pas.
L'expiration produit `mission_reply_timeout` et le nettoyage. Cette échéance
de mission est distincte d'un délai d'attente ou de transport du parent.
Le résultat reste absent tant qu'il n'est pas publié. Les lectures ne déclenchent
aucun lancement ni progression. Un timer natif pilote les phases acceptées.

La clé de demande et son enveloppe sont persistées avant lancement. Un rejeu
identique rend la tâche existante. Une enveloppe différente donne
`envelope_mismatch`. Les références enfant et mission sont stables. La définition
du fournisseur est figée à l'acceptation. Une modification ultérieure du registre
ne substitue pas une autre cible au milieu d'une reprise.

La mission est remise par `SendIdempotent`. La réponse enfant est capturée sous
sa corrélation de mission avant accusé. Des descendants actifs empêchent sa
publication. La remise au parent garde une clé stable. Une nouvelle lecture ne
réémet pas le résultat. Les liens de flotte sont fermés après nettoyage supervisé.

Limites natives : 16 tâches actives par parent, 128 globales, 4096 enregistrements,
profondeur maximale8. Aucun recrutement interprojet automatique.
Un refus métier rend une erreur MCP explicite, sans substitution ni retry caché.

Les grants humains passent par le protocole de contrôle natif et la CLI humaine.
Ils ne sont pas accessibles aux quatre outils MCP. Les secrets de session T3
sont exclus de l'environnement enfant même si `pass_env` les demande.
La révocation est conservée par AgentId stable jusqu'à un nouveau grant humain.
Une reprise d'instance exige une preuve native et le départ de l'ancienne
présence et de sa connexion primaire. Une erreur de capture persistante produit
`native_result_not_persisted`, sans accusé ni livraison directe hors de la saga.
