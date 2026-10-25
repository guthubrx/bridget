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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/127-corrections-simples/spec.md#extrait-1).
