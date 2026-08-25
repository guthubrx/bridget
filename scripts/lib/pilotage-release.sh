#!/usr/bin/env bash
# Frontière commune : un outil actif provient d'un commit admis, jamais du chantier.

pilotage_release_hash() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

pilotage_refuse() {
  echo "REFUS activation pilotage: $*" >&2
  return 1
}

# Contrat : le caller fournit la racine, le chemin Git, le nom public et le
# drapeau --force. Succès = artefact exact + preuve + lien actif ; refus Git =
# zéro effet. Les seules écritures sont sous ~/.local/{share,bin} après toutes
# les préconditions. Toute corruption d'une release existante est terminale.
pilotage_install_release() {
  local root_dir="$1"
  local source_relative="$2"
  local command_name="$3"
  local force="$4"
  local branch head remote_head status
  local installed_command release_base release_dir release_command origin_file
  local prepare_dir prepared_command prepared_origin artifact_hash
  local bin_dir link_prepare prepared_link

  case "$command_name" in
    bridget-idle|bridget-ronde) ;;
    *) pilotage_refuse "outil inconnu: $command_name"; return 1 ;;
  esac

  # Un worktree lié porte un fichier .git ; seul le checkout principal porte
  # le répertoire .git. Ce refus doit précéder tous les autres et toute écriture.
  if [[ ! -d "${root_dir}/.git" ]]; then
    pilotage_refuse "worktree lié interdit; activer depuis le checkout principal après jury et merge"
    return 1
  fi

  branch="$(git -C "$root_dir" symbolic-ref --quiet --short HEAD 2>/dev/null || true)"
  if [[ "$branch" != main ]]; then
    pilotage_refuse "branche main requise; tête actuelle: ${branch:-detachee}"
    return 1
  fi

  status="$(git -C "$root_dir" status --porcelain=v1 --untracked-files=normal)"
  if [[ -n "$status" ]]; then
    pilotage_refuse "arbre Git sale; committer ou restaurer avant activation"
    return 1
  fi

  head="$(git -C "$root_dir" rev-parse --verify 'HEAD^{commit}')" || {
    pilotage_refuse "HEAD Git illisible"
    return 1
  }
  installed_command="${HOME}/.local/bin/${command_name}"
  release_base="${HOME}/.local/share/bridget/pilotage/releases"
  release_dir="${release_base}/${head}"
  release_command="${release_dir}/${command_name}"
  origin_file="${release_command}.origin"

  if [[ -e "$installed_command" || -L "$installed_command" ]]; then
    if [[ -d "$installed_command" && ! -L "$installed_command" ]]; then
      pilotage_refuse "entrée active est un répertoire et ne peut pas être remplacée: $installed_command"
      return 1
    fi
    if [[ ! -L "$installed_command" || "$(readlink "$installed_command")" != "$release_command" ]]; then
      if [[ "$force" != 1 ]]; then
        pilotage_refuse "entrée active déjà présente: $installed_command (utiliser --force pour migrer explicitement)"
        return 1
      fi
    fi
  fi

  remote_head="$(git -C "$root_dir" rev-parse --verify 'refs/remotes/origin/main^{commit}' 2>/dev/null || true)"
  if [[ -z "$remote_head" ]]; then
    pilotage_refuse "origin/main absente; exécuter git fetch origin puis recommencer"
    return 1
  fi
  if ! git -C "$root_dir" merge-base --is-ancestor "$head" "$remote_head"; then
    pilotage_refuse "HEAD non admis par origin/main; pousser, faire juger et merger avant activation"
    return 1
  fi
  if ! git -C "$root_dir" cat-file -e "${head}:${source_relative}" 2>/dev/null; then
    pilotage_refuse "outil absent du commit admis: ${source_relative}"
    return 1
  fi

  mkdir -p "$release_base"
  prepare_dir="$(mktemp -d "${release_base}/.prepare-${command_name}.XXXXXX")"
  prepared_command="${prepare_dir}/${command_name}"
  prepared_origin="${prepare_dir}/${command_name}.origin"
  if ! git -C "$root_dir" show "${head}:${source_relative}" >"$prepared_command"; then
    rm -rf "$prepare_dir"
    pilotage_refuse "extraction Git impossible: ${head}:${source_relative}"
    return 1
  fi
  chmod 0555 "$prepared_command"
  artifact_hash="$(pilotage_release_hash "$prepared_command")"
  {
    printf 'format=bridget-pilotage-release-v1\n'
    printf 'remote=origin\n'
    printf 'source_ref=refs/remotes/origin/main\n'
    printf 'commit=%s\n' "$head"
    printf 'artifact=%s\n' "$command_name"
    printf 'sha256=%s\n' "$artifact_hash"
  } >"$prepared_origin"
  chmod 0444 "$prepared_origin"

  if [[ -e "$release_command" ]] && ! cmp -s "$release_command" "$prepared_command"; then
    rm -rf "$prepare_dir"
    pilotage_refuse "release corrompue pour ${head}: $release_command"
    return 1
  fi
  if [[ -e "$origin_file" ]] && ! cmp -s "$origin_file" "$prepared_origin"; then
    rm -rf "$prepare_dir"
    pilotage_refuse "preuve origine corrompue pour ${head}: $origin_file"
    return 1
  fi

  mkdir -p "$release_dir"
  if [[ ! -e "$release_command" ]]; then
    mv "$prepared_command" "$release_command"
  fi
  if [[ ! -e "$origin_file" ]]; then
    mv "$prepared_origin" "$origin_file"
  fi
  rm -rf "$prepare_dir"

  bin_dir="${HOME}/.local/bin"
  mkdir -p "$bin_dir"
  if [[ -L "$installed_command" && "$(readlink "$installed_command")" == "$release_command" ]]; then
    echo "déjà en place: $installed_command -> $release_command" >&2
  else
    link_prepare="$(mktemp -d "${bin_dir}/.activate-${command_name}.XXXXXX")"
    prepared_link="${link_prepare}/${command_name}"
    ln -s "$release_command" "$prepared_link"
    if ! mv -f "$prepared_link" "$installed_command"; then
      rm -rf "$link_prepare"
      pilotage_refuse "activation atomique impossible: $installed_command"
      return 1
    fi
    rmdir "$link_prepare"
    echo "posé: $installed_command -> $release_command" >&2
  fi
  echo "origine: refs/remotes/origin/main contient $head; preuve: $origin_file" >&2

  PILOTAGE_INSTALLED_COMMAND="$installed_command"
  PILOTAGE_RELEASE_COMMAND="$release_command"
  PILOTAGE_RELEASE_SHA="$head"
}
