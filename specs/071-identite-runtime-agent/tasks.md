# Tâches - SPEC-071

## User Story 1 - Identifier le runtime réel - P1

- [x] T001 - Ajouter dans `crates/bridget-daemon/src/ui.rs` les témoins Rust de
  projection de `transport`, `mode`, `model` et `effort`. Preuve : un
  `AgentInfo` complet restitue exactement les quatre champs et un enregistrement
  historique sans option reste sérialisable.
- [x] T002 - Étendre `UiAgentRowV1` et `compose_agent_rows` dans
  `crates/bridget-daemon/src/ui.rs` pour transmettre uniquement les faits déjà
  attestés. Preuve : T001 passe sans modification du protocole `AgentInfo`.
- [x] T003 - Ajouter dans `crates/bridget-daemon/assets/ui/app.js` la matrice de
  tests de normalisation du runtime et du mode. Preuve : Codex, Claude Code,
  Cursor, Gemini CLI, TMUX, ACP/FLUX, CLI géré, CLI inconnu et données absentes
  ont chacun un verdict exact ; un suffixe de nom n'influence rien.
- [x] T004 - Étendre `normalizeAgentRow` et ajouter les fonctions pures de
  catalogue et de mode dans `crates/bridget-daemon/assets/ui/app.js`. Preuve :
  T003 passe et le fournisseur réel du modèle n'est jamais reconstruit.

## User Story 2 - Reconnaître les produits par leur marque - P1

- [x] T005 - Importer les quatre SVG officiels dans
  `crates/bridget-daemon/assets/ui/providers/` et consigner source, date,
  variante et SHA-256 dans `providers/NOTICE.md`. Preuve : chaque runtime connu
  possède un asset exact et aucun SVG ne contient script, URL externe,
  gestionnaire d'événement ou données raster embarquées.
- [x] T006 - Ajouter dans `crates/bridget-daemon/src/ui.rs` les témoins Rust des
  routes `/providers/*.svg`. Preuve : type `image/svg+xml`, ETag, `no-cache`,
  réponse 304 et corps octet pour octet sont vérifiés.
- [x] T007 - Embarquer et servir les quatre SVG via `include_bytes!` et
  `write_asset` dans `crates/bridget-daemon/src/ui.rs`. Preuve : T006 passe et
  aucun chargement tiers n'est requis.

## User Story 3 - Lire une fiche compacte et accessible - P1

- [x] T008 - Ajouter dans `crates/bridget-daemon/assets/ui/app.js` les témoins
  Node des données de fiche, du positionnement aux bords, de
  `aria-describedby`, du maintien au survol, de la fermeture Échap et du
  nettoyage lors d'un nouveau rendu, d'un scroll ou d'un redimensionnement.
  Preuve : chaque invariant échoue si le comportement productif correspondant
  est retiré.
- [x] T009 - Remplacer le tooltip imbriqué par une fiche globale unique dans
  `crates/bridget-daemon/assets/ui/app.js`, sans modifier la sélection de ligne.
  Preuve : T008 passe, le focus reste sur la ligne, une seule fiche peut être
  visible et aucune référence à une ligne DOM remplacée ne subsiste.
- [x] T010 - Remplacer les styles `.agent-row__tooltip` par la fiche structurée
  dans `crates/bridget-daemon/assets/ui/theme.css`. Preuve : largeur maximale
  360 pixels, extrait limité à deux lignes, logo sans fond propre, puce de mode
  lisible et aucune bordure visible, ombre ou gradient.

## Vérification et traçabilité

- [x] T011 - Exécuter `node crates/bridget-daemon/assets/ui/app.js`, les tests
  Rust ciblés de `ui.rs` et `cargo test -p bridget-daemon --test
  ui_relay_test`. Preuve : toutes les commandes passent ou chaque blocage
  préexistant est isolé et documenté.
- [x] T012 - Vérifier la matrice manuelle décrite dans
  `specs/071-identite-runtime-agent/quickstart.md`, mettre à jour les preuves de
  `spec.md` et réaliser la self-review de simplicité. Preuve : exigences
  FR-7101 à FR-7114 et SC-7101 à SC-7106 possèdent une preuve code, test ou un
  écart explicitement non vérifié.

## Résultat d'exécution

- 81/81 tests Node passent.
- 43/43 tests Rust `ui::` passent.
- 21/21 tests d'intégration `ui_relay_test` passent.
- La prévisualisation Chromium à 1280 par 720 mesure une fiche de 344 pixels,
  avec logo, libellés, puce et extrait lisibles.
- Le parcours live sur le daemon n'a pas été exécuté, car il exigerait de
  remplacer ou d'arrêter le relais de production. Il reste au gate de
  déploiement.
- La suite complète rencontre trois échecs dans `managed_parity_test`, tous
  reproduits sans la session 071 sur `origin/main` au commit `a63cf97`.
- Clippy strict rencontre deux avertissements devenus erreurs dans
  `bridget-transport`, également reproduits sur `origin/main` et hors des
  fichiers modifiés par cette session.
