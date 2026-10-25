# Spécification 129 - Remise aux fils occupés quand rien n'est à relier

## Fiche synthèse
Spec: 129-remise-en-cours-de-tour | Statut: Implemented | Priorité: P1 | Date: 2026-10-02
Branche: session-129-remise-en-cours-de-tour | Retour d'expérience opus2D, point 3.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/129-remise-en-cours-de-tour/spec.md#extrait-1).

## Exigences
- **FR-001** : un message sans réponse attendue (groupable) en tête de file, pour un fil occupé
  dont le fournisseur est Claude ou Cursor, est remis aussitôt.
- **FR-002** : Codex, les demandes suivies, les sollicitations de fil et les notifications
  attendent la fin du tour (inchangé).

## Hors périmètre
Avertir l'expéditeur d'une remise différée (nouveau message de protocole) ; la réponse finale à un
`reply=no` non relayée est traitée par la consigne de la 127.
