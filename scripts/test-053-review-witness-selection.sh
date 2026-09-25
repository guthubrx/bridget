#!/usr/bin/env bash
set -u -o pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
gate="${root_dir}/scripts/check-review-witness-selection.sh"
temp_parent="${TMPDIR:-/tmp}"
[[ "$temp_parent" == /* ]] || {
  printf 'TMPDIR doit être absolu\n' >&2
  exit 2
}
fixture_root="$(mktemp -d "${temp_parent%/}/bridget-review-witness-test.XXXXXX")" || exit 2
passed=0
failed=0

cleanup() {
  case "$fixture_root" in
    */bridget-review-witness-test.*) rm -rf -- "$fixture_root" ;;
  esac
}
trap cleanup EXIT

full_list="${fixture_root}/full.list"
if ! cargo test -p bridget-daemon --lib -- --list >"$full_list" 2>"${full_list}.stderr"; then
  sed -n '1,80p' "${full_list}.stderr" >&2
  exit 2
fi
omitted_witness="$(awk '
  /^daemon::presence_tests::.*: test$/ {
    sub(/: test$/, "")
    print
    exit
  }
' "$full_list")"
[[ -n "$omitted_witness" ]] || {
  printf 'univers de contrôle muet: aucun témoin presence_tests\n' >&2
  exit 2
}

selection_amputee_est_refusee() {
  local output status
  output="$($gate -- --skip "$omitted_witness" 2>&1)"
  status=$?
  printf '%s\n' "$output"
  [[ "$status" -ne 0 ]] || {
    printf 'la sélection amputée a été acceptée: %s\n' "$omitted_witness" >&2
    return 1
  }
  [[ "$output" == *"MANQUANT ${omitted_witness}"* ]] || {
    printf 'le refus ne nomme pas le témoin amputé: %s\n' "$omitted_witness" >&2
    return 1
  }
}

selection_complete_est_acceptee() {
  local output
  output="$($gate 2>&1)" || {
    printf '%s\n' "$output" >&2
    return 1
  }
  printf '%s\n' "$output"
  [[ "$output" == *'sélection complète'* ]]
}

selection_vide_est_inobservable() {
  local output status
  output="$($gate -- --exact session_053_aucun_test 2>&1)"
  status=$?
  printf '%s\n' "$output"
  [[ "$status" -ne 0 ]] || {
    printf 'une sélection vide a été acceptée\n' >&2
    return 1
  }
  [[ "$output" == *'sélection inobservable'* ]]
}

tests=(
  selection_amputee_est_refusee
  selection_complete_est_acceptee
  selection_vide_est_inobservable
)

printf 'univers session-053 (3 scénarios): sélection amputée, sélection complète, sélection vide\n'
for test_name in "${tests[@]}"; do
  if "$test_name"; then
    printf 'session-053 %s ... ok\n' "$test_name"
    passed=$((passed + 1))
  else
    printf 'session-053 %s ... FAILED\n' "$test_name"
    failed=$((failed + 1))
  fi
done

printf 'session-053 result: %d passed / %d failed / 0 ignored\n' "$passed" "$failed"
[[ "$failed" -eq 0 ]]
