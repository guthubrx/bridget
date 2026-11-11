# Compilation et nettoyage automatique

[English](build-cache.md) · [Français](build-cache.fr.md)


Les commandes `make build`, `make release` et `make test` passent par
`scripts/build.py`. Les caches de compilation reconnus de ce dépôt sont entretenus
avant et après Cargo, même en cas d'échec. Aucun daemon n'est relancé.

- Budget commun : **10 Gio**, pas 10 Gio par worktree.
- Ancienneté : **7 jours**, d'après la dernière modification du contenu du cache.
- Les profils les plus anciens sont purgés en premier si le budget est dépassé.
- Seuls `deps`, `build`, `incremental` et `.fingerprint`, dans les profils
  `target/debug` et `target/release`, sont supprimés. La prochaine compilation
  d'un profil purgé prendra donc plus longtemps.
- Exécutables au sommet du profil, sources, sauvegardes et état Bridget restent
  en place. Les caches d'autres projets ne sont pas concernés.
- Verrou Cargo occupé ou fichier ouvert dans le profil : ce profil est conservé.
  Si `lsof` ne permet pas un contrôle fiable, le nettoyage est refusé.
  Le budget est donc une cible, pas une limite forcée.
- Les arguments et le code de sortie de Cargo sont conservés. Un échec d'entretien
  affiche un avertissement sans masquer le résultat de la compilation. Aucun
  worktree n'est retiré après un échec. Garder les journaux de diagnostic hors
  des caches intermédiaires : ces caches peuvent être purgés même après un échec.

Prérequis : Unix avec Python **3.11 ou plus**, Git, Cargo et `lsof`. Aucun paquet
Python supplémentaire. Les tailles affichées sont logiques : les clones APFS
peuvent rendre le gain physique différent.

## Commandes

Depuis le dépôt ou son worktree :

```sh
make build
make release
make test
make clean-builds DRY_RUN=1  # Simulation, aucune suppression
make clean-builds          # Entretien suivant âge et budget
make test-build-cleanup    # Tests isolés du nettoyeur

# Arguments Cargo habituels, avec entretien avant et après exécution :
python3 scripts/build.py cargo test --workspace --release
python3 scripts/build.py cargo check --locked

# Surcharges ponctuelles, ou variables à exporter dans le shell :
BRIDGET_BUILD_CACHE_GIB=15 BRIDGET_BUILD_CACHE_DAYS=14 make release
```

`make clean` applique désormais cette même politique : il ne lance plus
`cargo clean`, qui effacerait aussi les binaires potentiellement utilisés.
Une limite égale à zéro purge tous les caches admissibles ; les protections
restent applicables. Une valeur négative, infinie ou invalide est refusée.

## Compilation dans un répertoire temporaire

Les répertoires temporaires nommés `bridget-build-*` directement dans le dossier
`.cache` du compte utilisateur sont enregistrés par le script avant et après la compilation,
lorsqu'ils existent, si `CARGO_TARGET_DIR` ou `--target-dir` les désigne. Un marqueur
les rattache à ce dépôt. Ils participent alors au même budget que les worktrees.
Les anciens répertoires non marqués et ceux d'un autre dépôt sont ignorés.

Après une installation indépendante et vérifiée, la commande suivante peut
purger immédiatement les intermédiaires du temporaire :

```sh
python3 scripts/build.py installed --target "$bridget_build_dir" --binary "$bridget_installed_binary"
```

Les variables doivent désigner les chemins absolus réellement utilisés. La
commande exige un temporaire enregistré et un binaire installé hors de celui-ci,
avec la même empreinte SHA-256. L'exécutable temporaire et le verrou Cargo restent
en place. `make install` effectue cet appel après installation ; ce document ne
constitue pas une autorisation de remplacer une installation active.

## Stockage sur un autre volume

Le compte peut choisir un volume pour toutes les prochaines compilations de ce
dépôt. Préparer sur le volume monté un dossier réel, appartenant au compte,
avec le mode `0700`. Il doit résider sur un autre système de fichiers que `/`.
Le volume doit prendre en charge les permissions Unix ; un volume FAT ne convient
pas à cette configuration. Ne pas remplacer une configuration existante sans
vérifier les caches qui en dépendent.

Générer une identité avec `python3 -c 'import uuid; print(uuid.uuid4().hex)'`.
Créer dans ce dossier le fichier `.bridget-build-root.json`, mode `0600` :

```json
{"version":1,"id":"0123456789abcdef0123456789abcdef"}
```

Créer ensuite `~/.config/bridget/build.json`, mode `0600`, en remplaçant le chemin
d’exemple par celui du dossier préparé et les deux identités par la valeur générée :

```json
{"version":1,"root":"/volume/compilation/bridget-cache","root_id":"0123456789abcdef0123456789abcdef"}
```

`python3 scripts/build.py target-dir` affiche la cible sans compiler. Make et
l’installateur source utilisent ce même résolveur. Le script choisit un enfant
`bridget-build-*` stable par dépôt. Un ancien `CARGO_TARGET_DIR` hors de cette
racine doit être retiré ; une cible explicite peut désigner un autre enfant
direct `bridget-build-*` pour une isolation utile.

Sans configuration, les parcours existants sont conservés. Avec configuration,
un volume absent ou une ancre invalide refuse cette compilation avant Cargo,
sans repli vers le disque système. Le ménage local reste possible. Le budget
commun inclut les caches externes lorsque le volume est disponible ; aucune
taille ni purge de caches d’un volume absent n’est prétendue. Si le volume
disparaît pendant Cargo, le résultat reste à vérifier et les fichiers sont
conservés. Ce réglage ne déplace pas les toolchains ni le cache global Cargo.

## Périmètre et limites

Un appel direct à `cargo` contourne cet entretien. Pour les nouveaux builds,
utiliser Make ou le script, y compris depuis un agent. L'installateur K1 utilise
le script lorsqu'il est présent ; son ancien mode autonome garde son repli Cargo.
Le déploiement SSH historique reste inchangé : pas de nettoyage automatique à
distance ni de nouveau prérequis imposé aux hôtes distants.

Les profils personnalisés, les sorties de compilation croisée et les répertoires
externes arbitraires ne sont pas parcourus. Le cache global Cargo n'est pas
touché. Le nettoyage intervient avant et après les commandes, pas en arrière-plan.
Les worktrees anciens doivent utiliser le nouveau script pour déclencher cet
entretien ; leurs caches standards sont néanmoins visibles depuis le dépôt.

L'entretien accompagne les builds ; aucun minuteur ni modèle en arrière-plan
n'est nécessaire. Les consignes du dépôt imposent Make ou le script aux agents.
Réutiliser le target du checkout au lieu de le copier à chaque test. Si une
isolation est nécessaire, utiliser un répertoire temporaire enregistré. Un cache
créé par un build échoué est enregistré à son retour. Les exécutables installés
et les copies de retour restent hors budget et sont à traiter à la livraison.

## Contrôles avant livraison

Avant de livrer, lancer `make lint` : il exécute l'analyse statique de Rust avec
`-D warnings` (un avertissement refuse la livraison) puis le contrôle de format.
Le contrôle de format est bloquant (lot 10 du programme 260) : lancer `make fmt` avant chaque commit.
`make lint-scripts` contrôle les scripts Python et shell (`ruff` et `bash -n`).
Le contrôle distant `Rust` rejoue l'analyse statique, le format et les tests ; la
liste des contrôles attendus est dans `specs/260-assainissement-code/contracts/controles-ci.md`.
