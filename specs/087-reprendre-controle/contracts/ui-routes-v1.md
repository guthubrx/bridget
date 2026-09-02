# Contrat : routes du relais UI

Toutes les routes passent par la socket du daemon. Aucune n'ouvre `bridget.db`.

| Méthode | Route | Trame daemon | Corps / réponse |
|---|---|---|---|
| GET | `/v1/control/state` | `ControlStateRead` | `ControlState` + `inbox_open_count` + `focus` (projection de `maicie status --json`, bloc `control`) |
| POST | `/v1/control/state` | `ControlStateSet` | `{ paused?, budget_cap?, reason?, expected_generation }` → `ControlState` ou `{ code }` |
| POST | `/v1/control/focus` | `ServiceRequest { payload: Delegate { goal, focus: { project_id, on_conflict } } }` | `{ text, project_id, on_conflict: "replace" \| "queue" \| null }` → reçu guichet `{ request_id }` puis statut par `maicie_request_status` ; `409 focus_conflict` avec le focus courant si `on_conflict` absent et un focus existe |
| GET | `/v1/inbox` | `HumanInboxList` | `{ items, open_count }` |
| POST | `/v1/inbox/<id>/resolve` | `HumanInboxResolve` | `{ choice }` → item résolu ou `{ code }` |

Éléments d'interface :

- **Bandeau d'état de contrôle**, toujours visible en tête de l'écran principal : pause (durée, bouton Reprendre) ou bouton Pause ; focus courant (but tronqué, projet) ; compteur d'items ouverts qui ouvre le panneau ; budget `consommé/plafond`.
- **Saisie « Travaille sur… »** dans la colonne Projets : un champ texte, le projet courant présélectionné, un bouton. En cas de focus existant, une confirmation à deux choix : remplacer ou mettre en file.
- **Panneau boîte de réception** : liste des items ouverts, contexte déplié, boutons de décision issus de `options`, historique repliable.
- **Réglage du plafond** dans Paramètres du serveur, sous les dossiers de projets, avec la même prévisualisation puis enregistrement automatique que les emplacements.

Textes affichés en français courant, sans identifiant technique : « Pause depuis 2 h 10 », « Focus : Corriger l'import CSV · projet Cartae », « 3 décisions vous attendent », « 4 objectifs automatiques sur 5 ».
