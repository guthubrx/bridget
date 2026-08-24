#!/bin/sh
# Vérifie le corpus public Bridget A publié au gate G-1601.
set -eu

root=$(git rev-parse --show-toplevel)
cd "$root"

pinned=cb7a6d8c6f122da38dccc4c2ebe643f4bf0cac74
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT HUP INT TERM

git merge-base --is-ancestor "$pinned" HEAD || {
    echo "G-1601 invalide : le producteur A $pinned n'est pas ancêtre de HEAD" >&2
    exit 1
}

specs/016-coordination-active/contracts/verify-amont-015.sh >/dev/null

expected_sha() {
    case "$1" in
        specs/016-coordination-active/contracts/fixtures/service-frames-v1.jsonl)
            echo 42e0ea32690e267221804a8af9aee538162f7c141a9cffacf6d6a44a9a8cda1f
            ;;
        specs/016-coordination-active/contracts/fixtures/coordination-events-v1.jsonl)
            echo 257215547c2858ee83fb64c61aeacdecbbdaee96df5661ced4e7403aaf40b590
            ;;
        specs/016-coordination-active/contracts/fixtures/coordination-stream-v2.jsonl)
            echo a384b492b53644a9357c5254df2d46b80b36b7d5761987d88d21c336dacdf00c
            ;;
        *)
            echo "fixture hors corpus A : $1" >&2
            exit 2
            ;;
    esac
}

verify_fixture() {
    candidate=$1
    pinned_path=$2
    expected=$(expected_sha "$pinned_path")
    source="$tmpdir/$(basename "$pinned_path")"

    git show "$pinned:$pinned_path" > "$source"
    actual=$(shasum -a 256 "$candidate" | awk '{print $1}')
    test "$actual" = "$expected" || {
        echo "corpus A divergent : $candidate" >&2
        return 1
    }
    cmp -s "$source" "$candidate" || {
        echo "corpus A et objet Git épinglé différents : $candidate" >&2
        return 1
    }
}

fixtures='
specs/016-coordination-active/contracts/fixtures/service-frames-v1.jsonl
specs/016-coordination-active/contracts/fixtures/coordination-events-v1.jsonl
specs/016-coordination-active/contracts/fixtures/coordination-stream-v2.jsonl'

if [ "${1:-}" = "--self-test" ]; then
    fixture=specs/016-coordination-active/contracts/fixtures/coordination-stream-v2.jsonl
    mutated="$tmpdir/coordination-stream-v2-mutated.jsonl"
    sed '1s/{/[/' "$fixture" > "$mutated"
    if verify_fixture "$mutated" "$fixture"; then
        echo "auto-test invalide : la mutation d'un octet a été acceptée" >&2
        exit 1
    fi
    echo "auto-test vérifié : la mutation d'un octet est refusée"
    exit 0
fi

for fixture in $fixtures; do
    verify_fixture "$fixture" "$fixture"
done

echo "G-1601 vérifié : corpus Bridget A relisible octet pour octet"
