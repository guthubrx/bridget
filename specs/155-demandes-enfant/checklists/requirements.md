# Specification Quality Checklist: Remontée des demandes de l'enfant

**Purpose**: Valider la spec avant planification
**Created**: 2026-10-11
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] Pas de détail d'implémentation inutile (les noms d'outils et de refus sont le contrat public)
- [x] Centrée sur la valeur : l'enfant ne reçoit plus de décision aveugle
- [x] Écrite pour un lecteur non spécialiste (phrases courtes)
- [x] Toutes les sections obligatoires présentes

## Requirement Completeness

- [x] Aucun marqueur [NEEDS CLARIFICATION] (3 questions avec défaut, section dédiée)
- [x] Exigences testables (FR-001 à FR-032)
- [x] Critères de succès mesurables (SC-001 à SC-011)
- [x] Scénarios d'acceptation définis (US1 à US7)
- [x] Cas limites identifiés : silence du parent, hors droits, secret, mode URL, pause, redémarrage, chaîne de 3 niveaux, rejeu, budget de demandes
- [x] Périmètre borné (Claude, ACP, texte libre humain, décisions durables hors périmètre)
- [x] Dépendances et hypothèses identifiées (150, 151, schéma Codex à figer en T002)

## Feature Readiness

- [x] Chaque FR a un test prévu dans tasks.md
- [x] Garde-fous de sécurité explicites : limite absolue (humain compris), aucun gain, pas de boucle, délai avec issue, accord non remis annulé
- [x] Changement de la décision 149 expliqué et daté (ADR en T036)

## Notes

- Le verdict échoue fermé : toute incertitude donne un refus automatique, pas un item humain.
- Le détail des champs Codex n'est pas lu dans ce dépôt. T002 le fige avant tout code.
- Point conditionnel : T034 dépend de la livraison de la session 151.
- Contre-revue Codex du 2026-10-11 (BLOCKED, 11 objections) traitée : voir `adversarial-review-codex.md`.
