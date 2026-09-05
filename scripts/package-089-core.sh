#!/usr/bin/env bash
# Paquet de preuve seulement : aucune installation, réseau ou flotte de production.
set -euo pipefail
umask 077
source_root=$(cd "$(dirname "$0")/.." && pwd -P)
if [[ $# != 1 || "$1" != /* || -e "$1" || -L "$1" ]]; then
  echo 'usage: package-089-core.sh /chemin/absolu/nouveau-paquet (doit être absent)' >&2
  exit 2
fi
package_root=$1
# Une liste d’autorisation de sources, pas un checkout complet masqué par features.
if [[ -n "$(find "$source_root/crates" -type l -print -quit)" ]]; then
  echo 'refus : lien symbolique dans les sources du paquet' >&2
  exit 2
fi
mkdir "$package_root"
cp "$source_root/Cargo.toml" "$source_root/Cargo.lock" "$source_root/rust-toolchain.toml" "$source_root/LICENSE" "$package_root/"
cp -R "$source_root/crates" "$package_root/crates"
for spec in 015-guichet-maicie 016-coordination-active 089-communication-core; do
  mkdir -p "$package_root/specs/$spec/contracts"
  cp -R "$source_root/specs/$spec/contracts/fixtures" "$package_root/specs/$spec/contracts/fixtures"
done
# Aucun script historique de déploiement, aucun code Maicie : les fixtures du
# protocole public restent nécessaires aux tests, pas leur implémentation métier.
printf '%s\n' "$package_root"
