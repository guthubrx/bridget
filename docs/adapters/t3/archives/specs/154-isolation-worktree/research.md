# Archives techniques de l’adaptateur T3

Document d’origine : [specs/154-isolation-worktree/research.md](../../../../../../specs/154-isolation-worktree/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Dossier de l'enfant.** Avec un parent attesté, le dossier demandé est
  canonicalisé et doit rester sous le dossier du parent
  (`native_delegation.rs:421-428`, `path.starts_with(&root)`). Sans fait
  attesté, `authorize_cwd` applique la racine d'un grant humain ou le projet
  commun (`native_delegation.rs:227-257`). Le dossier devient `Task.cwd`
  (`native_delegation.rs:489`). Le lancement revérifie qu'il est canonique
  (`native_delegation.rs:544-546`).
- **Droits.** `child_policy` calcule la politique de l'enfant. Pour Claude vers
  Claude, elle exige `parent.cwd == cwd` (`native_permissions.rs:229-230`) et
  revérifie le contexte de lancement par chemins de réglages
  (`native_permissions.rs:169-181`). Pour Codex vers Claude, elle capture un
  contexte neuf au dossier de l'enfant (`native_permissions.rs:224`). Pour
  Codex vers Codex et Claude vers Codex, le dossier n'intervient pas.
- **Aucun code Git d'espace de travail.** Le recensement `worktree` du dépôt ne
  montre que la résolution du projet commun (`communication.rs:64-127`), la
  carte de reprise (`wrapper.rs:329-400`) et des champs de contrat T3. Bridget
  ne crée ni ne supprime aucun worktree.
- **Carte de reprise.** Le wrapper lit déjà la branche, la tête et les
  fichiers modifiés du dossier de l'enfant (`wrapper.rs:623-651`) et alerte si
  ce dossier est le checkout principal d'un dépôt à worktrees liés
  (`wrapper.rs:552-615`). Ces lectures lancent `git` sans couper
  `core.fsmonitor` ni les verrous optionnels (`wrapper.rs:572-590`,
  `:623-641`). L'enfant d'un worktree reçoit ses faits Git sans code nouveau,
  mais ces lectures doivent être durcies (R8).
- **Collision.** `FileCollision` compare des chemins absolus normalisés sur
  30 secondes (`observation.rs:441-460`). Deux chemins différents ne se
  rencontrent jamais.
- **Nettoyage de tâche.** `cancel_tree` arrête les enfants puis pose
  `cleanup_done` (`native_delegation.rs:1380-1478`). Le tick pousse aussi les
  nettoyages après remise du résultat (`native_delegation.rs:1071-1080`,
  `:1283-1285`).
- **Verrou global.** `handle_with_fact` prend le verrou de l'état du daemon et
  appelle `handle_locked_with_project` pendant toute l'admission
  (`native_delegation.rs:301-312`). Tout Git exécuté là bloque les autres appels.
- **Git hors verrou, déjà pratiqué.** `resolve_communication_project` s'exécute
  avant le verrou et retire `GIT_DIR`, `GIT_WORK_TREE`, `GIT_COMMON_DIR`
  (`native_delegation.rs:283-296`, `communication.rs:85-100`).
- **Processus du wrapper.** Le wrapper est un processus séparé du daemon. Il lance
  déjà `git` directement, sans confinement (`wrapper.rs:623-641`). Le bac à
  sable du fournisseur est une politique appliquée par le fournisseur, pas par
  le wrapper.

<a id="extrait-2"></a>

## Extrait 2

- **Décision** : objet optionnel `workspace {strategy, base_ref?, branch?}`,
  absent par défaut. Même patron que `reply_timeout` de la session150 :
  `skip_serializing_if = "Option::is_none"`, donc mêmes octets sans le champ.
- **Raison** : un seul appel reste un seul appel. Aucun nouvel outil MCP.
- **Rejeté** : un outil `bridget_workspace` séparé ; une option de CLI seule.
- **T3** : `root` devient `shared`. `existing_worktree` est déjà couvert par
  `cwd`. `startFromOrigin` est exclu (réseau).

<a id="extrait-3"></a>

## Extrait 3

**Décision de l'utilisateur (2026-10-11)** : même modèle que T3, durci. Le
daemon exécute Git pour tout parent autorisé. Les durcissements de R5 sont tous
conservés. Le risque résiduel est accepté et écrit tel quel dans la spec et dans
l'ADR 154. Le cadre est celui de l'ADR 011 : un agent qui tourne sur cette
machine peut déjà exécuter des commandes et écrire des fichiers (ADR 011,
« Contexte »). L'hypothèse d'un même compte Unix pour le daemon et le dépôt est
celle de la spec.

<a id="extrait-4"></a>

## Extrait 4

**Alternatives écartées** : Git confiné (`sandbox-exec` présent, déprécié ;
`bwrap` absent ici ; profil plus large que `workspace-write` ; 33 tâches) ;
isolation coopérative par lecture des fichiers `.git` (isolation non imposée,
enfants confinés exclus, rapport non fiable) ; réservation aux parents non
confinés (valeur réduite, T3 ne le fait pas).

<a id="extrait-5"></a>

## Extrait 5

- Affichage de la branche dans Lineage T3 : contrat 149 fermé.
- Retrait automatique d'un worktree : décision humaine.
- Mesure d'un dépôt volumineux : T002 mesure le dépôt Bridget seulement.
- Fenêtre de course sur un lien `.worktrees` remplacé entre le contrôle et Git :
  recontrôle après création, résiduel accepté (R3).
