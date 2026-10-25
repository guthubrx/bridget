# Installer Bridget communication à côté de l'existant

[English](communication-installation.md) · [Français](communication-installation.fr.md)


## Installer ou mettre à jour en une commande

Dernière release publiée, déjà compilée :

```sh
curl -fsSL https://cartae.app/bridget/install.sh | sh
```

Dernier code de `main`, compilé sur la machine :

```sh
curl -fsSL https://cartae.app/bridget/install.sh | sh -s -- --source
```

Release précise :

```sh
curl -fsSL https://cartae.app/bridget/install.sh | sh -s -- --version v0.4.1
```

Le même script est disponible directement sur GitHub :

```sh
curl -fsSL https://raw.githubusercontent.com/guthubrx/bridget/main/scripts/install.sh | sh
```

Pour lire le script avant de l'exécuter, téléchargez-le dans un fichier, examinez
son contenu puis lancez `sh` sur ce fichier. HTTPS vérifie le serveur ; le fichier
SHA-256 vérifie l'intégrité de l'archive. Ce n'est pas une signature indépendante.

Lancez la commande avec votre utilisateur habituel, sans sudo. Le mode release
demande curl, tar et sha256sum ou shasum. La version 0.4.1 fournit Linux x86-64
et Mac Apple Silicon. Une plateforme sans paquet est refusée ; aucune compilation
n'est déclenchée sans l'option `--source`.

Le mode source demande Git, Python 3.11+, un compilateur C et ses outils de
développement (Xcode Command Line Tools sur Mac, chaîne C sur Linux).
Il installe Rust par le script officiel rustup si nécessaire. Il utilise la
toolchain et Cargo.lock du dépôt et le wrapper de compilation existant.
Un checkout dédié reste dans `.local/state/bridget-install/source` pour garder
une identité de cache stable entre les mises à jour. Le cache suit le budget
commun de 10 Gio et 7 jours. Les intermédiaires sont purgés après installation.
Un checkout de l'installateur modifié est conservé et refusé, jamais réinitialisé.
Un outil système absent produit une erreur claire, sans demande sudo.

Le programme est installé dans `.local/bin` du compte utilisateur. Le reçu et
le binaire précédent distinct sont conservés dans `.local/state/bridget-install`.
Une réinstallation identique conserve le précédent. Les profils Bash ou Zsh sont
sauvegardés avant d'ajouter le PATH. Dans le terminal courant :

```sh
. "$HOME/.local/share/bridget/env"
bridget --version
```

Sur une installation neuve, la commande installe et démarre un service utilisateur.
Elle attend une vraie réponse de Bridget avant d'annoncer le succès. Sur Mac,
launchd le relance à l'ouverture de session. Sur Linux, systemd le démarre au
démarrage du système et le conserve après déconnexion grâce au maintien de
session (lingering). Aucun terminal ne doit rester ouvert.

Sur Mac, l'utilisateur doit avoir une session graphique ouverte, même si
l'installation passe par SSH. Sur Linux, le gestionnaire systemd utilisateur doit
être disponible. Si nécessaire, l'installateur tente d'activer le maintien de
session sans demander de mot de passe. Un refus indique précisément le besoin
d'une autorisation administrateur. Ce n'est jamais annoncé comme un succès.

Le service s'appelle `app.cartae.bridget`. Un daemon ou une fédération qui répond
déjà est réutilisé sans interruption ni modification de sa définition. Une
fédération en panne ou une socket ambiguë est refusée ; aucun deuxième maître
n'est démarré. Les comptes et données sont conservés. Aucun worker, tunnel ou
processus T3 n'est relancé. Les services actifs gardent leur version chargée
jusqu'à une relance volontaire. Un service créé ici qui ne répond pas est
désactivé ; le programme et les données restent disponibles pour réparation.

Le PATH et le namespace choisi sont mémorisés pour les prochains terminaux.
Depuis votre projet, lancez `bridget claude` ou `bridget codex`. L'installation
et l'identification personnelle du fournisseur restent des prérequis distincts.
Pour rejoindre un autre maître, suivez le
[guide de fédération SSH](federation-services.fr.md).
Le binaire précédent ne suffit pas à restaurer une base.
Si le nettoyage échoue après une installation source, le message précise que
le programme est installé mais que le cache n'a pas été nettoyé. Cette erreur
ne signifie pas que l'ancien binaire est encore sur disque.
Une interruption peut laisser `.local/state/bridget-install/lock`. Consultez son
fichier `pid` et vérifiez qu'aucun installateur ne tourne avant de retirer ce
seul dossier de verrou. Ne supprimez ni les sources ni les données pour le libérer.

## Essai séparé depuis un checkout existant

Cette procédure ne remplace aucun binaire global, service, registre, profil de
fournisseur, skill globale ni base existante. Elle ne démarre pas le service compagnon ni GUI.
Les recettes Claude/GLM et le gate final restent soumis aux résultats consignés
dans la session 089 ; un paquet construit n'est pas une validation de compte.

## 1. Sources indépendantes et construction

Depuis le chantier actuel :

```sh
cd /Users/moi/Nextcloud/10.Scripts/64.bridget
package_parent=$(mktemp -d /tmp/bgpackage.XXXXXX)
package_parent=$(cd "$package_parent" && pwd -P)
bash scripts/package-089-core.sh "$package_parent/source"
cd "$package_parent/source"
shasum -a 256 -c SOURCE-MANIFEST.sha256
PATH=/Users/moi/.cargo/bin:$PATH BRIDGET_BUILD_ID="$(cat BUILD_ID)" \
  python3 scripts/build.py cargo build --locked --offline --release -p bridget-daemon
```

Rust 1.92.0 et les dépendances verrouillées doivent déjà être disponibles.
Un cache incomplet échoue hors ligne : aucune installation implicite. L'archive
contient les trois crates, les fixtures nécessaires, la skill, les deux scripts
SSH audités et cette documentation, pas les sources le service compagnon/UI/runtime.
BUILD_ID identifie la source ; le manifeste SHA-256 identifie les octets précis,
y compris un paquet de travail marqué `-dirty`. Sans BUILD_ID à la compilation,
le binaire d'une archive sans Git annonce honnêtement `unknown`.

## 2. Une instance privée, explicite

```sh
umask 077
bridget_state=$(mktemp -d /tmp/bgcomm.XXXXXX)
export BRIDGET_HOME="$bridget_state"
export BRIDGET_SOCKET="$bridget_state/bridget.sock"
bridget_bin="$package_parent/source/target/release/bridget"
"$bridget_bin" daemon
```

Conserver les trois valeurs affichées/résolues par le shell (`bridget_bin`,
`BRIDGET_HOME`, `BRIDGET_SOCKET`). Les reprendre **à l'identique** dans un autre
terminal, puis `"$bridget_bin" status` et `"$bridget_bin" agents --json --global`.
Ne pas refaire `mktemp` pour un client : il interrogerait une autre instance.
La racine doit être privée 0700, les fichiers privés 0600, sans lien symbolique.
Le chemin socket est court pour rester compatible macOS/Linux.

L'identité et l'annuaire sont interrogés sur la socket avec des budgets bornés.
Un inventaire inaccessible est une erreur explicite, pas une liste vide.
`status` ne lit pas SQLite côté client et ne prétend pas connaître un total des
messages absent du protocole ; `ledger --limit 20` consulte le maître.

L'annuaire de communication est local au projet par défaut (138). Le diagnostic
`agents --json --global` est volontaire et n'autorise aucun envoi extérieur.
Depuis un client autonome, donner la racine explicite du projet à consulter :
`"$bridget_bin" who --project-root /Users/moi/Nextcloud/10.Scripts/64.bridget`.
Sans racine attestée, le projet est inconnu : liste locale vide et avertissement,
pas panne d'inventaire. Les agents T3/wrapper héritent de leur contexte.
Pour envoyer vers un autre projet connu, porter `--cross-project-reason 'motif'`
sur send/thread, ou `cross_project_reason` en MCP. Les clients vérifient
`communication_projects_v1` ; mettre à jour les composants ensemble, sans
retirer le motif ni contourner le refus d'un serveur ancien. Un envoi legacy
inconnu reste possible avec avertissement. Aucun second accord humain obligatoire.
Les fils mixtes exigent le motif même en silence : tous leurs membres lisent.
Ces recettes ne prouvent pas qu'un daemon ou catalogue MCP déjà vivant est actualisé.

Consommateurs Python legacy : `bridget_agents()` sans racine n'est plus un
annuaire global. Pour vérifier un UUID ou un rôle déjà mandaté, choisir
explicitement `global_scope=True` ou le helper `bridget_agents_by_id` du moteur
Agent Loop. Cette consultation n'est ni recrutement, ni suggestion, ni motif
d'envoi. Transmettre toujours la racine de contexte et le motif requis à l'envoi.
Le moteur canonique et les nouvelles boucles utilisent déjà la consultation
adéquate ; un wrapper Psychologie ancien nécessitait l'adaptation de son seul
appel d'inventaire. Aucun contenu de mission n'est nécessaire à ce contrôle.

## 3. Fournisseurs et permissions

Le HOME du fournisseur porte son abonnement ; ne pas le remplacer ou copier
ses secrets par défaut. Une recette automatisée emploie un profil privé et une
autorisation explicite. GLM via Claude Code reste possible par configuration
déclarée ; aucun repli vers une API facturée n'est autorisé.

Un registre propre à cette instance peut être placé dans
`$BRIDGET_HOME/agents.json` (variable à résoudre avant toute manipulation).
Commande absolue, protocole, modèle et effort doivent correspondre à la matrice
de capacités déclarée ; ne pas ajouter une capacité pour contourner un refus.
La politique de lancement restrictive reste conservée : l'extraction retire le
runtime de projet, pas les droits ni les contrôles de facturation. Un refus est
à instruire, jamais à transformer automatiquement en lancement permissif.
La skill fournie reste un fichier à installer volontairement dans le profil
choisi, pas une écriture automatique dans tous les profils.

### Déclarer un fournisseur compatible Claude ou un second compte Codex

Les types par défaut sont `codex`, `claude`, `cursor` et `gemini`. Un autre
fournisseur se déclare dans `$BRIDGET_HOME/agents.json` (fichier 0600, sans lien
symbolique), lu au démarrage du daemon. Chaque type déclaré reçoit
automatiquement sa variante lecture seule `project-discovery-<type>`, utilisée
sous la posture `discovery`. Exemple pour GLM servi par Claude Code, avec un
lanceur qui fixe l'environnement du fournisseur et lit sa clé dans le trousseau
du système plutôt que dans un fichier :

```json
{
  "agents": {
    "glm": {
      "command": "/chemin/absolu/lanceur-glm",
      "args": ["--model", "glm-5.3", "--dangerously-skip-permissions", "--permission-mode", "bypassPermissions"],
      "protocol": "claude_stream_json",
      "forbidden_env": ["ANTHROPIC_API_KEY"],
      "claude_config_dir": "/chemin/absolu/profil-claude-glm",
      "mcp": {"interactive": "claude", "acp_session": false},
      "capabilities": {"execution_paths": ["claude_stream_json"], "models": {"glm-5.3": {}}}
    }
  }
}
```

Le dossier `claude_config_dir` doit être en 0700, sinon le lancement est refusé
« profil Claude invalide ». Un fournisseur lent à démarrer se lance avec
`--timeout <secondes>` : la commande attend alors cette durée.

Un second compte Codex se déclare de la même façon avec le protocole
`codex_app_server` et un lanceur qui fixe `CODEX_HOME`. Le modèle de `args` doit
figurer dans `capabilities.models`. Vérifier ensuite par un vrai lancement :
`bridget spawn glm --no-persistent --agent-id <UUID> --cwd <chemin>`, puis
`bridget stop <UUID>`.

## Codex interactif (090)

Les sources de cette évolution se trouvent dans
`/Users/moi/Nextcloud/10.Scripts/64.bridget`.
Pour construire un binaire de test sans remplacer l'installation release :

```sh
cd /Users/moi/Nextcloud/10.Scripts/64.bridget
PATH=/Users/moi/.cargo/bin:$PATH python3 scripts/build.py cargo build --locked -p bridget-daemon
```

Employer ensuite ce binaire avec le daemon privé et les variables de la section 2.
Dans un vrai terminal, depuis le répertoire de travail choisi :

```sh
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug/bridget codex
```

Le Codex déclaré doit prendre en charge `app-server --listen unix://…` et
`resume --remote unix://…` ; la recette porte sur 0.153.4. Le registre choisit
le chemin du binaire ; l'utilisateur garde son HOME et son abonnement. Les
permissions viennent de sa configuration native et de ses options explicites,
pas des arguments permissifs du mode géré. Une racine Bridget privée et courte
est nécessaire pour la socket de contrôle locale. Aucun service public ajouté.

Cette session n'est pas persistante : quitter ferme la TUI, le serveur privé
et sa présence. `spawn --persistent` reste le parcours d'arrière-plan. Une
commande de navigation vers un autre fil doit fermer l'intégration, jamais
laisser les prochains messages arriver invisiblement dans l'ancien fil.
Lancer une autre session explicitement pour changer de fil.

Pour reprendre dès le lancement un fil identifié et nommer sa présence :

```sh
bridget codex --name coderBridget resume <UUID-du-fil-Codex>
```

Ajouter `--yolo` uniquement si le contournement natif des approbations et du
sandbox est voulu. Le nom passe par le service partagé de profils ; l'identité
reste l'UUID Bridget, réutilisable explicitement avec `--agent-id`.

Après adoption de la release, la commande habituelle est simplement
`bridget codex`, depuis le répertoire de travail souhaité. Le lien
`/Users/moi/.local/bin/bridget` vise
`/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget`.
Les skills Codex, Claude et agents partagent la source canonique
`/Users/moi/Nextcloud/10.Scripts/64.bridget/skills/bridget/SKILL.md`.

L'adoption explicitement demandée le 2026-09-06 sauvegarde la release et l'état
du daemon sous `/Users/moi/.cache/bridget-adoptions/` avant remplacement.
L'arrêt gracieux de com.bridget.daemon n'est permis qu'après vérification de
l'annuaire ; aucune session active ne doit être interrompue implicitement.
Le reçu d'adoption local indique les chemins exacts, versions et vérifications.
Le retour arrière de cette adoption consiste à arrêter le seul daemon, restaurer
le binaire sauvegardé (et l'état arrêté sauvegardé si nécessaire), puis relancer
son plist inchangé. Ce n'est pas la procédure d'installation isolée ci-dessous.

### Dépanner le pont T3

`identity_not_found` signifie que le processus MCP n'a reçu aucune identité
utilisable. Ce code ne prouve pas que l'appelant est un sous-agent interne.
Dans un fil T3, appeler `t3_code.bridget_session` pour distinguer une attestation
absente d'un mauvais montage MCP. Si cette lecture réussit mais que Bridget
refuse toujours, comparer le chemin réel de `mcp_servers.bridget.command` dans
la configuration Codex effective avec l'exécutable choisi par T3 :
`T3CODE_BRIDGET_EXECUTABLE` s'il est défini, sinon le lien installé
`$HOME/.local/bin/bridget`. T3 refuse de transmettre sa preuve à un autre
exécutable, même si son nom est aussi `bridget`.

Pour une ancienne entrée officielle, remplacer uniquement son chemin de
commande par le lien installé stable. Conserver ses autres réglages, les
serveurs personnalisés et les désactivations explicites. Ne jamais inscrire
une preuve de session dans la configuration. Un chemin `target/release` ou
celui d'une release datée devient périmé après une livraison. Vérifier ce
raccordement à chaque installation, en plus de la santé du connecteur.

Une configuration corrigée ne modifie pas un processus MCP déjà lancé.
Le rattachement ou la reprise native du fil doit remonter la définition
vérifiée. Ne pas interrompre les autres fils ni relancer une mission pour
provoquer ce remontage. Un nouveau tour seul n'est pas une garantie : le
fournisseur peut conserver son montage. Le rétablissement est acquis après
un `bridget_status` réussi dans le fil concerné, pas après la seule attestation
T3. Hors T3, lancer l'agent avec `bridget codex` ou `bridget claude` ; un
sous-agent interne rend son résultat à son parent.

`t3_session_unavailable` signifie que la preuve privée présentée par le fil n'est
plus acceptée. Avec T3 Code `0.0.45-local.166` ou plus récent, une reprise Codex
remonte automatiquement la preuve courante avant son premier tour. Si le remontage
échoue, T3 bloque ce tour. Il ne faut pas enchaîner les désinstallations et les
réinstallations. Vérifier d'abord les versions réellement chargées, puis relancer
T3 une fois seulement si l'application ouverte exécute encore une ancienne version.

`Too many open files` indique un défaut de cycle de vie des liens ou un ancien
pont encore chargé. Le pont corrigé ferme les liens des sous-agents terminés après
leur drain et limite à 32 les reprises simultanées. La limite souple normale de
4 096 fichiers suffit. L'augmenter masque le symptôme sans fermer les ressources.

## 4. Fédération SSH

`scripts/federate-ssh.sh --help` décrit le transfert maître → client ;
`scripts/deploy-remote.sh --help` décrit le déploiement **client-only**.
Fournir explicitement hôte/port/utilisateur, clé privée, fichier known_hosts
prérempli, label et chemins de sockets neufs. Lancer `--dry-run` d'abord.
Le transfert est au premier plan ; aucun launchd/systemd n'est installé.
Un socket occupé est refusé, jamais effacé. La vérification de clé d'hôte reste
stricte ; aucune redirection d'agent SSH ni config SSH héritée.
Le déploiement client-only prend une racine Git explicite et un Cargo distant
déjà installé : il ne fonctionne pas directement depuis l'archive sans Git.
Pour ce cas, transférer volontairement le paquet et suivre la construction
hors ligne ci-dessus ; ne pas réutiliser un ancien script de déploiement.

L'archive ne contient aucune clé. Les recettes macOS↔Linux et coupure/reprise
déjà exécutées sont documentées, sans déclarer leur résultat transposable à
n'importe quel réseau. Un tunnel rompu ne justifie pas un renvoi sous une
nouvelle clé ni un redémarrage du fournisseur.

## 5. Arrêt et retour arrière

Arrêter d'abord les seuls équipiers de cette instance avec leur UUID (commande
`stop`), puis interrompre le daemon de test au premier plan et le tunnel de test.
Vérifier PID/commande et namespace avant toute intervention sur un processus.
Ne pas utiliser une commande globale `launchctl kickstart` ou `systemctl restart`
de l'ancien Bridget : le diagnostic de version du noyau n'en recommande aucune.

Le retour arrière consiste à quitter cette instance et reprendre les anciens
terminaux : aucun élément de production n'a été remplacé. Garder l'état privé
pour diagnostic, ou l'archiver après arrêt. Aucune suppression automatique,
aucun import/migration de la base utilisateur : les migrations n'ont été
éprouvées que sur copies de fixtures. L'adoption de la flotte est une décision
distincte, après les gates et avec un plan de retour arrière dédié.
