# Plan 110 — Reprise de nom

## Décision

Le magasin de profils ne connaît pas l'annuaire vivant, qui est un état mémoire du daemon. Plutôt
que d'y introduire une dépendance, `rename_display_name` accepte un prédicat : « ce détenteur
est-il vivant ? ». Le daemon fournit la réponse depuis son annuaire ; les tests fournissent un
prédicat explicite.

Pas de seuil de temps. Le champ `updated_at` ne date pas la dernière activité mais le dernier
changement de profil : un agent connecté de longue date le porte ancien, et un critère d'ancienneté
lui volerait son nom. La présence est le seul signal juste.

## Implémentation

- `crates/bridget-daemon/src/agent_profile.rs` : `rename_display_name` prend un paramètre
  `holder_is_live: impl Fn(&str) -> bool`. Sur conflit, consulter le prédicat. Si vivant, refuser
  comme avant. Sinon, dans la même transaction, attribuer au détenteur un nom dérivé via
  `available_display_name`, incrémenter sa révision, puis poser le nom demandé.
- `crates/bridget-daemon/src/daemon.rs` : au point d'appel, passer une fermeture qui interroge
  l'annuaire vivant (route active et présence dans la rétention).
- Les autres appelants passent un prédicat qui refuse tout transfert, pour un comportement inchangé.

## Tests

- Unitaires dans `agent_profile.rs` : transfert accordé quand le détenteur est éteint, refus quand
  il est vivant, nom dérivé distinct, révisions incrémentées, atomicité sur erreur.
- Intégration : l'extension de nom 089 reste verte.

## Vérification et livraison

`fmt`, `clippy -D warnings`, recette complète. Puis reconstruction du binaire et relance du daemon
et du pont, sur autorisation. Contrôle final : les trois fils portent leur nom, le journal ne
montre plus de refus.
