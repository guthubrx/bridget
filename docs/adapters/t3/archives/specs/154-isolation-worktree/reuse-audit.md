# Archives techniques de l’adaptateur T3

Document d’origine : [specs/154-isolation-worktree/reuse-audit.md](../../../../../../specs/154-isolation-worktree/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Requete | Portee | Resultat |
|---|---|---|
| `rg -i worktree` | `crates/` | contrat T3, carte de reprise, projet commun ; aucune création ni suppression de worktree |
| `rg "Command::new\(\"git\"\)"` | `crates/bridget-daemon/src` | `build_identity.rs:45`, `communication.rs:88`, `wrapper.rs:574/:625`, plus fixtures/tests ; aucun exécuteur partagé borné |
| `rg "check-ref-format"` | `crates/` | aucun |
| `rg "try_wait\|wait_timeout"` | `crates/bridget-daemon/src` | boucle d'attente dans `daemon.rs` (tests) ; des attentes bornées de processus existent ; aucun exécuteur Git partagé borné parmi les appels directs relus |
| `rg "info/exclude"` | `crates/` | aucun ; écriture rejetée (D11) |
| `rg "file_collision\|FileCollision"` | `crates/bridget-daemon/src` | `observation.rs` : détection ; pas de prévention |
| `rg "fn is_disallowed_control"` | `crates/` | `bridget-core/src/text_guards.rs:40` |
| `rg "mod .*_tests"` | `native_delegation.rs` | `:9698-9699` : patron de module de tests enfant |
| `rg "git" wrapper.rs` (lectures) | `wrapper.rs:572-590`, `:623-641` | deux lectures sans `fsmonitor` coupé : durcies |
| `rg "workspace" native_delegation.rs` | daemon | seulement `sandbox_workspace_write` du bac à sable Codex |

<a id="extrait-2"></a>

## Extrait 2

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Exécuteur Git | creer nouveau module `workspace_git` | 3 usages, sécurité en un point, aucun équivalent borné | 2026-10-11 |
| Dépendance `git2` | rejeter | dépendance lourde, binaire Git déjà requis par le projet | 2026-10-11 |
| Exécution de Git | **arbitrage utilisateur** : le daemon exécute Git pour tout parent autorisé, par le contrat natif (`GitVcsDriverCore.ts:3418-3419`, `:3789`), durci ; risque résiduel accepté (ADR 011) ; confinement et lecture de fichiers écartés | 2026-10-11 |
| Retrait automatique de worktree | rejeter | contre-revue 2, 3, 4, 7 : risques de perte ; Bridget ne supprime jamais | 2026-10-11 |
| Création du worktree par le wrapper | rejeter | le wrapper n'est pas confiné ; chemin absent à l'admission ; échec asynchrone | 2026-10-11 |
| Écriture de `info/exclude` | rejeter | surface de lien symbolique ; refus `workspace_dir_not_ignored` | 2026-10-11 |
| Prévention de collision par refus | rejeter | changerait le comportement 149 | 2026-10-11 |
| `existing_worktree` de T3 | rejeter | déjà couvert par `cwd` sous la racine | 2026-10-11 |
| `observation.rs` | ne pas modifier | chemins distincts, aucun code nécessaire | 2026-10-11 |
