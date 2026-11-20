#!/bin/sh
# Produce a detached OpenSSH signature (<file>.sig) in the bridget-release namespace.
set -eu

usage() {
    printf '%s\n' 'Usage: scripts/sign-release.sh --key <private-ssh-key> <file>...' >&2
    exit 2
}

key=${BRIDGET_RELEASE_SIGNING_KEY:-}
while [ $# -gt 0 ]; do
    case "$1" in
        --key)
            [ $# -ge 2 ] || usage
            key=$2
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        --)
            shift
            break
            ;;
        -*)
            usage
            ;;
        *)
            break
            ;;
    esac
done

[ -n "$key" ] || { printf '%s\n' 'Error: signing key required (--key or BRIDGET_RELEASE_SIGNING_KEY).' >&2; exit 2; }
[ -f "$key" ] && [ ! -L "$key" ] || { printf '%s\n' "Error: invalid signing key file: $key" >&2; exit 2; }
[ $# -ge 1 ] || usage
command -v ssh-keygen >/dev/null 2>&1 || { printf '%s\n' 'Error: ssh-keygen is required.' >&2; exit 2; }

for target in "$@"; do
    [ -f "$target" ] && [ ! -L "$target" ] || { printf '%s\n' "Error: target file missing or symlink: $target" >&2; exit 2; }
    sig="${target}.sig"
    [ ! -L "$sig" ] || { printf '%s\n' "Error: signature path is a symlink: $sig" >&2; exit 2; }
    rm -f "$sig"
    ssh-keygen -Y sign -f "$key" -n bridget-release "$target" </dev/null >/dev/null
    printf '%s\n' "$sig"
done
