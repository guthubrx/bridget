# Archives techniques de l’adaptateur T3

Document d’origine : [specs/160-reprise-quota/reuse-audit.md](../../../../../../specs/160-reprise-quota/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVIII | complexité documentée | Lookup O(log A + F), SQL O(log N), coût chaîne et tri/union décrits au plan actuel |
| `~/.speckit/constitution.md` XIX | minimalisme | pas de module, extractions à deux usages |
| `~/.speckit/constitution.md` VIII | forensic avant correction | T002, tests rouges T003 à T008 |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart |
| `AGENTS.md` et contrat utilisateur courant | livraison complète | Git/build/activation Bridget/nettoyage après gates ; T3 conservé |
| contrat 148 | pas de substitution implicite | FR-015, T026 |

<a id="extrait-2"></a>

## Extrait 2

| Requete | Portee | Resultat |
|---|---|---|
| `sed -n 1296,1450p execution_store.rs` | reconstruction | exclusion de `failed`, raison en dur |
| `grep "'waiting_user_input', 'interrupting'"` | `execution_store.rs` | `:531`, `:1327`, `:1378`, `:1395`, `:1467`, `:1481` |
| `grep CREATE TABLE execution_continuations` | `execution_store.rs` | `:1588`, `mode` fermé, `reason` libre |
| `grep execution_schema_migrations` | `execution_store.rs` | versions 1 à 11, prochaine 12 |
| `sed -n 18060,18135p daemon.rs` | gestionnaire `DeliveryRejected` | suppression du suivi et notification du parent |
| `sed -n 6215,6262p wrapper.rs` | publication de l'échec | transition `failed` puis `DeliveryRejected` |
| `sed -n 4228,4520p daemon.rs` | `schedule_execution_recovery` | queue extractible, garde native, gouvernance |
| `grep reserve_governed_continuation` | `managed_supervisor.rs` | `:64`, sources `InactiveParent`, `RecoveryAfterIdleWrapper` |
| `git show origin/main:.../UsageLimitRecoveryWorker.ts` | T3 | identité, garde de fenêtre passée, armement puis relance |
| lecture `specs/152-reprise-missions/*` | primitives | noms et tâches citées |
| lecture `specs/157-catalogue-quotas/*` | mémoire de limites | `Seen`, `Quota`, FR-008 |
