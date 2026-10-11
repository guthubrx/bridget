# Audit de reutilisation de l'existant - 154-isolation-worktree

## Decision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation/specs/154-isolation-worktree

Conclusion courte: Le plan réutilise la règle de dossier, le calcul de droits,
le chemin de fin de tâche et la purge de la session150. Il crée un seul module,
`workspace_git`, parce qu'aucun exécuteur Git borné n'existe et qu'il y a trois
usages réels. Aucune duplication évidente. Deux points restent des questions
utilisateur, pas des doublons.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 14 |
| Items audites | 14 |
| Reutilisations deja prevues | 9 |
| Existants potentiellement pertinents | 4 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 6 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Dossier de l'enfant sous la racine du parent | contrôle `canonicalize` + `starts_with(root)` | `crates/bridget-daemon/src/daemon/native_delegation.rs:421-428` | le worktree est sous la racine, la règle ne change pas |
| Droits hérités sur le dossier de l'enfant | `child_policy`, `snapshot` | `native_delegation.rs:436-440`, `native_permissions.rs:206-260` | appelés une seconde fois sur le worktree réel |
| Projet commun d'un worktree | `resolve_communication_project` (common-dir) | `crates/bridget-daemon/src/communication.rs:64-127` | un worktree reste dans le même projet ; non modifié |
| Fin de tâche | pose de `cleanup_done` | `native_delegation.rs:1071-1080`, `:1283-1285`, `:1474-1477` | le rapport final (lecture seule) s'insère avant ce drapeau |
| Arrêt confirmé avant rapport final | `cancel_tree` | `native_delegation.rs:1380-1478` | le rapport est lu après l'arrêt, rien n'est supprimé |
| Purge sans perte | conditions de purge de la session150 | `specs/150-socle-delegation/research.md` R2 | inchangées grâce à D7 |
| Octets inchangés sans nouveau champ | patron `reply_timeout` de la session150 | `specs/150-socle-delegation/data-model.md` | même `skip_serializing_if` |
| Catalogue par fournisseur | calcul `inherit` / `development` | `native_delegation.rs:380-391` | `worktree` ajouté avec la même logique |
| Caractères de contrôle | `is_disallowed_control` | `crates/bridget-core/src/text_guards.rs:40` | assainissement des sujets de commit |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Exécuteur Git borné | `git_common` interne à `resolve_communication_project` (retire `GIT_*`) | `communication.rs:85-100` | Reprendre la liste des variables retirées dans l'exécuteur ; la fonction interne reste (un usage de lecture, hors périmètre) |
| Délai d'un processus | boucle `try_wait` jusqu'à une échéance | `crates/bridget-daemon/src/daemon.rs:12059-12063` | Reprendre le patron, pas le code (il est dans un test) |
| Faits Git donnés à l'enfant | carte de reprise `managed_resume_worktree` | `crates/bridget-daemon/src/wrapper.rs:623-651` | Réutilisée ; seul changement : `fsmonitor` et verrous optionnels coupés (2 lignes, contre-revue 8) |
| Alerte « checkout principal » | `is_protected_principal_checkout` | `wrapper.rs:552-615` | Même durcissement de 2 lignes ; plus aucun désarmement puisque Bridget ne retire plus de worktree |

## Items ajoutés par la ronde 2

| Item | Existant | Preuve | Décision |
|---|---|---|---|
| Lectures du wrapper par l'exécuteur | `managed_resume_worktree`, `is_protected_principal_checkout` | `wrapper.rs:572-590`, `:623-651` | Remplacer les appels directs par l'exécuteur : une seule implémentation durcie |
| Table de réservations | état du daemon sous verrou | `native_delegation.rs:301-312` | Structure en mémoire dans l'état existant, pas de table SQL |
| Rapport hors verrou | collecte `cleanups` puis exécution après le verrou | `native_delegation.rs:1283-1291` | Même patron « noter sous verrou, exécuter dehors » |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| aucun | - | - | - |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `~/.speckit/constitution.md` XVI | worktrees sous `.worktrees/NNN-nom` | convention reprise pour le nom du dossier |
| `~/.speckit/constitution.md` XVIII, XIX, XX | complexité, minimalisme, responsabilité | exécuteur unique, pas de dépendance, pas de futur hypothétique |
| `docs/delegation-native.md` | « sans gain de droits », dossier `.git` jamais racine | pas d'élargissement du bac à sable ; worktree sous la racine |
| `AGENTS.md` du projet | livraison sans relance de service | aucune relance |
| mémoire `bridget-tests-umask-077-env-prive` | recette de test | quickstart |
| règle utilisateur 154 | Bridget n'est pas un environnement de développement | liste blanche de sous-commandes, pas de PR, fusion ni push |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 148-identite-delegation | contrat `bridget_delegate`, rejeu par `request_id` | `workspace` entre dans l'enveloppe |
| 149-sous-agents-lineage | droits hérités, Lineage fermé | aucun nouvel état Lineage |
| 150-socle-delegation | purge par arbre, `cleanup_done`, refus nommés, `reply_timeout` | purge inchangée, même patron de champ |
| 100-observation-partage | `file_collision` sur chemins absolus | inchangée ; test de non-collision |
| 138 (projet commun) | worktrees partagent le common-dir | un worktree reste dans le projet |
| 145 (communication git) | résolution Git hors verrou | avant-vol hors verrou |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -i worktree` | `crates/` | contrat T3, carte de reprise, projet commun ; aucune création ni suppression de worktree |
| `rg "Command::new\(\"git\"\)"` | `crates/bridget-daemon/src` | `build_identity.rs`, `communication.rs`, `daemon.rs` (tests) ; aucun exécuteur partagé borné |
| `rg "check-ref-format"` | `crates/` | aucun |
| `rg "try_wait\|wait_timeout"` | `crates/bridget-daemon/src` | boucle d'attente dans `daemon.rs` (tests) ; aucun utilitaire de délai |
| `rg "info/exclude"` | `crates/` | aucun ; écriture rejetée (D11) |
| `rg "file_collision\|FileCollision"` | `crates/bridget-daemon/src` | `observation.rs` : détection ; pas de prévention |
| `rg "fn is_disallowed_control"` | `crates/` | `bridget-core/src/text_guards.rs:40` |
| `rg "mod .*_tests"` | `native_delegation.rs` | `:2714` : patron de module de tests enfant |
| `rg "git" wrapper.rs` (lectures) | `wrapper.rs:572-590`, `:623-641` | deux lectures sans `fsmonitor` coupé : durcies |
| `rg "workspace" native_delegation.rs` | daemon | seulement `sandbox_workspace_write` du bac à sable Codex |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Exécuteur Git | creer nouveau module `workspace_git` | 3 usages, sécurité en un point, aucun équivalent borné | 2026-10-11 |
| Dépendance `git2` | rejeter | dépendance lourde, binaire Git déjà requis par le projet | 2026-10-11 |
| Exécution de Git | **arbitrage utilisateur** : le daemon exécute Git pour tout parent autorisé, comme T3 (`GitVcsDriverCore.ts:3418-3419`, `:3789`), durci ; risque résiduel accepté (ADR 011) ; confinement et lecture de fichiers écartés | 2026-10-11 |
| Retrait automatique de worktree | rejeter | contre-revue 2, 3, 4, 7 : risques de perte ; Bridget ne supprime jamais | 2026-10-11 |
| Création du worktree par le wrapper | rejeter | le wrapper n'est pas confiné ; chemin absent à l'admission ; échec asynchrone | 2026-10-11 |
| Écriture de `info/exclude` | rejeter | surface de lien symbolique ; refus `workspace_dir_not_ignored` | 2026-10-11 |
| Prévention de collision par refus | rejeter | changerait le comportement 149 | 2026-10-11 |
| `existing_worktree` de T3 | rejeter | déjà couvert par `cwd` sous la racine | 2026-10-11 |
| `observation.rs` | ne pas modifier | chemins distincts, aucun code nécessaire | 2026-10-11 |

## Gate avant tasks

- [x] Arbitrage utilisateur sur l'exécution de Git (spec.md, 2026-10-11)
- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
