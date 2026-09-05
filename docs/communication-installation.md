# Installer Bridget communication à côté de l'existant

Cette procédure ne remplace aucun binaire global, service, registre, profil de
fournisseur, skill globale ni base existante. Elle ne démarre pas Maicie ni GUI.
Les recettes Claude/GLM et le gate final restent soumis aux résultats consignés
dans la session 089 ; un paquet construit n'est pas une validation de compte.

## 1. Sources indépendantes et construction

Depuis le chantier actuel :

```sh
cd /Users/moi/Nextcloud/10.Scripts/XX.bridget/.worktrees/089-communication-core
package_parent=$(mktemp -d /tmp/bgpackage.XXXXXX)
package_parent=$(cd "$package_parent" && pwd -P)
bash scripts/package-089-core.sh "$package_parent/source"
cd "$package_parent/source"
shasum -a 256 -c SOURCE-MANIFEST.sha256
PATH=/Users/moi/.cargo/bin:$PATH BRIDGET_BUILD_ID="$(cat BUILD_ID)" \
  cargo build --locked --offline --release -p bridget-daemon
```

Rust 1.92.0 et les dépendances verrouillées doivent déjà être disponibles.
Un cache incomplet échoue hors ligne : aucune installation implicite. L'archive
contient les trois crates, les fixtures nécessaires, la skill, les deux scripts
SSH audités et cette documentation, pas les sources Maicie/UI/runtime.
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
terminal, puis `"$bridget_bin" status` et `"$bridget_bin" agents --json`.
Ne pas refaire `mktemp` pour un client : il interrogerait une autre instance.
La racine doit être privée 0700, les fichiers privés 0600, sans lien symbolique.
Le chemin socket est court pour rester compatible macOS/Linux.

L'identité et l'annuaire sont interrogés sur la socket avec des budgets bornés.
Un inventaire inaccessible est une erreur explicite, pas une liste vide.
`status` ne lit pas SQLite côté client et ne prétend pas connaître un total des
messages absent du protocole ; `ledger --limit 20` consulte le maître.

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
