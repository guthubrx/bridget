# Contre-revue adverse — implémentation 104

- Date : 2026-09-18
- Demandeur : bdget (Claude, `127bccff-8490-453a-8182-884b749ec41e`)
- Agents interrogés (fournisseur différent : Cursor) :
  1. cursor-listen (`04c521fe-3282-41e2-bbc7-e1590ce3c351`), demande envoyée 05:17 (borne 25 min, jusqu'à 05:42) ;
  2. horizon-cursor (`7e9dec19-82b4-47a3-a1e8-229e10416a2a`), demande de repli envoyée 05:29 (borne 20 min, jusqu'à 05:49).
- Contrainte annoncée : lecture seule, aucune écriture, aucun commit, aucune dépense.
- Question posée : verdict `APPROVE` / `APPROVE_WITH_CHANGES` / `BLOCKED` puis défauts logiques, risques, trous de
  couverture, changements minimaux (≤ 8 points, `fichier:ligne`), sur cinq points précis :
  1. curseur (hex JSON : `actor`, `fingerprint` SHA-256, `upper`, `before`) : saut, doublon ou fuite d'une ligne tierce ?
  2. ordre `(ts, id, target)` DESC : row-values SQL vs `MessageKey::page_cmp` cohérents ?
  3. budgets 128 / 1 Mio / 16 Mio / 60 Kio : un cas où la page ne progresse pas ?
  4. `match_offset` via `folded_len` exact pour tous les caractères de `fold_char` ?
  5. identité / permis : trou entre `ledger_read_context` et la publication ?
- Artefacts transmis (chemins absolus) : spec, plan, contrat, modèle, plan de tests, `ledger.rs`, `ledger_requests.rs`,
  `daemon.rs`, `mcp.rs`, `cli.rs`, `protocol.rs`, `search_104_test.rs`.

## Issue

**Pas de réponse dans le délai** (constaté à 05:50, bornes 05:42 et 05:49 expirées). Le pipeline continue
sans contre-revue externe ; toute réponse arrivant ensuite sera vérifiée contre le code et les tests, puis
consignée ici dans un commit de suite (`docs(104)`), sans régénération de fichier.

| Agent | Remise | Verdict | Objections |
|---|---|---|---|
| cursor-listen | en vol, jamais accusée (agent occupé ou wrapper sans accusé) | — | — |
| horizon-cursor | accusée (`reçu`) vers 05:35, aucune réponse à la borne | — | — |

Auto-revue de substitution (Article XX) sur les cinq questions, avec preuves de tests :
1. Curseur : `before` = dernière clé consommée, `upper` fixé en première page ; les bornes sont réappliquées en
   SQL avec la participation de l'acteur (`ledger_requests.rs:one_branch`) — un curseur forgé ne relit que ses propres
   lignes (S11), une purge/insertion entre pages ne duplique ni ne saute une clé existante (S13, S01/S08).
2. Ordre : row-values `(ts, id, target) <= / <` en collation BINARY ↔ `page_cmp` sur `as_bytes()` (S01/S08 sur 2000
   lignes avec doublons d'identifiant et de seconde ; test unitaire de `page_cmp`).
3. Progrès : le premier résultat d'une page passe toujours le budget de réponse ; la première ligne admissible est
   toujours sélectionnée (budget 0 < 1 Mio) ; une ligne > 16 Mio est consommée sans corps (S07, S09, S10).
4. `match_offset` : `folded_len` = longueur du repli minuscule du caractère, même règle que `fold_for_search` ;
   vérifié sur `Ÿ`/`É` (changement de longueur) et emoji (S02, S30, test unitaire).
5. Identité : `ledger_read_context` prend identité + permis sous verrou, puis `ledger_identity_still_live` compare la
   même connexion et le même acteur avant publication (S22 : owner coupé pendant le repli → `identity_unavailable`).
   Fenêtre résiduelle : une révocation entre la revérification et l'écriture sur la socket est équivalente à une
   réponse déjà envoyée ; le contenu appartient de toute façon à l'acteur révoqué (pas de fuite tierce).

## Objections

| Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|
| (aucune reçue dans le délai) | | | |
