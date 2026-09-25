#!/usr/bin/env bash
set -euo pipefail

readonly FORBIDDEN_AUTHORSHIP_PATTERN='^[[:space:]]*co-authored-by[[:space:]]*:'
pre_push_temp_dir=""

message_has_forbidden_authorship() {
  local message_file="$1"
  LC_ALL=C grep -Eiq "$FORBIDDEN_AUTHORSHIP_PATTERN" "$message_file"
}

refuse() {
  printf 'REFUS pre-push: %s\n' "$*" >&2
  exit 1
}

cleanup_pre_push_temp_dir() {
  case "$pre_push_temp_dir" in
    /*/bridget-pre-push.*)
      [[ ! -d "$pre_push_temp_dir" ]] || rm -rf -- "$pre_push_temp_dir"
      ;;
  esac
}

is_zero_oid() {
  local object_id="$1"
  [[ -n "$object_id" && "$object_id" != *[!0]* ]]
}

is_hex_oid() {
  local object_id="$1"
  [[ "$object_id" =~ ^[0-9a-fA-F]+$ ]]
}

append_commit_tip() {
  local object_id="$1"
  local destination="$2"
  local description="$3"
  local commit_id peeled_id peeled_type

  git cat-file -e "${object_id}^{object}" 2>/dev/null ||
    refuse "objet absent localement pour ${description}: ${object_id}; exécuter git fetch --prune --tags"

  if commit_id="$(git rev-parse --verify --quiet "${object_id}^{commit}")"; then
    printf '%s\n' "$commit_id" >>"$destination"
    return 0
  fi

  peeled_id="$(git rev-parse --verify --quiet "${object_id}^{}")" ||
    refuse "objet impossible à éplucher pour ${description}: ${object_id}"
  peeled_type="$(git cat-file -t "$peeled_id" 2>/dev/null)" ||
    refuse "type illisible pour ${description}: ${object_id}"
  case "$peeled_type" in
    tree|blob) return 0 ;;
    *) refuse "type non pris en charge pour ${description}: ${peeled_type}" ;;
  esac
}

main() {
  [[ "$#" -eq 2 ]] || refuse "deux arguments Git attendus: nom et destination du distant"
  local remote_name="$1"
  local remote_location="$2"
  [[ -n "$remote_name" && -n "$remote_location" ]] ||
    refuse "distant sans nom ou destination"

  local temp_parent="${TMPDIR:-/tmp}"
  [[ "$temp_parent" == /* ]] || refuse "TMPDIR doit être un chemin absolu"
  pre_push_temp_dir="$(mktemp -d "${temp_parent%/}/bridget-pre-push.XXXXXX")" ||
    refuse "création du répertoire temporaire impossible"
  trap cleanup_pre_push_temp_dir EXIT

  local updates_file="${pre_push_temp_dir}/updates"
  local local_objects_file="${pre_push_temp_dir}/local-objects"
  local local_commits_file="${pre_push_temp_dir}/local-commits"
  local remote_refs_file="${pre_push_temp_dir}/remote-refs"
  local remote_commits_file="${pre_push_temp_dir}/remote-commits"
  local remote_state_file="${pre_push_temp_dir}/remote-state"
  local revisions_file="${pre_push_temp_dir}/revisions"
  local introduced_file="${pre_push_temp_dir}/introduced"
  local message_file="${pre_push_temp_dir}/message"
  : >"$updates_file"
  : >"$local_objects_file"
  : >"$local_commits_file"
  : >"$remote_commits_file"

  local local_ref local_oid remote_ref remote_oid extra
  local update_count=0
  while IFS=' ' read -r local_ref local_oid remote_ref remote_oid extra; do
    [[ -n "${local_ref:-}" && -n "${local_oid:-}" &&
       -n "${remote_ref:-}" && -n "${remote_oid:-}" &&
       -z "${extra:-}" ]] ||
      refuse "ligne de mise à jour mal formée"
    is_hex_oid "$local_oid" || refuse "identifiant local invalide pour ${local_ref}"
    is_hex_oid "$remote_oid" || refuse "identifiant distant invalide pour ${remote_ref}"
    git check-ref-format "$remote_ref" >/dev/null 2>&1 ||
      refuse "référence distante invalide: ${remote_ref}"
    printf '%s\t%s\t%s\t%s\n' "$local_ref" "$local_oid" "$remote_ref" "$remote_oid" >>"$updates_file"
    update_count=$((update_count + 1))
    if ! is_zero_oid "$local_oid"; then
      printf '%s\n' "$local_oid" >>"$local_objects_file"
    fi
  done

  [[ "$update_count" -gt 0 ]] || exit 0

  if ! git ls-remote --refs -- "$remote_location" >"$remote_refs_file"; then
    refuse "état du distant ${remote_name} impossible à observer"
  fi

  local tab=$'\t'
  # O(R + U) : chaque référence observée et chaque mise à jour sont indexées une fois.
  if ! LC_ALL=C awk -F "$tab" '
    FILENAME == ARGV[1] {
      remote_count[$2] += 1
      remote_oid[$2] = $1
      next
    }
    FILENAME == ARGV[2] {
      remote_ref = $3
      announced_oid = $4
      if (remote_count[remote_ref] > 1) {
        printf "duplicate\t%s\n", remote_ref
        exit 10
      }
      observed_oid = remote_count[remote_ref] == 1 ? remote_oid[remote_ref] : ""
      if (announced_oid ~ /^0+$/) {
        if (observed_oid != "") {
          printf "concurrent-creation\t%s\n", remote_ref
          exit 11
        }
      } else if (observed_oid != announced_oid) {
        printf "divergent\t%s\n", remote_ref
        exit 12
      }
    }
  ' "$remote_refs_file" "$updates_file" >"$remote_state_file"; then
    local remote_state remote_state_ref remote_state_extra
    IFS=$'\t' read -r remote_state remote_state_ref remote_state_extra <"$remote_state_file" || true
    [[ -n "${remote_state_ref:-}" && -z "${remote_state_extra:-}" ]] ||
      refuse "comparaison de l'état distant impossible"
    case "$remote_state" in
      duplicate)
        refuse "référence distante dupliquée dans l'observation: ${remote_state_ref}"
        ;;
      concurrent-creation)
        refuse "la référence annoncée neuve existe désormais: ${remote_state_ref}"
        ;;
      divergent)
        refuse "état distant divergent pour ${remote_state_ref}; actualiser puis retenter"
        ;;
      *) refuse "comparaison de l'état distant impossible" ;;
    esac
  fi

  [[ -s "$local_objects_file" ]] || exit 0

  while IFS= read -r local_oid; do
    append_commit_tip "$local_oid" "$local_commits_file" "tête locale"
  done <"$local_objects_file"

  local advertised_oid advertised_ref advertised_extra
  while IFS=$'\t ' read -r advertised_oid advertised_ref advertised_extra; do
    [[ -n "${advertised_oid:-}" && -n "${advertised_ref:-}" &&
       -z "${advertised_extra:-}" ]] ||
      refuse "observation distante mal formée"
    is_hex_oid "$advertised_oid" ||
      refuse "identifiant distant observé invalide pour ${advertised_ref}"
    git check-ref-format "$advertised_ref" >/dev/null 2>&1 ||
      refuse "référence distante observée invalide: ${advertised_ref}"
    append_commit_tip "$advertised_oid" "$remote_commits_file" "référence distante ${advertised_ref}"
  done <"$remote_refs_file"

  [[ -s "$local_commits_file" ]] || exit 0
  cat "$local_commits_file" >"$revisions_file"
  if [[ -s "$remote_commits_file" ]]; then
    printf '%s\n' '--not' >>"$revisions_file"
    cat "$remote_commits_file" >>"$revisions_file"
  fi

  if ! git rev-list --stdin <"$revisions_file" >"$introduced_file"; then
    refuse "calcul des commits introduits impossible"
  fi

  local commit_id
  local forbidden_count=0
  # O(C) : C est le nombre de commits nouvellement atteignables.
  while IFS= read -r commit_id; do
    [[ -n "$commit_id" ]] || continue
    if ! git show -s --format=%B "$commit_id" >"$message_file"; then
      refuse "message illisible pour le commit ${commit_id}"
    fi
    local authorship_status
    if message_has_forbidden_authorship "$message_file"; then
      authorship_status=0
    else
      authorship_status=$?
    fi
    case "$authorship_status" in
      0)
        printf 'REFUS pre-push: co-autorat interdit dans le commit %s\n' "$commit_id" >&2
        forbidden_count=$((forbidden_count + 1))
        ;;
      1) ;;
      *) refuse "inspection du message impossible pour le commit ${commit_id}" ;;
    esac
  done <"$introduced_file"

  [[ "$forbidden_count" -eq 0 ]] ||
    refuse "${forbidden_count} commit(s) introduit(s) portent un co-autorat interdit"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
