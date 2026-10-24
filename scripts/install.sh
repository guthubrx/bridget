#!/bin/sh
# Bridget user installer. O(archive size); no service or account changes.
set -eu
umask 077

fail() { printf 'Bridget: %s\n' "$*" >&2; exit 1; }
say() { printf 'Bridget: %s\n' "$*" >&2; }
need() { command -v "$1" >/dev/null 2>&1 || fail "Missing prerequisite: $1"; }
download() {
    curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL \
        --connect-timeout 15 --max-time 300 --retry 2 -o "$2" "$1"
}
digest() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d ' ' -f 1
    else
        shasum -a 256 "$1" | cut -d ' ' -f 1
    fi
}

main() {
    mode=release
    version=
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --source) mode=source; shift ;;
            --version)
                [ "$#" -ge 2 ] || fail '--version needs a release tag.'
                version=$2; shift 2 ;;
            --help|-h)
                printf '%s\n' 'Usage: sh install.sh [--source | --version vX.Y.Z]' \
                    'Default: latest published release. --source: build current main.' \
                    'User installation; no sudo, service restart or provider login.'
                return ;;
            *) fail "Unknown option: $1" ;;
        esac
    done
    [ "$mode" != source ] || [ -z "$version" ] || fail '--source and --version are mutually exclusive.'
    [ "$(id -u)" != 0 ] || fail 'Run as your normal user, not root or sudo.'
    case "${HOME:-}" in /*) ;; *) fail 'HOME must be an absolute directory.' ;; esac
    case "$HOME" in *'
'*) fail 'HOME must not contain a newline.' ;; esac
    HOME=$(cd "$HOME" && pwd -P)
    export HOME
    need curl
    need tar
    command -v sha256sum >/dev/null 2>&1 || need shasum
    os=$(uname -s)
    arch=$(uname -m)
    case "$os" in Darwin) platform=darwin ;; Linux) platform=linux ;; *) fail "Unsupported OS: $os" ;; esac
    case "$arch" in arm64|aarch64) arch=arm64 ;; x86_64|amd64) arch=x86_64 ;; *) fail "Unsupported architecture: $arch" ;; esac
    repo=https://github.com/guthubrx/bridget
    bindir=$HOME/.local/bin
    state=$HOME/.local/state/bridget-install
    envdir=$HOME/.local/share/bridget
    # Reject redirected installation directories. HOME itself may be canonicalized by the OS.
    for dir in "$HOME/.local" "$bindir" "$HOME/.local/state" "$state" "$HOME/.local/share" "$envdir"; do
        [ ! -L "$dir" ] || fail "Installation directory is a symlink: $dir"
    done
    mkdir -p "$bindir" "$state" "$envdir"
    chmod 700 "$state"
    lock=$state/lock
    mkdir "$lock" 2>/dev/null || fail "Another installation is running (or interrupted): inspect $lock"
    work=
    staged=
    new=
    trap 'test -z "$staged" || rm -f "$staged"; test -z "$new" || rm -f "$new"; test -z "$work" || rm -rf "$work"; rm -f "$lock/pid"; rmdir "$lock"' EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    printf '%s\n' "$$" > "$lock/pid"
    work=$(mktemp -d "$state/stage.XXXXXXXX")
    candidate=$work/bridget

    if [ "$mode" = release ]; then
        if [ -z "$version" ]; then
            latest=$(curl --proto '=https' --proto-redir '=https' --tlsv1.2 -fsSL \
                --connect-timeout 15 --max-time 60 -o /dev/null -w '%{url_effective}' "$repo/releases/latest")
            case "$latest" in "$repo"/releases/tag/*) version=${latest##*/} ;; *) fail 'Cannot resolve the latest release.' ;; esac
        fi
        case "$version" in *'
'*) fail 'Invalid release tag.' ;; esac
        printf '%s\n' "$version" | LC_ALL=C grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+([.-][A-Za-z0-9.-]+)?$' || fail 'Invalid release tag.'
        stem=bridget-${version#v}-$platform-$arch
        archive=$stem.tar.gz
        say "Downloading $version for $platform-$arch."
        download "$repo/releases/download/$version/$archive.sha256" "$work/checksum" || fail 'No checksum/package for this platform or version. Use --source explicitly if needed.'
        expected=$(awk 'NR == 1 {print $1}' "$work/checksum")
        [ "${#expected}" = 64 ] || fail 'Invalid SHA-256 sidecar.'
        case "$expected" in *[!0-9a-fA-F]*) fail 'Invalid SHA-256 digest.' ;; esac
        expected=$(printf '%s' "$expected" | tr 'A-F' 'a-f')
        download "$repo/releases/download/$version/$archive" "$work/package.tar.gz"
        [ "$(digest "$work/package.tar.gz")" = "$expected" ] || fail 'SHA-256 mismatch. Installed Bridget was not changed.'
        # Stream only the known member, never extract arbitrary archive paths or links.
        tar -xzOf "$work/package.tar.gz" "$stem/bridget" > "$candidate"
        identity=$version
    else
        need git
        need python3
        need cc
        python3 -c 'import sys; sys.exit(sys.version_info < (3, 11))' || fail 'Python 3.11 or later is required for --source.'
        say 'Fetching current main. Source builds take longer than release installs.'
        source=$state/source
        [ ! -L "$source" ] && [ ! -L "$source/.git" ] || fail 'Source checkout must not be a symlink.'
        if [ ! -e "$source" ]; then
            git -c core.hooksPath=/dev/null clone --quiet --depth 1 --single-branch --branch main "$repo.git" "$work/source"
            mv "$work/source" "$source"
        else
            [ -d "$source/.git" ] || fail "Invalid installer checkout: $source"
            [ "$(git -C "$source" remote get-url origin)" = "$repo.git" ] || fail 'Installer checkout has a different origin.'
            [ -z "$(git -C "$source" status --porcelain --untracked-files=all)" ] || fail "Installer checkout was edited: preserve your work in $source before retrying."
            git -C "$source" fetch --quiet --depth 1 origin refs/heads/main
            git -C "$source" -c core.hooksPath=/dev/null checkout --quiet --detach FETCH_HEAD
        fi
        identity=$(git -C "$source" rev-parse HEAD)
        export PATH="$HOME/.cargo/bin:$PATH"
        BRIDGET_BUILD_ID=$identity
        export BRIDGET_BUILD_ID
        unset RUSTUP_TOOLCHAIN
        if ! command -v rustup >/dev/null 2>&1; then
            download https://sh.rustup.rs "$work/rustup.sh"
            sh "$work/rustup.sh" -y --profile minimal --no-modify-path --default-toolchain none
        fi
        need cargo
        # Same registered cache on every invocation; the existing wrapper owns its budget.
        CARGO_TARGET_DIR=$HOME/.cache/bridget-build-installer
        export CARGO_TARGET_DIR
        (cd "$source" && python3 scripts/build.py cargo build --locked --release -p bridget-daemon)
        cp "$CARGO_TARGET_DIR/release/bridget" "$candidate"
    fi
    [ -s "$candidate" ] || fail 'Package contains no Bridget executable.'
    chmod 755 "$candidate"
    "$candidate" --version > "$work/version" || fail 'Candidate cannot run. Installed Bridget was not changed.'
    grep -qi 'bridget' "$work/version" || fail 'Unexpected candidate version response.'

    # Prepare PATH before replacing the executable; preserve existing shell settings.
    [ ! -L "$envdir/env" ] || fail "Symlink refused: $envdir/env"
    printf '%s\n' 'case ":$PATH:" in *":$HOME/.local/bin:"*) ;; *) export PATH="$HOME/.local/bin:$PATH" ;; esac' > "$envdir/env"
    profiles="$HOME/.profile"
    case "${SHELL:-}" in
        */zsh) profiles="$profiles
$HOME/.zshrc
$HOME/.zprofile" ;;
        */bash) profiles="$profiles
$HOME/.bashrc"
            # Do not create .bash_profile: it would hide existing .profile/.bash_login.
            if [ -e "$HOME/.bash_profile" ] || [ -L "$HOME/.bash_profile" ]; then
                profiles="$profiles
$HOME/.bash_profile"
            elif [ -e "$HOME/.bash_login" ] || [ -L "$HOME/.bash_login" ]; then
                profiles="$profiles
$HOME/.bash_login"
            fi ;;
    esac
    oldifs=$IFS
    IFS='
'
    # Validate every destination before changing the first profile.
    for profile in $profiles; do
        [ ! -L "$profile" ] || fail "Shell profile is a symlink: $profile. Configure PATH manually."
    done
    for profile in $profiles; do
        if ! grep -Fq '. "$HOME/.local/share/bridget/env"' "$profile" 2>/dev/null; then
            if [ -f "$profile" ] && [ ! -e "$profile.before-bridget-install" ] && [ ! -L "$profile.before-bridget-install" ]; then
                cp -p "$profile" "$profile.before-bridget-install"
            fi
            staged=$(mktemp "$profile.bridget.XXXXXXXX")
            if [ -f "$profile" ]; then cp -p "$profile" "$staged"; fi
            { printf '%s\n' '. "$HOME/.local/share/bridget/env"'; test ! -f "$profile" || cat "$profile"; } > "$staged"
            mv -f "$staged" "$profile"
        fi
    done
    IFS=$oldifs
    [ ! -d "$bindir/bridget" ] || fail "Destination is a directory: $bindir/bridget"
    if [ -e "$bindir/bridget" ] && [ "$(digest "$bindir/bridget")" != "$(digest "$candidate")" ]; then
        cp -pL "$bindir/bridget" "$work/previous"
        mv -f "$work/previous" "$state/previous-bridget"
    fi
    # Stage on the destination filesystem, then rename without following the old symlink.
    new=$(mktemp "$bindir/.bridget-install.XXXXXXXX")
    if ! cp "$candidate" "$new" || ! chmod 755 "$new" || ! mv -f "$new" "$bindir/bridget"; then
        rm -f "$new"
        fail 'Could not replace the executable.'
    fi
    printf '%s\n' "channel=$mode" "source=$identity" "sha256=$(digest "$bindir/bridget")" > "$work/installed.txt"
    mv -f "$work/installed.txt" "$state/installed.txt"
    if [ "$mode" = source ]; then
        (cd "$source" && python3 scripts/build.py installed --target "$CARGO_TARGET_DIR" --binary "$bindir/bridget")
    fi
    say "Installed $(cat "$work/version") ($identity) at $bindir/bridget"
    say 'For this terminal: . "$HOME/.local/share/bridget/env"'
    say 'No services were started or restarted. Existing processes keep their loaded version.'
    say 'Standalone use: bridget daemon. Join an existing master: see bridget federate --help.'
    say 'Claude/Codex accounts and Bridget data were preserved.'
}

# Run a complete function: a truncated download cannot execute half an installation.
main "$@"
