# Bridget — communication entre agents

Bridget relie des agents de fournisseurs différents. Un daemon conserve les
identités, les messages, les demandes suivies et les faits de livraison ;
CLI, MCP et observation de journal sont des accès à cette même autorité.

**Cette reprise est centrée sur la communication.** Pas de GUI, de serveur web,
de runtime Docker/projet ni d'implémentation Maicie dans ce produit. Maicie peut
rester un service extérieur utilisant le protocole public. La voie principale
utilise les pilotes natifs Codex/Claude ou ACP ; tmux n'est pas requis.

## État de cette branche

Extraction physique et paquet autonome validés dans la session
089 : 1 199 tests automatiques réussis, 50 crashs réels rejoués, fmt/clippy verts
et revue indépendante avec correctifs vérifiés. Codex natif a répondu réellement
avec journal attachable. La recette Claude
réelle reste ouverte sur l'authentification locale ; GLM n'est pas encore validé.
L'échange SSH interserveur, la coupure/reprise et la charge de 600 événements
sur 60 secondes ont passé leurs recettes isolées. Les migrations sur copies et
les gates de sécurité ciblés sont exécutés. Les comptes fournisseurs non validés
restent explicitement hors du verdict. Rien n'a été déployé dans la flotte.

Preuves, refus observés et tâches ouvertes :
[réalisation](specs/089-communication-core/implementation.md),
[tâches](specs/089-communication-core/tasks.md),
[garanties](specs/089-communication-core/contracts/communication.md).

## Se servir de la communication

Depuis une session Bridget enregistrée, dans le namespace de cette session :

Ces exemples supposent le **binaire extrait**, pas le `bridget` déjà installé
par l'ancien produit. Suivre d'abord le guide d'installation indépendante ;
aucun alias, service ou PATH global n'est remplacé automatiquement.

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

Les outils MCP `bridget_who`, `bridget_send`, `bridget_cancel` et `bridget_ledger` utilisent la
même socket. Avec MCP, répondre avec `to`, `body` et `in_reply_to`.
La [skill livrée](skills/bridget/SKILL.md) contient les exemples et la conduite de
reprise ; elle n'est pas installée dans les profils globaux automatiquement.

## Codex interactif, sans tmux (session 090)

Depuis un vrai terminal, dans le répertoire de travail souhaité :

```sh
bridget codex
bridget codex -m gpt-5.6-terra
bridget codex --name coderBridget --yolo resume <UUID-du-fil-Codex>
bridget codex --name horizon-original --yolo resume horizon-original
bridget codex --name horizon-original --yolo resume
```

Le binaire de cette branche ouvre la **TUI officielle Codex**, reliée à un
app-server local privé. La saisie humaine et les messages Bridget arrivent dans
le même fil. L'UUID affiché au lancement est l'adresse de cette session ; Codex
répond aux autres agents par `bridget_send` avec `in_reply_to`. Une réponse finale
à l'écran n'est **pas** envoyée automatiquement au correspondant.

Les messages ordinaires attendent la fin d'un tour humain. Les permissions restent
dans la TUI : aucun accord automatique ajouté par Bridget. Modèle, profil,
configurations `-c`, sandbox et politique d'approbation explicitement demandés
sont relayés ; aucune option permissive du registre géré n'est réutilisée.

Contrat vérifié sur Codex **0.153.4** : connexion distante Unix expérimentale,
historique `legacy`, un fil par lancement. `/new`, `/resume` vers un autre fil,
fork et sous-agents internes Codex qui chargent un second fil terminent
l'intégration au lieu de conserver une identité pointant vers l'ancien fil.
Les autres agents Bridget restent indépendants et joignables. Pour changer de répertoire,
quitter puis relancer depuis ce répertoire ; `--cd`, images, fournisseur local
ne sont pas proposés par cette première version.
Une option non prise en charge est refusée, jamais ignorée.

`resume <UUID>` ou `resume <nom-Codex>` choisit le fil initial et conserve
son historique et son titre. Le nom Codex est recherché exactement ; absent
ou ambigu, il est refusé sans ouvrir de session. `resume` seul propose un
menu Bridget alimenté par `thread/list` de Codex : numéro puis Entrée,
`n`/`p` pour paginer, `q` ou Ctrl-C pour annuler. Ce n'est pas le sélecteur
graphique natif Codex : la TUI officielle démarre après le choix, lorsque
Bridget est déjà inscrit et son journal actif. Aucun fil provisoire créé.
Le menu liste les conversations interactives non archivées, tous répertoires
affichés, par activité récente. Lecture bornée à 1 000 fils/10 s ; un catalogue
incomplet est refusé, jamais utilisé pour choisir arbitrairement.
Le nom après `resume` désigne la conversation **Codex**, pas l'agent Bridget.
`--name` retrouve l'identité Bridget portant ce nom (80 caractères maximum),
ou la crée si le nom est nouveau. Aucun UUID Bridget à connaître : un nom
inactif est réutilisable, un agent encore actif reste protégé contre une
seconde ouverture. Avec `resume <UUID-Codex>`, son historique est repris ;
sans `resume`, une nouvelle conversation commence sous la même identité.
Le lien fil→identité est conservé localement, donc `resume <UUID-Codex>` sans
`--name` retrouve aussi l'identité d'un fil déjà lancé par cette version.
Pour un fil ancien sans liaison, préciser son nom Bridget une première fois.
Un nom et un fil désignant deux identités distinctes sont refusés sans mutation.
`--agent-id` reste une option avancée, pas un prérequis de reprise humaine.
L'agent peut se renommer avec `bridget rename <nom>`, sans changer son UUID.
À la fermeture, recopier la dernière ligne `Reprendre : bridget codex …` :
la commande Codex générique `--remote …` affichée plus haut pointe vers une
socket temporaire désormais fermée. Aucun prompt initial n'est rejoué.
`--yolo` est l'alias de `--dangerously-bypass-approvals-and-sandbox` : il désactive
explicitement sandbox et approbations Codex ; il n'est jamais ajouté par défaut.

| Commande | Durée de vie et usage |
|---|---|
| `bridget codex` | Interaction native au premier plan ; quitter ferme cette session. |
| `bridget spawn codex --persistent …` | Agent supervisé, indépendant du terminal. |
| `bridget attach <uuid>` | Journal et saisie de messages, pas reprise de la TUI Codex. |

Pour confier du développement à un nouveau Codex, l'humain lance depuis son
terminal `bridget spawn codex --persistent --cwd "$PWD" --posture development`.
Ce profil autorise l'écriture dans ce répertoire, pas le réseau des commandes
shell ni une extension automatique des droits. Les outils MCP déclarés restent
un accès distinct. Il est limité à cet ordre et exige une entrée/sortie TTY.
Sans `--posture`, la politique globale reste applicable ; `--posture discovery`
impose la lecture seule. `relaunch` conserve les droits figés de l'ancien agent :
il ne transforme pas un agent découverte en développeur. Aucun réglage global
n'est nécessaire pour ce lancement individuel.

Le daemon Bridget peut se reconnecter sans recréer le fil. Le socket Bridget
configuré peut être celui d'un tunnel SSH ; l'app-server Codex reste local,
dans un répertoire privé. Aucun nouveau serveur HTTP ou accès navigateur.
Cette évolution n'installe ni ne remplace automatiquement le binaire en service.

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

**Parler dans attach :** taper un message puis Entrée l'envoie à l'agent ;
Ctrl-C quitte la vue sans arrêter l'équipier. Ce n'est pas une reprise de la TUI
fournisseur ni un écran permettant d'accorder des permissions. La ligne de statut
du terminal affiche client/type, modèle, effort et état depuis le même annuaire
que `who`. Elle devient indisponible sans renouvellement ; le fournisseur réel
n'est jamais déduit de « Claude » ou « Codex ». Les commandes et demandes
d'autorisation Codex sont rendues comme des faits, avec sortie neutralisée.

### Skill, MCP ou CLI ?

La skill est le mode d'emploi ; MCP exécute les outils structurés de communication.
Les agents privilégient MCP pour envoyer, répondre, consulter et annuler leurs
demandes. Si les outils sont différés, ils découvrent le catalogue avant de passer
au shell. Un refus de socket dans le shell sandboxé n'est pas une preuve de panne
MCP. Les réponses finales des équipiers gérés peuvent être relayées par le wrapper ;
la TUI humaine, elle, exige une réponse liée explicite. Ne pas doubler les deux.

Le lancement, l'arrêt et la relance restent des commandes CLI explicitement
autorisées, pas des outils MCP de supervision. Toujours lire le reçu et le profil
effectif : connecté ne signifie pas autorisé à écrire. Les outils Maicie encore
annoncés sont une façade vers un service extérieur, non une dépendance pour
communiquer. Les artefacts restent inertes et n'ouvrent aucune interface graphique.

Les contenus référencés conservent bytes, provenance et accès. Un document HTML
est du contenu inerte : aucun rendu, script ou navigateur dans le noyau.

## Construire sans toucher à l'installation existante

Répertoire de réalisation :
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/089-communication-core`.

Rust est épinglé dans rust-toolchain.toml. Construire avec Cargo disponible :

```sh
cd /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/089-communication-core
PATH=/Users/moi/.cargo/bin:$PATH cargo build --locked -p bridget-daemon
```

Choisir une racine d'état privée NEUVE, courte et absolue. Ne pas pointer vers
l'ancien cache. Le HOME fournisseur n'est pas remplacé en production : il porte
les abonnements. Aucun repli vers une API facturée n'est ajouté.

```sh
bridget_state=$(mktemp -d /tmp/bgcore.XXXXXX)
export BRIDGET_HOME="$bridget_state"
export BRIDGET_SOCKET="$BRIDGET_HOME/bridget.sock"
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/089-communication-core/target/debug/bridget daemon
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
