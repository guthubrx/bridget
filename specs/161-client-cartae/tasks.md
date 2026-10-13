# Tasks SPEC-161 — Client Bridget à jour sur Cartae

**Statut :** Implemented. **Progression :** 14/14. **Pipeline :** In Progress. **Date :** 2026-10-13. **Branche :** session-161-client-cartae.

Réemploi PASS lu avant génération ; contre-revueClaude APPROVE_WITH_CHANGES traitée. Templates et check-prerequisites absents : tentative RC127, protocole Tasks appliqué manuellement sans créer un toolkit. Chemins ci-dessous relatifs au worktree /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/161-client-cartae.

## Phase1 — Préparation commune

- [x] T001 Figer propriété et état initial dans specs/161-client-cartae/validation/preflight161.md : refs Git, processus protégés, lien096, toolchain explicite, configurations Cargo utiles sans secret, droits/possession/groupe des parents et ressources existantes ; consigner inventaire du cache et décision de copie contrôlée Mac→Cartae sans téléchargement tiers (cache complet encore non attesté) (FR009/012/013/014).

## Phase2 — Fondations de la source approuvée

- [x] T002 Étendre scripts/tests/federation_089_test.sh pour exprimer les refus stricts --expect-commit : HEAD différent, dirty, assume-unchanged/skip-worktree et valeur invalide avant SSH ; conserver le cas legacy sans option. Observer les cas nouveaux en échec avant implémentation (FR001/002/014).
- [x] T003 Étendre scripts/deploy-remote.sh avec --expect-commit optionnelle40hex et gardes strictes avantSSH, sans casser mode legacy/dry-run ; vérifier les casT002 et documenter décision de réemploi existante (FR001/002/014).

## Phase3 — US1 : préparer le client exact (P1)

**Résultat indépendant :** le script refuse une source ou un transfert non conforme avant build et borne ses ressources ; les fixtures ne prouvent pas encore la livraison réelle.

- [x] T004 [US1] Étendre scripts/tests/federation_089_test.sh pour manifest corrompu, fichier manquant/en trop, lien distant, espaces permis, newline/backslash refusés et modesGit/exclusions ; observer refus avant appel Cargo build (FR002/005/014).
- [x] T005 [US1] Adapter scripts/deploy-remote.sh au format SOURCE-MANIFEST.sha256 existant : même liste filtrée pour transfert/hash, SHAmanifest attendu transmis parSSH, ensemble distant exact et absence de liens avantbuild ; vérifierT004 sans restreindre aux seuls crates (FR001/002/005/014).
- [x] T006 [US1] Adapter scripts/tests/federation_089_test.sh pour contrôler target-dir explicite, --jobs2 et nice au processusCargo, outilabsent/offlinefail/préfixeoccupé/pipefail ; garder tous ces refus sans activation (FR003/004/005).
- [x] T007 [US1] Borner scripts/deploy-remote.sh à une invocationCargo locked/offline, jobs2/nice10 et CARGO_TARGET_DIR explicite, en réemployant les gardes et la toolchain fournie ; vérifierT006 et propager les échecs (FR003/004/005).

## Phase4 — US2 : bascule et retour arrière (P1)

**Résultat indépendant :** l'extrait versionné de livraison fonctionne dans une fixture et refuse les références étrangères ; la bascule Cartae reste un gate réelPhase8.

- [x] T008 [US2] Vérifier/figer l'extrait Python os.replace dans specs/161-client-cartae/contracts/client-delivery.md, réemployé depuis pilotage-release, avec lienvoisin, comparaison avantaller/retour, refusnomoccupé et relecturefinale ; relier quickstart à cet unique extrait et expliciter limiteTOCTOU (FR007/008).
- [x] T009 [US2] Étendre scripts/tests/federation_089_test.sh pour extraire et exécuter le bloccontractuel exact sur fixturesaller/retour, cibleétrangère, temporaireoccupé et cibleinexécutable ; vérifier absence de mutation étrangère et retourversancienclient (FR007/008/014).
- [x] T010 [US2] Compléter specs/161-client-cartae/quickstart.md avec commandes paramétrées réellement valides de status/who candidat, recettePython095 séparée et lecture/status aprèsrollback ; préciser inscriptions/messagesautorisés et zérorestart/éditiondirecteDB (FR006/008/009/010).

## Phase5 — US3 : préparer une livraison vérifiable (P2)

**Résultat indépendant :** procédure et reçu distinguent preuves à produire et preuves observées. Aucun déploiement précoce pour cocher une tâche documentaire.

- [x] T011 [US3] Mettre à jour docs/federation-services.md avec la mise à jour161, source160/runner161 distincts, préfixeneuf/r2, garde --expect-commit, contrôle candidat/rollback et limiteclient-only ; aucun guide concurrent (FR001/006/007/008/012).
- [x] T012 [US3] Préparer specs/161-client-cartae/validation/livraison161.md avec rubriques non exécutées : sources/runner/docs/binaire, manifest, Git/push, activation/rollback, status/who, roundtripPython et ses écritures, processus, cleanup ; préparer inventaireexact et commandes outilbuild.py sans supprimer de ressources (FR011/012/013).

## Phase6 — Validation du développement

- [x] T013 Exécuter bash -n scripts/deploy-remote.sh puis bash scripts/tests/federation_089_test.sh sur sources finales ; enregistrer commandes/RC/cas/limites dans specs/161-client-cartae/validation/tests161.md. Répéter uniquement les cas affectés si correction ; aucune FULL Rust (SC006).
- [x] T014 Relire le diff et les décisions de specs/161-client-cartae/reuse-audit.md : aucune création non arbitrée, pas de dépendance/helper superflu, compatlegacy et14FR/6SC reliés ; consigner self-review et plan d'exécution des gates réels dans specs/161-client-cartae/implementation.md (FR001–014).

## Dépendances et ordre

T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009 → T010 → T011 → T012 → T013 → T014. US1 fondeUS2 ; US3 assemble leurs preuves. Pas de parallélisme de code : script et harness partagés. Seule relecture documentaire peut se faire parallèlement avec ownership explicite. Aucun MVP partiel n'est la cible ; toutes les tâches et tous les gates ci-dessous sont requis.

## Gates après développement — obligatoires pour livrer, distincts des cases de code

Les14cases décrivent développement et préparation vérifiables avantcommit. Les cocher ne satisfait pas les critères de livraison. Converge confronte chaqueFR au code/fixture ou à sa procédure opérationnelle pour ces14cases ; les SCréels restent non atteints jusqu'au reçuPhase8/9. Le statut implémentation peut être Implemented à14/14, mais pipeline reste In Progress jusqu'àG8/G9. Aucun FR n'est retiré pour éviter une dépendance circulaire.

**G6/G7 :** Converge après dernière implémentation, contre-revuecode autre fournisseur et audit applicable ; défaut bloquant résolu avantcommit.

**G8a — Git outil :** committer/intégrer/pousser le runner161 après gates ; vérifier ls-remote. Figer sonSHA distinct du checkout160 propre utilisé pourcompiler.

**G8b — Installation réelle :** invoquer le runner161 avec --expect-commit160, manifestpré-build et target-dirneuf ; buildLinuxofflinejobs2/nice10. Vérifier architecture/SHA/modes/BuildID, status/who candidat, référencealias avantactivation. Cacheincomplet → SC001nonatteint, lotnonlivré, lien096intact, copie contrôlée Mac→Cartae autorisée selon la décision bornée du plan ; si impossible, blocage sans autre approvisionnement implicite.

**G8c — Activation et recette :** basculeexacte contractuelle puis readlink/status/who du clientcourant ; recettePython095 distincte aller/retour ; deuxagents/deuxmessages synthétiques tracés ; zérorestart et aucune éditiondirecteDB. Suréchec rollbackconditionnel puis readlink/status ; aucun succès inventé.

**G8d — Reçus :** compléter les preuves réellement observées dans livraison161.md, intégrer/pousser les métadonnées et vérifier SHArefdistante. Ne pas attribuer au binaire le SHA des documents.

**G9 — Nettoyage :** archiver/vérifierpreuves, revenircheckoutdurable, traiter stages/source/target via outils/guards existants, retirerWT/branchepropres fusionnés nonutilisés sansforce. Garder096rollback,161actif et cachesutiles justifiés. Chaque candidat examiné, aucuneDB supprimée, aucune taillelogique vendue commegainphysique.

## Couverture des exigences

| Exigences | Développement/préparation | Preuve réelle obligatoire |
|---|---|---|
| FR001/002 | T002–T005,T011 | G8b : source160 exacte, manifest et binaire |
| FR003/004/005 | T006/T007 | G8b : build offline borné, préfixe neuf |
| FR006 | T010/T011 | G8b/G8c : candidat puis alias actifs |
| FR007/008 | T008/T009/T010/T011 | G8c : lien et rollback prêt/exercé si échec |
| FR009/010 | T001/T010 | G8c : processus et recettes distinctes |
| FR011/012/013 | T001/T012/T014 | G8a/G8d/G9 : Git, reçu, cleanup |
| FR014 | T002–T009/T014 | G8b/G8c : gardes et permissions |
| SC001/002/003/004 | T002–T011 | G8b/G8c : résultats réels, pas fixtures seules |
| SC005 | T012/T014 | G8a/G8d/G9 |
| SC006 | T013 | Rapport tests ciblés et limites |

## ETA après Tasks

14tâches, dont majorité de contrôles courts sur deux fichiers existants. Développement/préparation45–90min ; revue/convergence/audit15–30min ; livraison/build/réseau45–90min ; nettoyage/métadonnées15–30min. Total120–240min. Cacheincomplet ou blocageSSH peut dépasser cette borne et impose recalibrage immédiat ; attente d'autorisation d'approvisionnement horsETA.
