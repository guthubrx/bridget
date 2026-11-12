# Changelog

## 0.8.49 - 2026-11-12

- Le refus d’un appel au catalogue distingue un champ version mal typé de
  l’action catalogue inexistante. Le type attendu est un nombre JSON, sans
  conversion implicite ni élargissement de permissions.

## 0.8.48 - 2026-11-12

- Le statut de continuation vérifie aussi la présence des enfants locaux avant de proposer une reprise.
- Les sondes détaillent leur temps de contrôle par phase, sans changer leurs délais, leurs verdicts ou les vérifications de droits.

## 0.8.47 - 2026-11-12

- Une remise qui attend la fin du tour ne relit plus tout l’inventaire local toutes les 500 ms.
- Le relevé commun permet la reprise ; toute remise garde sa lecture fraîche, les droits, les annulations et les expirations.

## 0.8.46 - 2026-11-12

- L’entretien conserve les configurations MCP des fournisseurs ; le propriétaire garde son nettoyage à la fermeture.
- Un diagnostic de démarrage tardif précise un échec générique sans rouvrir la tâche ni écraser une cause précise.

## 0.8.45 - 2026-11-12

- Une liaison SSH indisponible ne masque plus une cible unique attestée pendant la préparation.
- Les refus d’ambiguïté et de politique restent bloquants ; sans cible attestée, l’erreur de transport originale est conservée.

## 0.8.44 - 2026-11-12

- Les reçus des demandes suivies annoncent le délai de réponse, 60 secondes par défaut, et son inclusion de l’attente en file.
- Les refus avant remise précisent leur portée et les conditions d’une suite autorisée.
- Les diagnostics de préparation SSH sont horodatés et reliés entre maître et worker source sans conserver de sortie privée.

## 0.8.43 - 2026-11-12

- Les connexions natives reprennent leur preuve de projet au retour du worker, sans relancer la mission.
- Une annonce plus récente, un arrêt ou une connexion retirée invalident les anciennes réponses de preuve.

## 0.8.42 - 2026-11-12

- Les échecs de préparation SSH conservent une signature de diagnostic fermée, sans sortie privée ni provenance supposée.
- Les résolutions de projet vides laissent un diagnostic corrélé à la connexion, sans modifier les droits ni les missions.

## 0.8.41 - 2026-11-12

- Sans résultat métier durable, le guichet indique une application inconnue au lieu de applied:false.
- Le résultat et le guidage distinguent le guichet des admissions natives ; le rejeu peut admettre une mission jamais reçue et exige un mandat encore valide.

## 0.8.40 - 2026-11-11

- La reprise d’un démarrage refusé conserve le contrat du worktree ou de la vue de commit effectivement préparé.
- Les changements réels de définition et de droits restent refusés ; le contexte partagé garde sa comparaison à la sonde source.

## 0.8.36

- Report the native fleet quota refusal and finish the request without claiming an execution started.
- Preserve prepared workspaces with an unavailable report and no implicit deletion.
- Explain unsupported guidance before dispatch and settle correlated rejections without changing the active mission.
- Create new build caches privately before Cargo, preserving existing cleanup guards.

## 0.8.35 - 2026-11-11

- Le refus de journal reconnaît une tâche terminée et nettoyée depuis son état durable, même après disparition de sa présence en mémoire.
- Les routes actives restent prioritaires ; aucun arrêt n’est déduit d’une tâche encore active ou sans nettoyage confirmé.

## 0.8.34 - 2026-11-11

- CPU et charge élevés orientent le placement automatique sans interdire la machine imposée ; protection mémoire conservée.
- Une inspection en lecture seule ne demande plus de place de création ni de bootstrap disponible.
- Les refus de préparation distinguent les bornes internes de la charge matérielle, avec portée et corrélation explicites.

## 0.8.33 - 2026-11-11

- Aide locale de send accessible avec --help, -h et help, sans connexion ni envoi.
- Refus de capacité MCP : plafond local, appel non transmis et reprise précise ; limites inchangées.
- Délai de lecture JSONL explicitement nommé, distinct de la limite de taille.

## 0.8.32

- Clore aussi le suivi d’une mission dont l’expiration native est déjà confirmée, même après son nettoyage.
- Conserver son échec, ses résultats et ses fichiers ; ne pas relancer son enfant ni reclasser une autre cause d’échec.

## 0.8.31

- Capturer les réponses finales locales sans leur imposer le dépôt d’un worker distant.
- Clore les demandes après sauvegarde du résultat et reprendre une clôture SQL différée sans rejouer la mission.
- Empêcher les rappels répétés du même niveau ; conserver le palier suivant et l’identifiant complet.
- Refuser explicitement une finale tardive non capturée, afin de préserver son texte et les tâches déjà closes.

## 0.8.30

- Choisir le mode de remise de chaque nouvelle boucle : message ordinaire, guidage du tour en cours ou interruption.
- Autoriser le contrôle autonome d'une boucle pour son seul mode, son projet et ses destinataires désignés, sans emprunter une identité humaine ou parentale.
- Conserver ce choix dans les reprises ; révoquer le mandat à la clôture et relire les anciens reçus sans nouvelle remise.
- Conserver les gardes de pause, de tranquillité, de rythme et de mission native active, avec refus explicite sans changement de mode.

## 0.8.29

- Borner les dossiers ouverts par l’audit à la profondeur de la vue, pour respecter les limites de descripteurs des systèmes.

- Conserver les comptes rendus précédents quand une tâche de fond retarde le résultat final.
- Préparer une vue locale avec le registre du worker attesté, sans consulter une autre identité de profil.
- Expliquer la ronde déjà terminée avant un refus de guidage et conserver les causes précises des limites de préparation.
- Préciser les outils interdits en discovery, les ressources externes non transférées et la lecture des journaux après arrêt.
- Journaliser l’étape et le type fermé des erreurs de transport, sans contenu ni chemin.

## 0.8.28

- Transmettre le refus d’un chemin protégé dans l’admission, avec sa cause constatée et une capacité négociée compatible avec les anciens pairs. Conserver les causes distinctes d’un même lot.
- Expliquer le dépôt Git ancêtre détecté et son HEAD introuvable, dans la CLI comme dans l’admission. Identifier aussi une version Git refusée dans l’environnement exécuté.
- Conserver la référence complète d’une réponse expirée, pour les agents et la boîte humaine. Préciser l’issue à vérifier avant toute suite.


## 0.8.27

- Relire le quota sur la session conservée au repos pour observer un reset du fournisseur, sans tour de modèle ni assouplissement des contrôles.
- Donner la fenêtre, le retour et la date d'observation du quota de l'enfant dans le refus de reprise, avec sa tâche et sa clé.

## 0.8.26

- Les erreurs du contrôle préalable MCP indiquent que la commande n’a pas été transmise, avec l’étape constatée et la conservation des reçus antérieurs. Aucun délai ni droit changé.

## 0.8.25 — 2026-11-10

- Piloter un enfant existant depuis sa conversation humaine : envoi, guidage, interruption et sélection confirmée du modèle/effort selon les capacités de sa session.
- Lire le catalogue du pilote ciblé, local ou distant, sans modifier le parent ni créer de mission. Les anciens wrappers restent compatibles.
- Conserver les commandes et leur clé après une réponse perdue ; une relecture ne double pas la remise.
- Préciser la reconnexion du compte existant quand le fournisseur atteste une session OAuth expirée ; conserver api_error et distinguer les refus de cause inconnue.

## 0.8.23 — 2026-11-09

- Exposer les six formes de références du brief avec leurs champs requis et rappeler kind/label/host/path dans le guidage de continuation. Validateur natif inchangé.

## 0.8.22 — 2026-11-09

- Corriger le guidage des refus de capacité : la reprise suit le reçu d’admission réel, sans présumer un refus durable ni changer les seuils.

## 0.8.21
- Authenticated local preparation uses the origin worker registry even when a different profile registry exists on the account. No new permissions or content transfer.

## 0.8.20
- Allow existing local non-Git directories through the origin relay for general/shared tasks, without copying or a self-SSH tunnel.
- Resolve native child projects through the admitted placement worker when the OS hostname differs from its fleet machine ID.
- Make the commit_view/discovery/source-SHA requirement explicit in the MCP description.

## 0.8.19

- Projet Git des enfants natifs distants vérifié par leur worker, sans confondre racine de droits et projet de communication.
- Les notifications sans réponse attendue conservent les instructions de résultat final des missions natives.


## 0.8.18

- Diagnostic hors Git avec machines concernées, limites UTF-8 explicites du brief et durée renouvelée par ronde. Aucun changement de permissions ou de transport documentaire.


[English](CHANGELOG.md) · [Français](CHANGELOG.fr.md)

Published versions are added to the repository history under `vX.Y.Z` tags.
Versions follow semantic versioning and dates use YYYY-MM-DD. Validation
figures below describe the named release's evidence, not the current checkout.

## 0.8.17 — 2026-11-09

- Store direct information without a model turn through `intent=information`, with durable receipts, existing ledger access and no ordinary-delivery fallback.
- Target maintenance actions to affected work and preserve observation facts without waking dormant owners.

## 0.8.11 — 2026-11-08

- Share observed Claude account quota refusals across model catalogues using the same launcher. Preserve their original observation and reset dates across the cache-key migration. Keep other accounts and machines separate.
- Explain the required version and read actions in the specialists MCP description and invalid-call guidance.

## 0.8.9 — 2026-11-07

- Show the current local folder in recovery cards. An unavailable Git check no longer declares a usable commit view unavailable.

## 0.8.8 — 2026-11-07

- Keep each newly admitted mission's authority after the originating turn ends. Continue to check its owner, native revocations, cancellation, source policy and frozen placement before launch.
- Recover an early registration disconnect within the original deadline and move slow view cleanup outside ordinary replies. Preserve one request and one task after a lost response.
- Reuse already authorized local folders, including non-Git sources, without a mirror or a second directory grant. Keep remote projection and control-store protections.
- Remove the fixed 32 GiB view, 128 worktree and 64 project-mapping quotas. Check physical storage, checked counters and bounded metadata instead.
- Qualify recognized command tools and the child profile before placement. Distinguish read commands from known build writes; do not add shell rights to read-only profiles.

## 0.8.7 — 2026-11-07

- Keep the remote channel when a known initial preparation refusal arrives after its wait expires. Unknown tuples and stale epochs remain rejected; late refusals cannot create tasks.
- Preserve capacity and connection failures during pre-release reattestation, and explain preparation timeouts without claiming a stopped server or changed permissions.

## 0.8.6 — 2026-11-07

- Preserve specialist administration for devices paired with the exact legacy global administrator scope. Partial scopes, expired credentials and revoked devices remain denied.
- Apply an already recorded human decision with its original key and exact operation, without asking the user to answer again. Explain how to resolve an administrative scope refusal.

## 0.8.5 — 2026-11-06

- Reuse the local roots of an existing placement while preparing its destination machine, without rewriting the provisioned mapping or changing permissions.
- Accept observations for an exact known managed command without requiring a delegation record. Preserve worker, instance, generation and correlation checks.

## 0.8.4 — 2026-11-06

- Admit origin-side commit-view preparation when the target is the same provisioned machine. Keep other workspace strategies and all authority checks unchanged.
- Report the closed validation field and cause for origin-relay refusals, with the size limit when known. A generic refusal no longer implies an older client image.

## 0.8.3 — 2026-11-05

- Read only the selected launch definitions when processing worker observations, instead of decoding the full retained history for every command.
- Preserve a worker connection when a late refusal refers to the exact conserved view of a closed task. Identity, connection epoch and live-request correlation remain checked.
- Report closed worker refusal diagnostics without command bodies or credentials.

## 0.8.2 — 2026-11-05

- Give explicitly selected Git inventories60seconds and remove the pause between successful reads. Timeout diagnostics retain a closed command, timings, volumes and stop state.
- Fix64MiB view-reservation readers and add a narrowly scoped cold quarantine recovery that preserves receipts and starts no process.
- Allow256MiB public view maps, stream closed-reference manifest reads and include the map in free-space checks. Compatible reader floors reject older rollback images.
- Keep cancelled, unpublished view preparations scheduled until native conservation is confirmed.

## 0.8.1 — 2026-11-04

- Raise selected commit-view payloads to32GiB, inventories/manifests to64MiB and helper deadlines to30minutes. Preserve disk, object integrity and mission checks.
- Keep an admitted task visible across interrupted origin attestations and preserve the request key and unknown outcomes.

## 0.8.0 — 2026-11-03

- Native sleeper agents: history, explicit wake-up and refusal of new direct, group and scheduled requests. Previously admitted work and its results are kept.
- Classification actions sync both ways by default for linked clients, with a Bridget-only setting, durable receipts and the same command key while its outcome remains unknown.
- Active/Sleeper views in the app, `agents sleep|wake` commands, CLI filters and caches respecting server classification. Retained state requires a reader that understands dormancy.

## 0.7.8 — 2026-11-03

- Atomically relay verified context for prepared projects, including linked checkouts and previously authorized additional roots.

## 0.7.7 — 2026-11-03

- Declare the specialists input schema as an object and expose composed schemas through complete local references, so strict MCP clients can discover all37tools at startup. Accepted actions, permissions and guards are unchanged.

## 0.7.6 — 2026-11-02

- Explain unconfirmed remote starts with task-scoped checks and preserved request references. Historical start errors no longer hide later recorded work or results. No change to retries, admission or permissions.

## 0.7.5 — 2026-11-02

- Keep live bridge connections during bounded read timeouts. Suspend observations until a fresh check succeeds. Report closed RPC failure causes without replaying uncertain deliveries.

## 0.7.4 — 2026-11-01

- Add explicit global --table output for finite CLI inventories, details and receipts, before or after the command. Family columns, hostnames, Unicode wrapping and colour-free pipes share one native renderer. JSON, scopes, confirmations and exit codes stay unchanged.

## 0.7.3 — 2026-11-01

- Update status includes every registered machine, including missing and stale reports.
- Local updater setup supports machine-specific worker units on SYS, KS and macOS.
- Stopped services stay stopped. Missing observation dates stay unknown.
- Linux package built on Debian12/glibc2.36; Android7003 uses the existing signing certificate.

## 0.7.2 — 2026-11-01

- Specialist lists, searches and details also return the current human display name from the agent profile, separately from roles and stable IDs. Renames appear on the next read, including while offline.

## 0.7.1 — 2026-11-01

- Admission recovery waits for the parent's live identity and permission evidence instead of treating missing evidence as changed permissions. Explicit revocation still refuses admission.
- Bounded client recovery keeps reading the same receipt through transport interruptions and returns observed terminal refusals. An unreadable outcome stays unknown. Historical requests17/18 are not replayed.
- The Android0.7.0 application is unchanged.

## 0.7.0 — 2026-11-01

Add declared roles and human-designated responsibilities, versioned specialist profiles, durable mandates, on-demand sessions, sourced cross-project memory, definition import/export and optional team compositions. MCP, CLI and the Android catalogue share native authority and receipts. No extra work budget is enabled by default. A compatible specialist-request reader is required before rollback.

## 0.6.5 — 2026-10-30

- Task status explains a provider model rejection and the decision required from the user.
- The catalogue reads the current configuration and excludes disabled connections.
- No automatic model substitution, account change or task replay.

## [0.6.4] - 2026-10-29

- Correct the authenticated native permission ceiling across engines without exporting the origin application's authority.
- Offer missing provider programs during installation with explicit selection and existing accounts preserved.
- Distinguish installed programs, compatibility, personal sign-in and actual model access.
- Publish native fixes delivered since0.6.3: replayable admissions, bounded diagnostics, compact preparation and an explicit fleet catalogue.
- Rebuild the Android package as0.6.4 to align the release version, with the same interface, certificate and signing lineage.

## [0.6.3] - 2026-10-25

- Add a left conversation drawer with favorites, search and explicit attention states.
- Keep drafts and reading positions per agent; add loaded-history search and editable system dictation.
- Prepare files and photos locally before explicit sending; show code languages and exact copy controls.
- Add isolated interactive HTML previews alongside static and source modes, with no privileged bridge or network access.
- Project correlated native context-compaction activity without inferred progress.

## Unreleased

### Android message queue

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-1).

### Android and permanent federation access

- Add the native Android app: QR pairing, scoped directory, messages, files,
  task actions, questions, durable retries and protected local storage.
- Add the HTTPS mobile gateway with revocable device credentials and bounded work.
- Add optional minimal notifications, local quiet hours and signed APK updates.
- Add origin attestations and project preparation over the authenticated worker
  channel, plus a private workstation connection to a permanent server master.


### Documentation and repository hygiene

- English is the default README and public-guide language. French editions remain available.
- Public navigation separates current guides from original-language historical records.
- Deployment addresses and a personal address in tracked documents use example values.
- Absolute workstation dependency symlinks are removed.
- Secret scanning checks immutable historical findings separately from new changes.
  See [the audit and its limits](docs/security-audit.md).

## [0.4.1] - 2026-10-16

### Fixed

- An accepted remote response closes its tracked request even without the parent's
  transport acknowledgment, avoiding a false no-response alert.
- Maintenance retries that closure after storage failure, even if task cleanup
  already finished. It retains the result without restarting work.
- Correlation checks mission, round, child and parent. A genuinely absent reply
  still follows the configured deadline.

### Compatibility and validation

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-2).

## [0.4.0] - 2026-10-15

### Added

- Fleet catalog entries expose declared machine, engine, connection, model and
  effort support. `bridget delegate --catalogue --json` works through the native Bridget interface.
- Tasks can constrain each selection axis independently. Omitted effort keeps
  the provider's native behavior.
- Routing considers fresh load facts, bounded Git continuity, declared build
  capacity and scoped account quota/credit observations.
- A configurable decision chain applies deterministic filters, an optional
  compatible decision service, an optional lightweight selector and deterministic
  fallback scoring. Selectors choose execution combinations, not task results.
- Decision services are declared per machine. Disabled/unavailable services do
  not prevent authorized fallback levels. Every choice remains within admissible
  combinations.
- Durable state and Lineage v4 retain the selected option, provenance and bounded
  facts. Exact replay returns the saved receipt without another decision.

### Git and remote execution

- Remote startup checks the expected source commit before preparing work.
- Separate tasks can use separate worktrees. Routing does not itself push or
  merge delivered work; delivery remains explicit coordination.
- A declared model does not prove account access or unknown quota/credit.
  Each machine retains its own authorized connections.

### Compatibility and operation

- Lineage v1–v3 keep their existing shapes. V4 adds decision metadata.
- V2 admission and probe capabilities are negotiated explicitly, without silent
  conversion to an older admission.
- Existing receipts survive store migration. After a V2 admission receipt is
  written, do not return to a 0.3.0 binary solely because routing was disabled.
  Use a compatible binary or a validated coherent restoration that loses no work.
- Bridget provides terminal access to its native engine.

### Validation and known limits

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-3).

## [0.3.0] - 2026-10-15

### Added

- `bridget delegate <MISSION>` provides a short native child workflow.
- `bridget task` exposes list, status, wait, continue, respond, cancel and read ACK.
- `--json` supports scripts. A terminal without agent identity is refused before
  agent operations and receives registration guidance.

### Limits

- Coordination needs a live Bridget agent identity; a human terminal does not
  automatically become an agent.
- `task list` is a human global projection. Other operations remain scoped to
  their owning coordinator.

## [0.2.1] - 2026-10-15

### Fixed

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-4).

### Operation and validation

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-5).

## [0.2.0] - 2026-10-14

### Added

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-6).

### Fixed and limits

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-7).

## [0.1.3] - 2026-10-02

### Fixed

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-8).

### Documentation and changes

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-9).

## [0.1.2] - 2026-10-02

### Documentation

- Read-only managed launches are documented separately from human-terminal
  authorization for development. An undeclared type is not a read-only policy failure.

### Fixed

- A turn immediately followed by another between bridge reads is no longer
  reported as an observation gap when its origin was observed.

## [0.1.1] - 2026-09-25

### Changed

- `bridget --help` reports the actual package version instead of a fixed 0.1.0.

### Documentation

- Updated README and changelog.
- Publication adds versioned history instead of replacing it.

## [0.1.0] - 2026-09-25

First versioned public release. Earlier publications replaced history.

### Added

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-10).

### Fixed

[Note technique de l’adaptateur](docs/adapters/t3/archives/CHANGELOG.md#extrait-11).
