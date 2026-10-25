# Archives techniques de l’adaptateur T3

Document d’origine : [specs/154-isolation-worktree/spec.md](../../../../../../specs/154-isolation-worktree/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-11. Accord utilisateur : feuille de route « Bridget cinq étoiles
en communication et délégation » (ADR 150b, ligne 154). Critère visé :
Isolation, de 1 à 5 (`specs/150-socle-delegation/research-t3-bridget.md:23`).

<a id="extrait-2"></a>

## Extrait 2

T3 Code propose trois stratégies pour un fil lancé : `root`, `existing_worktree`
et `worktree{baseRef, branch, startFromOrigin}`
(`packages/contracts/src/orchestrationV2.ts`, `OrchestrationV2ThreadLaunchWorkspaceStrategy`).
Mais un enfant lancé par `delegate_task` partage le worktree du parent.

<a id="extrait-3"></a>

## Extrait 3

**Correspondance avec T3.** `root` correspond à `shared`. `existing_worktree`
correspond déjà à `cwd` pointant vers un worktree existant sous la racine du
parent (aucun code nouveau). `startFromOrigin` est exclu : il demande un accès
réseau.

<a id="extrait-4"></a>

## Extrait 4

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

<a id="extrait-5"></a>

## Extrait 5

**Décision.** Option « exécution directe de Git, durcie » : le daemon Bridget exécute lui-même
Git, avec ses droits, pour tout parent autorisé par les règles de la 149. Il n'y
a pas de réservation aux parents en accès complet. L'utilisateur a demandé d'être
« au moins aussi propre que T3 » et s'en remet au détail du plan.

<a id="extrait-6"></a>

## Extrait 6

**Fait vérifié dans T3** (`origin/main`, `apps/server/src/vcs/GitVcsDriverCore.ts:3418-3419`
et `:3789`, `GitVcsDriver.ts:430`) : le serveur T3 exécute `git worktree add` avec
ses propres droits pour tout appelant. Il coupe seulement `core.fsmonitor`. Il ne
neutralise ni les hooks ni les filtres. Il retire un worktree avec
`-c status.showUntrackedFiles=normal`.

<a id="extrait-7"></a>

## Extrait 7

**Ce que la 154 fait de plus que T3** : hooks coupés ; `fsmonitor` coupé ;
`GIT_ALLOW_PROTOCOL=` vide, `protocol.allow=never`, récupération paresseuse
coupée ; `--no-pager` ; signatures et programmes externes coupés ; configuration
système et globale de l'utilisateur ignorées ; refus des dépôts à filtre
exécutable, `include`, clone partiel, `worktreeConfig` et LFS ; exécuteur borné
en temps et en sortie ; liste blanche de sous-commandes ; Git jamais sous le
verrou global ; réservations concurrentes ; aucune suppression de worktree ni de
branche (T3 retire des worktrees).

<a id="extrait-8"></a>

## Extrait 8

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
