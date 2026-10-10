#!/bin/bash
umask 077
export TMPDIR=/tmp/b149t-r6 CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0
V=/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation
for spec in "ws-r5 debug" "ws-r6 debug" "ws-r6 release" "ws-r5 release"; do
  set -- $spec; ws=$1; prof=$2
  flag=""; [ "$prof" = release ] && flag="--release"
  cd /tmp/b149t-r6/$ws || exit 1
  CARGO_TARGET_DIR=/tmp/b149t-r6/t-$ws /usr/bin/time -p cargo test -p bridget-daemon --lib --locked --offline $flag --no-run > $V/hash-latency149-build-$ws-$prof.log 2>&1
  echo "$ws $prof RC=$?" >> /tmp/b149t-r6/build-bench.status
done
echo DONE >> /tmp/b149t-r6/build-bench.status
