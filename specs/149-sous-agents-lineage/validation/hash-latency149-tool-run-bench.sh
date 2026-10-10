#!/bin/bash
# usage: run-bench.sh <ws-r5|ws-r6> <debug|release> <bin-client> <label> <sortie.json>
umask 077
export TMPDIR=/tmp/b149t-r6
ws=$1; prof=$2; client=$3; label=$4; out=$5
bin=$(ls -t /tmp/b149t-r6/t-$ws/$prof/deps/bridget_daemon-* | grep -v '\.d$' | head -1)
cd /tmp/b149t-r6/$ws/crates/bridget-daemon || exit 1
BENCH_LABEL=$label BENCH_CLI=/Users/moi/.local/share/claude/versions/2.1.296 \
BENCH_EXPECTED_SHA=c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937 \
BENCH_CLIENT_BIN=$client BENCH_GCLAUDE=/Users/moi/.local/bin/gclaude \
"$bin" --ignored bench149_latence_hachage_cli_reel --nocapture --test-threads=1 > "$out.raw" 2>&1
echo "exit=$?" >> "$out.raw"
grep -o 'BENCH149 .*' "$out.raw" | sed 's/^BENCH149 //' > "$out"
