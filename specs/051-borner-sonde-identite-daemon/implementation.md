# Journal d’implémentation — Session 051

**Spec** : 051-borner-sonde-identite-daemon
**Branche** : session-051-borner-sonde-identite-daemon
**Base** : f893b1e4a010a333dc8ba2348df9b8ee8e45663c

## État initial

- `daemon_identity` clone la socket et appelle `read_line` sans délai.
- `get_status` reçoit une `Option` qui confond les échecs de lecture avec une
  identité absente, puis ouvre une seconde connexion pour l’annuaire.
- La carte de reprise possède déjà une source `status: Result<...>` capable de
  porter l’indisponibilité ; elle est aujourd’hui alimentée par un succès forcé.
- DevKMS n’est pas disponible sur cette machine (`mem: command not found`).

## Progression

### T001 — Oracle du pair silencieux

- **Statut** : terminé.
- **Fichier** : `crates/bridget-daemon/tests/identity_probe_timeout_test.rs`.
- **Instrument** : vrai binaire `bridget status`, vraie socket Unix jetable,
  première trame décodée comme `RoleHandshake(Client)`, pair ensuite muet.
- **Univers** : 1 test, 0 benchmark, listé avant le tir.
- **Preuve rouge** :
  `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.02s`.
- **Diagnostic** : le client dépasse la borne externe de quatre secondes. Le
  parent lui envoie SIGTERM, attend sa fin et libère la fixture ; aucun enfant
  ni répertoire `bg51-*` ne reste après le tir.

### T002 — Sonde bornée et résultat explicite

- **Statut** : terminé.
- **Commit** : `2cfa21a` — borne de deux secondes posée sur le flux avant le
  premier `read_line` ; la sonde rend désormais un résultat qui distingue
  absence et indisponibilité.
- **Compatibilité** : un `ClientWelcome` suivi d’un rapport ancien sans
  `instance_id` conserve le daemon présent et ses attributs non attestés.
- **Auto-revue** : `Path::exists()` a été retiré dans `bb1c8cd`, car une erreur
  de permission rend aussi `false`. La connexion est maintenant l’instrument
  unique. Le contrôle `7cc396c` oppose socket absente, daemon ancien répondant
  et chemin inaccessible.

### T003 — Propagation sans absence inventée

- **Statut** : terminé.
- **CLI** : `status`, `agents` et `who` rendent code 1 avec la cause, avant
  toute seconde connexion de collecte.
- **Reprise** : le `Result` existant porte directement l’erreur et la carte
  n’affiche jamais « aucun agent connecté ».
- **Reaper** : l’observation échoue au lieu de convertir la source en tableau
  vide ; le contrôle positif conserve le vrai daemon absent comme liste vide.
- **Preuves** : 2/0 pour la propagation, 16/0 pour le banc CLI voisin.

### T004 — Mutant et validations

- **Statut** : terminé.
- **Mutant causal final** : retrait unique de
  `set_read_timeout(Some(DAEMON_IDENTITY_READ_TIMEOUT))` ; l’oracle du vrai
  binaire meurt à 0/1 après 4,02 s.
- **Restauration** : SHA-256 avant/après identique,
  `b68741bc79664e589d1f1938c1d078560e8e124a4a2a8ea2eee862b3abf494de` ;
  le même oracle repasse à 1/0 en 2,06 s.
- **Contrôles** : matrice identité 3/0, propagation 2/0, CLI 16/0.
- **Composition** : `cargo check --workspace --all-targets` vert.
- **Format** : `rustfmt --check` vert sur les cinq fichiers Rust du delta ;
  `git diff --check` vert.
- **Clippy** : le tir strict `-D warnings` est rouge sur dix dettes hors delta
  (`wrapper.rs`, interfaces du superviseur, `managed_supervisor.rs`, `ui.rs`,
  `store.rs` et un `collapsible_if` ancien de `daemon.rs`). Aucun vert n’est
  inventé et ces dettes ne sont pas corrigées dans 051.
- **Limite** : le banc daemon complet n’est pas lancé ; plusieurs fixtures de
  cet univers ont une non-terminaison connue. Aucun compte global n’est donc
  revendiqué.

## REX — Retour d’expérience

**Date** : 2026-08-27
**Tâches complétées** : 4/4

### Ce qui a fonctionné

- Le parent du banc borne et termine son propre client par SIGTERM : le mutant
  ne peut ni suspendre la campagne ni laisser un enfant vivant.
- La première trame est décodée avant le silence ; le test prouve que le pair a
  réellement vu la sonde.
- La carte de reprise possédait déjà le bon type `Result` : la correction retire
  un succès forcé au lieu d’ajouter une nouvelle couche.

### Difficultés et décisions

- Une première commande `cargo fmt` a touché des fichiers hors périmètre. Le
  statut était propre juste avant ; son seul delta mécanique a été retiré et
  les formats suivants ont été contrôlés fichier par fichier.
- Tester uniquement le silence aurait laissé `Path::exists()` classer une
  source inaccessible comme absente. L’auto-revue a ajouté la contre-épreuve
  de permission sans modifier le périmètre fonctionnel.

### Responsabilité future

- Complexité O(1), trois réponses au plus, aucune dépendance ni option ajoutée.
- Potentiel minimalisme : aucune abstraction ou ligne de production identifiée
  comme suppressible à comportement constant après factorisation des trois
  écritures de négociation.
- Le code ajouté porte un besoin observé ; les helpers ont chacun trois usages
  réels et centralisent le même diagnostic.
