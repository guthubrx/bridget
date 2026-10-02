# Spécification 129 - Remise aux fils occupés quand rien n'est à relier

## Fiche synthèse
Spec: 129-remise-en-cours-de-tour | Statut: Implemented | Priorité: P1 | Date: 2026-10-02
Branche: session-129-remise-en-cours-de-tour | Retour d'expérience opus2D, point 3.

## Problème observé (données réelles)
Le 01/10 à 02:47:11Z, treize messages ont été remis d'un coup au coordinateur `opus_city_ai`
après 2 h 20 à 8 h 47 d'attente dans la file du pont. Ce coordinateur est resté dans un seul tour
de 17:52Z à 02:37Z (8 h 45), en travaillant. Le pont ne remet qu'à un fil au repos, pour relier une
réponse à sa demande ; un message sans réponse attendue n'a pourtant rien à relier.
T3 sait piloter le tour en cours (« steer ») pour Claude (donc GLM) et Cursor, pas pour Codex
(adaptateurs T3, test `steers a running turn instead of opening a new one on mid-turn sendTurn`).

## Exigences
- **FR-001** : un message sans réponse attendue (groupable) en tête de file, pour un fil occupé
  dont le fournisseur est Claude ou Cursor, est remis aussitôt.
- **FR-002** : Codex, les demandes suivies, les sollicitations de fil et les notifications
  attendent la fin du tour (inchangé).

## Hors périmètre
Avertir l'expéditeur d'une remise différée (nouveau message de protocole) ; la réponse finale à un
`reply=no` non relayée est traitée par la consigne de la 127.
