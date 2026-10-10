#!/bin/bash
umask 077
D=/Volumes/8TB2/50-repos-archives/validation-cache
O=/tmp/b149t-r6/bench-out
R5=$D/bridget149-debug/bridget-ec6b18b5d468
R6=$D/bridget149-debug/bridget-620e729fca53
REL=$D/bridget149-release/bridget-abfb346e23cc
run(){ /tmp/b149t-r6/tools/run-bench.sh "$1" "$2" "$3" "$4" "$O/$4.json"; echo "$4 fini $(date +%T)" >> $O/status; }
rm -f $O/status
# Passe A
run ws-r5 debug $R5 A1-debug-r5
run ws-r6 debug $R6 A2-debug-r6
run ws-r6 release $REL A3-release-r6
run ws-r5 release $REL A4-release-r5-ref
# Passe B (ordre inverse)
run ws-r5 release $REL B1-release-r5-ref
run ws-r6 release $REL B2-release-r6
run ws-r6 debug $R6 B3-debug-r6
run ws-r5 debug $R5 B4-debug-r5
echo DONE >> $O/status
