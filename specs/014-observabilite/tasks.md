# Tasks 014 — Observabilité des modes & amorçage de reprise

**Base** : branche `session-14-observabilite` depuis main (5e7d86e).
**Règles** : docs/regles-chantier.md (17 règles) + validations avant commit,
commit en review immuable, auteur ≠ relecteur, zéro trace IA, jamais de push.
**Couloirs** : T1401-T1402 = daemon/protocol/cli (coderBridget) ;
T1403+T1406 = wrapper (cxbridget) ; T1404-T1405 = attach/journal (prospective).
Cargo.toml et protocol.rs : propriétaire coderBridget, les autres passent
commande (règle 4).

- [ ] T1401 [FR-1401] Champ mode de présence distinct du transport
  (protocol/daemon) : renseigné à l'enregistrement par chaque chemin (wrapper
  ACP, wrapper tmux, cli), migration douce des présences existantes ;
  `attach` refuse sur le mode réel.
  **Observable** : test des 3 chemins d'enregistrement + refus attach cité.
- [ ] T1402 [FR-1402] `who` : colonne mode + localisation tmux
  `session:window.pane` (meilleure connaissance, `—` sinon).
  **Observable** : sortie who réelle sur les 3 cas ; test non-TTY.
- [ ] T1403 [FR-1403] Sonde runtime claude (transcript JSONL) : modèle
  remonté, effort si présent ; étiquette opaque.
  **Observable** : who affiche le modèle d'un claude réel.
- [ ] T1404 [FR-1404] Corrélation toolCallId à l'écriture du journal : les
  updates héritent du titre (champ additif v1) ; le renderer n'affiche plus
  « inconnu » pour un appel titré.
  **Observable** : fixture multi-updates + session réelle ; golden non-TTY
  avec delta déclaré.
- [ ] T1405 [FR-1405] Heure locale dans le renderer attach.
  **Observable** : test avec TZ forcée ; goldens ajustés, delta déclaré.
- [ ] T1406 [FR-1406] Wrapper : sous-commandes codex hors has_prompt +
  amorçage de reprise (identité + découverte MCP différée) ; tests d'argv
  (override conservé, amorçage en position prompt) et de régression du texte.
  **Observable** : SC-1404 rejoué en réel — reprise puis réponse liée par
  outil sans intervention.
- [ ] T1407 Non-régression + revue hostile finale (bancs 008/012/013,
  workspace complet, clippy -D warnings ; revue croisée par constats).
