# Compilation et nettoyage automatique

Les commandes `make build`, `make release` et `make test` passent par
`scripts/build.py`. Après un succès, les caches de compilation des worktrees de
ce dépôt sont entretenus automatiquement. Aucun daemon n'est relancé.

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
- Un échec de compilation ne déclenche aucun nettoyage. Un échec d'entretien
  affiche un avertissement sans transformer un build réussi en échec.

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

# Arguments Cargo habituels, avec entretien après succès :
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
`.cache` du compte utilisateur sont enregistrés après une compilation réussie
par le script, si `CARGO_TARGET_DIR` ou `--target-dir` les désigne. Un marqueur
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

## Périmètre et limites

Un appel direct à `cargo` contourne cet entretien. Pour les nouveaux builds,
utiliser Make ou le script, y compris depuis un agent. L'installateur K1 utilise
le script lorsqu'il est présent ; son ancien mode autonome garde son repli Cargo.
Le déploiement SSH historique reste inchangé : pas de nettoyage automatique à
distance ni de nouveau prérequis imposé aux hôtes distants.

Les profils personnalisés, les sorties de compilation croisée et les répertoires
externes arbitraires ne sont pas parcourus. Le cache global Cargo n'est pas
touché. Le nettoyage intervient après les commandes, pas en arrière-plan.
Les worktrees anciens doivent utiliser le nouveau script pour déclencher cet
entretien ; leurs caches standards sont néanmoins visibles depuis le dépôt.
