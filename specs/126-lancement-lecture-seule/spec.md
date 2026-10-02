# Spécification 126 - Lancement d'équipiers en lecture seule : règle claire et autorisée d'office

## Fiche synthèse
Spec: 126-lancement-lecture-seule | Statut: Implemented | Priorité: P2 | Date: 2026-10-02
Branche: session-126-lancement-lecture-seule | Demande de l'utilisateur.

## Problème observé
Un agent a renoncé à lancer des équipiers GLM pour une tâche de lecture (28 morceaux), croyant que
tout lancement exigeait un terminal humain, et a utilisé des sous-agents plus coûteux. La skill
plaçait la garde du terminal (propre à `--posture development`) au cœur de la section de
lancement, et la référence classait `spawn`/`stop` comme « CLI humain ».

Vérifications du 2026-10-02 :
- `bridget spawn claude` sans `--posture`, depuis une session sans terminal : accepté, type
  `project-discovery-claude` (lecture seule), arrêté proprement par `bridget stop`.
- `bridget spawn glm` : refusé « capacité manquante posture_decouverte » : aucun type `glm` dans le
  registre du daemon (`~/.cache/bridget-core/agents.json` absent ; types par défaut codex, claude,
  cursor, gemini), et la clé `ZAI_API_KEY` du lanceur `gclaude` n'est connue que de T3.

## Exigences
- **FR-001** : la skill autorise d'office le lancement en lecture seule pour une tâche de lecture,
  et recommande le type le moins coûteux disponible pour un gros volume.
- **FR-002** : la garde du terminal humain est décrite comme propre au droit d'écrire (Codex).
- **FR-003** : un refus « posture_decouverte » est lu comme un type non déclaré, jamais comme la
  règle du terminal.
- **FR-004** : la référence des commandes est cohérente (spawn/stop utilisables par un agent dans
  ces limites).

## Hors périmètre (décision utilisateur)
Déclarer un type `glm` lançable par Bridget suppose de lui fournir la clé GLM.
