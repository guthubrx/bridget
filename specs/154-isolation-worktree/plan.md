# Plan - Session154 isolation optionnelle dans un worktree

**Spec** : `specs/154-isolation-worktree/spec.md`
**Branche** : `session-154-isolation-worktree`
**Worktree** : le worktree courant du pipeline 150-158 (exécution séquentielle,
sans commit automatique, diff enregistré par session)
**Date** : 2026-10-11 (révisé après deux rondes de contre-revue Codex)
**Statut** : prêt pour l'implémentation. Arbitrage utilisateur reçu le 2026-10-11 : le daemon exécute Git pour tout parent autorisé, comme T3, avec les durcissements ci-dessous (`spec.md`, « Arbitrage utilisateur - exécution de Git »). 28 tâches.

## Résumé

Un paramètre optionnel `workspace` sur `bridget_delegate`. Avec `worktree`,
Bridget crée un worktree Git sous le dossier du parent, sur une nouvelle
branche, sans tenir le verrou du daemon pendant Git. L'enfant y travaille avec
les droits du parent, sans extension. Le résultat dit la branche et les
commits. Bridget ne supprime jamais un worktree ni une branche. Il lit l'état
final et le déclare `kept`. L'humain nettoie. Bridget n'ajoute aucune autre
fonction Git.

## Contexte technique

- Langage : Rust 2024, travail dans `crates/bridget-daemon` et
  `crates/bridget-transport`.
- Stockage : charge JSON de `Task` (SQLite existant). Aucune table nouvelle.
- Git : binaire système, version 2.44 ou plus.
- Aucune nouvelle dépendance Rust.
- Tests : recette `umask 077` (quickstart), dépôts Git jetables sous `$R`.

## Décisions d'architecture

| # | Décision | Alternative rejetée | Raison |
|---|---|---|---|
| D1 | Champ `workspace` sur `Delegate` | outil `bridget_workspace` séparé | un appel reste un appel |
| D2 | Worktree sous `<cwd>/.worktrees/<task8>` | hors racine, ou dans `.git` | `starts_with(root)` suffit, aucun droit ajouté |
| D3 | Admission en deux passes dans le daemon : réserver sous verrou, Git hors verrou, tout revalider sous verrou, droits calculés sur le worktree réel | création sous verrou ; création dans le wrapper ; création dans le tick | verrou jamais tenu pendant Git ; le contexte de droits Claude dépend des fichiers du dossier ; refus synchrone préservé |
| D4 | Refus précoces `workspace_requires_development` et `workspace_inheritance_unavailable` | laisser échouer à `settings_revision_changed` | refus lisibles, aucun effet disque |
| D5 | Exécuteur Git unique, durci, liste blanche sans suppression, configurations système et globale ignorées | `std::process::Command` partout, ou `git2` | sécurité en un point, pas de dépendance |
| D6 | Bridget ne supprime jamais un worktree ni une branche | retrait automatique sans force ; retrait après contrôle de propreté | quatre risques de perte écartés par construction (contre-revue 2, 3, 4, 7) |
| D7 | `cleanup_done` après le rapport final, état `kept` | nouveau drapeau de purge | la purge 150 reste inchangée |
| D8 | Rapport lu dans Git (dont `head_on_branch`), ligne de fin écrite par Bridget, objet dans `task_status` | texte de l'enfant comme source | un enfant ne forge pas les faits |
| D9 | `shared_writers` informatif | refus de la 2e délégation partagée | compatibilité 149, usages en lecture |
| D10 | Refus des dépôts à configuration exécutable (filtres, inclusions, clone partiel) | neutraliser chaque filtre par `-c` | liste incomplète si une inclusion en ajoute (contre-revue 1) |
| D11 | `.worktrees/` doit être déjà ignoré, plafond de 32 worktrees, aucune écriture dans la configuration du dépôt | écrire `info/exclude` ; pas de plafond | supprime une surface de lien symbolique ; borne l'accumulation (contre-revue 5, 9) |
| D12 | Rapport hors verrou : le tick note un travail, Git lit sans verrou, un verrou court range le rapport | Git dans le tick | verrou global jamais tenu pendant Git (ronde 2, point 4) |
| D13 | Table de réservations en mémoire (clé et plafond) ; la seconde passe ne refait pas les contrôles d'état Git créés par la requête | pas de réservation ; revalidation totale | deux appels de même clé ne créent pas deux worktrees ; la branche réservée et la 32e place ne se refusent pas elles-mêmes (ronde 2, points 5 et 7) |
| D14 | Exécution de Git par le daemon pour tout parent autorisé, comme T3, durcie ; risque résiduel de changement de configuration accepté | Git confiné ; Git par fichiers ; réservation aux parents non confinés | décision utilisateur du 2026-10-11 ; T3 fait de même avec moins de durcissements |

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | `WorkspaceRequest`, `Delegate.workspace` optionnel |
| `crates/bridget-daemon/src/lib.rs` | déclaration du module `workspace_git` |
| `crates/bridget-daemon/src/workspace_git.rs` (nouveau) | exécuteur Git, validations, avant-vol, création, rapport |
| `crates/bridget-daemon/src/workspace_git_tests.rs` (nouveau) | tests de l'exécuteur et des validations |
| `crates/bridget-daemon/src/delegation.rs` | `Task.workspace`, `WorkspaceRecord`, `visible_changed` l'ignore |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | avant-vol hors verrou, admission en deux passes, contrôles précoces, rapport au résultat et à la fin, catalogue, `shared_writers`, `task_view` |
| `crates/bridget-daemon/src/daemon/native_workspace154_tests.rs` (nouveau) | tests `native154_*` |
| `crates/bridget-daemon/src/delegation_mcp.rs` | schéma, liste blanche et lecture de `workspace` |
| `crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs` | clés de catalogue figées |
| `crates/bridget-daemon/src/wrapper.rs` | deux lectures Git de la carte de reprise passent par l'exécuteur durci (sortie et temps bornés, `status` omis si filtre exécutable) |
| `crates/bridget-daemon/src/observation.rs` | test seul : chemins distincts sans collision (aucun code) |
| `docs/delegation-native.md`, `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` | stratégie, refus, conservation, nettoyage humain, limites |
| `docs/decisions/154-isolation-worktree-optionnelle.md` | ADR |

Le code existant de droits (`native_permissions.rs`) ne change pas.

## Étapes d'implémentation

1. Mesurer le coût de `worktree add` (Article XXII) pour régler le délai.
2. Contrat : type `WorkspaceRequest`, octets inchangés sans le champ.
3. Exécuteur Git et validations, tests rouges d'abord.
4. Enregistrement durable dans `Task`.
5. Avant-vol hors verrou, refus de configuration exécutable.
6. Admission en deux passes : réservation, création hors verrou, revalidation,
   droits sur le worktree réel.
7. Catalogue et schéma MCP.
8. Rapport au passage à `result_available`, objet de statut, ligne de fin.
9. Rapport final pour échec et annulation, état `kept`.
10. `shared_writers`, test de non-collision, durcissement du wrapper.
11. Documentation, ADR, régression, relecture.

## Contrôle constitution

| Article | Statut | Justification |
|---|---|---|
| III SpecKit | OK | spec, plan, tâches avant code |
| VII ADR | OK | ADR 154 prévue (T024) |
| VIII Forensic | OK | mesure préalable (T002) ; tests rouges avant correction |
| XV Test-before-next | Adapté | pas de commit automatique ; chaque tâche testée avant d'être cochée |
| XVI Worktree | OK | session dans le worktree du pipeline ; le produit crée des worktrees sous `.worktrees/`, conforme à la convention |
| XVIII Complexité | OK | voir ci-dessous |
| XIX Minimalisme | OK | voir ci-dessous |
| XX Vertus LLM | OK | voir ci-dessous |

### Complexité (Article XVIII)

- Avant-vol : au plus 8 commandes Git de lecture, O(1) en nombre ; `worktree
  list` est O(w), w ≤ 32 enregistrés sous `.worktrees/` (borné par le plafond).
- Création : 1 `worktree add`, coût proportionnel à la taille de l'arbre
  extrait, mesuré en T002, hors verrou.
- Rapport : 5 commandes, bornées à 50 commits et 1 Mio de sortie.
- `shared_writers` : parcours des tâches actives, O(128) par la borne 150.

### Minimalisme (Article XIX)

- Un module nouveau, `workspace_git`, justifié par l'avant-vol, la création et
  le rapport, et par le point unique de sécurité.
- Pas de table, pas de dépendance, pas de nouvel état Lineage, pas de nouvel
  outil MCP, **pas de code de suppression** ni de retour arrière.
- Rejetés : `start_from_origin`, `existing_worktree` (déjà couvert par `cwd`),
  suppression de branche, retrait de worktree, commit automatique, écriture dans
  `info/exclude`, commande de liste, création par le wrapper.

Potentiel minimalisme : aucune ligne existante supprimée ; environ 300 lignes
ajoutées, dont 60 % de tests. La révision retire environ 120 lignes prévues
(retrait, retour arrière, états de rétention, exclusion).

### Responsabilité future (Article XX)

- Chaque décision a sa preuve `fichier:ligne` dans `research.md`.
- Ce qui n'est pas vérifié : le comportement réel d'un enfant Codex sous bac à
  sable dans un worktree lié (question 2) ; le coût sur un très gros dépôt ; le
  comportement de Git 2.44 à 2.54 (seule la 2.55.0 est testée).
- La charge future baisse pour le mainteneur (aucune décision de suppression)
  et monte pour l'humain (nettoyage), borné par le plafond de 32.

## Stratégie de test

| Niveau | Contenu |
|---|---|
| Rouge d'abord | création inexistante, refus absents, rapport absent |
| Unitaire | validations de noms, exécuteur (environnement, liste blanche, délai, version), enregistrement durable |
| Intégration | dépôts Git réels sous `$R` : création, refus hostiles (filtre, objet absent, inclusion, lien), rapport 0/2/60 commits et tête détachée, conservation (sale, emboîté, `showUntrackedFiles=no`), annulation, rejeu, purge 150, verrou libre pendant Git |
| Non-régression | filtres `native148`, `native149`, `native150`, `delegation149`, `spec100`, `spec_087` |
| Suite complète | recette `--workspace --no-fail-fast` avant clôture |

## Risques

| Risque | Parade |
|---|---|
| Un parent confiné modifie la configuration du dépôt entre le contrôle et l'exécution de Git et obtient l'exécution de code avec les droits du daemon | **Risque accepté par l'utilisateur au même niveau que T3** (ADR 011) ; documenté dans l'ADR 154 ; les durcissements réduisent la surface sans la fermer |
| Deux appels concurrents de même clé | table de réservations (D13), test SC-010 |
| Filtre ou hook du dépôt exécuté avec les droits du daemon | configurations système et globale ignorées ; refus des filtres, inclusions et clones partiels dans la configuration locale ; hooks et fsmonitor coupés ; tests hostiles |
| Récupération réseau implicite | refus des clones partiels ; `GIT_NO_LAZY_FETCH`, `protocol.allow=never` ; test d'objet absent |
| Daemon bloqué pendant Git | deux passes, Git hors verrou, délai 20 s, test de verrou libre |
| Lien `.worktrees` remplacé entre le contrôle et Git | recontrôle du chemin canonique après création ; résiduel accepté (attaquant = parent délégant, Git refuse un dossier non vide) |
| Restes après un échec tardif | journal avant l'appel, restes nommés dans le refus ; inoffensifs ; aucune suppression |
| Worktrees `kept` qui s'accumulent | plafond de 32, recette de nettoyage, chemin dans le résultat ; non automatisé par choix |
| Tête détachée ou branche changée par l'enfant | `head_on_branch` faux dans le rapport et la ligne de fin |
| Enfant sous `workspace-write` qui ne peut pas commiter | travail conservé dans le worktree, `dirty: true` ; question 2 |
| Paires Claude vers Claude refusées | refus explicite, question 1, session 154b |
| Sujets de commit hostiles | assainis et traités comme données ; champs structurés lus dans Git |
| Dépôts Git LFS refusés | documenté ; le partage `shared` reste disponible |
| Chevauchement avec la session151 (résultat structuré) | l'objet `workspace` est autonome ; 151 le déplace sans changer sa forme |

## Livraison

Code sur disque dans le worktree du pipeline, sans commit automatique ni
relance de service. Diff cumulé enregistré dans
`specs/154-isolation-worktree/validation/diff-154.patch`.
