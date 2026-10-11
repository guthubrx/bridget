# Compatibilité 151 avec le vrai ancien daemon — T018

Validation réelle réussie le 2026-10-11. Les deux outils rendent
`wait_unsupported` en moins de 5 secondes. Aucun outil nouveau n'est transmis
à l'ancien daemon. Le reçu complet conserve les octets reçus et relayés.

## Binaires contrôlés

Le daemon ancien est compilé depuis le commit de base de 150 :
`1f2c661d39a9ce106723a93ac883f76ab9fe52c6`.
Le worktree détaché était propre avant compilation et avant retrait.

| Binaire | SHA-256 |
|---|---|
| Ancien daemon | `263d0dc07fe89dbcc5f4ebade0a1f888fcdb0f6c0c13bdb87dd2ffd67ce6c8f1` |
| Client 151 | `07095719ff941ec54ac948b46c56c19cdb83821f94222a814632aed34f048855` |

Chemin du daemon testé :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/target/debug/bridget

Chemin historique du client privé, retiré après validation :
/private/tmp/old151.pfvh/current/bridget

La compilation ancienne a duré 32,46 secondes. Le cache Cargo partagé est conservé.
Le principal reconstruit le binaire 151 avant les vérifications suivantes.

## Résultats mesurés

| Appel MCP réel | Délai | Retour |
|---|---:|---|
| `bridget_task_wait`, une tâche synthétique, attente 3600 s | 44,710 ms | `native_delegation_refused`, `wait_unsupported`, `isError: true` |
| `bridget_request_wait`, une demande synthétique, attente 3600 s | 54,874 ms | `native_delegation_refused`, `wait_unsupported`, `isError: true` |

Le client envoie `initialize`, reçoit la réponse, puis envoie
`notifications/initialized`. L'identité provient d'un fichier privé, situé
dans son namespace. Tous les arguments métier sont synthétiques.

Le namespace naît sous `/private/tmp`, avec des dossiers privés et `umask 077`.
Chaque processus reçoit un environnement fermé. Le registre des fournisseurs
est vide : aucun fournisseur, modèle ou agent extérieur n'est lancé.

## Preuve du protocole transmis

Un relais Unix transparent relie le client 151 au véritable daemon ancien.
Il conserve tous les octets dans les deux sens, avant et après transmission.
Chaque connexion est jointe et fermée avant de conclure.
Le relais ne réécrit ni les trames, ni le rôle, ni l'identité.

Les captures de chaque outil contiennent exactement cette trame client :

```json
{"type":"DaemonIdentityRequest"}
```

Chaque trame comporte un LF final et mesure 33 octets, soit 66 octets clients
au total. Chaque réponse est un vrai `DaemonIdentityReport` sans champ
`features`. Les octets reçus et transmis sont identiques dans les deux sens.
Les captures ne contiennent ni `NativeDelegation::Wait`, ni `RequestWait`.
La sonde précède l'authentification auxiliaire ; aucune trame nouvelle n'est envoyée.

Le reçu conserve les données UTF-8, les octets hexadécimaux, les trames JSON
décodées, les réponses MCP et les délais monotones.

## Nettoyage observé

Le client MCP termine avec le code 0 après EOF.
L'ancien daemon est identifié par `ps -ww` : PID enfant direct et chemin du
binaire attendu. Il reçoit SIGTERM individuellement. Après 3 secondes,
son code de sortie est 0 et `ps` confirme son absence.

Les deux processus et le namespace privé sont retirés.
Le worktree ancien propre et la copie temporaire du client sont retirés.
Le cache Cargo partagé reste présent. Aucun service de production n'est touché.

Le premier passage est conservé comme échec du relais de recette : ses
demi-fermetures EOF recevaient `ENOTCONN` lorsque le client avait déjà fermé.
Les captures montraient déjà que tous les octets avaient été transmis.
La recette tolère cette erreur uniquement sur `shutdown` après un EOF et la
consigne dans le reçu. Les assertions de transmission restent inchangées.
Le second passage est entièrement vert.

## Preuves et reproduction

Reçu JSON complet :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/compat-old151.json

Sortie brute du passage final :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/compat-old151.log

Compilation ancienne :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/old-build151.log

Source intégrale de la recette Python, conservée comme journal reproductible :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation/compat-old151-source.log

Pour reproduire : créer un worktree propre au commit indiqué, conserver une
copie du client 151 avant de compiler l'ancien daemon, puis exécuter la source
Python ci-dessus avec trois arguments absolus : binaire ancien, binaire 151,
chemin du reçu JSON. La recette crée et retire son namespace privé. Elle
enregistre les empreintes et vérifie les deux limites de 5 secondes.

La compilation utilise `cargo build -p bridget-daemon --bin bridget`, avec
`env -i`, `umask 077` et le cache partagé ci-dessous :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/target

Les journaux du daemon, du client et du passage initial portent le même
préfixe `compat-old151` dans ce dossier de validation :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/151-attente-fin-de-tache/specs/151-attente-fin-de-tache/validation
