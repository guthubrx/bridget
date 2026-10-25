# Contre-revue adverse - Session154

## Ronde 1 (plan)

- **Date** : 2026-10-11
- **Agent** : Codex, modèle GPT-6.1-Sol, effort élevé, lecture seule
- **Question** : sûreté de la création et du retrait d'un worktree par le daemon,
  droits hérités, perte de commits, verrou global, lectures Git du wrapper.
- **Verdict reçu** : BLOCKED (9 points). La revue juge justifiés les refus précoces
  (posture `discovery` sans mutation ; Claude ou GLM sur le même dossier,
  `native_permissions.rs:229-230`) et le double calcul de `child_policy`.
- **Piste de simplification posée par le coordinateur** : (a) créer le worktree
  dans le wrapper de l'enfant ; (b) Bridget ne supprime jamais un worktree ni une
  branche, il rapporte et laisse le nettoyage à l'humain.

### Évaluation des pistes (Article XIX)

| Piste | Verdict | Preuve et raison |
|---|---|---|
| (a) création dans le wrapper | rejetée | Le wrapper est un processus à part, sans confinement : il lance déjà `git` directement (`wrapper.rs:623-641`). Le bac à sable est une politique appliquée par le fournisseur. Git y aurait donc les droits de l'utilisateur, comme dans le daemon : aucun gain de droits. Le chemin n'existe pas à l'admission, or le contexte de droits Claude dépend des fichiers du dossier (`native_permissions.rs:103-108`) et le lancement le revérifie (`native_delegation.rs:551-554`). Une erreur de création deviendrait une tâche `failed` asynchrone au lieu d'un refus sans reste, et demanderait un nouveau message wrapper vers daemon. |
| (b) Bridget ne supprime jamais | retenue | Retire `worktree remove`, le retour arrière, les états `removed` et `retained`, l'écriture dans `info/exclude`. Les objections 2, 3, 4 et 7 tombent ou se réduisent à un champ de rapport. Coût : accumulation sur le disque, bornée par un plafond de 32 et une recette de nettoyage humain. |

### Objections

| # | Objection | Vérifiée comment | Retenue | Raison |
|---|---|---|---|---|
| 1 | CRITIQUE : filtres smudge ou process (aussi par `include`) exécutés avec les droits du daemon ; clone partiel avec récupération réseau implicite | raisonnement sur `worktree add` (extraction avec filtres de la configuration) ; le plan ne lisait pas la configuration ; l'exécuteur n'ignorait ni la configuration globale ni les inclusions | oui | configurations système et globale ignorées ; refus `workspace_repo_unsupported` pour filtres, `include`, `includeIf`, clone partiel, `worktreeConfig` ; `GIT_NO_LAZY_FETCH`, `protocol.allow=never` ; tests filtre hostile (marqueur absent) et objet absent (T005, T007, T008) |
| 2 | CRITIQUE : retirer le worktree d'un enfant efface le worktree emboîté d'un petit-enfant, masqué par `.worktrees/` ignoré | cohérent avec le plan : le petit-enfant crée `<cwd enfant>/.worktrees/x`, motif ignoré, donc invisible au contrôle de propreté ; source de Git non relue localement (aucune copie disponible) | oui | sans objet par (b) : aucune suppression. Test de non-régression `native154_conservation_*` : le worktree emboîté reste intact (T017) |
| 3 | ÉLEVÉE : sans `--force`, Git supprime les fichiers non suivis si `status.showUntrackedFiles=no` | cohérent avec `research.md` d'origine (R6) qui s'appuyait sur « sans force = propre » | oui | sans objet par (b). Les lectures de Bridget donnent `--untracked-files=normal` en ligne de commande, qui prime sur la configuration ; test `native154_rapport_*` avec `showUntrackedFiles=no` (T014) |
| 4 | ÉLEVÉE : garder la branche ne protège pas le commit final (tête détachée, autre branche) | cohérent : le rapport lisait `HEAD` sans vérifier la branche | oui | le rapport ajoute `branch_commit` et `head_on_branch`, repris dans la ligne de fin ; test tête détachée (T014, T015) ; la suppression n'existe plus, donc aucun commit n'est perdu par Bridget |
| 5 | ÉLEVÉE : course sur les liens (`.worktrees` remplacé après `symlink_metadata`, `info/exclude` lié, dossier final non couvert) | lu le plan : Git reçoit un chemin ; l'écriture de `info/exclude` suivrait un lien | partiel | `info/exclude` : écriture supprimée, refus `workspace_dir_not_ignored` (surface retirée). Dossier final : recontrôle du chemin canonique du worktree et du dossier de départ sous la racine (T009). Fenêtre sur `.worktrees` : un descripteur ouvert sans suivi de lien ne peut pas être donné à Git sur macOS ; résiduel accepté, attaquant = le parent délégant, Git refuse un dossier non vide ; noté dans l'ADR |
| 6 | ÉLEVÉE : verrou global tenu jusqu'à 20 s pendant la création | lu `native_delegation.rs:301-312` : `handle_locked_with_project` s'exécute sous verrou | oui | admission en deux passes avec une seule fonction : réserver sous verrou, Git sans verrou, tout revalider sous verrou, puis calculer les droits sur le worktree réel ; test de verrou libre avec un `git` ralenti (SC-009, T007, T009) |
| 7 | ÉLEVÉE : retour arrière incomplet, branche orpheline après délai, échec ou crash | cohérent : Git crée la branche avant d'achever l'extraction | partiel | retour arrière supprimé (aucune suppression). Garde avant l'appel : le journal nomme chemin et branche ; refus qui nomme les restes ; restes inoffensifs (extrait propre, branche sur la base). Pas de trace durable en base : le journal suffit pour un cas rare et sans danger |
| 8 | ÉLEVÉE : le wrapper lance `git status` sans couper `fsmonitor` ni borner | lu `wrapper.rs:572-590` et `:623-641` : aucun `-c core.fsmonitor=false`, aucun `GIT_OPTIONAL_LOCKS` | partiel | `fsmonitor` et verrous optionnels coupés dans les deux lectures (T022, 2 lignes). Bornage de la sortie non traité : la carte tronque déjà l'affichage à 40 entrées, hors périmètre |
| 9 | MOYENNE : Git 2.24 n'interprète pas sûrement `core.fsmonitor=false` ; worktrees `retained` non bornés et sans fiche après purge | version minimale choisie sans test ; états `retained` sans plafond | oui | version minimale 2.44 contrôlée avant toute commande (`workspace_git_unsupported`) ; plafond de 32 worktrees (`workspace_limit`) ; recette de nettoyage humain dans le guide et dans le contrat (`git worktree list`, `git branch --list 'bridget/*'`) à la place d'une fiche |

### Décision de conception retenue

Création par le daemon, en deux passes, hors verrou. Aucune suppression de
worktree ni de branche par Bridget. Rapport en lecture seule, durci. Dépôts à
configuration exécutable refusés. Les artefacts corrigés sont `spec.md`,
`plan.md`, `research.md`, `data-model.md`, `contracts/workspace154.md`,
`tasks.md`, `reuse-audit.md`, `quickstart.md`, `checklists/requirements.md` et
`analyze-report.md`.

### Points non vérifiés

- Le code source de Git (`builtin/worktree.c`) n'est pas disponible localement :
  les objections 2 et 3 sont acceptées sur le raisonnement du réviseur et rendues
  sans objet par la décision de ne plus supprimer.
- Le comportement de Git 2.44 à 2.54 n'est pas testé ; seule la 2.55.0 est
  installée.

Statut : objections bloquantes fermées par la conception. Pas de ronde 2 avant
l'implémentation ; les tests rouges de T005, T007, T014, T017 et T022 portent
chaque objection.

## Ronde 2 (plan révisé)

- **Date** : 2026-10-11
- **Agent** : Codex GPT-6.1-Sol, effort élevé, lecture seule
- **Verdict reçu** : BLOCKED. Objections 2, 3 et 4 de la ronde 1 sans objet ;
  7 traitée ; 1, 5, 6 et 8 restent ouvertes ; 9 partielle. Sept points nouveaux.
- **Diagnostic du coordinateur** : la racine commune est que le daemon exécute
  Git avec ses droits sur un dépôt dont la configuration est modifiable par un
  parent confiné. Les correctifs par configuration ne convergent pas.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/adversarial-review-codex.md#extrait-1).

**Notes du réviseur intégrées** : `GIT_ALLOW_PROTOCOL=` vide et `--no-pager`
imposés (FR-014, T005). Les alias ne remplacent pas les commandes internes de
Git : aucune action, notée dans FR-014.

### Faisabilité de l'Option 1 (vérifiée sans exécuter)

`/usr/bin/sandbox-exec` existe (macOS 26.2, fichier du 22 novembre 2025). Il est
déprécié mais présent. `bwrap` est absent de cette machine. Bridget n'a aucun
lanceur confiné aujourd'hui, seulement la reconnaissance de leurs refus
(`refusals.rs:94-95`).

### Décision

Les points 4, 5, 6, 7 et les notes sont intégrés. Les points 1, 2 et 3 dépendent
de la racine (exécution de Git avec les droits du daemon) et ont été soumis à
l'utilisateur sous trois options.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/adversarial-review-codex.md#extrait-2).
