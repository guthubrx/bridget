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

- **Statut** : en attente.

### T003 — Propagation sans absence inventée

- **Statut** : en attente.

### T004 — Mutant et validations

- **Statut** : en attente.
