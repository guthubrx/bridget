# Audit des dépendances et des scripts — 089

Exécuté le 2026-09-05, Cargo.lock SHA-256 `b1dad808f86e619985e8a02050cce82851388a88b0cf9a3d7e67eff4d0754c67`.

## RustSec et maintenance

`cargo-audit 0.22.2` installé avec `cargo install --version 0.22.2 --locked --root /private/tmp/b89-audit-N5l6N5 cargo-audit` : outil privé, aucune installation globale. Invocation effective :

```sh
env -i HOME=/private/tmp/b9t011.7nVSnj/home CARGO_HOME=/Users/moi/.cargo PATH=/Users/moi/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin /private/tmp/b89-audit-N5l6N5/bin/cargo-audit audit --file Cargo.lock --db /private/tmp/b89-audit-N5l6N5/advisory-db --json
```

Sortie brute dans `artifacts/rustsec-audit.json` : exit 0, 94 dépendances, zéro vulnérabilité et zéro avertissement, aucune exception ignorée. Base de 1 239 avis, commit `5a0ebedfe8bdd2e295b171f4162f8c977bcad9a5`, dernière mise à jour annoncée 2026-09-02. Ce résultat est une photographie des avis connus, pas une certification de sûreté du code ni des exécutables externes. Méthode : [RustSec](https://rustsec.org/) et [documentation officielle cargo-audit](https://github.com/rustsec/rustsec/blob/main/cargo-audit/README.md).

Point de maintenance conservé explicitement : `serde_yaml 0.9.34+deprecated`, utilisé uniquement par `reprise::load_pin` pour lire les pins YAML historiques. Aucun avis RustSec retourné, mais le marqueur deprecated reste une dette. Pas de suppression silencieuse du format ni de remplacement par un parseur maison. Un changement de bibliothèque devra conserver le corpus YAML et traiter le contenu comme non fiable ; l'audit présent ne prouve pas une borne de profondeur/allocation pour ce parseur.

## Licences et usage

Inventaire reproductible dans `artifacts/dependency-licenses.json`, issu de `cargo metadata --offline --locked --format-version 1` (toutes plateformes, y compris crates non compilées ici). Toutes les entrées déclarent une licence. Les trois crates locales sont MIT. Les alternatives `MIT OR Apache-2.0`, `Unlicense OR MIT`, `Apache-2.0 OR BSL-1.0` et `MIT OR Apache-2.0 OR LGPL-2.1-or-later` ne rendent pas toutes leurs branches simultanément obligatoires. `unicode-ident` ajoute Unicode-3.0 ; `foldhash` est Zlib. Les expressions historiques avec slash sont conservées telles que publiées, pas transformées en fausse analyse juridique. Les notices des dépendances restent à conserver lors d'une redistribution ; cet inventaire n'est pas leur texte intégral. Le paquet SQLite embarqué doit également conserver ses informations de provenance, pas seulement celles du binding libsqlite3-sys.

Lecture statique des imports et manifests : serde/serde_json pour le fil et la persistance, uuid pour les identifiants, rusqlite pour les transactions, sha2 pour canons/contenus/définitions, libc pour sockets/poll/termios/groupes, log/env_logger pour stderr, signal-hook pour l'arrêt, unicode-width pour le terminal, serde_yaml pour les pins. Aucune dépendance directe orpheline identifiée ; ce n'est pas un résultat cargo-machete. `libc` réutilise désormais l'unique version workspace dans les deux crates. Le graphe interdit Maicie/UI/Docker via le test T013, en plus de l'absence physique de leurs sources dans le paquet.

## Scripts distribués

Périmètre relu : `package-089-core.sh`, `federate-ssh.sh`, `deploy-remote.sh`, `verify-089-contracts.sh` et leurs tests. Aucun curl-pipe-shell, installation Rust implicite, écrasement de profil, clé copiée ou effacement de socket occupée. Sources du paquet sélectionnées par liste d'autorisation ; transfert distant limité aux fichiers suivis Git, sans symlink, fichiers DB ou sockets. Les modifications suivies sont transférées et marquées dirty ; ce n'est pas un export prétendument identique à HEAD. SSH exige une clé et des clés d'hôte explicites privées, vérification stricte sans TOFU, configuration héritée neutralisée, aucun forward d'agent ni LocalCommand. Les arguments distants ont un alphabet fermé et sont quotés. Pas de service launchd/systemd installé.

Limites : sécurité même UID selon threat-model.md ; les préflights shell ne constituent pas une protection contre un attaquant du même UID remplaçant les chemins entre contrôle et usage. Un échec de transfert/build peut laisser un préfixe privé neuf pour diagnostic ; le script refuse sa réutilisation au lieu de le nettoyer sans preuve. Les scripts historiques encore présents dans l'historique du clone ne font pas partie du paquet autorisé ; T035 doit rendre cette distinction explicite dans la livraison finale.
