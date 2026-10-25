# Journal 114 — Trois causes résiduelles de non-joignabilité

- **Base** : main `9661c127` — **Date** : 2026-09-20 — **Statut** : Implemented, livré en production le 2026-09-20 07:07

## A. Une seconde application adaptateur de bureau ne révoque plus les identités

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/implementation.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/implementation.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/implementation.md#extrait-3).

## B. Une rafale de messages n'ouvre plus un tour par message

### Diagnostic
Le 2026-09-19 à 19:16, `horizon-3D` a reçu 26 messages en 2 min 30 de la part de `opus-horizon`,
`claude-horizon` et `horizon-cursor`. Vingt-trois étaient des comptes rendus sans réponse attendue ;
chacun a ouvert son tour. `correlate` exige autant de tours assistants que de messages utilisateur
dans la fenêtre lue : un seul tour sans texte casse l'égalité pour toute la fenêtre. Deux demandes
suivies ont donc fini « au repos sans appariement certain », sans réponse pour leur expéditeur.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/implementation.md#extrait-4).

## C. Les fils Cursor sont attestés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/implementation.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/implementation.md#extrait-6).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/implementation.md#extrait-7).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/implementation.md#extrait-8).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/implementation.md#extrait-9).

## T009 — Republication
Copie filtrée régénérée (311 fichiers, 238 substitutions, aucun motif interdit, aucun lien cassé),
formatée, puis recette complète sur la copie : **1532 réussis, 0 échec, 52 ignorés**, clippy propre.
Dépôt public `guthubrx/bridget` remis à `63f2b51`, commit unique anonyme. Clone de contrôle : aucun
motif interdit, une seule branche, aucun tag.
