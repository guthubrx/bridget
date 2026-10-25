# Archives techniques de l’adaptateur T3

Document d’origine : [specs/154-isolation-worktree/plan.md](../../../../../../specs/154-isolation-worktree/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| # | Décision | Alternative rejetée | Raison |
|---|---|---|---|
| D1 | Champ `workspace` sur `Delegate` | outil `bridget_workspace` séparé | un appel reste un appel |
| D2 | Worktree sous `<cwd>/.worktrees/<task8>` | hors racine, ou dans `.git` | `starts_with(root)` suffit, aucun droit ajouté |
| D3 | Admission en deux passes dans le daemon : réserver sous verrou, Git hors verrou, tout revalider sous verrou, droits calculés sur le worktree réel | création sous verrou ; création dans le wrapper ; création dans le tick | verrou jamais tenu pendant Git ; le contexte de droits Claude dépend des fichiers du dossier ; refus synchrone préservé |
| D4 | Refus précoces `workspace_requires_development` et `workspace_inheritance_unavailable` | laisser échouer à `settings_revision_changed` | refus lisibles, aucun effet disque |
| D5 | Exécuteur Git unique, durci, liste blanche sans suppression, configurations système et globale ignorées | `std::process::Command` partout, ou `git2` | sécurité en un point, pas de dépendance |
| D6 | Bridget ne supprime jamais un worktree ni une branche | retrait automatique sans force ; retrait après contrôle de propreté | quatre risques de perte écartés par construction (contre-revue 2, 3, 4, 7) |
| D7 | `cleanup_done` après le rapport terminé et arrêt confirmé/fin de garde153, état `kept` indépendant des droits d’écriture | nouveau drapeau de purge | la purge 150 reste inchangée |
| D8 | Rapport lu dans Git (dont `head_on_branch`), ligne de fin écrite par Bridget, objet dans `task_status` | texte de l'enfant comme source | un enfant ne forge pas les faits |
| D9 | `shared_writers` informatif | refus de la 2e délégation partagée | compatibilité 149, usages en lecture |
| D10 | Refus des dépôts à configuration exécutable (filtres, inclusions, clone partiel) | neutraliser chaque filtre par `-c` | liste incomplète si une inclusion en ajoute (contre-revue 1) |
| D11 | `.worktrees/` doit être déjà ignoré, plafond de 32 worktrees, aucune écriture dans la configuration du dépôt | écrire `info/exclude` ; pas de plafond | supprime une surface de lien symbolique ; borne l'accumulation (contre-revue 5, 9) |
| D12 | Rapport hors verrou : le tick note un travail, Git lit sans verrou, un verrou court range rapport+ligne+result_available ensemble si waiting_for_children+mission+round correspondent, chaque ronde recalcule ses faits | Git dans le tick | verrou global jamais tenu pendant Git (ronde 2, point 4) |
| D13 | Réservations via Arc<Mutex> séparé, plafond par union de chemins ; la seconde passe ne refait pas les contrôles d'état Git créés par la requête | pas de réservation ; revalidation totale | deux appels de même clé ne créent pas deux worktrees ; la branche réservée et la 32e place ne se refusent pas elles-mêmes (ronde 2, points 5 et 7) |
| D14 | Exécution de Git par le daemon pour tout parent autorisé, exécution directe de Git, durcie ; risque résiduel de changement de configuration accepté | Git confiné ; Git par fichiers ; réservation aux parents non confinés | décision utilisateur du 2026-10-11 ; T3 fait de même avec moins de durcissements |

<a id="extrait-2"></a>

## Extrait 2

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
| Chevauchement avec la session151 (résultat structuré) | l'objet `workspace` est autonome ; le contrat151 livré reste intact |
