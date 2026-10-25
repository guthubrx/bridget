# Plan - Session154 isolation optionnelle dans un worktree

**Spec** : `specs/154-isolation-worktree/spec.md`
**Branche** : `session-154-isolation-worktree`
**Worktree** : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree
**Base** : `89f7a4163c2fb27229587f50a46414295ba3674d`, incluant153 `7c848850ba6067cf1d0d94b8edf0dd11896a1b24` livrée et active
**Date** : 2026-10-11 (révisé après deux rondes de contre-revue Codex)
**Statut** : prêt pour l'implémentation. Arbitrage utilisateur reçu le 2026-10-11 : le daemon exécute Git pour tout parent autorisé, avec les durcissements ci-dessous (`spec.md`, « Arbitrage utilisateur - exécution de Git »). 28 tâches.

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/plan.md#extrait-1).

## Fichiers touchés

| Fichier | Changement |
|---|---|
| `crates/bridget-transport/src/protocol.rs` | `WorkspaceRequest`, `Delegate.workspace` optionnel |
| `crates/bridget-daemon/src/lib.rs` | déclaration du module `workspace_git` |
| `crates/bridget-daemon/src/workspace_git.rs` (nouveau) | exécuteur Git, validations, avant-vol, création, rapport |
| `crates/bridget-daemon/src/workspace_git_tests.rs` (nouveau) | tests de l'exécuteur et des validations |
| `crates/bridget-daemon/src/delegation.rs` | `Task.workspace`, `WorkspaceRecord`, `visible_changed` l'ignore |
| `crates/bridget-daemon/src/daemon/native_delegation.rs` | rejeu/autorisation du cwd canonique avant avant-vol Git hors verrou, admission en deux passes, contrôles précoces, rapport au résultat et à la fin, catalogue, `shared_writers`, `task_view` |
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
- Rapport : nombre constant de commandes, bornées à 50 commits et 1 Mio de sortie.
- `shared_writers` : parcours des tâches actives, O(128) par la borne 150.

### Minimalisme (Article XIX)

- Un module nouveau, `workspace_git`, justifié par l'avant-vol, la création et
  le rapport, et par le point unique de sécurité.
- Pas de table, pas de dépendance, pas de nouvel état Lineage, pas de nouvel
  outil MCP, **pas de code de suppression** ni de retour arrière.
- Rejetés : `start_from_origin`, `existing_worktree` (déjà couvert par `cwd`),
  suppression de branche, retrait de worktree, commit automatique, écriture dans
  `info/exclude`, commande de liste, création par le wrapper.

Potentiel minimalisme : aucune ligne existante supprimée ; volume final à mesurer après implémentation ; aucune estimation de lignes ne vaut preuve. La révision retire environ 120 lignes prévues
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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/plan.md#extrait-2).

## Livraison

Code sur disque dans le worktree du pipeline, sans commit automatique ni
relance de service. Diff cumulé enregistré dans
`specs/154-isolation-worktree/validation/diff-154.patch`.

## Réutilisation et séparation des responsabilités

- Exécuteur Git : `workspace_git.rs`, `workspace_git_tests.rs`, déclaration
  `lib.rs`, lectures T022 dans `wrapper.rs`. API proposée :
  `run_git(cwd: &Path, args: &[&str], timeout: Duration) -> Result<GitOutput, String>`,
  `preflight(cwd: &Path, base_ref: Option<&str>, branch: Option<&str>) -> Result<WorkspacePreflight, String>`,
  `create(preflight: &WorkspacePreflight, task_id: &str) -> Result<WorkspaceRecord, String>`,
  `report(workspace: &WorkspaceRecord) -> Result<WorkspaceRecord, String>`.
  `GitOutput` et `WorkspacePreflight` sont des valeurs locales/transitoires ; aucun
  stockage ni dépendance. Les types restent ceux nécessaires au module prévu.
- Protocole et stockage : `protocol.rs`, `delegation.rs`, `delegation_mcp.rs`,
  payload optionnel et enveloppe ; conserver `keep_alive_secs`, `brief` et leurs
  octets153. Aucun champ public de ronde dans `WorkspaceRecord`.
- Natif : `native_delegation.rs`, tests154, catalogue149 et test observation T021.
  Seul propriétaire de l’admission, des réservations, du tick et du CAS.
  Capturer `task_id`, `mission`, `round_no()` sous verrou ; recharger avant
  sauvegarde. `kept` ne ferme pas les droits. `cleanup_done` exige rapport terminé
  et arrêt confirmé/fin de garde153 ; ronde1 avec garde active ne suffit pas.

Le propriétaire Git ne modifie ni `communication.rs`, ni la résolution du
projet commun. Les signatures sont recommandations d’intégration ; aucun
helper supplémentaire ni champ public nouveau n’est autorisé sans preuve.

## Décisions de contre-revueR1 — en cours d’implémentation

Référence : adversarial-review-claude.md,15hypothèses. Échec de rapport : une
tentative bornée, faitsNone, mention indisponible ; publication/cleanup avancent
selon leurs conditions153. Chaque annulation rapporte tous les membres worktree
de l’arbre hors verrou. `begin_round` vide les faits courants et conserve kept.
Le budget du texte enfant retire la taille exacte de la ligne finale puis coupe
en UTF8. Une imitation de cette ligne est assainie ; l’objet workspace fait foi.

La whitelist valide les formes/options. Git2.55 refuse `status --no-textconv`
et `status --no-ext-diff` (RC129 constaté par le principal). Ces options ne
sont appliquées qu’aux formes compatibles de log ; environnement/configuration
restent communs. Aucun refus global des enfantsClaude : les contrôles149
full+never et les schémas fermés existants restent les autorités.

H11 : ancre du registreGitparent retenue ci-dessous, implémentation à vérifier.
H12 : borne du lot de rapports encore en conception.
Ne pas inventer de quota ni déclarer leur correction acquise. Le marqueur privé Task.workspace_report_mission est autorisé par le complément
du principal ci-dessous ; aucun champ public de ronde ni table nouvelle.

### Complément technique du principal — marqueur et ancre

Marqueur privé retenu : `Task.workspace_report_mission`, default et omission
siNone. Il identifie la mission déjà rapportée. Aucun champ de ronde ajouté à
WorkspaceRecord ni au contrat public. CAS waiting_for_children+mission+round
protège le succès ; le marqueur protège aussi échec de livraison, cleanup et
reprise après panne, même avec faitsNone et kept. `begin_round` remet faits et
marqueur à vide. La condition de première publication vaut par mission.

AncreGit : utiliser le registre Git du parent hors dossier enfant, avec parcours
borné et correspondance des chemins toplevel/path enregistrés. Ne pas lire le
fichier.git contrôlable de l’enfant. Ancre absente : rapport indisponible, aucun
champ public/privé d’ancre supplémentaire. Implémentation à vérifier.

Neutraliser dans le texte enfant le préfixe `[Bridget] espace de travail` en
`[Enfant] espace de travail`, puis ajouter l’unique suffixe d’autorité. Son
budget exact en octets et une coupeUTF8 gardent la limite262144octets.

## ComplémentH12 et limiteSIGTERM

WorkspaceWorkArcMutexséparé unique : creationsMap etreportsSet des
(task,mission,round) en cours. Claim empêche tickconcurrent de relancerGit lent ;
DropRAII retireseulementl’entrée du verrou séparé. Aucunthread/backgroundengine
ni quota arbitraire. Choixautoriséroot ; testnatifconcurrence àprouver.

SIGTERM sur seulchild.id Git puis reapborné, pasSIGKILL. Programmehostile
ignorantSIGTERM peutpersister ; retourborné, pas arrêtabsolu. Parent/ancêtre
principal etregistreGitcommun restent la sourcefiable du modèleADR011.
FondationsR1 :15PASS/7FAIL ; T004/T021/T022acquises ; cinqcasesdone,23open.
T011attendcontratferméglobal ; T003/T005nonacquises. Aucun GREENglobal.
