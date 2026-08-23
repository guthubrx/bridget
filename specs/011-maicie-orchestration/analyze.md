# Rapport d'analyse SpecKit — Maicie v3 (après revue hostile cxbridget)

**Date** : 2026-08-22  
**Périmètre** : `spec.md`, `plan.md`, `tasks.md`, contrats et modèle de données.

## Résultat

**PASS_WITH_REQUIRED_GATES** — la seconde revue hostile a fait compléter les
deux sagas de reprise : payload exact pour message et command_id pour SpawnOrder.
Le MVP P1 reste déterministe et indépendant d'ACP live, MCP, GUI/TUI et gestion
de processus. Trois gates restent externes : idempotence/tombstone Bridget,
session 008 avant runtime ACP et session 009 avant SpawnOrder.

**Verdict cxbridget** : APPROVE après contre-vérification finale ; les deux
réserves documentaires sur `message_id` et `actor=local_human` ont été intégrées.

## Revue hostile intégrée

| ID | Sévérité initiale | Correction intégrée |
|---|---|---|
| R1 | Critique | outbox préalable, message_id client idempotent, états prepared/outcome_unknown/accepted, test Ack perdu |
| R2 | Critique | approbation mono-usage avec hashes, acteur, expiration, consommation atomique ; SpawnOrder 009 uniquement |
| R3 | Haute | permission ACP affichée comme auto-décidée, aucune attente humaine inventée |
| R4 | Haute | suppression du canal sémantique MCP/extension ACP du périmètre |
| R5 | Haute | MVP CLI : tags égaux, choix explicite, synthèse factuelle, aucun LLM |
| R6 | Haute | Bridget unique horloge active ; aucun timer ou relance Maicie |
| R7 | Haute | Subscribe session 008, seq/Gaps/fraîcheur ; snapshot dérivé non autoritaire |
| R8 | Moyenne | tâches réordonnées, conflits `[P]` supprimés, gate P1 T016 ajouté |
| R9 | Moyenne | benchmark SC-008 et analyse recalculée |
| R10 | Critique | OutboxDélégation enrichie de l'enveloppe filaire immuable ; prepared inclus dans lookup/replay ; trois crash barriers et idempotency_expired |
| R11 | Critique | ActivationOutbox command_id persistée avec dispatching ; lookup/replay SpawnOrder et consommation après issue durable |

## Couverture des exigences

| Requirement Key | Has Task? | Task IDs |
|---|---|---|
| FR-001–FR-007 | Oui | T001–T009 |
| FR-008–FR-009 | Oui | T017–T018 |
| FR-010–FR-012 | Oui | T008, T015, T019 |
| FR-013–FR-014 | Oui | T021–T023 |
| FR-015–FR-019 | Oui | T010–T015 |
| FR-020 | Oui | T004–T008 |
| FR-021 | Oui | T004–T005, T017–T018 |
| FR-022 | Oui | T010–T015 |
| FR-023 | Oui | T005–T008, T016, T023 |
| SC-001–SC-007 | Oui | T006–T019, T021–T025 |
| SC-008 | Oui | T020 |
| SC-009 | Oui | T008, T016, T023 |

## Constitution Alignment

- **Minimalisme** : PASS. Aucun scheduler, superviseur, LLM, GUI/TUI, MCP
  sémantique ou processus enfant Maicie.
- **Responsabilité future** : PASS. Les invariants de reprise, source/fraîcheur
  des snapshots et le contrat d'approbation sont inspectables.
- **Sécurité** : PASS sous gate 009. L'approbation est liée à l'action et
  revalidée contre TOCTOU ; Maicie ne possède aucune primitive de spawn.
- **État partagé** : PASS. Une outbox Maicie et un snapshot dérivé ne copient
  pas l'autorité de livraison Bridget.

## Gates avant implémentation

1. Vérifier que Bridget expose effectivement `message_id` client idempotent et
   la lecture d'issue par id. Sinon spécifier/implémenter cette petite extension
   Bridget avant T006 ; ne pas contourner par un nouvel envoi.
2. Ne démarrer T017–T020 qu'après compatibilité publique session 008.
3. Ne démarrer T021–T023 qu'après contrat public SpawnOrder session 009.

## Metrics

- Functional Requirements : 23
- Success Criteria : 9
- Tasks : 26
- Couverture : 100 %
- Problèmes critiques ouverts : 0
- Gates externes : 3 (idempotence publique, 008, 009)
- Problèmes de minimalisme ouverts : 0

## Next Action

L'implémentation reste volontairement non démarrée. Au moment de l'autorisation,
ouvrir `session-11-maicie-orchestration` et traiter T001–T016 comme MVP ; les
lots 008/009 restent bloqués par leurs contrats respectifs.
