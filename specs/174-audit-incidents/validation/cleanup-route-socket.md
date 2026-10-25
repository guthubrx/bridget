# macOS routing sockets and cleanup

The real macOS lsof capture had one rejected record: descriptor 8, type rte, empty name. The installed lsof manual defines rte as an AF_ROUTE socket. It is not a filesystem object. The existing closed non-file list now includes this exact type. Unknown types and inaccessible real files remain rejected.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/cleanup-route-socket.md#extrait-1).
