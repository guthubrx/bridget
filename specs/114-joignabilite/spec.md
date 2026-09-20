# Spécification 114 — Trois causes résiduelles de non-joignabilité

## Fiche synthèse
Spec: 114-joignabilite | Statut: In Progress | Priorité: P1 | Date: 2026-09-20
Branche: session-114-joignabilite | Suite du balayage demandé après la 113.

## Problèmes observés (2026-09-20)

**A. Une seconde application T3 révoque toutes les identités MCP.** Deux applications
t3code lancées sur le même dossier utilisateur partagent `server-runtime.json`. Avant chaque
attestation, le pont exigeait que ce fichier décrive encore SON serveur ; la seconde application
l'ayant réécrit, la comparaison échouait en boucle et le pont révoquait les marqueurs de TOUS les
fils. Symptôme vécu : `identity_not_found` sur les appels MCP de chaque fil T3, et 456 lignes
identiques dans le journal du pont en une journée. Le fichier est aussi effacé par l'application
qui se ferme, ce qui empêche un démarrage ultérieur du pont de retrouver le serveur.

**B. Une rafale de messages ouvre un tour par message.** Le 2026-09-19 à 19:16, le coordinateur
`horizon-3D` a reçu 26 messages en deux minutes et demie, venant de trois agents. Vingt-trois
étaient de simples comptes rendus sans réponse attendue, et chacun a ouvert son propre tour. La
corrélation entre messages utilisateur et tours exige leur égalité dans la fenêtre lue : un seul
tour sans texte la casse pour tout le monde. Deux demandes suivies voisines ont fini « au repos
sans appariement certain », donc sans réponse, alors que leur destinataire avait travaillé.

**C. Les fils Cursor ne peuvent pas être attestés.** Le pont sait relier un processus à son fil
pour Codex (rollout ouvert) et Claude (option de session). Cursor n'était pas reconnu : quatre
fils Cursor recevaient des messages mais leurs propres appels MCP restaient sans identité.

## Exigences
- **FR-001** : une seconde application T3 déclarée dans le fichier d'état ne révoque aucune
  identité. Ce qui doit être prouvé, c'est que NOTRE serveur n'a pas été remplacé pendant la
  collecte ; sa naissance le prouve, et couvre en plus le recyclage de PID.
- **FR-002** : le partage de dossier est signalé une fois dans le journal, avec le port et le PID
  de l'autre application, et affiché par `bridget t3 status`. Le cas « fichier effacé alors que le
  pont tourne » est nommé lui aussi.
- **FR-003** : un échec d'attestation répété n'est journalisé qu'au changement, et le rétablissement
  est dit.
- **FR-004** : les messages sans réponse attendue qui patientent pour un même fil partagent un seul
  tour, borné à 8 messages et 32 Kio.
- **FR-005** : ne sont jamais groupés une demande suivie, une sollicitation de fil, une
  notification. Un message seul garde mot pour mot son enveloppe actuelle.
- **FR-006** : dans un lot, chaque remise garde son accusé propre ; un rejeu déjà accusé n'est pas
  réinjecté et n'entre pas dans le lot.
- **FR-007** : un fil Cursor est attesté par le dossier `acp-sessions/<id>` que son processus tient
  ouvert, au même titre qu'un rollout Codex.
- **FR-008** : chaque fournisseur ne lit que ses propres fichiers de session ; un fichier étranger
  ouvert par un processus n'invalide pas l'inventaire.

## Hors périmètre
- Écrire dans l'état de t3code pour réparer le fichier : le pont ne modifie jamais t3code.
- Grouper des demandes suivies : une réponse par tour ne se partage pas (contre-revue d'evols-t3).
- Balayage des sagas d'envoi expirées, et espace disque (écarté par l'utilisateur).

## Critères de succès
- **SC-001** : avec un fichier d'état désignant un autre PID, l'attestation réussit.
- **SC-002** : trois comptes rendus en file produisent un seul tour portant les trois textes.
- **SC-003** : une demande suivie, une sollicitation ou une notification reste seule dans son tour.
- **SC-004** : un lot de deux remises idempotentes produit une injection et deux accusés.
- **SC-005** : un chemin `…/acp-sessions/<id>/store.db` rend `<id>`, et rien d'autre ne le rend.
- **SC-006** : recette complète verte ; après livraison, les fils Cursor apparaissent rattachés et
  le journal du pont ne répète plus le même avertissement.
