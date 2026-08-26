#!/usr/bin/env bash
# Harnais 018 de la frontière chantier/production des outils de pilotage.
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fixture_root="$(mktemp -d -t bridget-pilotage-install.XXXXXX)"
repo="${fixture_root}/repo"
linked_worktree="${fixture_root}/linked-worktree"

cleanup() {
  rm -rf "$fixture_root"
}
trap cleanup EXIT

fail() {
  echo "test-018-pilotage-install: $*" >&2
  exit 1
}

hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

file_mode() {
  if stat -c '%a' "$1" >/dev/null 2>&1; then
    stat -c '%a' "$1"
  else
    stat -f '%Lp' "$1"
  fi
}

commit_fixture() {
  local message="$1"
  GIT_AUTHOR_NAME=fixture GIT_AUTHOR_EMAIL=fixture@example.invalid \
  GIT_COMMITTER_NAME=fixture GIT_COMMITTER_EMAIL=fixture@example.invalid \
    git -C "$repo" commit -q -m "$message"
}

mkdir -p "${repo}/scripts/lib"
cp "${root_dir}/scripts/bridget-idle.py" "${repo}/scripts/bridget-idle.py"
cp "${root_dir}/scripts/bridget-ronde.py" "${repo}/scripts/bridget-ronde.py"
cp "${root_dir}/scripts/install-bridget-idle.sh" "${repo}/scripts/install-bridget-idle.sh"
cp "${root_dir}/scripts/install-bridget-ronde.sh" "${repo}/scripts/install-bridget-ronde.sh"
if [[ -f "${root_dir}/scripts/lib/pilotage-release.sh" ]]; then
  cp "${root_dir}/scripts/lib/pilotage-release.sh" "${repo}/scripts/lib/pilotage-release.sh"
fi
chmod 0755 "${repo}/scripts/"*.py "${repo}/scripts/"*.sh

git init -q "$repo"
git -C "$repo" checkout -q -b main
git -C "$repo" add scripts
commit_fixture "fixture initiale"
git -C "$repo" remote add origin https://example.invalid/bridget.git
admitted_sha="$(git -C "$repo" rev-parse HEAD)"
git -C "$repo" update-ref refs/remotes/origin/main "$admitted_sha"
printf '{}\n' >"${fixture_root}/maicie.json"

command_name() {
  printf 'bridget-%s\n' "$1"
}

run_installer() {
  local source_root="$1" tool="$2" home="$3"
  shift 3
  mkdir -p "$home"
  case "$tool" in
    idle)
      HOME="$home" "${source_root}/scripts/install-bridget-idle.sh" "$@"
      ;;
    ronde)
      HOME="$home" "${source_root}/scripts/install-bridget-ronde.sh" \
        --config "${fixture_root}/maicie.json" \
        --report-dir "${home}/reports" \
        --skip-activate "$@"
      ;;
    *) fail "outil de fixture inconnu: $tool" ;;
  esac
}

expect_refusal() {
  local label="$1" needle="$2" source_root="$3" tool="$4" home="$5"
  shift 5
  local output rc
  set +e
  output="$(run_installer "$source_root" "$tool" "$home" "$@" 2>&1)"
  rc=$?
  set -e
  if [[ "$rc" -eq 0 ]]; then
    fail "$label: succès interdit pour $tool ($output)"
  fi
  if ! grep -Fq "$needle" <<<"$output"; then
    fail "$label: motif '$needle' absent pour $tool ($output)"
  fi
  printf 'refus_%s_%s: OK\n' "$label" "$tool"
}

assert_no_command() {
  local tool="$1" home="$2" command
  command="$(command_name "$tool")"
  if [[ -e "${home}/.local/bin/${command}" || -L "${home}/.local/bin/${command}" ]]; then
    fail "effet interdit: ${home}/.local/bin/${command} existe"
  fi
}

assert_no_round_unit() {
  local home="$1"
  if [[ -e "${home}/Library/LaunchAgents/com.bridget.ronde.plist" \
     || -e "${home}/.config/systemd/user/bridget-ronde.service" \
     || -e "${home}/.config/systemd/user/bridget-ronde.timer" ]]; then
    fail "une unité ronde a été écrite malgré le refus"
  fi
  [[ ! -e "${home}/reports" ]] || fail "le répertoire de rapports a été créé malgré le refus"
}

assert_round_unit_passive() {
  local home="$1" unit
  unit="${home}/Library/LaunchAgents/com.bridget.ronde.plist"
  [[ -f "$unit" ]] || unit="${home}/.config/systemd/user/bridget-ronde.service"
  [[ -f "$unit" ]] || fail "unité ronde absente après installation"
  if grep -Eq '(send|spawn|stop|approve)' "$unit"; then
    fail "une unité de ronde ne peut embarquer aucune action"
  fi
}

assert_release() {
  local tool="$1" home="$2" sha="$3" command source_relative target expected hash
  command="$(command_name "$tool")"
  source_relative="scripts/${command}.py"
  target="$(readlink "${home}/.local/bin/${command}")"
  expected="${home}/.local/share/bridget/pilotage/releases/${sha}/${command}"
  [[ "$target" == "$expected" ]] || fail "cible $tool inattendue: $target != $expected"
  [[ -f "$target" && ! -L "$target" ]] || fail "release $tool non régulière ou liée: $target"
  [[ "$(file_mode "$target")" == 555 ]] || fail "mode release $tool inattendu: $(file_mode "$target")"
  [[ -x "$target" ]] || fail "artefact $tool non exécutable: $target"
  case "$target" in
    "$repo"/*|"$linked_worktree"/*) fail "artefact $tool encore lié au chantier: $target" ;;
  esac
  git -C "$repo" show "${sha}:${source_relative}" >"${fixture_root}/expected-${tool}"
  cmp -s "$target" "${fixture_root}/expected-${tool}" || fail "octets $tool différents du blob Git"
  hash="$(hash_file "$target")"
  [[ -f "${target}.origin" && ! -L "${target}.origin" ]] || fail "preuve origine $tool non régulière ou liée"
  [[ "$(file_mode "${target}.origin")" == 444 ]] || fail "mode preuve origine $tool inattendu: $(file_mode "${target}.origin")"
  grep -Fxq 'format=bridget-pilotage-release-v1' "${target}.origin" || fail "format origine $tool absent"
  grep -Fxq 'source_ref=refs/remotes/origin/main' "${target}.origin" || fail "référence origine $tool absente"
  grep -Fxq 'remote=origin' "${target}.origin" || fail "remote origine $tool absent"
  grep -Fxq "commit=${sha}" "${target}.origin" || fail "SHA origine $tool absent"
  grep -Fxq "artifact=${command}" "${target}.origin" || fail "artefact origine $tool absent"
  grep -Fxq "sha256=${hash}" "${target}.origin" || fail "empreinte origine $tool absente"
}

# 1. Worktree lié : doit primer sur le nom de branche.
git -C "$repo" worktree add -q -b unsafe-worktree "$linked_worktree" HEAD
for tool in idle ronde; do
  home="${fixture_root}/home-worktree-${tool}"
  expect_refusal worktree 'worktree lié interdit' "$linked_worktree" "$tool" "$home"
  assert_no_command "$tool" "$home"
  [[ "$tool" != ronde ]] || assert_no_round_unit "$home"
done
git -C "$repo" worktree remove -f "$linked_worktree"

# 2. Branche hors main dans le checkout principal.
git -C "$repo" checkout -q -b unsafe-branch
for tool in idle ronde; do
  home="${fixture_root}/home-branch-${tool}"
  expect_refusal branche 'branche main requise' "$repo" "$tool" "$home"
  assert_no_command "$tool" "$home"
  [[ "$tool" != ronde ]] || assert_no_round_unit "$home"
done
git -C "$repo" checkout -q main

# 3. Arbre sale, y compris avant l'examen d'une cible existante.
cp "${repo}/scripts/bridget-idle.py" "${fixture_root}/idle-clean.py"
printf '\n# sale\n' >>"${repo}/scripts/bridget-idle.py"
for tool in idle ronde; do
  home="${fixture_root}/home-dirty-${tool}"
  mkdir -p "${home}/.local/bin"
  printf 'sentinelle\n' >"${home}/.local/bin/$(command_name "$tool")"
  expect_refusal sale 'arbre Git sale' "$repo" "$tool" "$home"
  grep -Fxq sentinelle "${home}/.local/bin/$(command_name "$tool")" || fail "sentinelle $tool modifiée sur arbre sale"
  [[ "$tool" != ronde ]] || assert_no_round_unit "$home"
done
mv "${fixture_root}/idle-clean.py" "${repo}/scripts/bridget-idle.py"

# 4. Copie régulière sans --force : refus avant toute unité ; avec --force : migration.
for tool in idle ronde; do
  home="${fixture_root}/home-copy-${tool}"
  mkdir -p "${home}/.local/bin"
  printf 'sentinelle\n' >"${home}/.local/bin/$(command_name "$tool")"
  expect_refusal copie 'entrée active déjà présente' "$repo" "$tool" "$home"
  grep -Fxq sentinelle "${home}/.local/bin/$(command_name "$tool")" || fail "copie $tool modifiée sans --force"
  [[ "$tool" != ronde ]] || assert_no_round_unit "$home"
  run_installer "$repo" "$tool" "$home" --force >/dev/null
  assert_release "$tool" "$home" "$admitted_sha"
  [[ "$tool" != ronde ]] || assert_round_unit_passive "$home"
done

# 5. Référence origin/main absente : pas de fetch implicite.
git -C "$repo" update-ref -d refs/remotes/origin/main
for tool in idle ronde; do
  home="${fixture_root}/home-no-origin-${tool}"
  expect_refusal origine_absente 'origin/main absente' "$repo" "$tool" "$home"
  assert_no_command "$tool" "$home"
  [[ "$tool" != ronde ]] || assert_no_round_unit "$home"
done
git -C "$repo" update-ref refs/remotes/origin/main "$admitted_sha"

# 6. main local en avance : commit propre mais non admis.
printf 'avance locale\n' >"${repo}/avance-locale.txt"
git -C "$repo" add avance-locale.txt
commit_fixture "avance locale"
local_ahead_sha="$(git -C "$repo" rev-parse HEAD)"
for tool in idle ronde; do
  home="${fixture_root}/home-ahead-${tool}"
  expect_refusal non_poussee 'HEAD non admis par origin/main' "$repo" "$tool" "$home"
  assert_no_command "$tool" "$home"
  [[ "$tool" != ronde ]] || assert_no_round_unit "$home"
done

# La même tête devient admise quand origin/main la contient.
git -C "$repo" update-ref refs/remotes/origin/main "$local_ahead_sha"

# 7. main local en retard : rollback vers un ancêtre déjà admis autorisé.
git -C "$repo" checkout -q -b remote-next
printf 'descendant distant\n' >"${repo}/descendant-distant.txt"
git -C "$repo" add descendant-distant.txt
commit_fixture "descendant distant"
remote_next_sha="$(git -C "$repo" rev-parse HEAD)"
git -C "$repo" update-ref refs/remotes/origin/main "$remote_next_sha"
git -C "$repo" checkout -q main
behind_home="${fixture_root}/home-behind"
run_installer "$repo" idle "$behind_home" >/dev/null
assert_release idle "$behind_home" "$local_ahead_sha"
echo 'rollback_ancetre_admis: OK'

# 8. Installation des deux outils, origine lisible et rejeu idempotent.
release_home="${fixture_root}/home-release"
for tool in idle ronde; do
  run_installer "$repo" "$tool" "$release_home" >/dev/null
  assert_release "$tool" "$release_home" "$local_ahead_sha"
  before_target="$(readlink "${release_home}/.local/bin/$(command_name "$tool")")"
  before_hash="$(hash_file "$before_target")"
  run_installer "$repo" "$tool" "$release_home" >/dev/null
  [[ "$(readlink "${release_home}/.local/bin/$(command_name "$tool")")" == "$before_target" ]] || fail "cible $tool non idempotente"
  [[ "$(hash_file "$before_target")" == "$before_hash" ]] || fail "octets $tool non idempotents"
  printf 'installation_%s_idempotente: OK\n' "$tool"
done

# 9. Une release de mêmes octets n'est identique ni sous forme de lien, ni
# avec un mode plus permissif.
linked_release_home="${fixture_root}/home-release-lien"
run_installer "$repo" idle "$linked_release_home" >/dev/null
linked_release="$(readlink "${linked_release_home}/.local/bin/bridget-idle")"
chmod 0555 "${repo}/scripts/bridget-idle.py"
rm "$linked_release"
ln -s "${repo}/scripts/bridget-idle.py" "$linked_release"
expect_refusal release_lien 'attendu=fichier_regulier_non_lien' "$repo" idle "$linked_release_home"
chmod 0755 "${repo}/scripts/bridget-idle.py"

mode_release_home="${fixture_root}/home-release-mode"
run_installer "$repo" idle "$mode_release_home" >/dev/null
mode_release="$(readlink "${mode_release_home}/.local/bin/bridget-idle")"
chmod 0755 "$mode_release"
expect_refusal release_mode 'mode release invalide' "$repo" idle "$mode_release_home"

# Les feuilles exactes ne suffisent pas si le répertoire SHA se résout dans le
# dépôt : un succès doit alors survivre au déplacement réel de cette source.
canonical_release_home="${fixture_root}/home-release-canonique"
run_installer "$repo" idle "$canonical_release_home" >/dev/null
canonical_release="$(readlink "${canonical_release_home}/.local/bin/bridget-idle")"
canonical_release_dir="$(dirname "$canonical_release")"
borrowed_release_dir="${repo}/.git/borrowed-release"
mv "$canonical_release_dir" "$borrowed_release_dir"
ln -s "$borrowed_release_dir" "$canonical_release_dir"
set +e
canonical_output="$(run_installer "$repo" idle "$canonical_release_home" 2>&1)"
canonical_rc=$?
set -e
if [[ "$canonical_rc" -eq 0 ]]; then
  [[ "$repo" == "${fixture_root}/repo" ]] || fail "independance_reelle: dépôt de fixture inattendu"
  moved_repo="${fixture_root}/repo-deplace"
  mv "$repo" "$moved_repo"
  set +e
  HOME="$canonical_release_home" "${canonical_release_home}/.local/bin/bridget-idle" --help >/dev/null 2>&1
  survival_rc=$?
  set -e
  fail "independance_reelle: chemin dans le dépôt accepté, exécution après déplacement rc=${survival_rc}"
fi
grep -Fq 'release résolue dans le dépôt source' <<<"$canonical_output" \
  || fail "independance_reelle: motif canonique absent ($canonical_output)"
echo 'independance_reelle_chemin_canonique: OK'

# 10. --force remplace l'entrée exacte même si elle est un lien vers répertoire,
# puis la cible réellement obtenue est relue.
directory_link_home="${fixture_root}/home-directory-link"
mkdir -p "${directory_link_home}/.local/bin" "${directory_link_home}/ancienne-cible"
ln -s "${directory_link_home}/ancienne-cible" "${directory_link_home}/.local/bin/bridget-idle"
set +e
directory_link_output="$(run_installer "$repo" idle "$directory_link_home" --force 2>&1)"
directory_link_rc=$?
set -e
[[ "$directory_link_rc" -eq 0 ]] || fail "force_remplace_entree_exacte: échec inattendu ($directory_link_output)"
grep -Fq "posé: ${directory_link_home}/.local/bin/bridget-idle ->" <<<"$directory_link_output" \
  || fail "force_remplace_entree_exacte: attestation de pose absente ($directory_link_output)"
assert_release idle "$directory_link_home" "$local_ahead_sha"
[[ ! -e "${directory_link_home}/ancienne-cible/bridget-idle" ]] \
  || fail "force_remplace_entree_exacte: lien préparé déplacé dans l'ancien répertoire"
echo 'force_remplace_entree_exacte: OK'

# 11. Même SHA mais configuration différente : refus non nul, unités intactes
# et aucune annonce finale de succès.
divergent_home="${fixture_root}/home-ronde-divergente"
config_a="${fixture_root}/maicie-a.json"
config_b="${fixture_root}/maicie-b.json"
printf '{"version":"A"}\n' >"$config_a"
printf '{"version":"B"}\n' >"$config_b"
HOME="$divergent_home" "${repo}/scripts/install-bridget-ronde.sh" \
  --config "$config_a" --report-dir "${divergent_home}/reports-a" \
  --skip-activate >/dev/null
service="${divergent_home}/.config/systemd/user/bridget-ronde.service"
timer="${divergent_home}/.config/systemd/user/bridget-ronde.timer"
cp "$service" "${fixture_root}/service-a"
cp "$timer" "${fixture_root}/timer-a"
set +e
divergent_output="$(HOME="$divergent_home" "${repo}/scripts/install-bridget-ronde.sh" \
  --config "$config_b" --report-dir "${divergent_home}/reports-b" \
  --skip-activate 2>&1)"
divergent_rc=$?
set -e
[[ "$divergent_rc" -ne 0 ]] \
  || fail "configuration_divergente_refusee_sans_faux_succes: succès interdit ($divergent_output)"
! grep -Fq 'ronde portable prête' <<<"$divergent_output" \
  || fail "configuration_divergente_refusee_sans_faux_succes: annoncée prête"
cmp -s "$service" "${fixture_root}/service-a" \
  || fail "configuration_divergente_refusee_sans_faux_succes: service A modifié"
cmp -s "$timer" "${fixture_root}/timer-a" \
  || fail "configuration_divergente_refusee_sans_faux_succes: timer A modifié"
echo 'configuration_divergente_refusee_sans_faux_succes: OK'

forced_output="$(HOME="$divergent_home" "${repo}/scripts/install-bridget-ronde.sh" \
  --config "$config_b" --report-dir "${divergent_home}/reports-b" \
  --skip-activate --force 2>&1)"
grep -Fq 'ronde portable prête' <<<"$forced_output" || fail "configuration B forcée non annoncée prête"
grep -Fq "$config_b" "$service" || fail "configuration B forcée absente du service"
! cmp -s "$service" "${fixture_root}/service-a" || fail "configuration B forcée n'a pas remplacé le service A"
echo 'configuration_divergente_remplacee_sous_force: OK'

# 12. Un SHA existant avec d'autres octets est corrompu, jamais « réparé » par --force.
corrupt_home="${fixture_root}/home-corrupt"
run_installer "$repo" idle "$corrupt_home" >/dev/null
corrupt_target="$(readlink "${corrupt_home}/.local/bin/bridget-idle")"
chmod u+w "$corrupt_target"
printf '\n# corruption\n' >>"$corrupt_target"
chmod 0555 "$corrupt_target"
expect_refusal corruption 'release corrompue' "$repo" idle "$corrupt_home" --force
grep -Fq '# corruption' "$corrupt_target" || fail "la corruption a été écrasée silencieusement"

# 13. La production ne dépend plus du cycle de vie du dépôt source.
[[ "$repo" == "${fixture_root}/repo" ]] || fail "cible de suppression de fixture inattendue"
rm -rf "$repo"
HOME="$release_home" "${release_home}/.local/bin/bridget-idle" --help >/dev/null
HOME="$release_home" "${release_home}/.local/bin/bridget-ronde" --help >/dev/null
echo 'survie_sans_depot_source: OK'

echo 'test-018-pilotage-install: checks OK'
