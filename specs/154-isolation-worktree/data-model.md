# Modèle de données - Session154

Aucune table nouvelle. Aucune migration. L'état vit dans la charge JSON de la
tâche (`Task`, `delegation.rs`), comme `reply_timeout` en session150.

## Demande (`bridget-transport/src/protocol.rs`, variante `Delegate`)

| Champ | Type | Défaut si absent | Rôle |
|---|---|---|---|
| `workspace` | `Option<WorkspaceRequest>` | absent = `shared` implicite | Stratégie d'espace de travail |

`WorkspaceRequest` est une énumération étiquetée par `strategy`, avec
`deny_unknown_fields` :

| `strategy` | Champs | Notes |
|---|---|---|
| `shared` | aucun | comportement 150 |
| `worktree` | `base_ref: Option<String>`, `branch: Option<String>` | `base_ref` défaut `HEAD` ; `branch` défaut `bridget/<task8>` |

Le champ porte `skip_serializing_if = "Option::is_none"`. Il est placé après
`reply_timeout` (session150), sans modifier `keep_alive_secs` et `brief` de153. Il entre dans l'enveloppe idempotente.

Validation syntaxique (avant tout Git) :

- `branch` : 1 à 100 caractères, `[A-Za-z0-9._/-]`, premier caractère
  alphanumérique, sans `..`, `//`, `@{`, fin `/` ou `.lock`. Puis
  `git check-ref-format --branch` confirme.
- `base_ref` : 1 à 200 caractères, sans espace ni caractère de contrôle, premier
  caractère différent de `-`.

## Réservation (mémoire, entre les deux passes d'admission)

| Champ | Rôle |
|---|---|
| `task_id` | décidé à la première passe, réutilisé à la seconde |
| `toplevel`, `path`, `offset` | racine du dépôt, racine du worktree, décalage du dossier du parent |
| `branch`, `base_commit` | branche et base figées |

Garde mémoire via Arc<Mutex> séparé du verrou global ; plafond par union de chemins.
Non stockée. Après un crash entre les deux passes, il reste au plus un
worktree et une branche inoffensifs, nommés dans le journal du daemon.

## Enregistrement durable (`Task.workspace`)

Champ ajouté avec `#[serde(default)]` : un payload 150 se relit avec `None`.

| Champ | Type | Rôle |
|---|---|---|
| `strategy` | `"worktree"` | seules les tâches en worktree ont un enregistrement |
| `state` | `"active"`, `"kept"` | `active` : premier rapport non stocké ; `kept` : dossier et branche conservés |
| `toplevel` | chemin canonique | racine du dépôt, pour `git -C` |
| `path` | chemin canonique | racine du worktree |
| `branch` | texte | branche créée ou choisie |
| `base_commit` | texte (40 hex) | base figée à l'admission |
| `head_commit` | `Option<texte>` | tête lue à la fin |
| `branch_commit` | `Option<texte>` | commit pointé par la branche à la fin |
| `head_on_branch` | `Option<bool>` | tête rattachée à la branche attendue |
| `commit_count` | `Option<u32>` | commits depuis la base jusqu'à la tête |
| `commits` | liste de `{sha, subject}` | 50 au plus, plus récents d'abord |
| `truncated` | booléen | vrai si `commit_count` > 50 |
| `dirty` | `Option<bool>` | modifications ou fichiers non suivis |

Règles :

- `Task.cwd` vaut le dossier de départ de l'enfant : `path` plus le décalage du
  dossier du parent dans le dépôt.
- `visible_changed` (Lineage) ignore `workspace`. Aucun signal, aucun nouvel
  état. La vue Lineage ne change pas.
- `state` passe `active` → `kept`, une seule fois, quand le rapport final est
  stocké. Jamais de retour vers `active`.
- `cleanup_done` exige le rapport terminé et l’arrêt confirmé ou la fin de garde153 selon les conditions existantes ; `kept` ne suffit pas.
- Le rapport appartient à la mission et à la ronde courantes. Réutiliser `Task.mission` et `Task.round_no()` pour comparer avant sauvegarde (CAS). Aucun champ public de ronde ajouté à `WorkspaceRecord`.
- Une nouvelle ronde recalcule le rapport avant sa publication. Les résultats déjà publiés restent immuables. `kept` reste `kept` et les droits hérités permettent de continuer à écrire.
- Aucun état « retiré » : Bridget ne supprime jamais.

## Cycle de vie

```
admission ──► active ──(premier rapport rangé)──► kept
                                               │
                              rondes suivantes : rapport recalculé, kept inchangé
```

## Exécuteur Git (pas de stockage)

Entrée : dossier, sous-commande de la liste blanche, arguments, délai.
Sortie : code, texte borné à 1 Mio. Aucun état.

## Reçu `shared_writers`

Clé ajoutée au reçu de `bridget_delegate` seulement si la valeur est au moins 1
et la stratégie est `shared` en développement. Calculée à l'admission, non
stockée.

## PrécisionsR1 approuvées

À l’ouverture d’une ronde, vider les faits courants ; conserver kept. Pour le
résultat, CAS sur waiting_for_children+mission+round_no ; stocker rapport, ligne
et result_available ensemble. Échec d’une tentative bornée : faitsNone, mention
indisponible ; aucune attente infinie. Pour failed/cancelled et chaque membre en worktree de l’arbre, l’arrêt est
confirmé avant le rapport. Le rapport est sauvegardé d’abord. Une seconde
sauvegarde pose cleanup_done seulement quand ce rapport est prêt. Ces deux
écritures sont ordonnées. Elles ne forment pas une sauvegarde unique.
L’ancreGit fiable est retenue ci-dessous pour H11 ; H12 utilise le claim des rapports en cours dans WorkspaceWork, sans quota arbitraire. Aucun champ public supplémentaire ; marqueur privé autorisé, implémentation nondéclarée acquise.

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
