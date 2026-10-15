# Audit de reutilisation de l'existant — Rattachement Bridget après reprise T3

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

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Lecture de la preuve courante | `McpProviderSession.readMcpProviderSession()` | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:1812` | L'autorité par fil existe déjà. |
| Préparation de la configuration | `threadRuntimeParams()` | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:1809` | Prépare déjà Bridget et T3 Code MCP. |
| Mémoire de la preuve montée | `bridgetCredentialsByThread` | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:1805` | Aucun registre parallèle n'est requis. |
| Reprise du fil natif | Requête `thread/resume` existante | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:6583` | Elle conserve le même fil et recharge sa configuration. |
| Publication des droits | `publishMcpProviderPermissions()` | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:6282` | L'attestation reste liée au run. |
| Test de protocole | Banc de rejeu Codex | `/Users/moi/.cache/t3-session166-rattachement/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.test.ts:1840` | Il contrôle l'ordre et la configuration des requêtes. |

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

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| SPEC-149 | Identité privée distincte par fil | La preuve reste indexée par `ThreadId`. |
| SPEC-163 | Session fournisseur partagée et reprise | Le processus partagé ne doit pas fusionner les identités. |
| SPEC-165 | Cycle de vie borné des liens T3 | Le défaut courant est séparé de la limite de fichiers déjà corrigée. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg "prepareMcpSession|bridgetCredentialsByThread|startNativeTurn"` | Adaptateur et gestionnaire T3 | Montage, mémoire et contrôle du tour trouvés. |
| `rg "thread/resume|resumeThread"` | Adaptateur et tests T3 | Reprise native et traitement des archives trouvés. |
| `rg "McpProviderSession|credential stable|shared app-server"` | Tests T3 | Couvertures proches trouvées, sans rotation avant tour. |
| Inspection SQLite et journaux expurgés | Runtime T3/Bridget | Session partagée et échec limité au fil repris confirmés. |

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
