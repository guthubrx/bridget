# Session154 - Isolation optionnelle de l'enfant dans un worktree Git

<!-- SPEC-FORMALISM:START -->
## Fiche Synthèse

Spec: 154-isolation-worktree
Titre: Isolation optionnelle de l'enfant dans un worktree Git
Statut: Livré le 2026-10-12 (28/28 ; CONVERGED P2 ; installé et actif ; nettoyage terminé)
Priorité: P1
Tâches: 28/28 acquises sur preuves distinctes
Tests: Suite unique 2194 PASS/9 FAIL/72 ignorés avant correctifs ; huit échecs historiques, fixture UTC corrigée 1 PASS puis ciblés 5/3/2 PASS ; aucune seconde suite ni GREEN global

Résumé:
- Contexte: Un enfant natif travaille dans le dossier du parent. Si le parent ou un autre enfant écrit dans le même dossier, les fichiers se mélangent. Bridget ne fait que prévenir après coup (`file_collision`).
- Objectif: Permettre au parent de demander un espace de travail dédié (worktree Git) pour un enfant, et de recevoir la branche et les commits produits. Bridget crée le worktree. Il ne le supprime jamais.
- Dépendances: SPEC-148, SPEC-149, SPEC-150 et SPEC-153 (livrées). La base inclut aussi les sessions151 et152 livrées ; leurs contrats existants sont conservés.

Fichiers:
- spec.md: ✓
- tasks.md: ✓
- plan.md: ✓
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-154-isolation-worktree`
**Created**: 2026-10-11
**Status**: Livré (2026-10-12, CONVERGED P2, installé/actif, nettoyage terminé)
**Priority**: P1
**Dependencies**: SPEC-148, SPEC-149, SPEC-150, SPEC-153

Date : 2026-10-11. Accord utilisateur : feuille de route « Bridget cinq étoiles
en communication et délégation » (ADR 150b, ligne 154). Critère visé :
Isolation, de 1 à 5 (`specs/150-socle-delegation/research-t3-bridget.md:23`).

## Besoin et périmètre

Aujourd'hui un enfant natif reçoit le dossier du parent, figé à l'admission
(`crates/bridget-daemon/src/daemon/native_delegation.rs:421-428`). Il écrit dans
le même arbre de fichiers que le parent et que les autres enfants. Bridget ne
sait que signaler un risque : l'événement `file_collision` se déclenche quand
deux auteurs écrivent le même chemin en moins de 30 secondes
(`crates/bridget-daemon/src/observation.rs:441-460`). Il n'empêche rien.

T3 Code propose trois stratégies pour un fil lancé : `root`, `existing_worktree`
et `worktree{baseRef, branch, startFromOrigin}`
(`packages/contracts/src/orchestrationV2.ts`, `OrchestrationV2ThreadLaunchWorkspaceStrategy`).
Mais un enfant lancé par `delegate_task` partage le worktree du parent.

Cette session ajoute à `bridget_delegate` un paramètre optionnel `workspace`.
Deux stratégies :

- `shared` : le comportement actuel. C'est le défaut.
- `worktree` : Bridget crée un worktree Git dédié, sur une nouvelle branche
  partie d'une base choisie. L'enfant travaille dedans. Le résultat dit la
  branche, la base et les commits produits.

**Contrainte forte de l'utilisateur.** Bridget n'est pas un environnement de
développement. Bridget crée le worktree et lit son état. Bridget ne fait rien
d'autre côté Git : ni commit, ni fusion, ni rebase, ni push, ni récupération
réseau, ni pull request. **Bridget ne supprime jamais un worktree ni une
branche.** Le nettoyage appartient à l'humain (décision de la contre-revue,
voir `adversarial-review-codex.md`). Cette règle supprime par construction tout
risque de perte de travail causé par Bridget.

**Correspondance avec T3.** `root` correspond à `shared`. `existing_worktree`
correspond déjà à `cwd` pointant vers un worktree existant sous la racine du
parent (aucun code nouveau). `startFromOrigin` est exclu : il demande un accès
réseau.

**Hors périmètre** :

- fusion, rebase, push, pull request, `fetch`, résolution de conflits ;
- suppression ou nettoyage de worktrees et de branches, par Bridget ;
- commande de liste des worktrees (la commande Git de l'humain suffit, voir
  la recette de la documentation) ;
- interface graphique, affichage de la branche dans le panneau Lineage T3
  (contrat 149 fermé) ;
- levée du refus pour les paires Claude ou GLM vers Claude ou GLM (question 1) ;
- élargissement des droits d'écriture du bac à sable de l'enfant (question 2) ;
- dépôts qui utilisent des filtres Git exécutables, des inclusions de
  configuration ou un clone partiel (refusés, voir FR-013) ;
- extension du résultat structuré général déjà livré en session151 : cette session ajoute seulement
  l'objet `workspace` et une ligne de fin ;
- extension de la reprise après panne déjà livrée en session152 ;
- sous-modules Git et fichiers ignorés par Git (non copiés, voir hypothèses).

## Scénarios utilisateur

### US1 - Un enfant travaille dans son propre worktree (P1)

Un parent délègue une réécriture à un enfant avec
`workspace: {strategy: "worktree", base_ref: "main"}`. Bridget crée un worktree
sous le dossier du parent, sur une nouvelle branche. L'enfant démarre dans ce
worktree. Le dossier du parent ne change pas : ni son HEAD, ni son index, ni
ses fichiers. Deux enfants lancés ainsi n'écrivent jamais le même fichier.

### US2 - La création est sûre et les droits sont hérités (P1)

Bridget valide tout avant d'écrire sur le disque. Un nom de branche invalide,
une base inconnue, un dossier hors dépôt Git, un dépôt à configuration
exécutable, un dossier `.worktrees` non ignoré, une posture en lecture seule ou
une paire de fournisseurs non prouvée donne un refus nommé. Un refus avant la
création ne laisse aucun worktree, aucune branche et aucune tâche. Le worktree
reste sous le dossier du parent. L'enfant reçoit exactement les droits que le
parent a prouvés, sans extension. Rejouer la même clé ne crée pas un second
worktree. Pendant que Git travaille, le daemon continue de servir les autres
appels.

### US3 - Le résultat dit la branche et les commits (P1)

À la fin de la mission, le parent reçoit : le nom de la branche, le commit de
base, le commit final, le nombre de commits produits (avec les 50 derniers),
un indicateur « modifications non commitées » et un indicateur « la tête est
bien sur la branche ». Ces faits viennent de Git, lus par Bridget. Ils ne
viennent pas du texte de l'enfant.

### US4 - Le travail de l'enfant n'est jamais supprimé par Bridget (P1)

Quand la mission est finie, échouée ou annulée et que l'enfant est arrêté,
Bridget relit l'état du worktree et le marque `kept` (conservé). Il ne
supprime ni le dossier, ni la branche. La purge de la session150 efface des
lignes de base, jamais des fichiers. L'humain retire un worktree avec sa
commande Git habituelle. Un plafond de 32 worktrees sous un même dossier
rappelle de nettoyer.

### US5 - Le partage reste le défaut et le risque est annoncé (P2)

Un appel sans `workspace` se comporte exactement comme en 150. Quand un enfant
en développement part dans un dossier déjà utilisé par un autre enfant actif en
développement, le reçu indique `shared_writers` et le parent peut choisir un
worktree. Rien ne bloque l'appel.

## Exigences fonctionnelles

**Contrat**

- **FR-001** : `bridget_delegate` accepte un objet optionnel `workspace`.
  `strategy` vaut `shared` ou `worktree`. Pour `worktree`, `base_ref` et
  `branch` sont facultatifs. Toute autre clé ou valeur est refusée
  (`invalid_request`).
- **FR-002** : Sans `workspace`, la requête produit les mêmes octets canoniques
  et le même comportement qu'en 150. `workspace` fait partie de l'enveloppe
  idempotente : omis et `shared` explicite sont deux enveloppes différentes.
- **FR-003** : Rejouer une clé admise rend la tâche existante. Aucun second
  worktree n'est créé.
- **FR-004** : `bridget_capabilities` annonce `workspace_strategies` et, pour
  chaque fournisseur, `worktree` (booléen) et `worktree_refusal`.

**Création sûre**

- **FR-005** : `worktree` exige une posture effective `development`. Sinon le
  refus est `workspace_requires_development`.
- **FR-006** : Une paire parent Claude ou GLM vers enfant Claude ou GLM est
  refusée avec `workspace_inheritance_unavailable`, avant tout effet sur le
  disque. Cause : la règle de droits actuelle exige le même dossier
  (`crates/bridget-daemon/src/native_permissions.rs:229-230`).
- **FR-007** : Le dossier demandé doit appartenir à un dépôt Git non nu. Sinon
  `workspace_not_git`.
- **FR-008** : `base_ref` vaut `HEAD` s'il est omis. Bridget le résout en un
  commit unique et le fige (`base_commit`). Une référence inconnue donne
  `workspace_base_unavailable`. Une valeur qui commence par `-` est refusée.
  Git doit être en version 2.44 ou plus, sinon `workspace_git_unsupported`.
- **FR-009** : `branch` vaut `bridget/<8 premiers caractères du task_id>` s'il
  est omis. Un nom invalide donne `workspace_branch_invalid`. Une branche déjà
  existante donne `workspace_branch_exists`. Bridget ne réécrit jamais une
  branche existante.
- **FR-010** : Le worktree est créé dans `<dossier du parent>/.worktrees/<8
  premiers caractères du task_id>`. Le dossier `.worktrees` doit être un vrai
  dossier, pas un lien symbolique. Après la création, Bridget recontrôle que le
  chemin canonique du worktree et le dossier de départ de l'enfant sont les
  chemins attendus et restent sous la racine du parent. Sinon
  `workspace_path_unsafe`.
- **FR-011** : `.worktrees/` doit déjà être ignoré par la configuration du
  dépôt (`.gitignore` ou `info/exclude`). Sinon `workspace_dir_not_ignored`.
  Bridget n'écrit dans aucun fichier de configuration ou d'exclusion du dépôt.
- **FR-012** : Si 32 worktrees ou plus sont déjà enregistrés sous
  `<dossier du parent>/.worktrees/`, le refus est `workspace_limit`.
- **FR-013** : Bridget refuse un dépôt dont la configuration locale contient un
  filtre exécutable (`filter.*.smudge`, `.clean`, `.process`), une inclusion
  (`include`, `includeIf`), un clone partiel (`extensions.partialClone`,
  `remote.*.promisor`) ou `extensions.worktreeConfig`. Refus
  `workspace_repo_unsupported`. Aucune commande Git de Bridget ne récupère un
  objet par le réseau. **Ce contrôle est un garde-fou, pas une garantie** : la
  configuration peut changer entre le contrôle et l'extraction ou le rapport.
  Le risque résiduel est accepté (voir « Arbitrage utilisateur - exécution de Git »).
- **FR-014** : Toute commande Git des parcours154 (avant-vol, création et
  rapport) et des deux lectures du wrapper couvertes par T022 passe par un
  exécuteur unique :
  variables `GIT_*` retirées, configuration système et globale de l'utilisateur
  ignorées, hooks désactivés (`core.hooksPath=/dev/null`),
  `core.fsmonitor=false`, `GIT_ALLOW_PROTOCOL=` vide, `protocol.allow=never`,
  récupération paresseuse coupée, `--no-pager`, signatures non vérifiées
  (`log.showSignature=false`, `--no-show-signature`), programmes externes coupés
  (`--no-textconv`, `--no-ext-diff`), délai maximal, sortie bornée en mémoire et
  en temps, arguments sans interpréteur de commandes, références après
  `--end-of-options`. Les alias ne remplacent pas les commandes internes de Git.
  Les lectures historiques de `communication.rs` et `derive_domain_at` restent
  hors de ce périmètre, comme prévu au plan et dans l’ADR154.
- **FR-015** : L'exécuteur n'accepte que les sous-commandes de la liste
  blanche : `--version`, `rev-parse`, `symbolic-ref`, `check-ref-format`,
  `check-ignore`, `show-ref`, `config --get-regexp`, `worktree add -b`,
  `worktree list`, `rev-list`, `log`, `status`. Aucune commande de suppression,
  de réécriture ou de réseau n'y figure.
- **FR-016** : Le dossier de départ de l'enfant est le worktree plus le
  décalage du dossier du parent dans le dépôt. Les droits, le contexte de
  lancement et le cliché de permissions sont calculés sur ce dossier.
- **FR-017** : Git ne tourne jamais pendant que le verrou global du daemon est
  tenu, ni à l'admission, ni dans le tick. L'admission se fait en deux passes :
  la première, sous verrou, valide et réserve (`task_id`, chemin, branche, base) ;
  Git crée le worktree sans verrou ; la seconde, sous verrou, refait les
  contrôles qui restent vrais après la création : rejeu, pause, droits sur le
  worktree réel, `task_limit`, `retention_saturated`, lignée, délai, plafond
  (voir FR-027), existence du chemin attendu sur le disque. Les contrôles qui
  portent sur l'état Git créé par Bridget (branche existante, `.worktrees/`
  ignoré, configuration du dépôt, liste des worktrees) ne se refont pas à la
  seconde passe : leur résultat de la première passe vaut. Git ne laisse pas
  Bridget pré-créer le dossier final : seul Git crée le dossier du worktree.
- **FR-018** : Bridget n'exécute jamais de suppression de worktree ou de
  branche, y compris après un échec. Si une étape échoue après la création, le
  worktree et la branche restent sur le disque. Le refus nomme leur chemin et
  leur branche, et le journal du daemon les a nommés avant l'appel à Git. Ces
  restes sont inoffensifs (extraction propre de la base).

**Rapport**

- **FR-019** : À la fin de la mission, Bridget lit `head_commit`, `branch_commit`,
  `head_on_branch` (la tête est rattachée à la branche attendue), le nombre de
  commits depuis la base, les 50 derniers (identifiant et sujet, 200 caractères
  au plus, sans caractère de contrôle) et `dirty`. La lecture est en lecture
  seule, avec `--untracked-files=normal` donné en ligne de commande.
- **FR-020** : `bridget_task_status` renvoie un objet `workspace` : `strategy`,
  `state`, `path`, `branch`, `base_commit`, `head_commit`, `branch_commit`,
  `head_on_branch`, `commit_count`, `commits`, `truncated`, `dirty`.
- **FR-021** : Le résultat remis au parent se termine par une ligne écrite par
  Bridget : branche, base, nombre de commits, modifications non commitées, tête
  sur la branche, état du worktree.
- **FR-022** : Le rapport est calculé à chaque passage à `result_available`, donc pour chaque mission et ronde153. Pour un échec ou une annulation, il est calculé après l'arrêt confirmé de l'enfant. Le tick note sous verrou le travail avec `task_id`, `Task.mission` et `Task.round_no()` existants. Git lit hors verrou. Un verrou court range le rapport seulement si la mission et la ronde courantes correspondent encore et si aucun rapport de cette ronde n'a été rangé. Le premier rapport gagne pour cette ronde ; un rapport devenu ancien est rejeté. La publication du résultat attend ce rapport. Une nouvelle ronde recalcule les faits Git. Les résultats déjà publiés restent immuables. L'état passe `active` → `kept` une seule fois, sans retour ; `kept` signifie dossier et branche conservés, et permet de continuer à écrire selon les droits hérités.

**Conservation et purge**

- **FR-023** : `cleanup_done` devient vrai seulement après le rapport terminé et l’arrêt confirmé ou la fin de garde153 prévue par le contrat existant. Un rapport de ronde1 avec garde active ne suffit pas. La purge de la
  session150 ne change pas. Elle efface des lignes de base, jamais des fichiers
  du disque. Un worktree d'une tâche purgée reste sur le disque.
- **FR-024** : Un worktree absent au lancement de l'enfant fait échouer la
  tâche (`cwd_unavailable`) : Bridget ne le recrée pas en silence.

**Collision**

- **FR-025** : Pour une délégation `shared` en développement, le reçu ajoute
  `shared_writers` (nombre d'autres tâches natives actives en développement
  avec le même dossier) seulement si ce nombre est au moins 1. Rien ne bloque.
- **FR-026** : La détection `file_collision` ne change pas. Un enfant en
  worktree et le parent écrivent des chemins différents : aucun événement.

**Concurrence et lectures du wrapper**

**Documentation**

- **FR-027** : Deux appels de même clé ou de même dossier ne créent pas deux
  worktrees ni ne dépassent le plafond de 32. Une table de réservations en
  mémoire, sous verrou, refuse une seconde création pour la même clé
  (`workspace_creation_in_progress`, à rejouer). Le plafond compte l’union
  des chemins enregistrés dans Git, des réservations en cours et des chemins
  de tâches encore présents ou dont la présence est indéterminée. Un chemin
  de tâche absent (`NotFound`) ne compte pas. Les enregistrements Git et les
  réservations restent comptés ; les ensembles ne sont jamais additionnés.
- **FR-028** : Les lectures Git du wrapper (carte de reprise) passent par le même
  exécuteur durci (lecture seule, sortie et temps bornés, filtres exécutables
  non lancés : `status` est omis et la carte le dit si la configuration locale
  contient un filtre).
- **FR-029** : Le guide `docs/delegation-native.md`, la skill Bridget et une ADR
  décrivent la stratégie, les refus, la conservation, la recette de nettoyage
  humain (`git worktree list`, `git branch --list 'bridget/*'`,
  `git worktree remove`) et ce que Bridget ne fait pas.

## Critères de succès

- **SC-001** : Après création, mission et rapport final, le HEAD, l'index et les
  fichiers du dossier du parent sont identiques à l'état de départ, dans 100 %
  des scénarios de test.
- **SC-002** : 0 travail supprimé par Bridget : l'exécuteur n'exécute aucune
  commande de suppression dans toute la suite de tests (liste blanche
  vérifiée), et un worktree sale, un worktree emboîté d'un petit-enfant et un
  worktree sous `status.showUntrackedFiles=no` restent intacts après la fin de
  la tâche.
- **SC-003** : 100 % des refus décidés avant la création laissent 0 worktree,
  0 branche et 0 tâche. Un échec après création laisse des restes nommés dans
  le refus.
- **SC-004** : Onze entrées hostiles sont refusées ou neutralisées sans effet :
  branche avec `..`, branche commençant par `-`, branche avec `@{`, `base_ref`
  en forme d'option, `.worktrees` lien symbolique, dossier hors dépôt, dossier
  hors racine, branche existante, filtre `smudge` hostile (le marqueur n'est
  pas créé), clone partiel avec objet absent (aucune récupération réseau),
  inclusion de configuration.
- **SC-005** : Toutes les sous-commandes Git exécutées pendant la suite de tests
  appartiennent à la liste blanche de FR-015 (test avec exécuteur enregistreur).
- **SC-006** : Un appel sans `workspace` produit la même tâche et le même
  résultat qu'en 150 (tests 148, 149 et 150 verts).
- **SC-007** : Le résultat contient la bonne branche et le bon nombre de commits
  pour 0, 2 et 60 commits (50 listés, `truncated` vrai pour 60), et
  `head_on_branch` faux quand la tête est détachée.
- **SC-008** : Le coût de `worktree add` sur le dépôt Bridget est mesuré et
  consigné avant l'implémentation (Article XXII). Il règle le délai de Git.
- **SC-009** : Pendant un `worktree add` ralenti à 2 s, une lecture de statut
  d'une autre tâche répond en moins de 500 ms.
- **SC-010** : Deux appels simultanés de même clé créent un seul worktree ; 40
  appels simultanés sur un dossier ne dépassent jamais 32 worktrees.
- **SC-011** : Pendant un rapport de fin, aucune commande Git ne tourne sous le
  verrou global.

## Entités et hypothèses

**Entités**

- **Demande d'espace de travail** : objet `workspace` de l'appel.
- **Réservation** : `task_id`, chemin, branche et base décidés sous verrou avant
  Git.
- **Enregistrement d'espace de travail** : état durable dans la tâche (chemin,
  branche, base, état, rapport).
- **Exécuteur Git** : point d'entrée unique et borné des commandes Git.
- **Rapport** : branche, base, tête, commits, indicateurs.

**Hypothèses**

- Git 2.44 ou plus est installé (récupération paresseuse coupée par variable,
  `--end-of-options`). Version constatée : 2.55.0.
- La configuration globale de l'utilisateur est ignorée par Bridget. Un
  `.worktrees/` ignoré seulement par une règle globale est donc refusé.
- Le worktree part d'un commit. Les modifications non commitées du parent n'y
  sont pas. C'est une différence voulue avec `shared`.
- Les fichiers ignorés par Git ne sont pas présents dans un worktree neuf.
- Les sous-modules ne sont pas initialisés dans le worktree.
- Les dépôts avec Git LFS ou un autre filtre exécutable sont refusés.
- Un enfant sous bac à sable Codex `workspace-write` ne peut pas écrire dans le
  dossier Git commun, donc ne peut pas commiter. Son travail reste dans le
  worktree (`dirty: true`). Bridget n'élargit pas ses droits.
- Les restes d'un échec après création sont rares et inoffensifs.
- Le plafond de 32 worktrees vaut deux fois le plafond de 16 enfants actifs.
- Le daemon et le dépôt appartiennent au même utilisateur (cadre de menace de
  l'ADR 011). Le risque résiduel de l'arbitrage est accepté dans ce cadre.
- Les tests suivent la recette `umask 077` (quickstart).

## Arbitrage utilisateur - exécution de Git (décision prise le 2026-10-11)

**Décision.** Option « comme T3, durcie » : le daemon Bridget exécute lui-même
Git, avec ses droits, pour tout parent autorisé par les règles de la 149. Il n'y
a pas de réservation aux parents en accès complet. L'utilisateur a demandé d'être
« au moins aussi propre que T3 » et s'en remet au détail du plan.

**Fait vérifié dans T3** (`origin/main`, `apps/server/src/vcs/GitVcsDriverCore.ts:3418-3419`
et `:3789`, `GitVcsDriver.ts:430`) : le serveur T3 exécute `git worktree add` avec
ses propres droits pour tout appelant. Il coupe seulement `core.fsmonitor`. Il ne
neutralise ni les hooks ni les filtres. Il retire un worktree avec
`-c status.showUntrackedFiles=normal`.

**Ce que la 154 fait de plus que T3** : hooks coupés ; `fsmonitor` coupé ;
`GIT_ALLOW_PROTOCOL=` vide, `protocol.allow=never`, récupération paresseuse
coupée ; `--no-pager` ; signatures et programmes externes coupés ; configuration
système et globale de l'utilisateur ignorées ; refus des dépôts à filtre
exécutable, `include`, clone partiel, `worktreeConfig` et LFS ; exécuteur borné
en temps et en sortie ; liste blanche de sous-commandes ; Git jamais sous le
verrou global ; réservations concurrentes ; aucune suppression de worktree ni de
branche (T3 retire des worktrees).

**Risque résiduel accepté, tel que décidé.** Un parent confiné qui peut modifier la configuration du dépôt entre le contrôle et l'exécution de Git peut faire exécuter du code avec les droits du daemon. Ce risque est accepté explicitement, au même niveau que T3, dans le cadre de menace de l'ADR 011. Il est documenté dans l'ADR 154
(tâche T024). Le cadre de l'ADR 011 est : un agent qui tourne sur cette machine
peut exécuter des commandes et écrire des fichiers. Les garde-fous de Git
réduisent la surface et ne forment pas une frontière de sécurité.

### Alternatives écartées

- **Git confiné** (`sandbox-exec` sur macOS, `bwrap` sur Linux). Faisabilité
  vérifiée sans exécution : `/usr/bin/sandbox-exec` est présent (macOS 26.2,
  fichier du 22 novembre 2025), déprécié mais livré ; `bwrap` est absent de cette
  machine ; Bridget n'a aucun lanceur confiné (`refusals.rs:94-95` ne reconnaît
  que des refus). Raison de l'écart : T3 n'en fait pas, le profil serait plus
  large que les droits d'un parent `workspace-write` (Git écrit dans `.git/`),
  il faudrait une variante par plateforme et une maintenance à chaque version de
  Git (environ 33 tâches contre 28). Reprise possible plus tard sur le même
  exécuteur.
- **Isolation coopérative sans exécution de Git** (l'enfant crée son worktree,
  Bridget lit les fichiers `.git`). Raison de l'écart : l'isolation n'est pas
  imposée ; un enfant `workspace-write` ne peut pas créer le worktree ; le
  rapport vient de fichiers que l'enfant peut écrire ; pas de `dirty` ; les
  objets empaquetés demanderaient une bibliothèque de décompression absente des
  dépendances. Le critère Isolation resterait vers 3.
- **Réserver `worktree` aux parents non confinés** (`workspace_parent_confined`).
  Raison de l'écart : T3 ne le fait pas ; la valeur de la fonction serait réduite
  pour le flux courant ; l'utilisateur a choisi d'accepter le risque au niveau de
  T3.

## Questions pour l'utilisateur

1. **Paires Claude ou GLM vers Claude ou GLM.** La règle de droits actuelle
   exige le même dossier pour l'enfant. Elle bloque donc le worktree pour le
   flux le plus courant (parent Claude, enfant GLM). Défaut : refus explicite
   `workspace_inheritance_unavailable` en 154, et une session 154b plus tard
   pour prouver l'équivalence des fichiers de réglages. Autre choix : inclure
   cette preuve maintenant (environ 4 tâches de plus, zone sensible).
2. **Droits Git de l'enfant sous Codex `workspace-write`.** Cet enfant ne peut
   pas commiter dans un worktree lié. Défaut : ne rien élargir (cohérent avec
   « sans gain de droits » de la 149). Le parent récupère le travail dans le
   worktree conservé. Autre choix : autoriser en écriture le seul dossier
   `.git/worktrees/<id>` de l'enfant.
3. **`.worktrees/` non ignoré.** Défaut : refus `workspace_dir_not_ignored`
   avec le message « ajoutez `.worktrees/` à `.gitignore` ». Autre choix :
   Bridget ajoute le motif à `info/exclude` lui-même (écriture dans le dépôt,
   rejetée par la contre-revue comme surface de lien symbolique).

## Précisions approuvées après contre-revueR1

Le rapport protège la publication par comparaison d’état waiting_for_children,
mission et ronde courantes. Une erreur Git après tentative bornée laisse les
faits inconnus et rend « rapport indisponible », sans attente infinie. Le
résultat courant inclut une seule ligne de fin ; les imitations sont assainies.
Les champs structurés portent les faits. Une nouvelle ronde vide les faits
courants sans changer kept. Aucun arrêt forcé à result_available avec garde153.
Failed/cancelled attendent l’arrêt confirmé avant rapport de tout l’arbre.
Le rejeu admis et l’autorisation du cwd canonique précèdent toute probeGit154.
Les réservations et tâches ne se comptent qu’une fois par chemin au plafond.
H11 ancreGit fiable du registreparent retenue ci-dessous, à vérifier ; H12 borne du lot reste en conception.

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
