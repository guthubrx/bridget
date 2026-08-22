# Contrat : ordres de cycle de vie (spawn/stop)

Extension de la socket daemon existante — mêmes principes que les contrats 007
(variantes typées) et 008 (rôles de connexion). Noms définitifs au reuse-audit.

## Spawn (client → daemon, synchrone borné)

`SpawnOrder { type, name?, cwd (absolu), persistent, command_id }` →
réponse unique sous délai absolu :

- `SpawnAccepted { name }` — émise **seulement** après le `Register` réel
  (et, pour un persistant, après l'écriture durable de l'état désiré **puis**
  de l'issue — ordre D-503) ;
- refus typés (table fermée SC-003) : `UnknownType`, `CommandMissing`,
  `BillingGuard { variable }`, `NameActive`, `EnvUnfit { detail }`,
  `CwdGone`, `NegotiationFailed { detail }`, `SpawnTimeout` (Register tardif
  rejeté), `QuotaExceeded { limit }`, `DaemonRecovering`,
  `IdempotencyExpired`.

Idempotence : même `command_id` → issue rejouée, ou rattachement à la
génération en vol (attente du vrai terminal) — table de vérité du data-model.
**Ordre des gardes en `Recovering`** : la recherche du `command_id` précède le
refus — issue terminale connue → rejouée ; génération connue en vol →
rattachement ; **seul un ordre nouveau et inconnu** reçoit `DaemonRecovering`
(sinon une reconnexion pendant la récupération casserait l'idempotence).
Trois tests dédiés.

## Stop (client → daemon, synchrone borné)

`StopOrder { name }` → `StopOutcome` (data-model). Séquence interne :
invalidation de la génération (table superviseur) → retrait durable de l'état
désiré → ordre d'arrêt ciblé au wrapper (handshake D-501 : annulation, drain
file **et** notifications, `Unregister`) → attente bornée → escalade `killpg`
→ récolte/vérification → réponse. `End` typé émis vers chaque abonnement
attach de l'équipier (FR-011ter).

## Événements du canal de statut

`BootstrapReady` / `StartupFailed` (data-model) — corrélés
`instance_id`/`command_id` ; stderr n'est jamais un protocole.

## Règles

- Aucune réponse de succès sur `BootstrapReady` seul.
- `who` : les équipiers daemon-gérés portent le marqueur de gestion ; le
  daemon en phase `Recovering` l'affiche.
- Cas de test imposés : deux `SpawnOrder` simultanés même nom ; retry après
  chaque point de crash D-502/D-503 ; `stop` en `Reserved`, en bootstrap
  bloqué, après `Register`, sur marqueur périmé ; attach actif pendant stop
  puis reprise.
