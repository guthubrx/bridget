# Audit de reutilisation de l'existant — Rattachement Bridget après reprise adaptateur de bureau

## Decision

Statut: PASS
Date: 2026-10-15
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise

Conclusion courte: Le plan étend le montage privé et la reprise native déjà présents. Il ne crée aucun service, stockage, contrat public ou dépendance. La seule nouvelle règle relie deux états existants : la preuve courante et la preuve montée.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 6 |
| Items audites | 6 |
| Reutilisations deja prevues | 6 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 3 |

## Reutilisations correctement identifiees

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/reuse-audit.md#extrait-1).

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Aucun | Aucun | — | Aucune |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun | — | Aucune |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `/Users/moi/.speckit/constitution.md` | Réutiliser avant de créer | Extraire un helper du chemin de reprise actuel. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/.specify/memory/constitution.md` | Preuves avant livraison | Oracle RED, contrôles ciblés et recette réelle. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/.specify/memory/standards.md` | Charger les standards ciblés | Tests et qualité appliqués sans nouveau framework. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/AGENTS.md` | Diagnostiquer avant de modifier | État runtime et flux source observés avant le code. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/AGENTS.md` | Refus sûr et tests obligatoires | Aucun repli d'identité ; résultats réels requis. |

## Specs livrees applicables

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/reuse-audit.md#extrait-2).

## Journal de recherche

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/reuse-audit.md#extrait-3).

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Autorité d'identité | reutiliser | `McpProviderSession` reste la source courante par fil. | 2026-10-15 |
| Remontage | reutiliser | `thread/resume` charge déjà une nouvelle configuration sans changer d'historique. | 2026-10-15 |
| Détection | reutiliser | La carte existante distingue déjà preuve montée et preuve courante. | 2026-10-15 |
| Attestation | reutiliser | Le fait `provider_turn` conserve les règles de droits actuelles. | 2026-10-15 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
