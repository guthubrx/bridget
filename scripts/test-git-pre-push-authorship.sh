#!/usr/bin/env bash
set -u -o pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
hook="${BRIDGET_PRE_PUSH_HOOK:-${root_dir}/scripts/git-pre-push-authorship.sh}"
test_temp_parent="${TMPDIR:-/tmp}"
[[ "$test_temp_parent" == /* ]] || {
  echo "TMPDIR de test doit être un chemin absolu" >&2
  exit 2
}
fixture_root="$(mktemp -d "${test_temp_parent%/}/bridget-pre-push-test.XXXXXX")"
passed=0
failed=0

cleanup() {
  case "$fixture_root" in
    /*/bridget-pre-push-test.*) rm -rf -- "$fixture_root" ;;
  esac
}
trap cleanup EXIT

fixture_git() {
  env GIT_AUTHOR_NAME=Fixture GIT_AUTHOR_EMAIL=fixture@example.invalid GIT_COMMITTER_NAME=Fixture GIT_COMMITTER_EMAIL=fixture@example.invalid git "$@"
}

commit_object() {
  local repository="$1"
  local parent="$2"
  local message="$3"
  local tree
  tree="$(git -C "$repository" mktree </dev/null)"
  if [[ -n "$parent" ]]; then
    printf '%s\n' "$message" |
      fixture_git -C "$repository" commit-tree "$tree" -p "$parent"
  else
    printf '%s\n' "$message" |
      fixture_git -C "$repository" commit-tree "$tree"
  fi
}

create_fixture() {
  local name="$1"
  local directory="${fixture_root}/${name}"
  local_repo="${directory}/local"
  remote_repo="${directory}/remote.git"
  mkdir -p "$local_repo"
  git -C "$local_repo" init -q
  git -C "$local_repo" symbolic-ref HEAD refs/heads/main
  base_commit="$(commit_object "$local_repo" "" "Base propre ${name}")"
  git -C "$local_repo" update-ref refs/heads/main "$base_commit"
  git clone -q --bare "$local_repo" "$remote_repo"
  git -C "$local_repo" remote add origin "$remote_repo"
  zero_oid="$(git -C "$local_repo" hash-object --stdin </dev/null | sed 's/./0/g')"
}

publish_ref() {
  local reference="$1"
  local object_id="$2"
  git --git-dir="$remote_repo" fetch -q "$local_repo" "$object_id"
  git --git-dir="$remote_repo" update-ref "$reference" "$object_id"
}

run_hook() {
  local input_file="$1"
  local destination="${2:-$remote_repo}"
  (
    cd "$local_repo"
    "$hook" origin "$destination" <"$input_file"
  )
}

expect_rejection() {
  local input_file="$1"
  local destination="${2:-$remote_repo}"
  if run_hook "$input_file" "$destination" >"${input_file}.stdout" 2>"${input_file}.stderr"; then
    echo "le hook a accepté une transaction qui devait être refusée" >&2
    return 1
  fi
  [[ -s "${input_file}.stderr" ]] || {
    echo "le refus doit expliquer sa cause sur stderr" >&2
    return 1
  }
}

expect_acceptance() {
  local input_file="$1"
  if ! run_hook "$input_file" >"${input_file}.stdout" 2>"${input_file}.stderr"; then
    sed -n '1,20p' "${input_file}.stderr" >&2
    return 1
  fi
}

test_new_branch_without_upstream() {
  create_fixture "new-branch"
  local hostile input
  hostile="$(commit_object "$local_repo" "$base_commit" $'Sujet\n\nCo-authored-by: Ariane Exemple <ariane@example.invalid>')"
  input="${fixture_root}/new-branch.input"
  printf 'refs/heads/topic %s refs/heads/topic %s\n' "$hostile" "$zero_oid" >"$input"
  expect_rejection "$input"
}

test_force_push() {
  create_fixture "force"
  local remote_old hostile input
  remote_old="$(commit_object "$local_repo" "$base_commit" "Ancienne tête propre")"
  publish_ref refs/heads/topic "$remote_old"
  hostile="$(commit_object "$local_repo" "$base_commit" $'Nouvelle histoire\n\nCo-Authored-By: Boris Exemple <boris@example.invalid>')"
  input="${fixture_root}/force.input"
  printf 'refs/heads/topic %s refs/heads/topic %s\n' "$hostile" "$remote_old" >"$input"
  expect_rejection "$input"
}

test_multi_ref_transaction() {
  create_fixture "multi-ref"
  local clean hostile input
  clean="$(commit_object "$local_repo" "$base_commit" "Branche propre")"
  hostile="$(commit_object "$local_repo" "$base_commit" $'Branche interdite\n\n co-authored-by : Céleste Exemple <celeste@example.invalid>')"
  input="${fixture_root}/multi-ref.input"
  {
    printf 'refs/heads/clean %s refs/heads/clean %s\n' "$clean" "$zero_oid"
    printf 'refs/heads/hostile %s refs/heads/hostile %s\n' "$hostile" "$zero_oid"
  } >"$input"
  expect_rejection "$input"
}

test_real_filter_with_unknown_name() {
  local message_file="${fixture_root}/unknown-author-message"
  printf '%s\n' 'Sujet' '' 'Co-authored-by: Dorine Inconnue <dorine@example.invalid>' >"$message_file"
  # shellcheck source=/dev/null
  source "$hook"
  message_has_forbidden_authorship "$message_file"
}

test_remote_legacy_and_clean_multi_ref() {
  create_fixture "legacy"
  local legacy clean_one clean_two input
  legacy="$(commit_object "$local_repo" "$base_commit" $'Dette déjà distante\n\nCo-authored-by: Émile Hérité <emile@example.invalid>')"
  publish_ref refs/heads/main "$legacy"
  clean_one="$(commit_object "$local_repo" "$legacy" "Correctif propre un")"
  clean_two="$(commit_object "$local_repo" "$clean_one" "Correctif propre deux")"
  input="${fixture_root}/legacy.input"
  {
    printf 'refs/heads/one %s refs/heads/one %s\n' "$clean_one" "$zero_oid"
    printf 'refs/heads/two %s refs/heads/two %s\n' "$clean_two" "$zero_oid"
  } >"$input"
  expect_acceptance "$input"
}

test_fail_closed() {
  create_fixture "fail-closed"
  local clean input missing_remote remote_only
  clean="$(commit_object "$local_repo" "$base_commit" "Tête locale propre")"
  input="${fixture_root}/fail-closed.input"
  printf 'refs/heads/topic %s refs/heads/topic %s\n' "$clean" "$zero_oid" >"$input"
  missing_remote="${fixture_root}/absent.git"
  expect_rejection "$input" "$missing_remote"

  remote_only="$(
    printf '%s\n' "Objet distant inconnu localement" |
      fixture_git --git-dir="$remote_repo" commit-tree "$(
        git --git-dir="$remote_repo" mktree </dev/null
      )" -p "$base_commit"
  )"
  git --git-dir="$remote_repo" update-ref refs/heads/remote-only "$remote_only"
  expect_rejection "$input"
}

test_syntax_mutation_blinds_filter() {
  local mutant="${fixture_root}/mutant-pre-push"
  local message_file="${fixture_root}/mutation-message"
  local mutations=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ "$line" == "readonly FORBIDDEN_AUTHORSHIP_PATTERN="* ]]; then
      printf "%s,'\n" "${line%?}" >>"$mutant"
      mutations=$((mutations + 1))
    else
      printf '%s\n' "$line" >>"$mutant"
    fi
  done <"$hook"
  [[ "$mutations" -eq 1 ]] || {
    echo "le motif de production n'a pas été trouvé exactement une fois" >&2
    return 1
  }
  printf '%s\n' 'Sujet' '' 'Co-authored-by: Farid Imprévu <farid@example.invalid>' >"$message_file"
  if (
    # shellcheck source=/dev/null
    source "$mutant"
    message_has_forbidden_authorship "$message_file"
  ); then
    echo "la mutation syntaxique devait rendre le motif aveugle" >&2
    return 1
  fi
}

run_test() {
  local name="$1"
  shift
  if (set -euo pipefail; "$@"); then
    printf 'PASS  %s\n' "$name"
    passed=$((passed + 1))
  else
    printf 'ROUGE %s\n' "$name"
    failed=$((failed + 1))
  fi
}

run_test "branche neuve sans amont" test_new_branch_without_upstream
run_test "envoi forcé" test_force_push
run_test "transaction multi-références" test_multi_ref_transaction
run_test "filtre réel avec nom inventé" test_real_filter_with_unknown_name
run_test "héritage distant et flux propre" test_remote_legacy_and_clean_multi_ref
run_test "échec fermé" test_fail_closed
run_test "mutation syntaxique du filtre" test_syntax_mutation_blinds_filter

printf 'RÉSULTAT: %d passés / %d rouges / 0 ignoré\n' "$passed" "$failed"
[[ "$failed" -eq 0 ]]
