# Spécification 114 — Trois causes résiduelles de non-joignabilité

## Fiche synthèse
Spec: 114-joignabilite | Statut: Implemented (livré 2026-09-20) | Priorité: P1 | Date: 2026-09-20
Branche: session-114-joignabilite | Suite du balayage demandé après la 113.

## Problèmes observés (2026-09-20)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/spec.md#extrait-1).

**B. Une rafale de messages ouvre un tour par message.** Le 2026-09-19 à 19:16, le coordinateur
`horizon-3D` a reçu 26 messages en deux minutes et demie, venant de trois agents. Vingt-trois
étaient de simples comptes rendus sans réponse attendue, et chacun a ouvert son propre tour. La
corrélation entre messages utilisateur et tours exige leur égalité dans la fenêtre lue : un seul
tour sans texte la casse pour tout le monde. Deux demandes suivies voisines ont fini « au repos
sans appariement certain », donc sans réponse, alors que leur destinataire avait travaillé.

**C. Les fils Cursor ne peuvent pas être attestés.** Le pont sait relier un processus à son fil
pour Codex (rollout ouvert) et Claude (option de session). Cursor n'était pas reconnu : quatre
fils Cursor recevaient des messages mais leurs propres appels MCP restaient sans identité.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/spec.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/114-joignabilite/spec.md#extrait-3).

## Critères de succès
- **SC-001** : avec un fichier d'état désignant un autre PID, l'attestation réussit.
- **SC-002** : trois comptes rendus en file produisent un seul tour portant les trois textes.
- **SC-003** : une demande suivie, une sollicitation ou une notification reste seule dans son tour.
- **SC-004** : un lot de deux remises idempotentes produit une injection et deux accusés.
- **SC-005** : un chemin `…/acp-sessions/<id>/store.db` rend `<id>`, et rien d'autre ne le rend.
- **SC-006** : recette complète verte ; après livraison, les fils Cursor apparaissent rattachés et
  le journal du pont ne répète plus le même avertissement.
