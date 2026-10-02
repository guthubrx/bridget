# Spécification 127 - Corrections simples issues du retour d'expérience d'opus2D

## Fiche synthèse
Spec: 127-corrections-simples | Statut: Implemented | Priorité: P2 | Date: 2026-10-02
Branche: session-127-corrections-simples | Retour d'expérience de l'agent d'étude d'opus2D (22/09-02/10).

## Problèmes observés (bilan opus2D, points 4, 7, 8, 9)
- « Aucune réponse inter-agent attendue » lu comme « aucune action à faire » ; un défaut signalé
  est resté sans traitement (27/09 10:19).
- `bridget spawn gclaude` refusé « capacité manquante posture_decouverte » : pris deux fois à tort
  pour la garde du terminal humain ; le vrai motif est un type absent du registre.
- UUID complet exigé, préfixes refusés (27/09).
- `~/.local/share/bridget/bridget.db` : fichier vide de l'ancien Bridget, source de confusion.

## Exigences
- **FR-001** : une consigne commune `NO_REPLY_NOTICE` distingue « pas d'accusé de réception » et
  « action éventuelle à faire », et dit comment rendre un résultat ; reprise par tous les pilotes
  et par le pont T3 (message seul, lot, notification).
- **FR-002** : un type absent du registre est refusé `UnknownType` (types connus, registre).
- **FR-003** : `send` CLI et MCP résolvent un nom exact ou un début d'UUID (≥ 6) unique ; toute
  ambiguïté est refusée.
- **FR-004** : le fichier vide de l'ancien Bridget est retiré.
