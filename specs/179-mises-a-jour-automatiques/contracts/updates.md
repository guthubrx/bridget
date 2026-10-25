# Human update contract

Authenticated mobile endpoints:
- GET /v1/updates
- PUT /v1/updates with the closed object {"automatic": boolean}
- POST /v1/updates/check with {}
- POST /v1/updates/install with the closed object {"version": "X.Y.Z"}

All require an unrevoked device, UpdatesManage and federation-wide scope. Responses expose policy, host observations, check generation, exact manual target and core version. Unknown fields do not add authority. The exact target cannot select another newer release. Turning automatic mode off clears it.

Local administration extends MobileAdministrationRequest with Updates and GrantUpdates, using the existing same-UID, non-agent negotiated role. Prepare/Finish carry one UUID token. Report carries a bounded host name, strict installed/available versions and a closed stage enum. Prepare reads activity under State and takes the gate while that observation remains protected. Policy files are private, bounded and atomically replaced.

DaemonToWrapper::UpdateInProgress means the ordinary request was not dispatched through this gate. The client poisons the connection and advises a later attempt with unchanged identifiers. It is not a success, a terminal mission failure, or an instruction to create a new request.

Host state files and lease files are local private implementation state, not an authority offered to the phone. No automatic task replay, task cancellation, data migration or project cleanup is part of this contract.
