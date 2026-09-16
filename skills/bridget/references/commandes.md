# Commandes Bridget et accès 094–100

Lire cette référence pour toute demande qui dépasse l'annuaire, l'envoi et la
réponse liée de base. Elle décrit le contrat étendu à la session 100 ; elle ne prouve
ni que le binaire installé correspond à cette version, ni qu'un serveur MCP déjà
ouvert a rechargé son catalogue.

## Inventaire stable des commandes

Décisions possibles : **MCP exposé**, **Équivalence MCP**, **CLI humain** ou
**Interne**. Une équivalence réutilise un outil existant au lieu de créer un
synonyme. Chaque ligne ci-dessous correspond à une racine acceptée par l'aide ou
le répartiteur CLI ; la première colonne reste volontairement stable pour le test
de couverture.

| Commande racine | Décision 094 | Accès ou équivalence | Acteur, usage et motif |
|---|---|---|---|
| `codex` | CLI humain | Aucun outil MCP de lancement | Alias interactif annoncé par l'aide et résolu dans le registre actif ; ouvre la TUI fournisseur au premier plan. Sa présence dans l'aide ne garantit pas que le registre ou le fournisseur soit disponible. |
| `claude` | CLI humain | Aucun outil MCP de lancement | Alias interactif résolu dans le registre actif ; session 097 : pseudo-terminal possédé par le wrapper, sans tmux, présence `claude_pty`, refus sans terminal ; les permissions restent humaines, aucun bypass implicite. |
| `gemini` | CLI humain | Aucun outil MCP de lancement | Alias interactif résolu dans le registre actif ; voie tmux : un pane attesté est requis, sinon refus au lancement (session 097). |
| `gclaude` | CLI humain | Aucun outil MCP de lancement | Alias interactif de type `claude` : même pseudo-terminal que `bridget claude` (session 097). |
| `--` | CLI humain | Aucun outil MCP de lancement | Lance explicitement une commande d'agent personnalisée reconnue par le registre ; ne contourne ni le registre ni les permissions du fournisseur. |
| `daemon` | CLI humain | Aucun | Démarre l'autorité Bridget au premier plan. Un agent ne démarre pas un nouveau daemon pour réparer un envoi ou changer de namespace. |
| `mcp` | Interne | Serveur stdio, pas outil auto-appelable | Point d'entrée lancé par le client MCP configuré. Il n'est ni un second daemon ni une permission globale. |
| `attach` | CLI humain | Aucun outil MCP de terminal | Observe le journal et permet une saisie humaine dans un double-TTY ; ce n'est ni la TUI fournisseur ni un écran d'approbation. |
| `journal` | MCP exposé (100) | `bridget_journal` | Extrait exact borné du journal ; `to` partage, `reply` suit une réponse. Source UUID, séquences, lacunes et reprise explicites. |
| `events` | MCP exposé (100) | `bridget_events` | types/sub/list/unsub, propriétaire attesté, once/TTL ; notification d'un fait, sans obligation métier ni exécution de script. |
| `federate` | CLI humain | Aucun outil MCP de fédération | Réutilise, installe, observe ou retire une liaison SSH persistante via le gestionnaire 095 embarqué. Le statut reste local ; une mutation appartient à l'humain et conserve les gardes SSH/natives. |
| `t3` | CLI humain | Aucun outil MCP d'administration ; les fils exposés se joignent par `bridget_send` | Installe, observe, retire ou sert le pont t3code (session 098) : session émise par le CLI officiel `t3`, un agent par fil, remise par `thread.turn.start`, réponse liée par rang FIFO ; t3code n'est jamais modifié. |
| `artifact` | Équivalence MCP | `bridget_read_artifact` pour `artifact read`; publication par `bridget_publish_artifact` | Lit des octets par références et bornes, sans chemin libre ni exécution. La publication structurée n'a pas de commande CLI jumelle. |
| `spawn` | CLI humain | Aucun outil MCP de supervision | Crée un équipier géré avec persistance, répertoire et posture explicites ; l'autorité propriétaire n'est pas déléguée au modèle. |
| `stop` | CLI humain | Aucun outil MCP de supervision | Arrête un équipier géré désigné ; effet de cycle de vie réservé à l'humain. |
| `relaunch` | CLI humain | Aucun outil MCP de supervision | Relance un équipier arrêté avec sa définition figée ; ne change pas sa posture ni ses droits. |
| `decommission` | CLI humain | Aucun outil MCP de supervision | Retire un équipier de la flotte visible en conservant l'historique ; décision propriétaire. |
| `adopt-stopped` | CLI humain | Aucun outil MCP de supervision | Importe explicitement des agents arrêtés dont l'historique géré est prouvé ; décision de flotte. |
| `send` | MCP exposé | `bridget_send` | Envoi ou réponse corrélée ; viser l'UUID attesté et conserver exactement la clé de rejeu et l'enveloppe. |
| `reply` | Équivalence MCP | `bridget_send` avec `in_reply_to` | Commodité CLI vers le dernier expéditeur ; l'outil structuré évite l'ambiguïté quand plusieurs demandes coexistent. |
| `cancel` | MCP exposé | `bridget_cancel` | Annule uniquement une demande suivie de l'appelant ; n'arrête pas l'agent et ne prouve pas la fin de la mission. |
| `requests` | Équivalence MCP | `bridget_ledger` avec `view=requests` | Consulte les demandes de l'appelant, ou la projection globale autorisée ; aucun outil synonyme n'est créé. |
| `rename` | MCP exposé | `bridget_rename` | Change seulement le nom d'affichage de l'identité propre ; UUID, instance, historique et rejeux restent liés à la même identité. |
| `control` | Équivalence MCP pour la lecture ; CLI humain pour les mutations | `bridget_control_status` remplace `control status`; `pause`, `resume`, `budget` et `posture` restent sans MCP | Le statut est observable ; les mutations appartiennent au référent humain et ne sont pas déléguées par 094. |
| `inbox` | CLI humain | `bridget_control_status` ne donne que le compte ouvert, pas la liste ni la résolution | Liste et tranche des décisions humaines. Exposer `resolve` au modèle confondrait observation et autorité de référent. |
| `runtime` | MCP exposé | `bridget_runtime` | Déclare le modèle/effort propres avec source `Declared`; ne sélectionne aucun modèle chez le fournisseur. |
| `identity` | CLI humain | Aucun | `identity migrate --dry-run|--apply` planifie ou applique une migration sur chemins locaux explicites ; `--apply` exige le daemon arrêté et peut écrire des sauvegardes. |
| `domain` | MCP exposé | `bridget_domain` | Change ou réinitialise le domaine propre avec le même canon technique ASCII que le CLI ; aucune cible, socket ou chemin n'est accepté du modèle. |
| `dnd` | MCP exposé | `bridget_dnd` | Active la disponibilité propre pour une durée bornée ou la lève uniquement avec `off` ; une réponse corrélée valide reste livrable. |
| `install-hooks` | CLI humain | Aucun | Modifie la configuration Claude après sauvegarde, ou retire l'entrée avec `--remove`; les sessions déjà ouvertes ne sont pas modifiées. |
| `who` | MCP exposé | `bridget_who` | Annuaire attesté, éventuellement filtré par domaine ; absence de projection disponible n'est pas une liste vide. |
| `agents` | Équivalence MCP | `bridget_who` | Variante CLI orientée machine (`--json`) du même annuaire ; aucun doublon MCP. |
| `status` | MCP exposé | `bridget_status` | Santé et inventaire assainis depuis le daemon ; aucun accès direct à la base, à la socket ou aux instances. |
| `ledger` | MCP exposé | `bridget_ledger` | Projection bornée des messages et demandes ; une coupure reste une indisponibilité, jamais un historique vide inventé. |
| `reprise` | CLI humain | Aucun outil MCP global | Agrège Git, pin, chemins locaux et repli base, et peut écrire une carte avec `--write`. Les observations sûres séparées restent `who`, `ledger`, `status` et `control_status`. |
| `reaper` | CLI humain | Aucun outil MCP | `reaper report` lit processus, fichiers et descripteurs locaux, puis écrit `observations.jsonl`; malgré son nom, ce n'est pas une simple lecture daemon. Il n'effectue aucun kill. |
| `version` | CLI humain | Aucun | Affiche la version du binaire invoqué ; ne prouve pas la version d'un serveur MCP déjà vivant. |
| `help` | CLI humain | Aucun | Affiche l'aide locale du binaire invoqué. |
| `managed-bootstrap` | Interne | Aucun | Couture de démarrage des processus gérés, appelée par le superviseur ; ce n'est pas une promesse de commande utilisateur. |
| `managed-wrapper` | Interne | Aucun | Lance un pilote géré depuis une définition figée fournie par le superviseur ; ne pas l'appeler directement. |
| `guichet` | Interne | Aucun outil Bridget public | Dépôt interne de requêtes de service pour des consommateurs historiques. Ce chemin n'est pas une promesse nominale ; les outils Maicie éventuels restent une façade séparée. |
| `hook` | Interne | Aucun | Entrée fail-soft appelée par les hooks Claude (`claude-runtime`, `claude-statusline`) ; elle lit stdin et n'est pas une commande conversationnelle. |
| `discover` | Interne | Utiliser `bridget_who` ou `who` | Alias de répartiteur vers l'annuaire, non annoncé comme interface publique stable. |
| `--version` | CLI humain | Alias exact de `version` | Alias orthographique local. |
| `-v` | CLI humain | Alias exact de `version` | Alias court local. |
| `--help` | CLI humain | Alias exact de `help` | Alias orthographique local. |
| `-h` | CLI humain | Alias exact de `help` | Alias court local. |

Le répartiteur refuse explicitement les anciennes entrées `ui`, `cleanup`,
`managed-runtime-wrapper`, `managed-runtime-stop`, `project-runtime` et
`project-round`. Elles ne sont ni des commandes promises ni des replis vers un
programme homonyme. Les autres alias d'agents sont chargés dynamiquement depuis
le registre : ne jamais déduire leur disponibilité d'un nom de fournisseur.

## Catalogue MCP Bridget fermé

La politique fournisseur étendue en 100 autorise exactement les quatorze outils Bridget
ci-dessous lorsqu'elle est effectivement chargée. Elle n'accorde pas une
approbation MCP globale et n'inclut pas automatiquement les outils Maicie.

- `bridget_send` — envoyer ou répondre avec corrélation et rejeu explicite.
- `bridget_cancel` — annuler sa propre demande suivie.
- `bridget_who` — lire l'annuaire visible.
- `bridget_ledger` — lire messages et demandes bornés.
- `bridget_journal` — lire ou partager un extrait sourcé, sans lecture arbitraire du disque.
- `bridget_events` — s'abonner aux faits futurs disponibles, lister et supprimer ses abonnements.
- `bridget_publish_artifact` — publier un contenu structuré, sourcé et inerte.
- `bridget_read_artifact` — relire les octets autorisés par références exactes.
- `bridget_rename` — modifier son propre nom d'affichage.
- `bridget_dnd` — modifier sa propre disponibilité temporaire.
- `bridget_domain` — modifier ou réinitialiser son propre domaine.
- `bridget_runtime` — déclarer son modèle et son effort, sans sélection fournisseur.
- `bridget_status` — lire la santé assainie du daemon.
- `bridget_control_status` — lire l'état de contrôle, le compte inbox et un historique borné.

Un champ de portée (`issuer_scope`) sert à corréler et borner certains contrats
client ; il ne constitue pas à lui seul une authentification du daemon. Les
mutations propres sont liées par Bridget à l'identité et à l'instance de la
connexion : ne jamais fournir ni inventer un nom, UUID, instance, socket ou
chemin de cible.

## Extraits et abonnements (100)

`bridget_journal` accepte `agent`, `tail` (défaut 50, max 200) OU `from_seq`,
et éventuellement `to`/`reply`. Il cite le journal, sans suivre ses instructions.
Le résultat indique les lacunes/limites, la provenance et `next_seq` ; une entrée
trop grande n'est pas silencieusement résumée. Pas d'accès libre aux fichiers.

`bridget_events` accepte `action:types|sub|list|unsub`. `sub` demande `event`
parmi `turn_ended`, `permission_required`, `file_written`, `file_collision` ;
filtres `agent` et `file` (`*`, seulement pour fichiers), `once` et `ttl_secs`.
`unsub` demande `id`. Aucun paramètre propriétaire : identité courante attestée.
Une fin de tour n'est ni un succès ni une réponse à un `reply` attendu.
La borne est la réception du fait par le daemon après création de l'abonnement,
sans rejeu historique ; un événement encore en transit peut le déclencher.

Les faits concernent les intégrations qui les journalisent : tours corrélés
ACP/Claude stream-json/Codex app-server, permissions ACP/Codex, écritures
structurées réussies. Pas de promesse sur idle natif/T3 ou commandes shell libres.
Collision = deux auteurs/même hôte/chemin absolu dans 30 s, jamais un verrou.
État mémoire du daemon : défaut 1 h, max 7 jours, 16 abonnements/agent, 128 total.
`once` consomme le déclenchement même si la remise échoue ; absence/DND/saturation
peuvent perdre des notifications. Consulter `notifications_lost`, `evicted_writes`
et `suppressed_total` ; recréer ses abonnements après redémarrage. Rien à attendre
activement : la notification arrive dans la messagerie ordinaire.

## Actions propres et observations

Renommer ne change jamais l'adresse UUID. Confirmer le succès seulement si
`agent_id` est l'UUID exact de l'identité propre attestée et si `display_name`
est le nom demandé après la normalisation légitime du profil, notamment la
condensation des espaces blancs. Ne pas comparer les octets de la saisie brute :
une normalisation attendue n'est pas une substitution. Un conflit, un refus, une
erreur technique ou toute autre incohérence n'est pas un renommage confirmé.

```json
{"name":"bridget_rename","arguments":{"display_name":"Équipe B"}}
```

DND accepte `on` ou `off`. Avec `on`, `duration` est une chaîne en secondes,
minutes ou heures (`90s`, `30m`, `2h`) ; sans durée, le défaut est `60m`. La
durée totale doit rester entre 1 seconde et 7 jours (604800 secondes). `off`
refuse une durée et correspond seul à `None`, donc à la levée. Une échéance
interne déjà expirée est refusée sans mutation : elle ne devient ni un ACK `on`
transformé en `off`, ni une prolongation d'une seconde. DND ne bloque pas une
réponse valide liée à une demande suivie.

```json
{"name":"bridget_dnd","arguments":{"mode":"on","duration":"30m"}}
{"name":"bridget_dnd","arguments":{"mode":"off"}}
```

Le domaine est un libellé technique de 1 à 100 octets ASCII, limité aux lettres,
chiffres, `-` et `_`, avec exactement la même validation que le CLI et le daemon.
Il est soit remplacé, soit réinitialisé ; les deux formes sont mutuellement
exclusives. Bridget confirme d'abord l'application en mémoire puis la persistance
atomique existante. `domain_persistence_failed` signifie que la mémoire a changé
mais que la durabilité n'est pas confirmée : ne pas annoncer un rollback ni un
succès durable. `reset:true` retire la surcharge et revient immédiatement au vrai
domaine dérivé du projet ; le wrapper conserve ce résultat après redémarrage du
daemon au lieu de réappliquer l'ancienne surcharge.

Le client sérialise localement mutation et persistance avec la lecture/réapplication
du wrapper, par identité UUID et à proximité de sa socket cliente. Le stockage
reste volontairement local : une socket daemon fédérée par SSH ne déplace pas le
fichier de domaine vers la machine distante. Cette garantie suppose que client et
wrapper coopératifs sont tous deux chargés ; un ancien writer ou wrapper vivant
n'en bénéficie pas et ne doit pas être annoncé comme actualisé.

```json
{"name":"bridget_domain","arguments":{"domain":"documentation"}}
{"name":"bridget_domain","arguments":{"reset":true}}
```

Le runtime est une **déclaration**, source `Declared`. Il renseigne l'annuaire ;
il ne sélectionne pas un modèle et n'a pas la même fonction que `/model` dans
attach ou dans la TUI fournisseur.

```json
{"name":"bridget_runtime","arguments":{"model":"gpt-5.6-terra","effort":"medium"}}
```

Les statuts sont lus littéralement. `agent_count` reste `null` si l'inventaire
n'est pas disponible ; hôte, build ou état absent reste inconnu. Une panne du
store de contrôle est une indisponibilité, jamais un compte inbox égal à zéro.
L'historique de contrôle est désactivé par défaut et borné de 0 à 50 entrées.

```json
{"name":"bridget_status","arguments":{}}
{"name":"bridget_control_status","arguments":{"history_limit":20}}
```

## Artefacts inertes

Publier un artefact exige une clé d'idempotence, un type, un titre, des données,
au moins une source et une raison. Conserver le reçu et transmettre ses
`artifact_ref`/`version_ref` plutôt que recopier un HTML en Markdown. HTML reste
du contenu inerte : ni rendu, ni JavaScript, ni navigateur, ni réseau.

```json
{"name":"bridget_publish_artifact","arguments":{"idempotency_key":"synthese-2026-09-07-v1","kind":"table","title":"Synthèse contrôlée","payload":{"columns":["état"],"rows":[["validé"]]},"sources":[{"source_kind":"agent_computed","locator":"analyse-locale","citation":"Synthèse calculée depuis les faits Bridget"}],"publication_reason":"initial"}}
```

Relire utilise uniquement les références reçues, un type fermé, un offset et
une limite de 1 à 16384 octets ; ne pas convertir un chemin local en référence.

```json
{"name":"bridget_read_artifact","arguments":{"version":1,"artifact_ref":"<artifact_ref_reçu>","version_ref":"<version_ref_reçu>","kind":{"kind":"manifest"},"offset":0,"limit":16384}}
```

## Administration de la fédération SSH (095–096)

Le binaire 096 fournit `bridget federate URL|status|remove` en embarquant le
script canonique ; aucune recherche de script adjacent n'est effectuée. La
distribution conserve aussi `scripts/federate-ssh.sh install|status|remove` :
LaunchAgent utilisateur sur macOS, unité `systemd --user` sur Linux. Cette
administration reste une action humaine explicite, pas un outil MCP. Le runner
installé est autonome ; aucune ancienne installation Bridget n'est requise.
Voir `docs/federation-services.md` dans la distribution pour les paramètres,
la clé d'hôte explicite et les limites de disponibilité.

La liaison publie la socket du maître par SSH inverse sur le client distant :
même annuaire et mêmes messages, pas deux bases répliquées. Un client fédéré
ne démarre pas de daemon de secours. `spawn` reste exécuté sur le maître ; pour
un agent travaillant sur le serveur, lancer le CLI fournisseur Bridget sur ce
serveur. Un service actif seul ne prouve pas la livraison : vérifier `who` et
un échange dans les deux sens. Ne jamais démarrer ou retirer cette liaison
pour contourner un refus MCP ou une restriction du shell.

Une destination suit la forme fermée `ssh://[utilisateur@]hôte[:port]`, avec
port 22 par défaut. Un alias DNS peut reconnaître sans mutation une IP déjà
enregistrée, mais ne remplace jamais cette cible ni sa clé d'hôte. Le statut
global n'utilise ni DNS ni SSH. Le retrait par alias DNS demande `--label` hors
double-TTY ; en double-TTY seulement, il peut exiger la confirmation littérale
du label, de l'hôte enregistré et du port affichés. Une ambiguïté, une résolution
hors budget ou des paramètres explicites divergents sont des refus sans mutation.

## Adaptateur t3code (098)

`bridget t3 install [--no-service]` émet une session dédiée par
`t3 auth session issue --subject bridget --label bridget-<id> --ttl 30d --json`,
la conserve en 0600 sous `<état Bridget>/t3code/`, et enregistre un
LaunchAgent (`com.bridget.t3`) ou une unité `systemd --user` qui lance
`bridget t3 serve`. `status` lit le serveur (`~/.t3/userdata/server-runtime.json`,
boucle locale seule, PID vivant), la session, le service et le dernier état
publié par le pont. `uninstall` retire le service, révoque la session par son
identifiant (à défaut par son libellé) et efface l'état. Aucune de ces commandes
n'écrit dans les fichiers de t3code.

Le pont présente chaque fil vivant doté d'une session fournisseur comme un
agent : identité stable dérivée de l'identifiant du fil, type = fournisseur
(`claude`, `codex`, …), transport `t3code`, mode `cli`, nom humain = titre du
fil (refusé si le nom existe déjà). Une remise attend un fil sans tour actif
(borne `BRIDGET_T3_TURN_WAIT_SECS`, 120 s), envoie `thread.turn.start` avec
`commandId` = identifiant du message Bridget (t3code déduplique), puis renvoie
comme réponse liée le texte du tour de même rang que le message, une fois le
tour clos. Un 401 déclenche un renouvellement unique ; un second 401 est un
échec explicite (`auth_failed`) visible par `status`. Le journal du fil est
projeté pour `attach` sans rejouer l'historique antérieur à l'installation.

## Version active et rechargement

Comparer le catalogue réellement retourné par le client avec les douze noms
ci-dessus avant d'annoncer la disponibilité de 094. Un binaire installé et un
serveur MCP vivant sont deux processus distincts : une ancienne session garde
son ancien binaire et son ancien catalogue. Les garanties de domaine exigent en
plus que les versions coopératives du client et du wrapper soient effectivement
chargées. Utiliser uniquement le mécanisme natif de rechargement du client
lorsqu'il est connu ; sinon signaler que la session doit être réouverte par
l'humain. Ne pas inventer de commande de reload, ne pas tuer une conversation et
ne pas relancer silencieusement un fournisseur.
