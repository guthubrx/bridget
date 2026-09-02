# Note de reprise — SPEC-087 « Reprendre le contrôle »

Écrite le 2026-09-02 vers 07:45 CEST, mise à jour le 2026-09-02 à 19:50 CEST après la passe d'intégration des deux versants pour qu'un autre agent reprenne sans relire toute la conversation. Lire dans l'ordre : cette note, `tasks.md`, `implementation.md`, `implementation-maicie.md` s'il existe, puis `contracts/`.

## 1. État exact

- **Rien n'est commité.** Tout le travail est dans l'arbre de travail local `/Users/moi/Nextcloud/10.Scripts/bridget` (branche `087-reprendre-controle`, créée depuis origin/main `bf89d4c7`) et dans le worktree serveur `/home/moi/bridget-referent/.worktrees/087-reprendre-controle` (même branche, même base). Les deux sont synchronisés par `rsync` (voir §3). Un `git stash` nommé `modifs-locales-hors-087` contient des modifications locales sans rapport (`.gitignore`, `apps/bridget-desktop/src-tauri/Cargo.toml`) : les rétablir sur `main` à la fin avec `git stash pop`, pas sur cette branche.
- **Versant daemon, transport, CLI, interface : livré et testé** (tâches cochées dans `tasks.md`, détail dans `implementation.md`).
- **Versant Maicie : en cours par un agent dérivé** au moment de cette note. Il n'édite que `plugins/maicie/**` et écrit son journal dans `implementation-maicie.md`. Si ce journal n'existe pas, l'agent a été interrompu : l'état réel est le `git diff -- plugins/maicie` et le fichier de test `plugins/maicie/tests/controle_referent_087.rs`. Ne rien supprimer ; compiler et lire les erreurs.

## 1 bis. État après la passe d'intégration (2026-09-02, 19:50 CEST)

- Les deux versants compilent ensemble et leurs gates sont verts (détail dans `implementation.md`, section « Passe d'intégration »). L'agent Maicie a été arrêté après avoir rendu son journal ; ses tâches livrées sont cochées dans `tasks.md`.
- **Toujours rien de commité.** Faire le §2 avant toute autre chose.
- Si une compilation dit `unresolved import DelegateOrigin` ou `no field named origin` alors que `protocol.rs` les contient : c'est un artefact périmé du cache ; `cargo clean -p bridget-transport -p maicie -p bridget-daemon` puis recompiler. Cela a coûté une heure une fois.
- Restent 17 tâches (`tasks.md`, cases ouvertes), dont 8 ajoutées par la convergence (T049 à T056). Deux d'entre elles demandaient un choix ; il est pris ci-dessous, ne pas rouvrir la question.

### Décision pour T052 (décision `reassign:<agent_id>` du référent)
Choisir la voie sans nouvelle trame : Maicie applique la décision en construisant elle-même le fait de réassignation, comme si la chaîne pré-autorisée avait désigné cet agent. Point d'entrée : `plugins/maicie/src/store.rs`, `apply_human_decision`, branche `reassign:` aujourd'hui `Unsupported` ; réutiliser le chemin de `apply_reassignment_fact` avec une génération ouverte vers l'agent choisi, dans la même transaction que `human_decisions_applied`, puis acquitter. Test : une délégation en `InterventionHumaineRequise`, décision `reassign:<uuid>`, on attend une génération vers cet agent et un acquittement ; rejeu de la même décision ⇒ rien de plus.

### Décision pour T055 (focus courant dans le bandeau)
Ne pas appeler `maicie status` depuis le relais. Ajouter au protocole une trame de lecture seule `WrapperToDaemon::ControlFocusRead` répondue par le daemon depuis une petite table `control_focus_projection (objective_id, but, project_id, updated_at)` que Maicie alimente à chaque relève par une trame service `ControlFocusPublish` (capacité `HumanInboxV1`, même connexion que les dépôts de boîte). Le daemon ne lit jamais la base Maicie. Classer les deux trames dans les deux matrices de rôles (voir §4 point 2). Le bandeau lit ensuite `focus` dans `GET /v1/control/state`.

### T056
Ne pas la faire sans le référent : elle branche un producteur de travail autonome qui n'existe pas encore sur main. La laisser ouverte et le dire dans le rapport.

## 2. Premier geste obligatoire : sauvegarder

```bash
cd /Users/moi/Nextcloud/10.Scripts/bridget
git add -A crates plugins specs docs tests
git commit -m "feat(087): Reprendre le controle - etat intermediaire"
git push -u origin 087-reprendre-controle
```

Puis sur le serveur, dans le worktree, `git fetch origin && git reset --hard origin/087-reprendre-controle` pour repartir d'un état identique. Aucune trace d'IA dans les messages de commit.

## 3. Méthode de travail (ne pas improviser)

- Éditer en local. Ne jamais éditer directement sur le serveur.
- Envoyer : `rsync -az --exclude target --exclude .git --exclude .worktrees --exclude .claude --exclude .gstack --exclude 'watch_*' --exclude node_modules /Users/moi/Nextcloud/10.Scripts/bridget/ cartae.app:/home/moi/bridget-referent/.worktrees/087-reprendre-controle/`
- Sur le serveur : `export PATH=$HOME/.cargo/bin:$PATH CARGO_TARGET_DIR=/home/moi/bridget-referent/bridget/target` (cache chaud) ; `cargo fmt -p bridget-transport -p bridget-daemon -p maicie` ; puis les tests.
- Rapatrier le formatage : `rsync -az cartae.app:/home/moi/bridget-referent/.worktrees/087-reprendre-controle/crates/ /Users/moi/Nextcloud/10.Scripts/bridget/crates/` et de même pour `plugins/`.
- Tests d'interface en local : `cd crates/bridget-daemon/assets/ui && node --test app.js` (attendu 135 passés).
- Gates de périmètre : `cargo test -p bridget-transport` (242 passés attendus) ; `cargo test -p bridget-daemon --lib --features test-support -- ui:: project_ referent_control human_inbox spec_087 focus_priority control_pause` (144 passés attendus) ; `cargo test -p maicie --lib` et `plugins/maicie/tests/controle_referent_087.rs`.
- Toujours annoncer le compte `passed/failed`, jamais un code retour derrière un `grep`.

## 4. Pièges déjà rencontrés, ne pas les redécouvrir

1. **`ClientHello` filtre les capacités** (`daemon.rs`, arm `WrapperToDaemon::ClientHello`, liste `matches!`) : une capacité absente de cette liste est ignorée en silence. `ControlStateV1` y est ajoutée. Idem `ServiceHello` : ensembles admis en dur, `[HumanInboxV1]` ajouté.
2. **Matrices de rôles exhaustives** dans `handle_wrapper_message` : toute trame neuve doit être classée dans la matrice service ET la matrice client, sinon rien ne compile. C'est voulu.
3. **Fixture de test `state_with_registered_agent` cassée sur main** (`InvalidAgentId("agent-2")`, identités UUID depuis 081) et elle tenait `HOME_REGISTRY_LOCK` : un panic empoisonnait le verrou et faisait échouer en cascade ~146 tests. Les prises du verrou relisent désormais un verrou empoisonné. Les tests 087 utilisent `spec_087_state` (UUID).
4. **Le greffe** : `~/.config/bridget/greffe-authorization.json` n'existe pas sur le serveur, toute mutation par guichet est refusée `PolicyUnavailable`. Pour le principal humain, le daemon dépose SANS la garde (ADR 027) ; Maicie ne doit pas exiger d'attestation greffe sur un `Delegate` à origine humaine.
5. **Scellé d'origine** : `bridget_transport::protocol::human_message_content_seal` = même algorithme que `maicie::domain::human_message_content_seal`. Le hash canonique se calcule sur la requête SANS `origin`. `message_id = "hmo-" + sha256_hex(issuer_scope + "\n" + request_id)[..32]`, `ts = issued_at`.
6. **Priorité de focus** : le daemon lit `focus:<objective_id>` dans `BridgetMessage.references` ; Maicie doit poser cette référence dans `outbox.rs` pour les délégations de focus.
7. **Deux écrivains sur le worktree serveur** (moi et l'agent Maicie) provoquaient des compilations cassées par intermittence. Un seul agent à la fois sur le serveur, ou un `rsync` complet avant chaque compilation.
8. **`maicie status --json` réconcilie des outboxes** : ne pas l'appeler depuis une route d'interface rafraîchie en boucle. C'est pourquoi le bandeau n'affiche pas encore le focus courant (T041).
9. **rm par motif** : ne jamais supprimer par glob sans `ls` du motif avant (leçon de la matinée).

## 5. Tâches restantes, dans l'ordre

Voir `tasks.md` (cases non cochées). Résumé :

- Maicie (agent dérivé, sinon à reprendre) : T009, T010, T015, T016, T017, T047, T021, T022, T023, T024 (référence `focus:`), T025, T027, T028, T032, T033, T037. Contrats à respecter : `contracts/human-inbox-v1.md` (acquittement après commit, jamais à la relève), `contracts/delegate-origin-v1.md`.
- Daemon restant : brancher `human_inbox::remind_overdue` dans le thread horaire de `daemon.rs` (`thread::spawn` de purge, `sleep_until_shutdown(Duration::from_secs(3600))`) avec `human_channel_config()` ; T041 (focus en tête de liste, nécessite une lecture Maicie sans effet de bord, à concevoir).
- T043 gates complets après fusion des deux versants : suite lib `bridget-daemon` complète avec comparaison des NOMS de rouges contre la base nue (`/tmp/baseu.txt` sur le serveur, 146 noms), clippy limité aux lignes du lot (4 erreurs préexistantes connues dans `bridget-transport`).
- T044 documentation (fait pour `regles-chantier.md`, `quickstart.md`, `implementation.md` ; fusionner `implementation-maicie.md`).
- T045 validation opérateur : dérouler `quickstart.md` avec le référent, consigner.
- T046 mise en service, uniquement après commit et fusion dans main : compilation release depuis le checkout principal, binaire versionné `~/.local/lib/bridget/bridget-<sha>-<hash>`, bascule du lien, redémarrage `bridget-ui.service` puis `bridget-daemon.service`, relance des agents gérés par UUID (`bridget agents --json` avant, `bridget relaunch <uuid>` après), vérification `bridget who` (ligne `Contrôle :`) et absence d'avertissement `identity_version`.

## 6. Converge et rapport

Après la dernière tâche : relire `spec.md` FR par FR et pointer `fichier:ligne` dans le code ; toute exigence sans preuve devient une tâche sous `## Convergence` en fin de `tasks.md`. Ne jamais poser `Implemented` tant qu'une case reste ouverte ; le statut honnête est `In Progress`.
