# Bridget — communication entre agents

Bridget relie des agents de fournisseurs différents. Un daemon conserve les
identités, les messages, les demandes suivies et les faits de livraison ;
CLI, MCP et observation de journal sont des accès à cette même autorité.

**Cette reprise est centrée sur la communication.** Pas de GUI, de serveur web,
de runtime Docker/projet ni d'implémentation le service compagnon dans ce produit. le service compagnon peut
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
réalisation,
tâches,
garanties.

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
bridget ledger search --query "pagination erreur" --limit 20
bridget ledger read --id '<id>' --target '<uuid>' --offset <match_offset> --digest <body_digest>
bridget attach '<agent_id_uuid>'
```

Les UUID sont exposés par `agents --json` (ou MCP who), tandis que `who` affiche
les noms humains. Les chevrons désignent les identifiants lus dans l'annuaire et le message reçu,
pas des valeurs à envoyer. `--reply` ouvre une demande suivie ; `in_reply_to`
lie une réponse à CETTE demande. Ne pas raccourcir l'identifiant. `bridget reply`
est une commodité visant le dernier expéditeur : pour plusieurs demandes
concurrentes, préférer le destinataire et la corrélation explicites ci-dessus.

L'adresse est l'UUID. `bridget rename "Équipe B"` ou `bridget_rename` change
seulement le nom affiché depuis la session propriétaire ; les rejeux, l'instance
et l'historique restent liés à la même identité. Un nom de fournisseur ou un
champ `--from` ne permet pas d'usurper cette identité.

### Garanties de communication et mise à jour 099

Un client MCP ou CLI rattaché à un agent doit présenter la preuve privée reçue
par son wrapper. Connaître l'UUID, l'instance ou une portée de rejeu ne suffit
pas. La preuve est renouvelée à la reconnexion du propriétaire ; ses anciennes
connexions auxiliaires perdent leurs droits. Elle reste dans l'état privé du
wrapper, y compris avec un daemon distant par SSH, jamais dans l'annuaire.

Mettre à jour ensemble daemon, wrappers et clients, puis reconnecter les wrappers
du **namespace vérifié**. Un ancien auxiliaire sans preuve reçoit un refus
explicite : aucun repli vers l'ancienne inscription déclarative. Cette correction
n'isole pas deux processus hostiles qui contrôlent le même compte système et
peuvent lire ses fichiers privés. Ne pas partager ce compte avec un tiers non fiable.

Une remise classique attend au maximum une seconde le verrou et l'écriture de
sortie, sans garder le verrou global du daemon. Une panne d'écriture ne produit
pas un accusé de réussite : une remise partielle peut rester indéterminée, et il
faut consulter le suivi avant un rejeu. Un accusé de remise n'atteste ni la fin
du travail fournisseur ni la qualité de sa réponse.

Le contrat 094 décrit une liste fermée de douze outils Bridget :
`bridget_who`, `bridget_send`, `bridget_cancel`, `bridget_ledger`, les deux outils
d'artefacts et les six ajouts `bridget_rename`, `bridget_dnd`, `bridget_domain`,
`bridget_runtime`, `bridget_status`, `bridget_control_status`. Ils utilisent la
même autorité ; ce catalogue n'est ni une approbation MCP globale, ni une
autorisation automatique des outils du guichet. Avec MCP, répondre avec `to`, `body`
et `in_reply_to`. La [skill livrée](skills/bridget/SKILL.md) contient les exemples
et la conduite de reprise ; son
[inventaire 094](skills/bridget/references/commandes.md) classe chaque commande
CLI. Ces fichiers ne prouvent ni l'installation du binaire, ni le catalogue
d'une session MCP déjà ouverte, et aucun profil global n'est modifié automatiquement.

## Administrer une fédération SSH

Le binaire 096 embarque le gestionnaire de fédération : aucun chemin de dépôt
ou de script n'est nécessaire pour l'usage humain courant.

```sh
bridget federate ssh://exemple.test -p 2222
bridget federate status
bridget federate remove ssh://exemple.test -p 2222 --label poste-beta-core
```

La première commande réutilise sans mutation une installation attestée lorsque
le nom DNS correspond à son IP enregistrée. Le DNS ne remplace jamais la cible
SSH ni le fichier `known_hosts`. `status` inventorie localement, sans SSH. Pour
un retrait trouvé par alias DNS, un appel non interactif exige `--label`; un
double terminal peut confirmer le label, l'hôte enregistré et le port affichés.
Une nouvelle destination demande les paramètres manquants seulement avec stdin
et stdout sur un terminal ; ailleurs, elle affiche les options requises et échoue.
Voir [le guide des services de fédération](docs/federation-services.md).

## Joindre les fils t3code (session 098)

Le binaire 098 embarque un pont vers [t3code](https://github.com/pingdotgg/t3code)
qui ne modifie jamais t3code : il lit son serveur local en boucle locale avec une
session émise par le CLI officiel `t3`, et présente chaque fil ouvert comme un
agent Bridget (`TYPE` = fournisseur du fil, `TRANSPORT` t3code, `MODE` cli, nom
humain = titre du fil).

```sh
bridget t3 install        # session t3 dédiée (t3 auth session issue) + service de pont
bridget t3 status         # serveur t3code, session, service, fils exposés
bridget send --to Alpha --reply -- 'Mission…'   # un tour démarre dans le fil « Alpha »
bridget t3 uninstall      # révoque la session, retire le service, efface l'état du pont
```

Prérequis : t3code démarré (application ou `t3 --mode web --no-browser`) et le
CLI `t3` installé (`npm i -g t3`). Un message remis attend que le fil soit libre
(sans limite de durée, file bornée à 64 remises par fil), démarre un tour avec le texte du message, et la réponse
de ce tour revient à l'expéditeur comme réponse liée : l'appariement suit
l'ordre FIFO des tours après le dernier tour clos au moment de la remise. La
session détenue est administrative (t3code 0.0.40 n'en émet pas d'autre) et
vit uniquement dans un fichier 0600 de l'état Bridget ; un refus 401 déclenche
un seul renouvellement, un second refus est un échec explicite dans `status`.
Le journal du fil (`bridget attach`) ne rejoue pas l'historique antérieur à
l'installation. Le fil archivé disparaît de l'annuaire, jamais de t3code.
Voir [l'ADR 034](docs/decisions/034-adaptateur-t3code.md).

Attente et écartement (sessions 111 à 113) : l'attente d'un fil occupé ne périme
rien par elle-même et ne consomme pas le budget d'exécution du tour, qui court à
partir du démarrage effectif. Une remise n'est écartée que pour un motif porté
par le message ou par le daemon : échéance de tour ou de réponse déjà passée à la
réception (`--timeout` côté expéditeur), demande suivie close, annulation, ou
file saturée. Un message sans échéance peut donc partir des heures après son
envoi, avec son contexte d'origine : c'est à l'expéditeur de poser une échéance
s'il en veut une. Chaque écartement est rapporté au daemon : la saga d'envoi
devient terminale (plus de « remise en vol » pour un message jeté) et
l'expéditeur d'une demande suivie encore ouverte reçoit l'échec avec son motif.

Le pont traite les annulations pendant l'attente d'un fil libre : une demande
annulée avant démarrage ne devient pas un tour ultérieur. L'expiration et les
rappels automatiques n'ouvrent pas de tour supplémentaire. Une annulation après
acceptation par t3code ne garantit pas l'interruption du fournisseur. La réponse
préparée est conservée jusqu'à confirmation, y compris si son destinataire se
déconnecte ou si le pont redémarre ; sa reprise ne relance pas le travail.
Le journal conserve les textes longs dans ses bornes ou signale explicitement
la lacune, sans coupe silencieuse à 4 096 caractères.

La session 105 réserve le relais automatique aux demandes `reply=true`.
Les réponses reçues et notifications `reply=false` ne produisent plus de réponse
automatique en retour. Les anciennes attentes dépourvues de contrat sont
conservées mais ne sont envoyées qu'après preuve d'une demande encore ouverte.
Les consignes T3, Claude, Codex, ACP et terminal demandent de ne pas accuser
réception inutilement. Une nouvelle question explicite reste possible : il
n'existe pas de filtre sur des mots comme « OK », ni de blocage des envois volontaires.

La session 102 ajoute les fils partagés : un historique commun à des membres
fixés, des dépôts silencieux par défaut, des sollicitations ciblées structurées
(`notify` : `[]`, UUID ou `"all"`), une lecture paginée avec reçu puis
confirmation, et `history` pour relire sans déplacer le repère. L'outil
`bridget_thread` et la famille `bridget thread` partagent le même contrat ;
l'alerte de fil est une remise idempotente `from: bridget` sans réponse
attendue, jamais relayée comme un message direct. Les wrappers annoncent la
capacité d'alerte après enregistrement ; un client ancien publie et lit mais
n'est pas sollicité.
La session 103 ajoute le dossier de passation : l'agent rédige objectif, résumé,
résultats déclarés, décisions, questions, prochain pas, références et limites ;
`bridget_handoff` (et `bridget handoff --json-stdin`) valide, rend un corps v1
déterministe borné à 16 Kio et le transmet par l'envoi idempotent existant. Rien
n'est lu ni certifié ; le dossier suit la conservation du journal.

La session 101 ajoute le rattachement automatique des appels MCP au fil T3
réel, lorsque son identité fournisseur et sa filiation de processus sont
attestées. Le pont lit uniquement les correspondances de sessions nécessaires
dans la base T3, sans la modifier. Ni le titre, ni le projet, ni le fichier le
plus récent ne servent de preuve. Une ambiguïté entraîne un refus explicite,
pas le choix arbitraire d'une conversation. Aucun redémarrage de T3 ou d'un
fournisseur n'est nécessaire à ce mécanisme. Les événements proposés restent
ceux annoncés par la source dans `bridget events types` ; voir
[l'ADR 037](docs/decisions/037-observations-t3-attestees.md).

## Claude Code interactif, sans tmux (session 097)

Depuis un vrai terminal (iTerm, Terminal, shell distant), dans le répertoire voulu :

```sh
bridget claude
bridget claude --resume
bridget claude --name relecteur --model claude-opus-5
```

Le wrapper possède un pseudo-terminal : Claude Code y démarre avec son
interface native (couleurs, redimensionnement, permissions), la frappe et
l'affichage sont relayés tels quels, et les messages Bridget sont collés dans
la conversation comme une saisie humaine (collage encadré puis Entrée). La
présence affichée par `who` est `claude | claude_pty | cli` : jamais `tmux`
sans tmux. Sans terminal, le lancement est refusé avec l'alternative
`bridget spawn claude`. Aucun bypass de permissions n'est ajouté ; un bypass
explicitement passé par l'utilisateur est relayé tel quel. Le journal de
session relaie les tours humain et assistant lus dans le transcript Claude ;
`bridget attach <UUID>` fonctionne comme sur Codex interactif. Une session
Claude hors T3 a donc deux routes vers une identité attestée : `bridget claude`
(ou ses alias historiques `gclaude` et `claude-son`, même type) et
`bridget spawn claude`. Une session Claude lancée autrement n'est pas joignable
par Bridget. Les autres
alias interactifs (`gemini`, agents personnalisés) restent sur tmux et sont
refusés au lancement s'ils n'ont aucun pane.

Claude géré (`bridget spawn claude`) hérite de la connexion au compte de
l'humain : HOME réel et `USER` sont nécessaires au CLI officiel ; aucune clé
d'API n'est lue. Recette réelle consignée dans
specs/097-claude-sans-tmux/implementation.md.

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
historique `legacy`, une identité Bridget liée au fil initial. Les sous-agents
internes restent gérés par Codex : leur création/reprise ne ferme pas la session
et ne crée aucune identité Bridget. Les notifications d'autres fils ne changent
pas la cible des messages. L'intégration ne suit pas implicitement `/new` ou
`/resume` vers un autre fil : relancer Bridget pour joindre cette conversation.
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
`ledger search` (ou `bridget_ledger` `action=search`) cherche une page bornée et
reprenable dans ses propres échanges ou un fil dont on est membre ; `ledger read`
relit un message exact par `id` + `target` avec son empreinte. Une recherche est
partielle par construction : continuer avec le curseur, ou l'annoncer comme telle.

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

La vue terminal présente les réponses en Markdown : titres, listes, emphase,
citations et code distinct visuellement, sous un en-tête compact séparé. Le nom
affiché reste cosmétique : l'adresse de communication est toujours l'UUID.
Les raisonnements et fins de tour ordinaires réussies sont masqués dans cette
vue seulement ; commandes, permissions, refus, erreurs et lacunes restent
visibles. La sortie non-TTY conserve le diagnostic technique, et le journal
source n'est pas modifié. Les blocs de code sont stylés, sans coloration lexicale
spécifique à chaque langage ni lien terminal actif.

Changer la largeur replie la réponse en cours ou la dernière réponse encore
gérée à l'écran, sans frappe supplémentaire, ainsi que la saisie et le statut.
Le préfixe déjà remis au scrollback appartient au terminal : Bridget ne réécrit
pas tout l'historique. Le prochain tour libère la réponse précédente retenue.

La saisie a un fond gris sur toute la largeur, ajusté aux lignes saisies ; le
statut coloré reste en dessous. Dans le TTY interactif (entrée et sortie terminal),
`Entrée` (CR) envoie, tandis que le LF distinct produit par `Shift+Entrée` ou
Ctrl-J insère une ligne à la position du curseur. `Option+Entrée` ainsi que
`Échap`, puis `Entrée` sont ignorés. Gauche/Droite se déplacent d'un caractère
UTF-8 entier et Option+Gauche/Droite d'un mot (suite non blanche). Ctrl-A/E vont
au début/à la fin de la ligne logique ; Ctrl-U/K effacent jusqu'à ces limites sans
supprimer le LF. Ctrl-W ou Option+Retour arrière effacent le mot précédent,
Option-D le suivant, et Ctrl-Y réinsère le dernier fragment ainsi supprimé. Le
retour arrière ordinaire efface seulement le caractère précédent et ne remplace
pas ce registre. Hors double-TTY, CR et LF gardent l'envoi historique et ces
nouveaux contrôles d'édition sont inertes. Haut/Bas rappelle les saisies émises
avec succès, place le curseur à leur fin et restaure le texte et la position du
brouillon courant après la plus récente. Le mode clavier enrichi est temporaire ;
sa désactivation sous `TERM=dumb` n'empêche pas la distinction CR/LF du double-TTY.
Cet historique est volatile, limité à l'ouverture attach courante, à 100 entrées
et à 1 Mio ; il est vide après réouverture. Le redimensionnement conserve le texte
entier de la saisie. `NO_COLOR` ou `TERM=dumb` désactive les couleurs ; `TERM=dumb` conserve
aussi le clavier traditionnel sans activation enrichie, et une sortie redirigée
reste sobre.

Dans attach sur un **Codex géré**, `/model gpt-5.6-terra medium` choisit le modèle
et l'effort pour les prochains tours du **même fil** : aucun redémarrage, aucune
mission cachée, aucun historique supprimé. Les deux valeurs sont obligatoires
et doivent être annoncées ensemble par le catalogue du Codex installé. Un refus
ne change rien ; une issue inconnue n'est pas une confirmation. Le statut reflète
ensuite le réglage attesté par Codex. Ce contrôle nécessite daemon et wrapper à
jour et Codex avec `thread/settings/update` (recette : 0.153.4). Il ne modifie pas
la définition de lancement figée : une relance Bridget recharge celle-ci.
Pour une session **interactive** `bridget codex`, utiliser le `/model` de la TUI
Codex elle-même ; les autres pilotes refusent explicitement ce contrôle attach.

### Skill, MCP ou CLI ?

La skill est le mode d'emploi ; MCP exécute les outils structurés de communication.
Les agents privilégient MCP pour envoyer, répondre, consulter et annuler leurs
demandes. Si les outils sont différés, ils découvrent le catalogue avant de passer
au shell. Un refus de socket dans le shell sandboxé n'est pas une preuve de panne
MCP. Les réponses finales des équipiers gérés peuvent être relayées par le wrapper ;
la TUI humaine, elle, exige une réponse liée explicite. Ne pas doubler les deux.

En 094, un agent peut agir sur **son propre état** par MCP, sans champ cible :
renommer son affichage (jamais son UUID), activer DND de 1 seconde à 7 jours
(`60m` par défaut) ou le lever, changer/réinitialiser son domaine et déclarer son
runtime. Seul `off` lève DND ; une échéance déjà expirée est refusée sans mutation
ni prolongation inventée. Une réponse corrélée valide reste livrable pendant DND.
Le domaine applique le même canon technique ASCII que le CLI ; son reset revient
au domaine réellement dérivé, y compris après redémarrage. La persistance
du domaine est confirmée séparément de son application en mémoire : une erreur
`domain_persistence_failed` interdit d'annoncer une durabilité ou un rollback.
`bridget_runtime` déclare un modèle/effort avec source `Declared` ; contrairement
à `/model` dans attach ou la TUI, il ne sélectionne rien chez le fournisseur.

`bridget_status` expose une santé assainie ; un inventaire indisponible donne un
compte inconnu, pas zéro. `bridget_control_status` lit l'état, le compte de
décisions ouvertes et, sur demande, 0 à 50 événements. Une panne du store reste
une indisponibilité. La portée technique d'un client ne constitue pas à elle
seule une authentification du daemon.

Le lancement, l'arrêt et la relance restent des commandes CLI explicitement
autorisées, pas des outils MCP de supervision. Toujours lire le reçu et le profil
effectif : connecté ne signifie pas autorisé à écrire. Les outils du guichet encore
annoncés sont une façade vers un service extérieur, non une dépendance pour
communiquer. Les artefacts restent inertes et n'ouvrent aucune interface graphique.

Les opérations de flotte, les décisions `control`/`inbox`, les hooks, la migration,
`reprise`, `reaper`, le daemon et le terminal restent humaines ou internes, avec
leur motif précis dans l'[inventaire exhaustif](skills/bridget/references/commandes.md).
Un ancien serveur MCP vivant conserve son binaire et son catalogue : utiliser le
mécanisme natif du client s'il est connu, sinon faire rouvrir la session par
l'humain. Les garanties de domaine exigent aussi que client et wrapper
coopératifs soient réellement chargés. Ne pas inventer de commande de
rechargement ni interrompre une conversation pour actualiser les outils.

Les contenus référencés conservent bytes, provenance et accès. Un document HTML
est du contenu inerte : aucun rendu, script ou navigateur dans le noyau.

## Construire sans toucher à l'installation existante

Pour les builds courants, utiliser `make build`, `make release`, `make test` ou
`python3 scripts/build.py cargo …` : l'entretien automatique conserve jusqu'à
10 Gio de caches récents, purge les caches anciens après 7 jours et préserve
les profils utilisés. `make clean-builds DRY_RUN=1` permet une simulation.
Un appel direct à Cargo ne déclenche pas cet entretien. Voir le
[guide des caches de compilation](docs/build-cache.md) pour les réglages et limites.

Répertoire de réalisation :
`/Users/user/projets/bridget/.worktrees/089-communication-core`.

Rust est épinglé dans rust-toolchain.toml. Construire avec Cargo disponible :

```sh
cd /Users/user/projets/bridget/.worktrees/089-communication-core
PATH=/Users/user/.cargo/bin:$PATH cargo build --locked -p bridget-daemon
```

Choisir une racine d'état privée NEUVE, courte et absolue. Ne pas pointer vers
l'ancien cache. Le HOME fournisseur n'est pas remplacé en production : il porte
les abonnements. Aucun repli vers une API facturée n'est ajouté.

La base `bridget.db` est en journal WAL : `bridget.db-wal` et `bridget.db-shm` l'accompagnent tant
que le daemon tourne, avec les mêmes droits privés. Une copie à chaud de la base seule est
incomplète ; copier les trois fichiers ou utiliser `VACUUM INTO`.

```sh
bridget_state=$(mktemp -d /tmp/bgcore.XXXXXX)
export BRIDGET_HOME="$bridget_state"
export BRIDGET_SOCKET="$BRIDGET_HOME/bridget.sock"
/Users/user/projets/bridget/.worktrees/089-communication-core/target/debug/bridget daemon
```

Ce daemon reste au premier plan. Les autres terminaux doivent recevoir EXACTEMENT
ces variables, et utiliser CE binaire. Aucun launchd, tunnel, profil fournisseur
ou processus de l'ancienne flotte n'est remplacé. Le
[guide d'installation indépendante et de retour arrière](docs/communication-installation.md)
décrit le paquet à liste de sources fermée, ses empreintes et les préconditions
de compte. Ce n'est pas une autorisation de bascule de la flotte.

## Observer et partager, sans bloquer le travail

Pour les agents : la [skill Bridget](skills/bridget/SKILL.md) explique quel outil
choisir ; les [recettes pratiques](skills/bridget/references/commandes.md#recettes-pratiques--observer-prolonger-partager)
détaillent l'abonnement, sa prolongation/annulation, le partage avec un relecteur
et les alertes de fichiers. Aucun nouveau profil ni formulaire à remplir.

Bridget sait transmettre un extrait exact du journal et prévenir un agent d'un
fait observable. Ces fonctions sont indépendantes de T3 et du service compagnon : elles
n'ajoutent ni mandat, ni validation obligatoire, ni verrou sur les fichiers.
Remplacer les UUID d'exemple par ceux de `bridget agents --json` :

```sh
bridget journal 'UUID_SOURCE' --tail 50 --to 'UUID_RELECTEUR' --reply
bridget events types
bridget events sub turn_ended --agent 'UUID_SOURCE' --once --ttl 1800
bridget events sub file_collision --file '/projet/src/*' --ttl 3600
bridget events list
bridget events unsub 'ID_ABONNEMENT'
```

Les mêmes opérations existent en MCP : `bridget_journal` et `bridget_events`.
`reply` suit une réponse attendue ; un abonnement signale un fait et ne résout
jamais cette demande. Une fin de tour ne prouve ni succès, ni fin de mission.
Après un abonnement confirmé, l'agent peut rendre la main : pas de sondage
continu. L'expiration met fin à l'abonnement sans envoyer d'alerte de délai.
Il n'existe pas de commande de prolongation : créer un remplacement confirmé
puis retirer l'ancien, en conservant ses filtres et en calculant la nouvelle
échéance. Ce remplacement peut brièvement se chevaucher ; voir les recettes.
L'extrait contient agent, séquences, entrées et notices ; 50 entrées par défaut,
200 maximum et moins de 64 Kio. `--from-seq` permet de reprendre à `next_seq`.
Une entrée trop grande reste explicitement incomplète ; aucun résumé inventé.

Types d'événements : `turn_ended`, `permission_required`, `file_written`,
`file_collision`. `events types` indique pour chacun sa disponibilité et les
UUID des sources compatibles. Seule une connexion principale attestée peut
annoncer ses capacités ; un ancien wrapper sans cette annonce n'est pas
considéré compatible. Une souscription visant un agent inconnu, indisponible
ou sans événement compatible est refusée immédiatement. Sans filtre d'agent,
la couverture reste limitée aux sources déclarées, pas à toute la flotte.

Filtres par UUID et motif de chemin (`*` uniquement). Le
propriétaire est l'identité active du client CLI/MCP, jamais un paramètre libre.
L'abonnement concerne les faits reçus par le daemon après sa création, sans
relecture de l'historique ; un fait déjà produit mais encore en transit peut
donc le déclencher. Ce n'est pas une synchronisation des horloges des agents.
Les fins de tour corrélées viennent des pilotes gérés ACP, Claude stream-json,
Codex app-server et de l'adaptateur T3. Pour T3, seules les fins explicites
`completed`, `error` et `interrupted` sont prises en compte, même sans texte
assistant, si l'origine du tour est attestée. Un état idle ou une déconnexion
ne constitue jamais une fin de tour. Les
permissions sont signalées lorsque le pilote les journalise, éventuellement
après leur traitement automatique : `permission_required` signifie « demande
observée », pas nécessairement « agent encore en attente d'une autorisation ».
Les écritures viennent d'outils structurés connus et terminés avec succès :
Claude/ACP Write, Edit et équivalents reconnus, Codex fileChange completed.
Ni lecture, ni commande shell libre, ni surveillance universelle du disque.

T3 fournit des événements `approval.requested` et certaines écritures Codex
attestées. **Les écritures de Claude dans T3 ne sont pas couvertes** : la
projection actuelle perd le chemin `input.file_path` et peut annoncer un outil
terminé sans résultat confirmant l'écriture. Le pont n'annonce donc pas
`file_written` pour ces fils. Cela ne change pas les capacités des wrappers
structurés Bridget utilisés sans T3.

Exemple : « préviens-moi quand Atelier-3D termine » devient un abonnement
ponctuel `turn_ended` sur son UUID, après vérification de sa présence dans
`events types`. La notification signifie **« tour terminé »**, jamais « projet
Atelier-3D fini ». Un refus d'abonnement ou une interruption doivent être
rapportés ; ils ne valent pas une surveillance active.

La couverture T3 reste bornée : `latestTurn` ne décrit que le dernier tour,
donc plusieurs fins entre deux lectures peuvent être manquées. La fenêtre
d'activités est limitée à 500 entrées avant compression côté T3, et la
projection à 12 chemins par activité. Une lacune détectée est signalée ; le
pont ne reconstruit pas les faits absents et ne promet pas d'exhaustivité.

Une collision est un **risque** : deux auteurs, même hôte et même chemin absolu
normalisé, moins de 30 secondes. Aucun accès au disque, verrou ou fusion Git ;
les alias par symlink, différences de casse et hôtes mal nommés restent des
limites. Les faits ne sont pas fédérés entre daemons. Une lacune détectée
(saturation ou projection T3 incomplète) donne une notice distincte, sans
consommer `once`. `events list` expose `facts_lost` pour les pertes quantifiées
et `observation_gaps` pour les lacunes, même de quantité inconnue ; chaque
abonnement expose ses propres compteurs. Ces compteurs concernent l'instance
courante du daemon, distinctement des notifications perdues à la remise.

Les abonnements ont une durée par défaut d'une heure, un maximum de sept jours,
une limite de 16 par agent et 128 au total. La fermeture du client auxiliaire
les conserve. Leur trace est persistée : après redémarrage du daemon, les
abonnements non expirés apparaissent dans `events list` en état `interrupted`,
sans reprendre automatiquement. Un avertissement est prévu au retour du
propriétaire ; consulter la liste reste nécessaire si cette remise échoue.
Supprimer l'ancien abonnement puis en créer un nouveau reprend uniquement sur
les faits futurs, sans rejouer la période manquante.

Pendant la vie du daemon, une source perdue fait passer l'abonnement à
`source_unavailable`. Son retour peut le rendre `active`, avec une notice de
reprise ; les changements de couverture sont signalés, sans rattrapage des
lacunes. Ces états sont consultables dans `events list` et les notices suivent
les mêmes limites de remise que les autres notifications.

`once` consomme le déclenchement même si
sa remise échoue. Agent absent, mode ne-pas-déranger, saturation : pas de
livraison durable. Les reçus exposent `notifications_lost`, `evicted_writes`
et les listes `suppressed_total` (limite de cinq notifications/s/abonnement).
La file de remise contient 64 messages ; budget d'une seconde par tentative,
expiration après cinq secondes. Le relais de faits est borné à 256 éléments,
le cache de fichiers à 4096 ; les pertes à la source sont journalisées sous
`observation_gap`. Les tours issus de notifications Bridget ne redéclenchent
pas d'observations ; une action manuelle ultérieure reste un nouveau travail.
La disponibilité réelle dépend des versions chargées et des capacités
annoncées : cette documentation ne constitue pas une preuve de déploiement ni
de notification reçue dans un vrai fil.

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
La carte de tests distingue leurs états.

Bridget ne remplace ni un orchestrateur de tâches, ni une revue humaine, ni une
preuve de travail terminé. L'adaptateur T3 reste séparé du cœur de communication ;
A2A et la greffière ne sont pas requis pour ces observations. Aucun framework
d'orchestration supplémentaire n'est introduit.
