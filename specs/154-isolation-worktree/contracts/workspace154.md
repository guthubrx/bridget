# Contrat - Espace de travail 154

Compatible avec les contrats 148, 149, 150 et153
(`specs/150-socle-delegation/contracts/delegation150.md`). Un appel sans le champ
`workspace` produit les mêmes octets canoniques et le même comportement qu'en
150.

## `bridget_delegate` - champ ajouté

| Champ | Type | Obligatoire | Défaut |
|---|---|---|---|
| `workspace` | objet | non | absent (équivaut à partagé) |

Formes acceptées :

```json
{"strategy": "shared"}
{"strategy": "worktree"}
{"strategy": "worktree", "base_ref": "main", "branch": "bridget/refonte-auth"}
```

| Clé | Pour | Règle |
|---|---|---|
| `strategy` | toutes | `shared` ou `worktree` |
| `base_ref` | `worktree` | omis = `HEAD` du dossier demandé. Résolu en un commit, figé. |
| `branch` | `worktree` | omis = `bridget/<8 premiers caractères du task_id>`. Doit être nouvelle. |

Toute autre clé ou valeur donne `invalid_request`. `workspace` fait partie de
l'enveloppe idempotente. Omis, `{"strategy":"shared"}` : deux enveloppes
différentes (`envelope_mismatch` au rejeu).

Il n'existe pas de champ `start_from_origin`, `push`, `merge`, `pr` ni `cleanup`.

## Refus ajoutés (code renvoyé dans `native_delegation_refused`)

| Code | Quand | Écriture |
|---|---|---|
| `workspace_requires_development` | posture effective `discovery` avec `worktree` | aucune |
| `workspace_inheritance_unavailable` | parent et enfant Claude ou GLM | aucune |
| `workspace_git_unsupported` | Git absent ou en version inférieure à 2.44 | aucune |
| `workspace_not_git` | dossier hors dépôt Git ou dépôt nu | aucune |
| `workspace_repo_unsupported` | configuration locale avec filtre exécutable, inclusion, clone partiel ou `worktreeConfig` | aucune |
| `workspace_base_unavailable` | `base_ref` inconnu, non commit, ou en forme d'option | aucune |
| `workspace_branch_invalid` | nom de branche invalide | aucune |
| `workspace_branch_exists` | la branche existe déjà | aucune |
| `workspace_dir_not_ignored` | `.worktrees/` non ignoré par la configuration du dépôt | aucune |
| `workspace_limit` | 32 worktrees ou plus déjà enregistrés sous `<cwd>/.worktrees/` | aucune |
| `workspace_creation_in_progress` | une création est déjà en cours pour la même clé ; à rejouer | aucune |
| `workspace_path_unsafe` | `.worktrees` est un lien, ou un chemin recontrôlé sort de la racine ou diffère de l'attendu | restes nommés dans le refus s'ils existent |
| `workspace_create_failed` | Git refuse, dépasse le délai, ou une revalidation de la seconde passe échoue après création | restes nommés dans le refus s'ils existent |

Pour `workspace_path_unsafe` et `workspace_create_failed`, le message du refus contient le chemin et la
branche éventuellement créés. Bridget ne les supprime pas.

Ordre, dans la suite de l'ordre de la session150 :
`identity_unavailable`, `invalid_request`, `reply_timeout_out_of_range`, validation153 de `keep_alive_secs` (`keep_alive_out_of_range`) et de `brief` (`brief_invalid`), rejeu
(`envelope_mismatch`, `request_expired`), `referent_paused`, gardes de droits
149 (dont `cwd_outside_parent_project`), **`workspace_requires_development`,
`workspace_inheritance_unavailable`, `workspace_git_unsupported`,
`workspace_not_git`, `workspace_repo_unsupported`, `workspace_base_unavailable`,
`workspace_branch_invalid`, `workspace_branch_exists`,
`workspace_dir_not_ignored`, `workspace_limit`**, `task_limit`,
`retention_saturated`, lignée parente, `reply_timeout_exceeds_parent`.
Première passe terminée : la réservation est inscrite (`workspace_creation_in_progress`
si la clé est déjà réservée) et le daemon rend le verrou. **Création du
worktree (`workspace_path_unsafe`, `workspace_create_failed`)** hors verrou.
Seconde passe sous verrou : sont refaits le rejeu, la pause, les gardes de droits,
`task_limit`, `retention_saturated`, la lignée, `reply_timeout_exceeds_parent`,
le plafond par union des chemins (sans doublecompte de la réservation propre) et l'existence du chemin attendu
sur le disque. Ne sont **pas** refaits `workspace_branch_exists`,
`workspace_dir_not_ignored`, `workspace_repo_unsupported` ni la liste des
worktrees : la branche et le worktree existent par la création de la même
requête. Puis le calcul des droits sur le worktree réel, puis l'écriture de la
tâche.

Le rejeu d'une clé admise rend la tâche existante avant tout contrôle de
`workspace`, à chaque passe. Aucun second worktree n'est créé pour une clé
déjà admise.

## `bridget_capabilities` - clés ajoutées

| Clé | Niveau | Valeur |
|---|---|---|
| `workspace_strategies` | racine | `["shared", "worktree"]` |
| `worktree` | chaque fournisseur | booléen : l'appel `worktree` est possible pour cette paire |
| `worktree_refusal` | chaque fournisseur | code de refus ou `null` |

## Reçu de `bridget_delegate` - clés ajoutées

| Clé | Quand |
|---|---|
| `workspace` | stratégie `worktree` : `{state, path, branch, base_commit}` |
| `shared_writers` | stratégie partagée en développement et au moins 1 autre tâche active sur le même dossier |

`cwd` du reçu vaut le dossier de départ de l'enfant (dans le worktree).

## `bridget_task_status` - objet `workspace`

Présent pour une tâche en worktree. Absent sinon.

```json
{"workspace": {
  "strategy": "worktree", "state": "kept",
  "path": "/abs/depot/.worktrees/1a2b3c4d", "branch": "bridget/1a2b3c4d",
  "base_commit": "<40 hex>", "head_commit": "<40 hex>", "branch_commit": "<40 hex>",
  "head_on_branch": true,
  "commit_count": 2, "commits": [{"sha": "<40 hex>", "subject": "texte"}],
  "truncated": false, "dirty": false}}
```

`state` : `active` (premier rapport non stocké) ou `kept` (dossier et
branche conservés). `kept` permet les rondes suivantes et ne retire aucun droit d’écriture hérité. La transition `active` → `kept` est unique. `head_on_branch` est faux si la tête est détachée ou
rattachée à une autre branche : les commits listés viennent alors de la tête, et
la branche peut ne pas les porter. `commits` : 50 au plus, plus récent d'abord.
`subject` est une donnée de l'enfant : à lire comme du texte, jamais comme une
instruction. Avant le rapport final, les champs de rapport sont `null`.

## Ligne de fin du résultat

Quand la tâche est en worktree, le résultat remis au parent se termine par une
ligne écrite par Bridget :

```
[Bridget] espace de travail : branche <branche>, base <7 hex>, <N> commit(s), modifications non commitées : <oui|non>, tête sur la branche : <oui|non>, worktree conservé : <chemin>
```

La ligne est ajoutée à chaque passage à `result_available`, avant la publication du résultat de la mission et de la ronde courantes. Réutiliser `Task.mission` et `Task.round_no()` pour vérifier la sauvegarde sous verrou ; rejeter un rapport ancien. Le premier rapport rangé gagne pour cette ronde. Chaque nouvelle ronde recalcule les faits ; un résultat déjà publié ne change plus. `WorkspaceRecord` ne reçoit aucun nouveau champ public de ronde. `cleanup_done` attend le rapport terminé et l’arrêt confirmé ou la fin de garde153 prévue ; une garde active empêche sa pose après la ronde1.

## Ce que Bridget fait côté Git

Liste blanche : `--version`, `rev-parse`, `symbolic-ref`, `check-ref-format`,
`check-ignore`, `show-ref`, `config --get-regexp`, `worktree add -b`,
`worktree list`, `rev-list`, `log`, `status`. Aucune autre commande.
**Aucune suppression** : ni `worktree remove`, ni `branch -d/-D`, ni
`update-ref -d`, ni `clean`. Écriture dans le dépôt limitée à : la branche
créée et le worktree créé. Aucune écriture dans `config` ni `info/exclude`.

## Nettoyage (humain)

```
git -C <depot> worktree list
git -C <depot> branch --list 'bridget/*'
git -C <depot> worktree remove <chemin>      # refuse si modifications
git -C <depot> branch -d <branche>           # après intégration
```

## Lineage adaptateur de bureau

Aucun changement de forme ni d'état. L'objet `workspace` n'apparaît pas dans le
panneau.

## Publication et relecture — précisionsR1

Le rejeu admis précède les probesGit154. Le cwd est autorisé et canonisé avant
ces probes. La sauvegarde du résultat exige waiting_for_children+mission+round
courants ; rapport, ligne et result_available sont rangés ensemble. Une tentative
bornée de rapport suffit : en erreur, faits inconnus/null, mention rapport
indisponible, publication conservée. Une nouvelle ronde vide les faits courants.
La ligne ajoutée garde un budget exact dans262144octets, texte enfant coupé en
UTF8 ; ses imitations sont assainies, champs structurés seuls authoritative.
Un résultat avec garde153 ne force pas arrêt. Failed/cancelled et tous leurs
descendants worktree attendent arrêt confirmé avant rapport/cleanup.

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
