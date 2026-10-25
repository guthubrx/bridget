# Journal d'implémentation — 110 Reprise de nom

## Métadonnées
- **Spec** : 110-reprise-nom — **Branche** : session-110-reprise-nom
- **Base** : main `1f6acc42` — **Date** : 2026-09-19 — **Statut** : Implemented, livré en production le 2026-09-19 14:41

## Diagnostic à l'origine

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/110-reprise-nom/implementation.md#extrait-1).

## Implémentation

- `agent_profile.rs` : `rename_display_name` prend un prédicat `holder_is_live`. Sur conflit,
  `display_name_holder` identifie le détenteur ; s'il est vivant, le refus `DisplayNameConflict`
  est inchangé ; sinon `release_display_name` lui attribue un nom dérivé disponible et avance sa
  révision, dans la même transaction que la pose du nom demandé.
- `daemon.rs` : le point d'appel fournit une fermeture qui interroge l'annuaire vivant, route
  active et connexion ouverte.
- Aucun seuil de temps. Le champ `updated_at` date le dernier changement de profil, pas la dernière
  activité : un agent connecté de longue date le porte ancien, et un critère d'ancienneté lui
  volerait son nom. La présence est le seul signal juste.

## Vérifications

- `cargo fmt --all -- --check` : OK. `cargo clippy --workspace --all-targets -- -D warnings` : OK.
- Recette complète : **1516 réussis, 0 échec, 52 ignorés**, dont trois tests nouveaux :
  transfert depuis une identité éteinte, refus face à une identité vivante, reprise de son propre
  nom sans transfert.

## T006 — Livraison en production (2026-09-19 14:41)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/110-reprise-nom/implementation.md#extrait-2).
