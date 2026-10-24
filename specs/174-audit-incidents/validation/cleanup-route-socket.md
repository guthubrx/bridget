# macOS routing sockets and cleanup

The real macOS lsof capture had one rejected record: descriptor 8, type rte, empty name. The installed lsof manual defines rte as an AF_ROUTE socket. It is not a filesystem object. The existing closed non-file list now includes this exact type. Unknown types and inaccessible real files remain rejected.

The existing kernel-object test includes the observed record. T3 useThreadActions removes a worktree explicitly after confirmation and does not use this process-file scanner. Bridget keeps its stronger observation guard; only the documented non-file type is corrected. No sudo rule, permission or force removal is added.
