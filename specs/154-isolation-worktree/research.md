# Recherche - Session154 isolation optionnelle dans un worktree

Date : 2026-10-11 (révisé après contre-revue Codex, voir
`adversarial-review-codex.md`). Source : lecture du checkout principal
`/Users/moi/Nextcloud/10.Scripts/64.bridget` (base `1f2c661d`, sans les
changements de la session150). Les numéros de ligne sont ceux de cette base. La
session150 est supposée livrée ; ses apports sont cités par sa spec.

## R1 - Ce que fait l'existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/research.md#extrait-1).

## R2 - Décision : un paramètre `workspace` sur `bridget_delegate`

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/research.md#extrait-2).

## R3 - Décision : worktree créé par le daemon, hors verrou, en deux passes

Deux simplifications ont été évaluées après la contre-revue.

**Piste (a) : créer le worktree dans le wrapper de l'enfant. REJETÉE.**

- Le wrapper n'est pas dans le bac à sable du fournisseur (R1). Git y tournerait
  avec les droits de l'utilisateur, comme dans le daemon. Le gain de droits
  attendu n'existe pas.
- Le chemin du worktree n'existerait pas à l'admission. Le contexte de droits
  Claude lit les réglages en remontant jusqu'au premier `.git`
  (`native_permissions.rs:103-108`). Dans un worktree lié, `.git` est un
  fichier. Un contexte capturé avant la création serait celui du parent. Le
  lancement le revérifierait sur le worktree réel et échouerait
  (`native_delegation.rs:551-554`).
- Une erreur de création arriverait après l'admission, donc comme une tâche
  `failed` et non comme un refus synchrone sans reste. Il faudrait un nouveau
  message wrapper vers daemon et une gestion de panne dans un second processus.
- Le verrou global se règle plus simplement dans le daemon (ci-dessous).

**Décision : deux passes dans le daemon.**

1. Première passe, sous verrou : tous les contrôles d'autorisation, de limites
   et de dépôt. Elle produit une réservation (`task_id`, chemin, branche, base)
   et rend le verrou.
2. Git crée le worktree sans verrou. Le journal du daemon nomme le chemin et la
   branche avant l'appel.
3. Seconde passe, sous verrou : le même code refait tous les contrôles (rejeu,
   pause, limites, droits), calcule `child_policy` et le cliché sur le
   worktree réel, puis écrit la tâche.

Une seule fonction d'admission sert aux deux passes. La seconde reçoit la
réservation et le worktree créé. Les contrôles identiques valent revalidation
sans code en plus.

- **Pourquoi sous le dossier du parent** : la règle `starts_with(root)` reste
  inchangée. Un worktree à côté du dépôt sortirait de la racine. Le dossier
  `.git` ne sert jamais de racine de droits.
- **Rejeté** : worktree dans `.git/` ; worktree hors racine avec un grant (ajoute
  un droit) ; création sous verrou (blocage du daemon jusqu'à 20 s, objection 6) ;
  création dans le tick (contexte de droits capturé sur un chemin absent).
- **Liens symboliques.** Un dossier `.worktrees` est contrôlé par
  `symlink_metadata` avant Git. Git exige un chemin et ne peut pas recevoir un
  descripteur ouvert sans suivi de lien sur macOS. Une fenêtre de course reste
  donc possible pour qui écrit déjà dans le dossier du parent. Bridget recontrôle
  après la création que le chemin canonique du worktree et celui du dossier de
  départ sont les chemins attendus, sous la racine. Écart accepté : l'attaquant
  est le parent délégant, Git refuse un dossier non vide existant, l'effet est un
  extrait propre d'un commit.

## R4 - Décision : droits exactement hérités, deux refus nets

- **Posture.** Un worktree sert à écrire. Avec une posture effective `discovery`
  (lecture seule) il ne sert à rien. Refus `workspace_requires_development`.
  `authorize_cwd` (voie 148 sans preuve) ne donne que `discovery` : la
  stratégie `worktree` ne passe que par la voie du parent attesté.
- **Claude vers Claude.** La règle actuelle (`native_permissions.rs:229-230`)
  refuse un dossier différent. De plus `.claude/settings.local.json` n'est pas
  suivi par Git : il manquerait dans le worktree et ferait perdre des règles
  `deny`. Refus explicite et précoce `workspace_inheritance_unavailable`. Question
  1 de la spec.
- **Autres paires.** Codex vers Codex conserve le schéma fermé des droitsCodex.
  Codex vers Claude en développement exige accèscomplet+never (:218-225)
  et recalcule le contexte du dossierréel ; les fichiersClaude ignorés du parent
  ne sont pas des droitsCodex. Claude avec règles versCodex est refusé
  (:253-268). Aucun refus global de toute cibleClaude n’est ajouté. Le sandbox Codex
  `workspace-write` garde ses racines inscriptibles vides
  (`native_delegation.rs:118-122`) : aucun droit n'est ajouté.
- **Catalogue.** Le catalogue calcule déjà `inherit` et `development` par
  fournisseur (`native_delegation.rs:714-725`). Il ajoute `worktree` et
  `worktree_refusal` avec la même logique de paire. Le test qui fige les clés
  (`native_delegation_permissions149_tests.rs:195`) est mis à jour.

## R5 - Décision : exécuteur Git unique, durci, dépôts à configuration exécutable refusés

- **Constat** : trois endroits ont besoin de Git (avant-vol, création, rapport).
  Un module unique est justifié par ces usages et par la sécurité.
- **Menaces et parades** :
  - injection d'option par `base_ref` ou `branch` : validation syntaxique, puis
    références après `--end-of-options` ;
  - **filtres smudge ou process, y compris par `include`** (objection 1) : `git
    worktree add` exécute une extraction qui applique les filtres de la
    configuration. L'exécuteur ignore les configurations système et globale
    (`GIT_CONFIG_NOSYSTEM=1`, `GIT_CONFIG_GLOBAL=/dev/null`). Avant toute
    création, `git config --get-regexp` sur la configuration locale refuse le
    dépôt s'il contient `filter.*.smudge`, `.clean`, `.process`, `include`,
    `includeIf`, `extensions.partialClone`, `remote.*.promisor` ou
    `extensions.worktreeConfig` (`workspace_repo_unsupported`). Un dépôt Git LFS
    est donc refusé : limite écrite dans la spec ;
  - **récupération réseau implicite d'un clone partiel** : refus du clone
    partiel ci-dessus, et défense en profondeur par `GIT_NO_LAZY_FETCH=1`,
    `-c protocol.allow=never`, `GIT_ALLOW_PROTOCOL=` vide. Un objet absent fait
    échouer la création (`workspace_create_failed`), sans réseau ;
  - hooks du dépôt : `-c core.hooksPath=/dev/null` ;
  - fsmonitor et pager : `-c core.fsmonitor=false`, `GIT_PAGER=cat`,
    `GIT_TERMINAL_PROMPT=0`. Cette valeur booléenne n'est fiable que sur un Git
    récent, d'où la version minimale 2.44 (objection 9), contrôlée par
    `git --version` avant toute commande (`workspace_git_unsupported`) ;
  - variables `GIT_*` héritées : retirées, comme dans `communication.rs:92-95` ;
  - blocage du daemon : Git ne tourne jamais sous verrou (R3) ; délai de 20 s
    pour la création, 10 s ailleurs, kill du processus Git lancé par Bridget,
    sortie bornée à 1 Mio ;
  - dérive vers un client Git : liste blanche de sous-commandes (FR-015) testée
    par un exécuteur enregistreur.
- **Avant-vol hors verrou.** Les commandes de lecture (`--version`, `rev-parse`,
  `config --get-regexp`, `check-ref-format`, `show-ref`, `check-ignore`,
  `worktree list`) tournent horsverrou seulement après lookup de rejeu et
  autorisation du cwdcanonique dans le premier verrou. La résolution héritée
  `resolve_communication_project` reste horspérimètre154. Elles n'exécutent ni hook ni fsmonitor.
  `status` et `worktree add` ne tournent jamais avant l'autorisation.
- **Rejeté** : la bibliothèque `git2` (nouvelle dépendance lourde, Article XIX) ;
  neutraliser les filtres un par un par `-c` (liste incomplète si une inclusion
  les ajoute, refus plus sûr).

## R5b - Ronde 2 et arbitrage : exécution de Git exécution directe de Git, durcie

La contre-revue de ronde 2 montre que les parades de R5 ne forment pas une
frontière : la configuration peut changer entre le contrôle et l'extraction ou le
rapport, et `log.showSignature` ou le filtre `clean` d'un `status` lancent un
programme choisi par le dépôt. Un parent confiné pourrait ainsi obtenir
l'exécution de code avec les droits du daemon.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/research.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/research.md#extrait-4).

Les corrections de ronde 2 valables dans tous les cas sont intégrées : rapport
hors verrou (R7), table de réservations, revalidation sans auto-contradiction,
lectures du wrapper bornées et durcies, `GIT_ALLOW_PROTOCOL=` vide, `--no-pager`,
signatures et programmes externes coupés. Le dossier final n'est jamais
pré-créé par Bridget.

## R6 - Décision : Bridget ne supprime jamais, l'humain nettoie

La contre-revue a montré quatre risques de perte dans le retrait automatique :

- le retrait d'un worktree d'enfant peut effacer le worktree emboîté d'un
  petit-enfant, car `.worktrees/` est ignoré et le contrôle de propreté ne le
  voit pas (objection 2) ;
- `status.showUntrackedFiles=no` rend « propre » un dossier avec des fichiers non
  suivis, que `worktree remove` sans force efface alors (objection 3) ;
- garder la branche ne protège pas le commit final si la tête est détachée ou
  sur une autre branche (objection 4) ;
- un retour arrière peut laisser une branche orpheline (objection 7).

Un correctif par risque est possible (conserver un worktree qui en contient un
autre, forcer `status.showUntrackedFiles=all`, tester la tête). Chacun ajoute du
code de décision de suppression dans un produit qui ne doit pas être un
environnement de développement. **Décision : supprimer la suppression.**

- **Ce qui est retiré** : `git worktree remove`, `update-ref -d`, le retour
  arrière, les états `removed` et `retained`, le motif de rétention, les
  tests de retrait. Quatre objections tombent par construction.
- **Ce que Bridget fait à la fin** : une lecture seule, durcie, qui produit le
  rapport (R7) et passe l'état à `kept`. Puis `cleanup_done` seulement après arrêt confirmé/fin de garde153 et rapport terminé.
- **Ce que cela coûte** : les worktrees s'accumulent sur le disque. Parades :
  plafond de 32 worktrees enregistrés sous `<cwd>/.worktrees/` (refus
  `workspace_limit`), chemin et branche dans le résultat remis au parent,
  convention de nommage (`.worktrees/<8 hex>`, branche `bridget/…`) et recette
  de nettoyage humain dans le guide. Il n'existe pas de fiche après la purge de
  la tâche : la recette `git worktree list` et `git branch --list 'bridget/*'`
  la remplace (objection 9).
- **Échec après création** : les restes (worktree, branche) sont laissés et
  nommés dans le refus. Le journal du daemon les nomme avant l'appel à Git. Ils
  sont inoffensifs : un extrait propre d'un commit et une branche posée sur ce
  commit.
- **Purge 150** : `cleanup_done` suit le rapport terminé et l’arrêt confirmé/fin de garde153. La purge efface des
  lignes de base et ne touche pas le disque. Aucun changement du code de purge.
- **Rejeté** : retrait automatique (risques ci-dessus) ; retrait différé à la
  purge ; commande de liste dans Bridget (la commande Git suffit) ;
  `--force` ; commit automatique.
- **Alerte de la règle 6.** Puisque Bridget ne retire plus de worktree, il ne
  désarme plus l'alerte du checkout principal (`wrapper.rs:560-566`).

## R7 - Décision : rapport lu dans Git, ligne de fin écrite par Bridget

- **Rapport.** Bridget lit : `rev-parse HEAD`, `symbolic-ref -q HEAD` (tête
  rattachée ?), `rev-parse <branche>` (`branch_commit`), `rev-list --count
  base..HEAD`, `log --max-count=50`, et `status --porcelain=v1
  --untracked-files=normal` borné. `head_on_branch` vaut vrai si la tête est
  rattachée à la branche attendue (objection 4 : le rapport signale une tête
  détachée ou changée de branche). Le sujet de commit vient de l'enfant : il est
  nettoyé (`bridget_core::is_disallowed_control`, 200 caractères) et reste une
  donnée.
- **Où.** Au passage à `result_available` (`native_delegation.rs:1111`) pour un
  résultat. Après l'arrêt confirmé de l'enfant pour un échec ou une annulation.
- **Comment le parent le reçoit.** Une ligne de fin, écrite par Bridget. L'objet
  complet est dans `bridget_task_status`. Le contrat151 déjà livré reste conservé.
- **Taille.** 50 commits au plus, 200 caractères par sujet : environ 12 Kio au
  maximum, bien sous la limite de résultat de 262 144 octets
  (`native_delegation.rs:1105`).

## R8 - Décision : lectures du wrapper par l'exécuteur unique

Les lectures de `managed_resume_worktree` et `is_protected_principal_checkout`
passent par `workspace_git` (T022). Le temps et les sorties sont bornés avant
la troncature de la carte. `status` est omis si un filtre exécutable est présent ;
la carte indique son indisponibilité. Le durcissement limité à deux lignes est
supersédé. Aucun changement de `communication.rs` ni de la racine/projet commun.

## R9 - Décision : collision, prévenir sans bloquer

- **Constat.** `file_collision` ne voit que les écritures déjà faites. Bloquer
  deux enfants sur un même dossier changerait le comportement 149.
- **Décision.** Le worktree supprime la collision par construction : chemins
  différents. Pour `shared` en développement, le reçu ajoute `shared_writers`
  s'il y a au moins un autre enfant actif en développement sur le même dossier.
  Parcours des tâches actives (au plus 128 par la session150) : O(128).
- **Rejeté** : refus de la deuxième délégation partagée ; modification de
  `observation.rs`.

## R10 - Effets croisés avec la session150

| Point | Effet de la 150 | Effet de la 154 |
|---|---|---|
| Purge par arbre | exige `cleanup_done` et accusé | `cleanup_done` suit le rapport final ; purge inchangée, disque intact |
| Pierre tombale | clé purgée → `request_expired` | inchangé ; aucun worktree recréé |
| Pause du référent | `referent_paused` à l'admission | refus avant toute création de worktree |
| Délai `reply_timeout` | champ optionnel de `Delegate` | `workspace` s'ajoute après, mêmes règles d'octets |
| Catalogue | clés `mission_reply_*` | clés `workspace_strategies`, `worktree`, `worktree_refusal` |
| Annulation | inchangée | rapport final lu après l'arrêt confirmé, aucune suppression |

## R11 - Hors périmètre constaté

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/research.md#extrait-5).

## R12 - Base153 livrée et rapports par ronde

Base documentaire : `89f7a4163c2fb27229587f50a46414295ba3674d`, avec153
`7c848850ba6067cf1d0d94b8edf0dd11896a1b24` livrée et active.
Preuves relues : `delegation.rs:82` (mission), `:139` (`round_no`),
`:161` (`continuable`) ; `native_delegation.rs:756-769` (keep_alive/brief avant
rejeu), `:728` (catalogue153), `:2229-2232` (garde avant nettoyage),
`:2500-2554` (arrêt confirmé puis cleanup).

Chaque ronde recalcule ses faits Git avant publication. Comparer mission et
ronde existantes avant sauvegarde ; le premier rapport gagne pour la ronde
courante, jamais pour toutes les rondes. Aucun helper, table ou état de ronde
public nouveau. `kept` conserve le dossier/branche ; les écritures restent
autorisées par les droits hérités. Les historiques publiés ne changent pas.

Validation externe primaire le2026-10-11 : [git-worktree](https://git-scm.com/docs/git-worktree)
confirme que les worktrees liés ont leurs métadonnées et partagent le dépôt ;
[git](https://git-scm.com/docs/git) documente les variables d’environnement et
les options de configuration. Ces lectures ne prouvent aucun test154.

## R13 — Contre-revueR1 et complément technique du principal

Quinzehypothèses arbitrées dans adversarial-review-claude.md. PublicationCAS
waiting_for_children+mission+round, rapporttentativebornée et indisponible en
erreur ; réserver budget exactdu suffixe262144octets et couperUTF8.

Task.workspace_report_mission privé default/skipNone est autorisé :
ND2193-2228 peut passer result_availableàfailed après refus de livraison.
CASsuccès seul ne couvre pas échec/cleanup/reprise. begin_round remet marqueur
et faits àvide. Aucun champ public de ronde ni migration/table.

L’ancrevient du registreGitparent horsenfant par scanborné et correspondance
toplevel/path. Aucun enfant.git lu, aucun champanchor nouveau. Ancre absente
→ faitsNone et mentionindisponible. H12bornelot reste enétude.
Whitelistformes/options, environnement/configurationcommuns ; no-textconv et
no-ext-diff seulement sur logcompatible, pas statusGit2.55(RC129constatéroot).
Annulation : chaque membreworktree de l’arbre, rapporthorsverrou puis cleanupCAS
après arrêtconfirmé. Résultatavecgarde153 neforcepasarrêt.

RéservationsArcMutexséparé ; plafondunionpaths sansdoubleréservationpropre.
Neutraliser lepréfixeenfantBridget→Enfant ; unique suffixeautoritaire et faits
structurés. Ces décisions restent à éprouver par les tests prévus ; aucun test
supplémentaire, aucun GREEN inventé.

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
