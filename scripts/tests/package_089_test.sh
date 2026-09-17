#!/usr/bin/env bash
# Aucun daemon/fournisseur lancé ; sorties uniquement dans une racine privée.
set -euo pipefail
umask 077
root=$(cd "$(dirname "$0")/../.." && pwd -P)
fixture=$(mktemp -d /tmp/b89-package-test.XXXXXX)
# macOS expose /tmp comme alias : utiliser son chemin réel pour le préflight.
fixture=$(cd "$fixture" && pwd -P)
package=$fixture/source
bash "$root/scripts/package-089-core.sh" "$package" >/dev/null
(cd "$package" && shasum -a 256 -c SOURCE-MANIFEST.sha256 >/dev/null)
for absent in .git apps plugins infra node_modules; do
  [[ ! -e "$package/$absent" ]]
done
for required in README.md README.en.md BUILD_ID skills/bridget/SKILL.md docs/communication-installation.md scripts/federate-ssh.sh specs/010-mcp/spike/fake-mcp-server.py docs/federation-services.md scripts/tests/federation_095_test.sh scripts/tests/federation_095_roundtrip.py specs/095-federation-services/spec.md scripts/tests/federation_096_test.sh specs/096-federate-cli/spec.md; do
  [[ -f "$package/$required" ]]
done
before=$(shasum -a 256 "$package/Cargo.toml")
for required in Makefile scripts/build.py scripts/tests/test_build_cleanup.py docs/build-cache.md; do
  [[ -f "$package/$required" ]]
done
if bash "$root/scripts/package-089-core.sh" "$package" >"$fixture/occupied.log" 2>&1; then
  echo 'destination occupée acceptée' >&2; exit 1
fi
[[ "$before" == "$(shasum -a 256 "$package/Cargo.toml")" ]]
ln -s "$fixture" "$fixture/alias"
if bash "$root/scripts/package-089-core.sh" "$fixture/alias/indirect" >"$fixture/symlink.log" 2>&1; then
  echo 'parent symbolique accepté' >&2; exit 1
fi
[[ ! -e "$fixture/indirect" ]]
# Le dernier composant est ordinaire : seul son parent redirige hors source.
# Travailler sur une copie du paquet conserve le dépôt et le paquet de référence.
source_fixture=$fixture/symlink-source
cp -R "$package" "$source_fixture"
mv "$source_fixture/skills" "$fixture/external-skills"
ln -s "$fixture/external-skills" "$source_fixture/skills"
[[ -d "$source_fixture/skills/bridget" && ! -L "$source_fixture/skills/bridget" ]]
if bash "$source_fixture/scripts/package-089-core.sh" "$fixture/refused-source" >"$fixture/source-parent.log" 2>&1; then
  echo 'parent source symbolique accepté' >&2; exit 1
else
  [[ $? == 2 ]]
fi
[[ ! -e "$fixture/refused-source" && ! -L "$fixture/refused-source" ]]
[[ $(readlink "$source_fixture/skills") == "$fixture/external-skills" ]]
cmp "$package/SOURCE-MANIFEST.sha256" "$source_fixture/SOURCE-MANIFEST.sha256"
(cd "$source_fixture" && shasum -a 256 -c SOURCE-MANIFEST.sha256 >/dev/null)
(cd "$package" && shasum -a 256 -c SOURCE-MANIFEST.sha256 >/dev/null)
# Mutation byte réelle : le manifeste n'est pas une simple déclaration.
printf '\n' >> "$package/README.md"
if (cd "$package" && shasum -a 256 -c SOURCE-MANIFEST.sha256 >"$fixture/mutation.log" 2>&1); then
  echo 'mutation non détectée' >&2; exit 1
fi
printf 'PASS paquet autonome, cible occupée, parents symboliques destination/source, mutation SHA-256 ; preuves : %s\n' "$fixture"
