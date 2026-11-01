# Bridget CLI tables

Use bridget --table specialists list or bridget specialists list --table.
Both forms are equivalent. Finite views and receipts cover agents, tasks,
budget/control, requests, inbox/queue, ledger, journals/events, wakes, threads,
handoffs, artifact metadata, accounts, models, updates, mobile administration,
compatibility and diagnostics.

Columns match native data. Specialist rows show human name, roles, project scope,
hostname and observed state. A human name is not an addressing identifier.
Existing JSON keeps full details and references.

Default output and --json remain unchanged. Combining --json and --table fails
before any operation. Option values, provider arguments and arguments after --
are preserved. Long cells wrap; OS terminal width, then COLUMNS, then120 columns.
NO_COLOR, pipes and TERM=dumb disable colours.

No extra request, provider call or permission. Unknown auth stays unverified.
Installed does not mean loaded. Human confirmations and exit codes are preserved.
Continuous streams, MCP protocols, pairing/QR and raw contents reject tables.

For send/reply, place --table before the body or before the command. Arguments after -- stay intact. who/discover retain their original authority and warning paths.
