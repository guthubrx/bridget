#!/usr/bin/env bash
set -u -o pipefail

test_script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
test_script="${test_script_dir}/${BASH_SOURCE[0]##*/}"
root_dir="$(cd "${test_script_dir}/.." && pwd)"
hook="${BRIDGET_PRE_PUSH_HOOK:-${root_dir}/scripts/git-pre-push-authorship.sh}"
test_temp_parent="${TMPDIR:-/tmp}"
[[ "$test_temp_parent" == /* ]] || {
  echo "TMPDIR de test doit être un chemin absolu" >&2
  exit 2
}
fixture_root=""
if ! fixture_root="$(mktemp -d "${test_temp_parent%/}/bridget-pre-push-test.XXXXXX")"; then
  echo "création du répertoire temporaire de test impossible" >&2
  exit 2
fi
passed=0
failed=0

cleanup() {
  case "$fixture_root" in
    /*/bridget-pre-push-test.*) rm -rf -- "$fixture_root" ;;
  esac
}
trap cleanup EXIT

if [[ "${BRIDGET_PRE_PUSH_TEST_INITIALIZATION_ONLY:-0}" == "1" ]]; then
  printf 'FIXTURE_ROOT_DERIVED=%s\n' "${fixture_root}/probe"
  exit 0
fi

write_executable() {
  local destination="$1"
  shift
  printf '%s\n' "$@" >"$destination"
  chmod +x "$destination"
}

assert_stderr_contains() {
  local input_file="$1"
  local expected="$2"
  LC_ALL=C grep -Fq -- "$expected" "${input_file}.stderr" || {
    printf 'diagnostic absent: %s\n' "$expected" >&2
    sed -n '1,20p' "${input_file}.stderr" >&2
    return 1
  }
}

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

test_filter_error_refuses_hostile_commit() {
  create_fixture "filter-error"
  local hostile input fake_bin
  hostile="$(commit_object "$local_repo" "$base_commit" $'Sujet hostile\n\nCo-authored-by: Gaëlle Inconnue <gaelle@example.invalid>')"
  input="${fixture_root}/filter-error.input"
  printf 'refs/heads/topic %s refs/heads/topic %s\n' "$hostile" "$zero_oid" >"$input"
  fake_bin="${fixture_root}/filter-error-bin"
  mkdir -p "$fake_bin"
  write_executable "${fake_bin}/grep" \
    '#!/usr/bin/env bash' \
    'printf "%s\n" "FAKE_GREP_READ_ERROR" >&2' \
    'exit 2'

  PATH="${fake_bin}:${PATH}" expect_rejection "$input" || return 1
  assert_stderr_contains "$input" "FAKE_GREP_READ_ERROR"
  assert_stderr_contains "$input" "inspection du message impossible pour le commit ${hostile}"
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

test_unknown_remote_refuses() {
  create_fixture "unknown-remote"
  local clean input missing_remote
  clean="$(commit_object "$local_repo" "$base_commit" "Tête locale propre")"
  input="${fixture_root}/unknown-remote.input"
  printf 'refs/heads/topic %s refs/heads/topic %s\n' "$clean" "$zero_oid" >"$input"
  missing_remote="${fixture_root}/absent.git"
  expect_rejection "$input" "$missing_remote" || return 1
  assert_stderr_contains "$input" "état du distant origin impossible à observer"
}

test_missing_remote_object_refuses() {
  create_fixture "missing-object"
  local clean input remote_only
  clean="$(commit_object "$local_repo" "$base_commit" "Tête locale propre")"
  input="${fixture_root}/missing-object.input"
  printf 'refs/heads/topic %s refs/heads/topic %s\n' "$clean" "$zero_oid" >"$input"
  remote_only="$(
    printf '%s\n' "Objet distant inconnu localement" |
      fixture_git --git-dir="$remote_repo" commit-tree "$(
        git --git-dir="$remote_repo" mktree </dev/null
      )" -p "$base_commit"
  )"
  git --git-dir="$remote_repo" update-ref refs/heads/remote-only "$remote_only"
  expect_rejection "$input" || return 1
  assert_stderr_contains "$input" "objet absent localement pour référence distante refs/heads/remote-only: ${remote_only}"
}

test_remote_divergence_refuses() {
  create_fixture "remote-divergence"
  local remote_actual clean input
  remote_actual="$(commit_object "$local_repo" "$base_commit" "État distant réel")"
  publish_ref refs/heads/topic "$remote_actual"
  clean="$(commit_object "$local_repo" "$remote_actual" "Tête locale propre")"
  input="${fixture_root}/remote-divergence.input"
  printf 'refs/heads/topic %s refs/heads/topic %s\n' "$clean" "$base_commit" >"$input"
  expect_rejection "$input" || return 1
  assert_stderr_contains "$input" "état distant divergent pour refs/heads/topic; actualiser puis retenter"
}

test_rev_list_failure_refuses() {
  create_fixture "rev-list-error"
  local clean input fake_bin real_git
  clean="$(commit_object "$local_repo" "$base_commit" "Tête locale propre")"
  input="${fixture_root}/rev-list-error.input"
  printf 'refs/heads/topic %s refs/heads/topic %s\n' "$clean" "$zero_oid" >"$input"
  fake_bin="${fixture_root}/rev-list-error-bin"
  mkdir -p "$fake_bin"
  real_git="$(command -v git)"
  write_executable "${fake_bin}/git" \
    '#!/usr/bin/env bash' \
    'if [[ "${1:-}" == "rev-list" ]]; then' \
    '  printf "%s\n" "FAKE_REV_LIST_ERROR" >&2' \
    '  exit 42' \
    'fi' \
    'exec "${BRIDGET_REAL_GIT:?}" "$@"'

  BRIDGET_REAL_GIT="$real_git" PATH="${fake_bin}:${PATH}" \
    expect_rejection "$input" || return 1
  assert_stderr_contains "$input" "FAKE_REV_LIST_ERROR"
  assert_stderr_contains "$input" "calcul des commits introduits impossible"
}

test_remote_lookup_is_linear() {
  create_fixture "linear-lookup"
  local input fake_bin scan_log clean index scan_count
  input="${fixture_root}/linear-lookup.input"
  fake_bin="${fixture_root}/linear-lookup-bin"
  scan_log="${fixture_root}/linear-lookup.scans"
  mkdir -p "$fake_bin"
  : >"$input"
  : >"$scan_log"
  for index in 1 2 3 4 5; do
    clean="$(commit_object "$local_repo" "$base_commit" "Tête propre ${index}")"
    printf 'refs/heads/topic-%s %s refs/heads/topic-%s %s\n' \
      "$index" "$clean" "$index" "$zero_oid" >>"$input"
  done
  write_executable "${fake_bin}/awk" \
    '#!/usr/bin/env bash' \
    'printf "%s\n" scan >>"${BRIDGET_AWK_SCAN_LOG:?}"' \
    'exec "${BRIDGET_REAL_AWK:?}" "$@"'

  BRIDGET_AWK_SCAN_LOG="$scan_log" BRIDGET_REAL_AWK="$(command -v awk)" \
    PATH="${fake_bin}:${PATH}" expect_acceptance "$input" || return 1
  scan_count="$(wc -l <"$scan_log" | tr -d ' ')"
  [[ "$scan_count" -le 1 ]] || {
    printf 'les références distantes ont été rescannées %s fois pour cinq mises à jour\n' \
      "$scan_count" >&2
    return 1
  }
}

test_remote_index_failure_refuses() {
  create_fixture "remote-index-error"
  local clean input fake_bin
  clean="$(commit_object "$local_repo" "$base_commit" "Tête locale propre")"
  input="${fixture_root}/remote-index-error.input"
  printf 'refs/heads/topic %s refs/heads/topic %s\n' "$clean" "$zero_oid" >"$input"
  fake_bin="${fixture_root}/remote-index-error-bin"
  mkdir -p "$fake_bin"
  write_executable "${fake_bin}/awk" \
    '#!/usr/bin/env bash' \
    'printf "%s\n" "FAKE_AWK_INDEX_ERROR" >&2' \
    'exit 2'

  PATH="${fake_bin}:${PATH}" expect_rejection "$input" || return 1
  assert_stderr_contains "$input" "FAKE_AWK_INDEX_ERROR"
  assert_stderr_contains "$input" "comparaison de l'état distant impossible"
}

test_mktemp_failure_stops_before_path_derivation() {
  local fake_bin fake_root stdout_file stderr_file
  fake_bin="${fixture_root}/mktemp-error-bin"
  fake_root="${fixture_root}/mktemp-must-fail"
  stdout_file="${fixture_root}/mktemp-error.stdout"
  stderr_file="${fixture_root}/mktemp-error.stderr"
  mkdir -p "$fake_bin"
  write_executable "${fake_bin}/mktemp" \
    '#!/usr/bin/env bash' \
    'printf "%s\n" "${BRIDGET_FAKE_MKTEMP_PATH:?}"' \
    'exit 1'

  if BRIDGET_FAKE_MKTEMP_PATH="$fake_root" \
    BRIDGET_PRE_PUSH_TEST_INITIALIZATION_ONLY=1 \
    PATH="${fake_bin}:${PATH}" \
    bash "$test_script" >"$stdout_file" 2>"$stderr_file"; then
    echo "le banc a ignoré l'échec de mktemp" >&2
    return 1
  fi
  LC_ALL=C grep -Fq -- "création du répertoire temporaire de test impossible" "$stderr_file" || {
    echo "le banc n'a pas nommé l'échec de mktemp" >&2
    return 1
  }
  if LC_ALL=C grep -Fq -- "FIXTURE_ROOT_DERIVED=" "$stdout_file"; then
    echo "un chemin a été dérivé après l'échec de mktemp" >&2
    return 1
  fi
}

test_runner_stops_on_first_fixture_error() {
  local stdout_file stderr_file
  stdout_file="${fixture_root}/runner-error.stdout"
  stderr_file="${fixture_root}/runner-error.stderr"
  if BRIDGET_PRE_PUSH_TEST_RUNNER_PROBE=1 BRIDGET_PRE_PUSH_HOOK="$hook" \
    bash "$test_script" >"$stdout_file" 2>"$stderr_file"; then
    echo "le runner a transformé une erreur de fixture en succès" >&2
    return 1
  fi
  LC_ALL=C grep -Fq -- "ROUGE erreur interne de fixture" "$stdout_file" || {
    echo "le runner n'a pas compté l'erreur de fixture" >&2
    return 1
  }
  if LC_ALL=C grep -Fq -- "FIXTURE_CONTINUED_AFTER_ERROR" "$stdout_file"; then
    echo "la fixture a continué après sa première erreur" >&2
    return 1
  fi
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
  local test_status
  shift
  (set -euo pipefail; "$@")
  test_status=$?
  if [[ "$test_status" -eq 0 ]]; then
    printf 'PASS  %s\n' "$name"
    passed=$((passed + 1))
  else
    printf 'ROUGE %s\n' "$name"
    failed=$((failed + 1))
  fi
}

if [[ "${BRIDGET_PRE_PUSH_TEST_RUNNER_PROBE:-0}" == "1" ]]; then
  test_probe_fixture_error() {
    false
    printf '%s\n' "FIXTURE_CONTINUED_AFTER_ERROR"
  }
  run_test "erreur interne de fixture" test_probe_fixture_error
  printf 'RÉSULTAT: %d passés / %d rouges / 0 ignoré\n' "$passed" "$failed"
  [[ "$failed" -eq 0 ]]
  exit
fi

run_test "branche neuve sans amont" test_new_branch_without_upstream
run_test "envoi forcé" test_force_push
run_test "transaction multi-références" test_multi_ref_transaction
run_test "filtre réel avec nom inventé" test_real_filter_with_unknown_name
run_test "erreur du filtre refuse le commit hostile" test_filter_error_refuses_hostile_commit
run_test "héritage distant et flux propre" test_remote_legacy_and_clean_multi_ref
run_test "distant inconnu refusé" test_unknown_remote_refuses
run_test "objet distant absent refusé" test_missing_remote_object_refuses
run_test "état distant divergent refusé" test_remote_divergence_refuses
run_test "échec du calcul rev-list refusé" test_rev_list_failure_refuses
run_test "index distant linéaire" test_remote_lookup_is_linear
run_test "échec de l'index distant refusé" test_remote_index_failure_refuses
run_test "échec mktemp avant dérivation" test_mktemp_failure_stops_before_path_derivation
run_test "runner arrêté sur erreur de fixture" test_runner_stops_on_first_fixture_error
run_test "mutation syntaxique du filtre" test_syntax_mutation_blinds_filter

printf 'RÉSULTAT: %d passés / %d rouges / 0 ignoré\n' "$passed" "$failed"
[[ "$failed" -eq 0 ]]
