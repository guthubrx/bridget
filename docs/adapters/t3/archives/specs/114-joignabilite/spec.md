# Archives techniques de l’adaptateur T3

Document d’origine : [specs/114-joignabilite/spec.md](../../../../../../specs/114-joignabilite/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**A. Une seconde application T3 révoque toutes les identités MCP.** Deux applications
t3code lancées sur le même dossier utilisateur partagent `server-runtime.json`. Avant chaque
attestation, le pont exigeait que ce fichier décrive encore SON serveur ; la seconde application
l'ayant réécrit, la comparaison échouait en boucle et le pont révoquait les marqueurs de TOUS les
fils. Symptôme vécu : `identity_not_found` sur les appels MCP de chaque fil T3, et 456 lignes
identiques dans le journal du pont en une journée. Le fichier est aussi effacé par l'application
qui se ferme, ce qui empêche un démarrage ultérieur du pont de retrouver le serveur.

<a id="extrait-2"></a>

## Extrait 2

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

<a id="extrait-3"></a>

## Extrait 3

## Hors périmètre
- Écrire dans l'état de t3code pour réparer le fichier : le pont ne modifie jamais t3code.
- Grouper des demandes suivies : une réponse par tour ne se partage pas (contre-revue d'evols-t3).
- Balayage des sagas d'envoi expirées, et espace disque (écarté par l'utilisateur).
