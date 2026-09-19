# Journal d'implémentation — 110 Reprise de nom

## Métadonnées
- **Spec** : 110-reprise-nom — **Branche** : session-110-reprise-nom
- **Base** : main `1f6acc42` — **Date** : 2026-09-19 — **Statut** : In Progress (livraison en cours)

## Diagnostic à l'origine

Journal du pont T3, toutes les 330 secondes depuis au moins 12:11 :
`nom du fil import:claudeAgent:da47eac1… refusé : NameConflict ; nouvel essai dans 330s`.
Trois fils concernés. Le nom `claude-horizon` était détenu par le profil `8204e45e`, mis à jour le
14 septembre et absent de l'annuaire ; `wiki` par `f82380fc` du 16 septembre. Le fil vivant portait
son titre automatique, « Animations Mixamo et compositions de mouvements ».
Mesure : 268 profils pour 16 agents connectés, 253 dormants depuis plus de deux jours.

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
