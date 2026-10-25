# Audit de reutilisation de l'existant - 154-isolation-worktree

## Decision

Statut: PASS
Date: 2026-10-11
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree

Conclusion courte: Le plan réutilise la règle de dossier, le calcul de droits,
le chemin de fin de tâche et la purge de la session150. Il crée un seul module,
`workspace_git`, parce qu'aucun exécuteur Git borné n'existe et qu'il y a trois
usages réels. Aucune duplication évidente. Les choix conservateurs approuvés restent applicables. Aucun arbitrage requis.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 20 |
| Items audites | 20 |
| Reutilisations deja prevues | 13 |
| Existants potentiellement pertinents | 7 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 6 |
| Specs existantes applicables | 7 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Dossier de l'enfant sous la racine du parent | contrôle `canonicalize` + `starts_with(root)` | `crates/bridget-daemon/src/daemon/native_delegation.rs:791-797` | le worktree est sous la racine, la règle ne change pas |
| Droits hérités sur le dossier de l'enfant | `child_policy`, `snapshot` | `native_delegation.rs:820-842`, `native_permissions.rs:206-232 et :328-329` | appelés une seconde fois sur le worktree réel |
| Projet commun d'un worktree | `resolve_communication_project` (common-dir) | `crates/bridget-daemon/src/communication.rs:64-127` | un worktree reste dans le même projet ; non modifié |
| Fin de tâche | pose de `cleanup_done` | `native_delegation.rs:2526-2555` | le rapport final (lecture seule) s'insère avant ce drapeau |
| Arrêt confirmé avant rapport final | `cancel_tree` | `native_delegation.rs:2354 et :2500-2555` | le rapport est lu après l'arrêt, rien n'est supprimé |
| Purge sans perte | conditions de purge de la session150 | `crates/bridget-daemon/src/delegation_lineage.rs:28-44` et `:200-218` | inchangées grâce à D7 |
| Octets inchangés sans nouveau champ | patron `reply_timeout` de la session150 | `crates/bridget-transport/src/protocol.rs:2023-2035` | même `skip_serializing_if` |
| Catalogue par fournisseur | calcul `inherit` / `development` | `native_delegation.rs:714-725` | `worktree` ajouté avec la même logique |
| Caractères de contrôle | `is_disallowed_control` | `crates/bridget-core/src/text_guards.rs:40` | assainissement des sujets de commit |
| Identité du rapport par ronde | `Task.mission`, `Task.round_no()` | `crates/bridget-daemon/src/delegation.rs:82`, `:139` | CAS sur ces valeurs existantes ; aucun champ public supplémentaire |
| Garde et continuation | `Task.continuable`, condition du tick | `delegation.rs:161`, `native_delegation.rs:2229-2232` | `kept` autorise les rondes suivantes ; cleanup ne suit pas seulement kept |
| Validation153 | validation avant canonical/rejeu | `native_delegation.rs:756-769` | conserver keep_alive et brief avant rejeu |
| Catalogue153 | clés keep_alive/brief/max_rounds | `native_delegation.rs:728` | ajouter workspace sans retirer les clés existantes |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| Exécuteur Git borné | `git_common` interne à `resolve_communication_project` (retire `GIT_*`) | `communication.rs:85-100` | Reprendre la liste des variables retirées dans l'exécuteur ; la fonction interne reste (un usage de lecture, hors périmètre) |
| Délai d'un processus | boucle `try_wait` jusqu'à une échéance | `crates/bridget-daemon/src/daemon.rs:12555-12565` | Reprendre le patron, pas le code (il est dans un test) |
| Faits Git donnés à l'enfant | carte de reprise `managed_resume_worktree` | `crates/bridget-daemon/src/wrapper.rs:623-651` | Lectures remplacées par l’exécuteur borné unique (T022) ; choix2lignes supersédé |
| Alerte « checkout principal » | `is_protected_principal_checkout` | `wrapper.rs:572-613` | Même exécuteur borné unique (T022) ; aucun désarmement puisque Bridget ne retire plus de worktree |

## Items ajoutés par la ronde 2

| Item | Existant | Preuve | Décision |
|---|---|---|---|
| Lectures du wrapper par l'exécuteur | `managed_resume_worktree`, `is_protected_principal_checkout` | `wrapper.rs:572-590`, `:623-651` | Remplacer les appels directs par l'exécuteur : une seule implémentation durcie |
| Table de réservations | état du daemon sous verrou, réservation154 absente | `crates/bridget-daemon/src/daemon.rs:977-1027` | Ajout en mémoire dans l’état existant ; aucune réservation154 préexistante, pas de tableSQL |
| Rapport hors verrou | collecte `cleanups` puis exécution après le verrou | `native_delegation.rs:1787-1792 et :2229-2242` | Même patron « noter sous verrou, exécuter dehors » |

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
| 153-rondes-contexte | mission, round_no, garde, validation brief | CAS existant, rapport à chaque ronde, cleanup conditionné |
| 150-socle-delegation | purge par arbre, `cleanup_done`, refus nommés, `reply_timeout` | purge inchangée, même patron de champ |
| 100-observation-partage | `file_collision` sur chemins absolus | inchangée ; test de non-collision |
| 138 (projet commun) | worktrees partagent le common-dir | un worktree reste dans le projet |
| 145 (communication git) | résolution Git hors verrou | avant-vol hors verrou |

## Journal de recherche

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/reuse-audit.md#extrait-1).

## Arbitrages

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/154-isolation-worktree/reuse-audit.md#extrait-2).

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Revalidation sur la base153 livrée

Base fournie par le principal : `89f7a4163c2fb27229587f50a46414295ba3674d`
(153 `7c848850ba6067cf1d0d94b8edf0dd11896a1b24`). Les anciennes références
numériques150 sont historiques. Les20items ont été reprobes pendant cette relecture ; voirP01–P20.

| Recherche ciblée | Preuve actuelle | Décision |
|---|---|---|
| `mission`, `round_no`, `continuable` | `delegation.rs:82`, `:139`, `:161` | étendre l’existant153, aucun état public neuf |
| `keep_alive_out`, `brief`, `canonical` | `native_delegation.rs:756-769` | garder validation153 avant rejeu |
| `cleanup_done`, `idle_deadline_at` | `native_delegation.rs:2229-2232`, `:2553` | ne pas fermer une garde active après ronde1 |
| `Command::new`, carte de reprise | `wrapper.rs:572-625`, `communication.rs:87-103` | T022 borné ; communication inchangée |

Gate5/5. Zéro duplication évidente non arbitrée. Sélection par chemin explicite :
le script `.specify/scripts/bash/check-prerequisites.sh` est absent du WT154.
Aucun fichier outil ni `feature.json` n’a été modifié pour cette sélection.

## Relecture fraîche des20items — ce tour

Méthode exécutée : `rg` sur canonicalize/child_policy/snapshot/cleanup_done/
mission/round_no/catalogue/Command/try_wait, puis lectures ciblées des blocs
actuels. La purge a été localisée dans `delegation_lineage.rs` (pas de fichier
`delegation_retention.rs`). Les ancres150 déplacées par153 sont remplacées.
Les20probes ci-dessous vérifient chacune les tokens du bloc source actuel ;
aucun PASS de code/tests n’est déduit de ce contrôle documentaire.

| Probe | Item | Source actuelle relue | Constat |
|---|---|---|---|
| P01 | Dossier enfant | `crates/bridget-daemon/src/daemon/native_delegation.rs:791` | Lecture réelle et tokens de contrôle vérifiés. |
| P02 | Droits hérités | `crates/bridget-daemon/src/daemon/native_delegation.rs:820` | Lecture réelle et tokens de contrôle vérifiés. |
| P03 | Projet commun | `crates/bridget-daemon/src/communication.rs:87` | Lecture réelle et tokens de contrôle vérifiés. |
| P04 | Fin de tâche | `crates/bridget-daemon/src/daemon/native_delegation.rs:2549` | Lecture réelle et tokens de contrôle vérifiés. |
| P05 | Arrêt confirmé | `crates/bridget-daemon/src/daemon/native_delegation.rs:2500` | Lecture réelle et tokens de contrôle vérifiés. |
| P06 | Purge sans perte | `crates/bridget-daemon/src/delegation_lineage.rs:28` | Lecture réelle et tokens de contrôle vérifiés. |
| P07 | Octets sans champ | `crates/bridget-transport/src/protocol.rs:2023` | Lecture réelle et tokens de contrôle vérifiés. |
| P08 | Catalogue fournisseur | `crates/bridget-daemon/src/daemon/native_delegation.rs:714` | Lecture réelle et tokens de contrôle vérifiés. |
| P09 | Contrôles texte | `crates/bridget-core/src/text_guards.rs:40` | Lecture réelle et tokens de contrôle vérifiés. |
| P10 | Identité mission/ronde | `crates/bridget-daemon/src/delegation.rs:78` | Lecture réelle et tokens de contrôle vérifiés. |
| P11 | Garde continuation | `crates/bridget-daemon/src/delegation.rs:161` | Lecture réelle et tokens de contrôle vérifiés. |
| P12 | Validation153 | `crates/bridget-daemon/src/daemon/native_delegation.rs:756` | Lecture réelle et tokens de contrôle vérifiés. |
| P13 | Catalogue153 | `crates/bridget-daemon/src/daemon/native_delegation.rs:728` | Lecture réelle et tokens de contrôle vérifiés. |
| P14 | Git common proche | `crates/bridget-daemon/src/communication.rs:87` | Lecture réelle et tokens de contrôle vérifiés. |
| P15 | Délai processus proche | `crates/bridget-daemon/src/daemon.rs:12555` | Lecture réelle et tokens de contrôle vérifiés. |
| P16 | Carte reprise | `crates/bridget-daemon/src/wrapper.rs:623` | Lecture réelle et tokens de contrôle vérifiés. |
| P17 | Checkout principal | `crates/bridget-daemon/src/wrapper.rs:572` | Lecture réelle et tokens de contrôle vérifiés. |
| P18 | Lectures wrapper | `crates/bridget-daemon/src/wrapper.rs:572` | Lecture réelle et tokens de contrôle vérifiés. |
| P19 | État réservation à étendre | `crates/bridget-daemon/src/daemon.rs:977` | Structure sous verrou présente ; réservation154 absente, ajout local prévu. |
| P20 | Travail hors verrou | `crates/bridget-daemon/src/daemon/native_delegation.rs:2229` | Lecture réelle et tokens de contrôle vérifiés. |

L’exécuteur unique a au moins trois usages déjà prévus : avant-vol d’admission
(T008), création (T009), rapport de chaque mission/ronde (T015/T018/T028),
et deux consommateurs existants du wrapper (T022, P16/P17). Les exécuteurs
Git proches relus utilisent `output()` sans borne (P14/P16/P17) ; ils ne
remplissent pas temps/sortie/listeblanche. La boucle P15 est un patron de
processus dans une fixture, pas un utilitaire Git réutilisable. Le module prévu
reste justifié. La résolution du projet communP03/P14 reste inchangée.

Gate5/5 reconfirmé après les20probes réels. Les mentions `kept` et CAS des
sections antérieures décrivent le plan ; elles ne prétendent pas que ces
fonctions154 existent déjà. Aucun plan, contrat, tasks ni modèle modifié ici.

## Marqueur privé autorisé après contre-revueR1

Preuve du principal : native_delegation.rs:2193-2228 (result_available peut
devenir failed sur refus de livraison). CAS waiting_for_children+mission+round
ne couvre donc pas tous les chemins. Le marqueur privé
Task.workspace_report_mission (default/skipNone) réutilise mission : premier
rapport/publication, cleanup de tout l’arbre et reprise après panne. begin_round
remet marqueur et faits àvide. Ces usages justifient un seul champ privé ; aucun
nouveau champ public WorkspaceRecord, table, dépendance ou helper.
Le registreGit parent hors enfant fournit l’ancre par scanborné ; aucun champ
anchor ajouté. Choix autorisés, code/test nonprouvés à ce stade.

## ArbitrageH12 — coordination mémoire unique autorisée

Un seul WorkspaceWork via ArcMutexséparé contient creationMap etreportSet.
Preuve fraîche relue : native_delegation.rs:16-43 (struct, clés, Drop) ;
daemon.rs:986 etinitialisation3432. Au moins trois usagesréels : réservation
création, claimde rapport task/mission/round empêchant tickconcurrent, libération
RAII mêmeenpanique. La coordinationprotége le travailGit horsverrou ; aucun
thread, moteurbackground, quota1/2 ou frameworknouveau. Leprincipal a autorisé
ce choix. Usagecomplet natif enimplémentation, recetteconcurrence prévue àprouver.

SIGTERM seulement sur child.id attesté puis reapborné. PasSIGKILL. Une cible
hostile quiignoreSIGTERM peutpersister ; la bornegarantit le retour, pas l’arrêt
inconditionnel. AncreGitparenthorsenfant, métadonnéescommon/registre recherchées
depuisparent/ancêtreprincipal. Parentreste sourcedeconfianceADR011. Cette
limite ne crée ni nouveloutil ni gain de droits et reste dans lerisqueaccepté.

## Annexe exécuteurGit — recherche de l’owner et lectures courantes

L’ownerGit rapporte avoir recherché GitOutput, WorkspacePreflight, run_git,
preflight, validate_branch etbase_ref avant d’écrire l’exécuteur154, sans
équivalent Git borné. Cette déclaration décrit la rechercheavantécriture ;
les fonctions154 existent maintenant et ne sont pas présentées comme absentes
par la lecturecourante. Les responsabilités proches ont été relues ici :

| Existant ou fonctionlocale | Preuve relue | Réutilisation / justification |
|---|---|---|
| git_common | communication.rs:87-104 | patron retraitGITenv ; exécuteur154 étend àtoutesGIT_ ; résolutioncommune horsscope |
| wrapper existant | wrapper.rs:574 et:621 | raccorder deuxfonctions existantes, aucun doublon de carte |
| collecte reprise | reprise.rs:341-365 | outputnonborné, horsscope ; ne couvre pas frontièretemps/sortieGit154 |
| terminate_bootstrap | managed_process.rs:473 | privé ; patronSIGTERM/try_wait repris ; pas helperpublicnouveau |
| read_bounded_stdin | cli.rs:2243 | privéstdinbloquant ; ne couvre pas deuxpipesGit etdeadline |
| contrôles de texte | bridget-core/src/text_guards.rs:40 | fonctionréutilisée pour références et sujets |
| sha/revision/range | workspace_git.rs:85-96 | guards privésSHA1quarantehex/référencelist ; contratsSHA256soixantequatrehex distincts |
| stop_git | workspace_git.rs:131-143 | unpointprivé pour les sorties d’erreur de capture, SIGTERM/reapborné |
| report_anchor | workspace_git.rs:349 | responsabilitéprouvée : ignorer.gitmutable enfant, registreparent ; pas champdurable |

Le seul capture alimente avantvol, création, rapport et wrapper. Aucun crate,
table, service ou outil nouveau. L’annexe justifie des fonctionsprivées du
moduleprévu, pas un nouveauframework. R2 reste interrompu ; ces lectures ne
prouvent pas GREENglobal ni livraison.

## Réutilisation finale FR001 — trois attributs, aucun helper nouveau

Recherche et lecture actuelles : protocol.rs:2011-2017, :2038-2040 et :2753.
Le générique deserialize_present_optional existait pour d’autres métadonnées.
Il conserve l’absence avec default et refuse null explicite pour un type fermé.

| Usage154 ajouté | Existant réutilisé | Preuve actuelle | Décision |
|---|---|---|---|
| Delegate.workspace | deserialize_present_optional | protocol.rs:2039 | REUTILISER ; absence compatible, null refusé |
| Worktree.base_ref | même générique | protocol.rs:2014 | REUTILISER ; chaîne présente seulement |
| Worktree.branch | même générique | protocol.rs:2016 | REUTILISER ; chaîne présente seulement |

Le test d’objet fermé existant prouve RED0/1 puis GREEN3/0. Le code ajoute
trois attributs, aucune fonction. Les reçus contrat154-final-red-r1.json et
contrat154-final-green-r1.json et leurs hashes sont dans validation/tests-154.md.
Cette correction n’ajoute aucun outil, table, dépendance ou cadre d’exécution.
Converge formel et gates T025/T026 restent ouverts à ce point.

## Point de clôture de l’implémentation

28/28 tâches acquises sur GO du principal, statut Implemented seulement. Les
preuves finales réutilisent le décodeur présent pour trois attributs et le
contrôle descendants_busy pour fermer la course CAS, sans nouveau helper.
CoreUTC corrige uniquement la fixture existante. Aucun nouveau service, outil,
dépendance ou table. Réutilisation confirmée ; aucune Converge formelle ni
livraison déclarée par cette note. Le patch complet sera réexporté après gel
documentaire pour inclure ce dernier cochage et la correction de whitespace.
