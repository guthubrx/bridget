# Journal d'implémentation 103

Statut : Implemented dans le worktree — 22/22 tâches ; non commité, non fusionné, non déployé.
Début 2026-09-17 à 22:20 CEST (enchaînement après la 102 dans le même pipeline).

## Socle (T001)

Worktree : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/103-dossier-passation
Branche : session-103-dossier-passation. Base : commit `adbd8dd7` docs(103) (artefacts de
préparation) puis `c6ecc307` merge(103) de main `f36804ea` (sessions 101, 105, 106
intégrées le même soir ; conflits feature.json/AGENTS.md résolus en gardant la 103).
103 reste indépendante de 102 et 104 : aucun fichier 102 copié ; le catalogue compte 14
outils sur ce socle, 15 avec `bridget_handoff`. À la fusion avec la 102 (qui ajoute
`bridget_thread`), les listes fermées (allowlist wrapper, inventaires MCP, golden Codex,
liste Claude) devront être réconciliées à 16 ; conflit textuel attendu et simple.
Aucune mutation de main.

## Écarts préexistants corrigés (mêmes correctifs que dans la 102, sur ce socle)

Prompt 105 des wrappers sans « ; » (tests `claude_interactive_097_test`), exemple de
réponse de SKILL.md raccourci (`core_089_skill_test`), liste attendue de
`claude_native_permissions_test` complétée (`bridget_events`, `bridget_journal`),
emprunt inutile t3code.rs:2780 (clippy).

## Progression

### T002 Harnais
- ✅ `tests/handoff_103_test.rs` : racine `/tmp`, home/état privés (support
  `idempotent.rs`), arrêt coopératif par SIGTERM et attente (`stop_cooperatively`), aucun
  SIGKILL émis par les tests, serveur MCP réel (`bridget mcp`) et CLI réelle par processus
  enfants suivis.

### T003–T005, T010 Validation et rendu v1 (`src/handoff.rs`, module déclaré dans lib.rs)
- ✅ Structures strictes (objet `draft` analysé champ par champ : inconnus, `null`, faux
  types, chaînes blanches refusés), bornes exactes en octets UTF-8, listes ≤ 12, références
  ≤ 16 avec union `kind` (file/url/message/journal/thread/artifact, chemin absolu lexical,
  URL http(s) sans identifiants, UUID canoniques, séquences cohérentes, `source_label`
  déclaratif), rendu = marqueur + `serde_json::to_string_pretty` (ordre canonique,
  `next_step` null, listes vides), refus > 16 384 octets, `preview_result`,
  `decorate_send_result` (reçu + `handoff_version`, `bytes`, `warnings`), requête commune
  `parse_request` (preview refuse le transport ; send exige `to` UUID, `id`+`issued_at`
  ensemble, `reply_timeout` avec `reply`).
- ✅ Tests unitaires spec103 : S01, S02/S03 (requête), S03, S04 (16 384 exact / 16 385),
  S05 (N/N+1 par champ et liste), S08, S10/S11, S22 (perf), S23 (fichier doré).

### T006–T007 MCP `bridget_handoff`
- ✅ Schéma fermé ; `preview` sans connexion ; `send` délègue à `execute_send` (corps exact,
  identité attestée, `id`/`issued_at` transmis), `canonical_send` inchangé ; inventaire 19
  outils (15 Bridget + 4 Maicie), matrice FR-009 mise à jour.

### T008, T011, T013 CLI `bridget handoff`
- ✅ `preview|send --json-stdin [--json]`, stdin borné à 65 536 octets (65 537e refusé avant
  analyse), action de l'objet cohérente avec la sous-commande, sortie humaine inerte
  (corps imprimé tel quel, limites rappelées), `--json` = objet MCP ; `send` réservé à une
  identité attestée, même chemin idempotent (`send_idempotent_to_daemon`) et même reçu
  (`mcp::send_issue_result`) ; codes 0/2/1 du contrat.

### T009, T012, T014, T017 Tests d'intégration (`handoff_103_test.rs`, 5/5)
- ✅ S02 (aperçu sans daemon ni socket ni identifiant ; transport refusé en aperçu) ;
  S06/S12/S13/S17 (remise réelle du corps exact avec auteur attesté, dix rejeux → une
  ligne de ledger, `envelope_mismatch` sans mutation, mise à jour = nouveau message) ;
  S09 (corps identique après redémarrage) ; S15/S16 (reply explicite, `reply_timeout`
  sans `reply` refusé, DND → statut `dnd`, aucune remise) ; S18/S19/S20 (parité CLI/MCP
  octet pour octet, exit 1 sur `in_flight`, stdin trop long, action incohérente, catalogue).
- Constat : deux dossiers identiques vers le même destinataire sont dédupliqués par 099
  (`duplicate`) avant le contrôle DND ; la fixture DND utilise un corps distinct.

### T015 Fichiers dorés
- ✅ `tests/fixtures/handoff_103_golden.json` (Unicode, `/`, `\\`, contrôles, listes vides,
  toutes les références, ordre des champs) ; test `spec103_s23_fichier_dore_v1_octets_inchanges`.

### T016 Liste fermée
- ✅ `BRIDGET_SAFE_MCP_TOOLS` 14 → 15 (`bridget_handoff`), golden Codex, liste Claude, `mcp::tests`.

### T018 Documentation
- ✅ commandes.md « Passation (103) » (contrat, recette en cinq étapes avec deux exemples
  JSON validés, formes CLI, trois exercices de reprise SC-001, conservation/visibilité),
  ligne `handoff` du tableau 094, `bridget_handoff` dans la liste des outils, « quinze
  outils » ; SKILL.md (description, deux demandes du quotidien, section « Passer un travail
  à un autre agent ») ; README.

### T019 Mesure
- ✅ `spec103_s22_deux_cents_rendus_de_16_kio_sous_100_ms` : 200 rendus ≈ 16 Kio, p95 ≈ 0,43 ms,
  max ≈ 0,45 ms (build debug, poste de recette) ; aucun réseau ni modèle.

### T021 Analyze, Converge, contre-revue
- ✅ analysis.md § « Converge — passage 1 » : CONVERGED, aucune tâche ajoutée. Contre-revue
  cursor-listen (Cursor ; Codex en erreur) : **APPROVE_WITH_CHANGES**, deux corrections
  retenues (schéma d'URL insensible à la casse ; description de l'outil : le validateur fait
  autorité sur le schéma), une note déjà couverte, un point cosmétique non retenu. Détail :
  adversarial-review-cursor-listen.md.

### T022 Point de reprise
- ✅ quickstart.md § « État au 2026-09-17 » (surfaces partagées avec la 102, réconciliation
  des listes fermées à la fusion, aucune autorisation de commit/installation implicite).

### T020 Recette complète (état définitif du code, 2026-09-17 22:52–22:58 CEST)
- ✅ Environnement : `umask 077`, `env -i HOME TMPDIR=/tmp/b.XXXX BRIDGET_HOME=/tmp/b102/ns
  PATH=~/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin`, `perl -e 'alarm 1800'`.
- `cargo fmt --all -- --check` : rc 0 ; `cargo clippy --workspace --all-targets -- -D warnings` : rc 0.
- `cargo test --workspace --release --no-fail-fast` : **1426 réussis, 0 échec, 50 ignorés**
  (/tmp/b102/r103c-release.log). Une première passe (22:39–22:44, avant les deux corrections
  de la contre-revue) avait donné le même bilan 1426/0/50 ; une relance intermédiaire dont le
  journal mélangeait deux exécutions a été refaite proprement.
- Ignorés : recettes réelles Codex/Claude/T3 conditionnées par variables d'environnement et
  worker de performance privé du harnais.

## Remise à l'humain

Livré dans le worktree (19 fichiers modifiés ou créés, base `c6ecc307`) : `handoff.rs`,
`bridget_handoff`, `bridget handoff`, allowlist 15, docs, tests, fichier doré, ADR 039
« Accepté ». Non fait : commit, fusion dans main, déploiement, recette avec agents réels.
Réconciliation attendue avec la 102 à la fusion (listes fermées d'outils : 16 ; inventaires
MCP : 20 ; « quinze » → « seize »). Commandes de recette : voir T020 ci-dessus.
