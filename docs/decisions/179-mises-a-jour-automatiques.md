# ADR-179 — Stable releases and safe automatic activation

Status: accepted for session 179. User authorization: automatic updates on by default, switchable in Android Settings, covering Android and Bridget on Mac/Cartae.

Use one stable GitHub release with versioned platform archives and a signed APK. Reuse the public installer's exact-member extraction and stable executable link, the mobile encrypted store and human scope, and the existing OS service managers. Add no package framework or new network authority.

The daemon owns a durable, bounded activation barrier. Acquiring it requires no active or uncertain native work and no pending operation. All new ordinary work is rejected before dispatch while it is held. UpdatesManage is a distinct human device capability; existing devices receive it only through an explicit administrative grant. The phone cannot select URLs, files or commands.

Each host has one updater lock and a durable activation receipt. Integrity is verified before execution. Services are checked after activation; rollback occurs before releasing the barrier. Lost release replies cannot trigger rollback after new work may have resumed. A restart reads the durable barrier. After expiry, recovery must acquire a fresh safe window.

T3's updateMachine/DesktopUpdates separate available/downloaded/active states, serialize operations and preserve prepared files on network failure. Reuse these behaviours. Bridget differs because activation covers independent hosts and missions, and Android owns the final package-install decision. Do not restart T3 or replace credentials to reload already running MCP clients.

GitHub release administration and the local OS account remain trusted. A compromised publisher is not addressed by a digest served by the same publisher. Android additionally verifies its signing lineage. macOS verifies the delivered code signature. No downgrade is automated. An unknown mission stop is a real blocker, not a candidate for state deletion.

The update services need an initial controlled installation on an old daemon. The feature does not create a bypass for that bootstrap. See docs/automatic-updates.md and the delivery receipt for actual activation and deferred components.
