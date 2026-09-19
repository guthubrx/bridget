#!/usr/bin/env bash
set -euo pipefail

readonly FAMILY_PREFIX='daemon::presence_tests::'
root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
temp_parent="${TMPDIR:-/tmp}"
fixture_root=""

refuse() {
  printf 'REFUS méta-gate 053: %s\n' "$*" >&2
  exit 1
}

cleanup() {
  case "$fixture_root" in
    */bridget-review-witnesses.*) rm -rf -- "$fixture_root" ;;
  esac
}

list_tests() {
  local output_file="$1"
  local error_file="$2"
  shift 2
  if ! (
    cd "$root_dir"
    "${CARGO:-cargo}" test -p bridget-daemon --lib -- --list "$@"
  ) >"$output_file" 2>"$error_file"; then
    sed -n '1,80p' "$error_file" >&2
    refuse "listing Cargo impossible"
  fi
}

extract_test_names() {
  local input_file="$1"
  local output_file="$2"
  awk '/: test$/ { sub(/: test$/, ""); print }' "$input_file" >"$output_file"
}

line_count() {
  awk 'END { print NR + 0 }' "$1"
}

selection_args=()
if [[ "$#" -gt 0 ]]; then
  [[ "$1" == "--" ]] || refuse "usage: $0 [-- <arguments libtest de sélection>]"
  shift
  selection_args=("$@")
fi

[[ "$temp_parent" == /* ]] || refuse "TMPDIR doit être absolu"
fixture_root="$(mktemp -d "${temp_parent%/}/bridget-review-witnesses.XXXXXX")" ||
  refuse "création du répertoire temporaire impossible"
trap cleanup EXIT

full_raw="${fixture_root}/full.raw"
selected_raw="${fixture_root}/selected.raw"
full_names="${fixture_root}/full.names"
selected_names="${fixture_root}/selected.names"
expected_family="${fixture_root}/expected.family"
selected_family="${fixture_root}/selected.family"
missing_family="${fixture_root}/missing.family"

list_tests "$full_raw" "${full_raw}.stderr"
list_tests "$selected_raw" "${selected_raw}.stderr" "${selection_args[@]}"
extract_test_names "$full_raw" "$full_names"
extract_test_names "$selected_raw" "$selected_names"

awk -v prefix="$FAMILY_PREFIX" 'index($0, prefix) == 1 { print }' \
  "$full_names" >"$expected_family"
awk -v prefix="$FAMILY_PREFIX" 'index($0, prefix) == 1 { print }' \
  "$selected_names" >"$selected_family"

expected_count="$(line_count "$expected_family")"
selected_total="$(line_count "$selected_names")"
[[ "$expected_count" -gt 0 ]] ||
  refuse "inventaire attendu inobservable: 0 témoin sous ${FAMILY_PREFIX}"
[[ "$selected_total" -gt 0 ]] ||
  refuse "sélection inobservable: 0 test listé; témoins attendus=${expected_count}"

# O(N) : chaque nom sélectionné puis attendu est visité une seule fois.
awk '
  FILENAME == ARGV[1] { selected[$0] = 1; next }
  !($0 in selected) { print }
' "$selected_family" "$expected_family" >"$missing_family"

missing_count="$(line_count "$missing_family")"
if [[ "$missing_count" -gt 0 ]]; then
  present_count=$((expected_count - missing_count))
  printf 'REFUS méta-gate 053: sélection incomplète: présents=%d/%d manquants=%d univers_sélection=%d\n' \
    "$present_count" "$expected_count" "$missing_count" "$selected_total" >&2
  while IFS= read -r missing_name; do
    printf 'MANQUANT %s\n' "$missing_name" >&2
  done <"$missing_family"
  exit 1
fi

printf 'méta-gate 053: sélection complète: présents=%d/%d univers_sélection=%d\n' \
  "$expected_count" "$expected_count" "$selected_total"
