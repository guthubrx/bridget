# Bridget — communication entre agents

Bridget relie des agents de fournisseurs différents. Un daemon conserve les
identités, les messages, les demandes suivies et les faits de livraison ;
CLI, MCP et observation de journal sont des accès à cette même autorité.

**Cette reprise est centrée sur la communication.** Pas de GUI, de serveur web,
de runtime Docker/projet ni d'implémentation Maicie dans ce produit. Maicie peut
rester un service extérieur utilisant le protocole public. La voie principale
utilise les pilotes natifs Codex/Claude ou ACP ; tmux n'est pas requis.

## État de cette branche

Extraction physique et garanties locales en cours de validation dans la session
089. Codex natif a répondu réellement avec journal attachable. La recette Claude
réelle reste ouverte sur l'authentification locale ; GLM n'est pas encore validé.
L'échange SSH interserveur, la coupure/reprise et la charge de 600 événements
sur 60 secondes ont passé leurs recettes isolées. Les migrations sur copies et
les gates de sécurité ciblés sont exécutés ; la non-régression globale et la
revue finale restent ouvertes. Rien n'a été déployé dans la flotte.

Preuves, refus observés et tâches ouvertes :
[réalisation](specs/089-communication-core/implementation.md),
[tâches](specs/089-communication-core/tasks.md),
[garanties](specs/089-communication-core/contracts/communication.md).

## Se servir de la communication

Depuis une session Bridget enregistrée, dans le namespace de cette session :

```sh
bridget who
bridget agents --json
bridget send --to '<agent_id_uuid>' --reply --timeout 120 -- 'Vérifie ce point et réponds avec ton résultat.'
bridget send --to '<emetteur_uuid>' --in-reply-to '<message_id_integral>' -- 'Résultat vérifié : …'
bridget ledger --limit 20
bridget attach '<agent_id_uuid>'
```

Les UUID sont exposés par `agents --json` (ou MCP who), tandis que `who` affiche
les noms humains. Les chevrons désignent les identifiants lus dans l'annuaire et le message reçu,
pas des valeurs à envoyer. `--reply` ouvre une demande suivie ; `in_reply_to`
lie une réponse à CETTE demande. Ne pas raccourcir l'identifiant. `bridget reply`
est une commodité visant le dernier expéditeur : pour plusieurs demandes
concurrentes, préférer le destinataire et la corrélation explicites ci-dessus.

L'adresse est l'UUID. `bridget rename "Équipe B"` change seulement le nom affiché
depuis la session propriétaire ; les retries, l'instance et l'historique restent
liés à la même identité. Un nom de fournisseur ou un champ `--from` ne permet
pas d'usurper cette identité.

Les outils MCP `bridget_who`, `bridget_send` et `bridget_ledger` utilisent la
même socket. Avec MCP, répondre avec `to`, `body` et `in_reply_to`.
La [skill livrée](skills/bridget/SKILL.md) contient les exemples et la conduite de
reprise ; elle n'est pas installée dans les profils globaux automatiquement.

## Ce que signifie un reçu

| Fait | Ce qu'il autorise à dire |
|---|---|
| `accepted` | Remise accusée par le transport, pas tâche intellectuelle achevée. |
| `in_flight` | Remise en cours attestée, pas raison de renvoyer sous une autre clé. |
| `outcome_unknown` | Résultat inconnu ; ne pas fabriquer un succès ou un refus. |
| `orphaned` | Remise orpheline ; décision explicite nécessaire avant réinjection. |
| `envelope_mismatch` | Même clé avec des arguments divergents : premier record inchangé. |
| `idempotency_expired` | Protection de rejeu expirée, pas permission de recréer silencieusement. |

Pour un retry MCP : mêmes `id` et `issued_at`, même instance et mêmes arguments
(corps, cible, délai, `reply`, `in_reply_to`). Préparer le couple avant le premier
appel si le workflow doit survivre à la perte du premier reçu. Ne pas renvoyer
avec une clé neuve après une issue ambiguë.

Le CLI ordinaire sans clé conserve son mode historique. Un envoi CLI ordinaire
idempotent demande `--id`, `--issued-at` et `--issuer-scope` ensemble. La réponse
CLI liée peut dériver la portée de l'instance ; conserver son couple id/instant
pour le rejeu. Ne pas supposer que tout exit 0 signifie un ACK durable.

## Observer sans inventer

`who` expose présence, modèle et signaux réellement reçus. Une absence de signal
reste inconnue. `ledger` lit le maître ; une coupure ne crée pas une base locale
vide. MCP peut consulter les demandes entrantes et sortantes (`mine`, défaut),
ou la portée globale (`requests_scope=all`).

`status`, `who` et `agents` refusent un inventaire non attesté au lieu de rendre
une liste vide. Leur sonde est bornée ; `status` n'ouvre aucune base côté client.
Le total des messages n'étant pas publié par le protocole, il reste explicitement
indisponible ; le ledger expose une vue bornée, pas un total prétendument exhaustif.

`attach` rejoue le journal puis suit le flux. La jonction conserve les séquences ;
un curseur devenu indisponible annonce une lacune (`Gap`). Source inaccessible,
fin de flux et rattrapage terminé sont distincts. Ni un journal frais ni une
connexion vivante ne prouvent qu'un agent travaille.

Les contenus référencés conservent bytes, provenance et accès. Un document HTML
est du contenu inerte : aucun rendu, script ou navigateur dans le noyau.

## Construire sans toucher à l'installation existante

Répertoire de réalisation :
`/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core`.

Rust est épinglé dans rust-toolchain.toml. Construire avec Cargo disponible :

```sh
cd /Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core
PATH=/Users/moi/.cargo/bin:$PATH cargo build --locked -p bridget-daemon
```

Choisir une racine d'état privée NEUVE, courte et absolue. Ne pas pointer vers
l'ancien cache. Le HOME fournisseur n'est pas remplacé en production : il porte
les abonnements. Aucun repli vers une API facturée n'est ajouté.

```sh
bridget_state=$(mktemp -d /tmp/bgcore.XXXXXX)
export BRIDGET_HOME="$bridget_state"
export BRIDGET_SOCKET="$BRIDGET_HOME/bridget.sock"
/Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core/target/debug/bridget daemon
```

Ce daemon reste au premier plan. Les autres terminaux doivent recevoir EXACTEMENT
ces variables, et utiliser CE binaire. Aucun launchd, tunnel, profil fournisseur
ou processus de l'ancienne flotte n'est remplacé. Le
[guide d'installation indépendante et de retour arrière](docs/communication-installation.md)
décrit le paquet à liste de sources fermée, ses empreintes et les préconditions
de compte. Ce n'est pas une autorisation de bascule de la flotte.

## Plusieurs serveurs

Le modèle reste un daemon maître et des clients via transfert de socket Unix
SSH. Même protocole, mêmes UUID/canons/ledger ; pas de HTTP public ni second
magasin de messages. Les scripts de transfert exigent chemins et identité SSH
explicites, une cible libre et une empreinte d'hôte connue. Une rupture du tunnel
ne justifie ni de relancer le fournisseur ni de déclarer une remise réussie.
La recette distante réelle et ses chiffres restent obligatoires avant livraison.

## Qualité et frontières

Trois crates : bridget-core (domaine), bridget-transport (protocole/pilotes/journal),
bridget-daemon (autorité, transactions, CLI/MCP). Un helper de canon et un helper
de clôture partagés ; aucune copie métier dans les façades. ACK, clôture et
événement liés restent transactionnels.

Les tests sont lancés avec HOME/BRIDGET_HOME/TMPDIR isolés, watchdog et nettoyage
des seuls enfants créés. Ne pas exécuter aveuglément les anciens harnais sur
le HOME réel. Les gates ignorés de compte/SSH ne valent pas des succès CI.
La [carte de tests](specs/089-communication-core/test-map.md) distingue leurs états.

Bridget ne remplace ni un orchestrateur de tâches, ni une revue humaine, ni une
preuve de travail terminé. A2A, une intégration T3 et la greffière sont des sujets
extérieurs : aucun framework supplémentaire n'est introduit dans cette reprise.
