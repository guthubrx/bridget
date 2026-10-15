# Validation169 — Preuves d'implémentation et limites

**État courant après R2 :14/14 tâches acceptées par ROOT, implémentation Implemented, pipeline In Progress.** ROOT a relu les9 deltas et vérifié les preuves T013/T014, puis clos le passage2 CONVERGED avec0 tâche ajoutée et tâches byte-identiques. Le bilan R2 est536/536 cas distincts ciblés finalement PASS après reprises. Il n'est ajouté ni aux1 020 cas R1 recouvrants ni aux124 PASS/1 SKIP MCP séparés. La contre-revue R2 est reçue et arbitrée par ROOT ; l'audit source est clos pass et J1 clos. FR017, SC003 réel, SC009 et J2–J6 restent ouverts.

Date : 2026-10-15. Le bilan initial R1 ci-dessous reste conservé avec ses incidents et ses reprises. Son acceptation des12 tâches décrit le gel antérieur. La présente mise à jour accepte les deux corrections R2 sur le GO ROOT ; elle ne produit aucun verdict de convergence, contre-revue R2, audit ou livraison.

Le responsable documentaire a lu les preuves existantes. Il n'a lancé aucun test, typecheck, lint, build, fournisseur ou service. Les niveaux ci-dessous restent distincts : contrat/texte, fixture SDK, intégration isolée avec fichier ou HTTP réels, régression héritée, puis recette réelle de fournisseur. Aucun résultat des quatre premiers niveaux ne devient une preuve d'obéissance d'un modèle réel.

## R2 — Corrections et preuves acceptées par ROOT

Reçu final lu : /Users/moi/.cache/bridget-delivery169/source-snapshot-r2/developpement169.md, SHA-256 `c088e76d0bc9d08b4e1c87435520d2c917fcb69f4797ec619507604997a7773b`. Source distante : /home/moi/.local/state/bridget-delivery169/developpement169.md. ROOT confirme65 membres réguliers et64 SHA d'archive vérifiés. Le manifeste /Users/moi/.cache/bridget-delivery169/source-snapshot-r2/source169-manifest.json a le SHA-256 `9fd4faf8e2541d13eb9b1c80bd9edb8f56a625923dbc4bdf73163204477ae165` :35 fichiers copiés sur Mac avec les mêmes SHA,9 deltas depuis R1 et26 fichiers inchangés.

| Correction | Preuve et limite |
|---|---|
| T013 — Reset OFF | Codex transmet le contexte même sans MCP et après reprise à cache vide ; le cache existant garde le reset pour la compaction. Le compositeur réutilisé traite l'état inconnu ACP/Cursor ; leurs adaptateurs produit restent inchangés. Les oracles couvrent reprise ordinale>1, état inconnu, slash brutes et transitions ; aucun cache avancé par un slash. OpenCode1/2 restent inchangés. |
| T014 — Montage avant premier offer | Le vrai runner et le vrai `prepareClaudeMcp` sont exercés avec une frontière SDK simulée. La candidate privée ne reçoit aucun prompt utilisateur. Après un non-montage, son `Query.return()` physique est attendu avant une seconde candidate blocked, unique et dotée de sa propre file. Identité d'entrée conservée ; divergence de montage ou erreur de préparation arrêtent avant offer. Le travail de fond et son refus restent préservés. Ce n'est pas une réponse de modèle réel. |

Le registre /Users/moi/.cache/bridget-delivery169/source-snapshot-r2/preuves/R2/tests-summary.json conserve la passe initiale437 PASS/99 FAIL, la reprise91 PASS/8 FAIL, puis la reprise8 PASS. Ce sont536 cas distincts finalement PASS sur6 bancs, pas une seule exécution verte. Les échecs initiaux restent conservés. Les87 transcriptions Codex adaptées à l'enveloppe attendue, le raccord Stream du runner, les oracles d'attente/réutilisation et le destructuring de `Tester.each` sont documentés dans le reçu. Aucun banc déjà réussi ou Pi/HTTP/MCP149 inchangé n'est rejoué sans cause.

| Contrôle R2 | Résultat et preuve |
|---|---|
| Types provider-core et provider-cursor | RC0 dans le registre /Users/moi/.cache/bridget-delivery169/source-snapshot-r2/preuves/R2/commandes169.json. |
| Types serveur | RC1,75 erreurs historiques, delta0. ROOT compare les75 blocs complets base/R1/R2 :`exact_raw_multiset_equal=true`. Reçu /Users/moi/.cache/bridget-delivery169/source-snapshot-r2/root-types-full-block-comparison-r2.json, SHA-256 `a4113d34fd74d4093aff8b399d56063e086bca2c5341cfd6ab5aa48396b6f414`. La preuve dépasse l'égalité des comptes ; le serveur ne devient pas PASS. |
| Lint et format | Lint des9 fichiers R2 :RC0,0 erreur,6 warnings hérités attestés sur la base. Format ciblé des9 fichiers conforme. Ce périmètre est distinct des12 warnings sur35 fichiers R1. Registre /Users/moi/.cache/bridget-delivery169/source-snapshot-r2/preuves/R2/commandes169.json. |

Le warning de fixture TS377033 reste conservé et non causal :instance unique, sans rupture de cycle de vie établie. Les qualifications natives Cartae/Mac ci-dessous couvrent seulement leurs deux paires SDK/CLI et une initialisation sans prompt. Pi réel, réponse et obéissance d'un modèle, admission native nouvelle, SC003 réel, activation cible et SC009 ne sont pas déduits des fixtures. Aucun nouveau test, typecheck, lint, build, service ou contrôle Git n'est lancé par le responsable documentaire. Le reçu /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/converge169.md est complété seulement après la clôture du passage2 ROOT.

## Provenance du gel et du bilan R1

- Reçu local final : /Users/moi/.cache/bridget-delivery169/source-snapshot/developpement169.md. SHA-256 vérifié : `5615751c6e7f9b3bb958ec5db0894dc851cbf14174175586df007173e70563ec`.
- Reçu de l'owner sur Cartae : /home/moi/.local/state/bridget-delivery169/developpement169.md.
- Registre exact des commandes, RC, durées, compteurs et SHA des logs : /Users/moi/.cache/bridget-delivery169/source-snapshot/commandes169.json ; source /home/moi/.local/state/bridget-delivery169/commandes169.json.
- Source préparée : /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget, base `76780a20bb22b9390eab2389eaf0db87e1349e7e`, branche `session-169-priorite-bridget`.
- Copie Mac des35 fichiers, attestée byte-identique par ROOT : /Users/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget. Reçu de copie : /Users/moi/.cache/bridget-delivery169/source-snapshot/copy-receipt169.json.

| Objet figé | Preuve locale | SHA-256 du reçu de copie |
|---|---|---|
| Patch source | /Users/moi/.cache/bridget-delivery169/source-snapshot/source169.patch | `cd362e6cd6e5f9765f1734af3831e80a9f517f51cb4e2c9166a8e377f3109e22` |
| Manifeste source | /Users/moi/.cache/bridget-delivery169/source-snapshot/source169-manifest.json | `efd17d140e0fe21bd882c11b0b008afd2788404b6ed81e56110b053a145ebba9` |
| Archive des35 fichiers | /Users/moi/.cache/bridget-delivery169/source-snapshot/source169.tar.gz | `75ba16488bf159bf1474155f50b660d27603a31418c270a33b19ea4921382d64` |

Le manifeste des preuves est /Users/moi/.cache/bridget-delivery169/source-snapshot/preuves169.sha256 ; source /home/moi/.local/state/bridget-delivery169/preuves169.sha256. Les logs complets restent privés sur Cartae. La présente passe n'a pas recalculé chaque SHA de log : elle vérifie le reçu final et lit les comparatifs copiés ; ROOT a confirmé leur provenance avant son GO.

**Contrôle indépendant ROOT après cette lecture documentaire.** ROOT confirme le téléchargement et l'archivage des126 preuves, puis le contrôle des127 membres d'archive et des126 SHA sous /Users/moi/.cache/bridget-delivery169/source-snapshot/preuves. SHA-256 de l'archive contrôlée : `eb1afc827ebc7a8985fd57f6d5ced241f4b2edc2228d0808f46567bfab0f921a`. ROOT a comparé les75 blocs complets d'erreur TS : ils sont identiques ; le passage de1 à2 warnings de types reste réservé ci-dessous. Le patch Mac, de préfixe SHA `d5124d4b`, diffère du patch source uniquement par ses index Git de10 chiffres au lieu de9. Les35 SHA de fichiers source sont identiques. Cette différence de représentation du patch n'est pas une dérive de source. Ce contrôle vient de ROOT ; le responsable documentaire n'a lancé aucune nouvelle campagne.

## Qualification réelle Cartae T014 — Cycle sans prompt

Reçu existant lu par SSH, sans autre lecture distante ni exécution de scénario : /home/moi/.local/state/bridget-delivery169/qualification-t014/result.json. SHA-256 des octets JSON reçus : `c0747f06ddfbb910c0b28cbeee636b43fc3e47179b475b1cde00ee82e45b41c0`. Le reçu porte sur le SDK0.3.276 et la CLI réellement installée2.1.251 : /home/moi/.local/share/claude/versions/2.1.251.

| Scénario qualifié | Résultat observé dans le reçu | Portée |
|---|---|---|
| Session neuve :`sessionId`, attente de `Query.return()`, même `sessionId` | Deux initialisations réussies, fermeture confirmée, sorties0 ; aucune conversation JSONL. Le protocole d'attente de `Query.return()` est précisé par ROOT. | Réinitialisation préparatoire avec identité d'entrée inchangée. |
| Témoin :initialisation neuve, puis `resume` sur l'ID sans conversation | Première initialisation réussie ; seconde refusée avec `No conversation found`, sortie1. | Ne pas convertir une entrée neuve en reprise d'une conversation inexistante. |
| Frames et fin des processus | Frames de contrôle `initialize` seulement, `userFrames:0`, aucune frame utilisateur/modèle observée ; `allTrackedChildrenStopped:true`. | Qualification SDK/CLI isolée ; fichiers de configuration et d'état temporaires distincts d'une conversation JSONL. |

Décision ROOT :une entrée neuve conserve `sessionId` ; une reprise existante conserve `resume`/`resumeSessionAt`. Aucun test positif de conversation existante n'est déduit du témoin négatif. Ce constat Cartae vaut pour cette seule paire SDK/CLI. ROOT atteste qu'aucun endpoint, authentification ou session de production n'a été touché. Cette qualification n'est pas une recette de modèle et n'est pas ajoutée au bilan dédupliqué historique ci-dessous.

Cette qualification éclaire le cycle de T014 ; elle ne prouve pas à elle seule son montage effectif ou sa première offre de prompt. L'acceptation R2 vient séparément du reçu et du GO ROOT ci-dessus. Le responsable documentaire a seulement lu le JSON ; il n'a relancé aucun scénario.

## Qualification réelle Mac T014 — Complément sans prompt

Reçu local lu, SHA-256 vérifié : /Users/moi/.cache/bridget-delivery169/qualification-t014-mac-2.1.300/result.json, `6f5d22cd4bc72b4d926193e68cf463b12c54cccc037127464011c18d4dbcd009`. Paire exécutée :SDK0.3.276 et CLI2.1.300 réellement installée dans /Users/moi/.local/share/claude/versions/2.1.300. Le RC0 de la qualification est transmis par ROOT ; le JSON donne1 782 ms.

| Observation Mac | Résultat et portée |
|---|---|
| Initialisation neuve, attente de `Query.return()`, même `sessionId` | Deux initialisations réussies, sorties0, aucune conversation JSONL ; identité d'entrée conservée. |
| Témoin `resume` sur ID sans conversation | Refus `No conversation found`, sortie1 attendue ; aucune preuve positive de reprise existante. |
| Frames et fermeture |0 frame utilisateur, aucune frame modèle observée ; quatre PID arrêtés naturellement, aucun signal ni erreur de fermeture, `allTrackedChildrenStopped:true`. |
| Isolement précisé par ROOT | Lanceur /usr/bin/env avec `-i`, identifiants d'authentification absents, HOME/configuration/répertoire de travail privés ; aucun modèle ou service de production sollicité. |

Les deux paires présentes sont donc qualifiées pour ce seul cycle sans prompt :SDK0.3.276/CLI2.1.251 sur Cartae et SDK0.3.276/CLI2.1.300 sur Mac. Les autres versions et une reprise existante positive restent hors preuve. La règle d'entrée reste `sessionId` pour une session neuve, `resume`/`resumeSessionAt` pour une reprise existante. Les qualifications ne sont pas ajoutées au bilan historique ci-dessous et ne valent pas à elles seules PASS du correctif R2. Son acceptation vient séparément des preuves R2 et du GO ROOT. Aucune qualification supplémentaire n'a été lancée par le responsable documentaire.

## Bilan historique du registre R1, sans addition de groupes recouvrants

Le registre dédupliqué de l'owner rapporte **1 020 cas distincts couverts sur17 bancs**. Ce bilan réunit des passes initiales et des reprises ciblées. Il ne décrit ni une campagne unique entièrement verte ni une suite globale du dépôt. Les groupes partiels et leurs reprises ne sont pas additionnés ici.

Les drivers MCP inchangés apportent séparément38 PASS Codex et86 PASS/1 SKIP Claude, soit le bilan réutilisé124 PASS/1 SKIP indiqué par l'owner. Ce bilan n'est pas ajouté aux1 020 cas. Le groupe qui les contient recouvre d'autres preuves déjà comptées.

| Preuve du registre | Résultat retenu et réserve |
|---|---|
| /home/moi/.local/state/bridget-delivery169/tests169-affectes-001.json | Première passe affectée :945 PASS/3 FAIL, RC1, campagne interrompue avec arrêt du seul worker de test. Les échecs et cet arrêt restent historiques. Les bancs terminés sont réutilisés selon le registre. |
| /home/moi/.local/state/bridget-delivery169/tests169-codex-reprise-001.json | Banc Codex repris :164 PASS. Pas un total à ajouter à la passe précédente. |
| /home/moi/.local/state/bridget-delivery169/tests169-pi-affectes-001.json ; /home/moi/.local/state/bridget-delivery169/tests169-pi-reprise-oracle-001.json | Pi :75 PASS/1 FAIL après correction du tampon privé ; l'ancien oracle de comptage est corrigé puis repris1 PASS. Les76 scénarios du banc sont comptés une seule fois par l'owner. |
| /home/moi/.local/state/bridget-delivery169/tests169-manager-reprise-001.json | Deux nouveaux cas de gestionnaire passent ; les cas inchangés ne sont pas rejoués. Intégration du vrai fichier/watch, de cinq tours et du builder Codex dans le même scénario. |
| /home/moi/.local/state/bridget-delivery169/tests169-claude-matrice-001.json | Quatre cas affectés passent, dont deux nouveaux. Le banc Claude dédupliqué est162 cas ; aucun cumul avec ses reprises précédentes. |
| /home/moi/.local/state/bridget-delivery169/tests-raccords-small-001.json | Montages/révocations/opt-outs des drivers inchangés réutilisés. Leur124 PASS/1 SKIP reste séparé du bilan dédupliqué. |
| /Users/moi/.cache/bridget-delivery169/source-snapshot/off-history-proof.json | Empreinte historique du registre R1 :6 931 octets UTF-8, sans preuve d'identité de l'enveloppe sans MCP R2 ; SHA `eb063f17b3aea507ea77a28081d44c8c55d30f9c0dda69bc83e54842f5a82eb3`. |

Les autres premiers échecs restent dans le registre : couche de test FileSystem absente, imports de fixtures, appel de helper Pi, oracle Cursor et nouvelle ligne de format. La tentative `types-core-001` n'a exécuté aucun compilateur, car `tsgo` était absent ; aucun RC de compilation n'en est déduit. Aucun de ces incidents n'est effacé par les résultats finaux.

## Types, lint et format R1 : comparaisons exactes

| Contrôle | Résultat final | Comparaison et lieux exacts |
|---|---|---|
| Types contracts, provider-core, provider-pi, provider-cursor, provider-acp, provider-opencode | RC0 rapporté par l'owner sur les paquets affectés. | Commandes exactes et logs dans /Users/moi/.cache/bridget-delivery169/source-snapshot/commandes169.json ; source /home/moi/.local/state/bridget-delivery169/commandes169.json. |
| Types serveur | **RC1,75 erreurs TS historiques, delta0 erreur. Le paquet serveur n'est pas PASS.** Les75 blocs d'erreur TS sont strictement identiques. Les warnings de types passent de1 à2 ; ce delta est distinct du lint. | /Users/moi/.cache/bridget-delivery169/source-snapshot/types-server-comparison-final.json ; source /home/moi/.local/state/bridget-delivery169/types-server-comparison-final.json. Compare /home/moi/.local/state/bridget-delivery169/types-server-baseline-001.json à /home/moi/.local/state/bridget-delivery169/types-server-005.json. |
| Lint des35 fichiers | RC0,12 warnings ; mêmes triplets règle/message/fichier que la base. Dernier delta de test/fixture :RC0,0 warning. | /Users/moi/.cache/bridget-delivery169/source-snapshot/lint-comparison-final.json ; source /home/moi/.local/state/bridget-delivery169/lint-comparison-final.json. Commandes /home/moi/.local/state/bridget-delivery169/lint-affectes-001.json, /home/moi/.local/state/bridget-delivery169/lint-baseline-affectes-001.json et /home/moi/.local/state/bridget-delivery169/lint-final-delta-001.json. |
| Format et espaces du diff | Format des35 fichiers conforme selon le reçu ; dernière addition Claude formatée séparément. `git diff --check` RC0. | /home/moi/.local/state/bridget-delivery169/format-check-002.json ; /home/moi/.local/state/bridget-delivery169/format-010.json ; /home/moi/.local/state/bridget-delivery169/diff-check-001.json. |

La comparaison serveur a d'abord montré77 erreurs contre75 sur la base. Les deux erreurs nouvelles du callback de test ont été corrigées. Les contrôles serveur003/004/005 retrouvent les mêmes75 erreurs complètes ; ils ne sont pas convertis en réussite globale.

La lecture indépendante de ROOT relève aussi un nouveau warning de types Effect `TS377033 multipleEffectProvide`, dans /home/moi/11.Repositories/t3code-local/.worktrees/169-priorite-bridget/apps/server/src/orchestration-v2/ProviderSessionManager.test.ts:4712. La fixture fournit le service de réglages configuré puis NodeServices pour ses dépendances. Le journal courant a2 warnings de types contre1 sur la base. Ce warning n'est ni une erreur TS ni l'un des12 warnings lint. Il sera soumis à la contre-revue ; aucune correction de code n'est faite par cette passe. Les diagnostics de types dans leur ensemble ne sont donc pas déclarés byte-identiques.

La base privée est /home/moi/.local/state/bridget-delivery169/baseline-types-76780a20. Elle utilise le même Node24.13.1, le même compilateur et le même patch Effect. Les preuves de résolution et d'identité sont :

- /Users/moi/.cache/bridget-delivery169/source-snapshot/baseline-module-resolutions.json ; source /home/moi/.local/state/bridget-delivery169/baseline-module-resolutions.json.
- /Users/moi/.cache/bridget-delivery169/source-snapshot/baseline-dependency-links.json ; source /home/moi/.local/state/bridget-delivery169/baseline-dependency-links.json.
- /Users/moi/.cache/bridget-delivery169/source-snapshot/baseline-config-manifest.json ; source /home/moi/.local/state/bridget-delivery169/baseline-config-manifest.json. Lock, package et configurations comparées sont byte-identiques.
- /Users/moi/.cache/bridget-delivery169/source-snapshot/toolchain169-manifest.json ; source /home/moi/.local/state/bridget-delivery169/toolchain169-manifest.json.
- /Users/moi/.cache/bridget-delivery169/source-snapshot/baseline-diagnostic-files.json ; source /home/moi/.local/state/bridget-delivery169/baseline-diagnostic-files.json.

Le responsable documentaire a lu les comparatifs : multiset complet des75 erreurs identique, aucun ajout/retrait ; multiset des12 warnings identique. Il n'a pas relancé les outils.

## Matrice des17 exigences

« Couvert » désigne le niveau de preuve indiqué, jamais une recette réelle implicite. Les détails par tâche et par banc viennent du reçu final et du registre cités ci-dessus.

| Exigence | Tâches | Niveau établi et limite |
|---|---|---|
| FR001 | T001,T005 | Contrat et fichier isolé :booléen d'environnement, défautfalse, exclu du patch public ; aucune UI ajoutée. |
| FR002 | T001,T005,J4–J5 | Absence/OFF prouvés en test. **ON de la cible installée reste à attester en livraison.** |
| FR003 | T001,T005 | Fichier/watch et cache :JSON invalide, I/O, erreur après ancienOFF, refus avant secret/write, fichier byte-identique, réparation. |
| FR004 | T002–T005,T007–T008,T013–T014 | Contexte par tour et fixtures SDK ; cinq tours fichier/watch. R2 complète reset OFF/cache inconnu et montage Claude avant offer. Pas d'obéissance LLM déduite. |
| FR005 | T003,T006–T008,T013 | Fil T3 demandé, outils hors délégation, slash natives et nom privé Pi réservés. R2 confirme slash ACP/Cursor brutes sans avancement du cache. |
| FR006 | T003,T006 | Contraintes workspace/expected_base_commit réelles du contrat168 présentes dans le texte, autres axes automatiques. Pas de nouvelle sélection contrainte exécutée par un modèle réel169. |
| FR007 | T003,T006 | Inspection du delta par owner et ROOT :aucun second moteur de choix, aucun routage natif Bridget modifié. |
| FR008 | T002,T004,T006,T014 | Référence MCP stable, montages/révocations/opt-outs réutilisés. R2 conserve l'identité d'entrée et attend le Query physique ; aucune nouvelle autorité. |
| FR009 | T003–T004,T007–T009,T013–T014 | Consigne ferme, reset OFF et non-montage Claude fidèle avant offer ; aucun nouveau dispatch de secours. Une panne future reste possible ; aucun verrou natif nouveau. |
| FR010 | T009–T010 | Mécanismes149/163/168 réutilisés, références de rejeu conservées et absence de nouveau dispatch constatée. **Aucune admission Bridget réelle169 n'est exécutée par un mock T3.** |
| FR011 | T005,T010 | Transitions du fichier et régressions149 :pas de migration ajoutée, anciens suivis conservés. Reprise de la cible enJ5. |
| FR012 | T010 | Lineage149 :23 PASS ; journal/statut/résultat et lectures restent natifs. Aucun changement de rendu. |
| FR013 | T006,T010 | Autonomie native héritée et moteur inchangé ; suivi149 réutilisé. Aucun nouveau parcours standalone réel169 n'est annoncé. |
| FR014 | T002–T004,T007–T008,T013–T014 | Contrats/fixtures de tous les raccords ; R2 corrige Codex/ACP/Cursor et prépare le montage Claude réel avant offer. Pi réel absent. |
| FR015 | T001–T003,T006,T008,T011,T014 | Source unique, input existant et runner Query réutilisé ; aucun état durable parallèle, dépendance ou protocole public ajouté. |
| FR016 | T012,T011 | Vrai handler HTTP isolé :sujet authentifié Bridget seul, champs utiles intacts, autres sujets et bounded inchangés. Simulation du gros fil distincte de la recette cible. |
| FR017 | T012,J2–J6 | **Ouverte.** Code préparatoire accepté ; paquet, installation, activation et reprise de la cible ne sont pas encore attestés. |

### Réemploi précis de T009/FR010

Les mécanismes hérités sont nommés pour éviter de transformer une fixture T3 en admission native prétendue :

- SPEC149 :identifiants de tâche/enfant stables, relation parent et projection unique ; source /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/149-sous-agents-lineage/spec.md.
- SPEC163 :décision durable avant admission, même demande rejouée sans nouvelle décision, contrôle final des gardes ; source /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/163-decision-admission/spec.md.
- SPEC168 :enveloppe et contraintes automatiques, choix durable, rejeu/reprise et non-double-admission natifs ; source /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/168-routage-flotte/spec.md.

L'owner puis ROOT ont lu l'absence de nouveau dispatch de sous-tâche ou de moteur de secours dans le delta169. Le moteur Bridget et ses stockages ne sont pas modifiés. Les consignes gardent les références de suivi/rejeu. Ces preuves établissent le réemploi et l'absence d'un nouveau chemin d'hôte ; elles ne rejouent pas une admission réelle149/163/168 et ne prouvent pas un choix d'outil d'un modèle réel.

## Matrice des9 critères de succès

| Critère | Preuve disponible | État de preuve |
|---|---|---|
| SC001 | Matrice de textes, transmission par tour et corrections R2 de reset/montage ; politique OFF conservée, enveloppe sans MCP différente. | Contrat/fixture/transport, sans résultat de modèle. |
| SC002 | I/O isolées, cache fiable, refus avant secret/write, fichier invalide byte-identique. | Couvert par les bancs réglages/manager. |
| SC003 | Texte et transmission préparés ; compteurs d'outils distingués dans le contrat. | **Ouvert, recette réelle et résultat corrélé enJ5.** |
| SC004 | Contraintes et gardes, identité stable ; R2 conserve les options d'entrée du runner et son fait de montage. | Fixture et qualification sans prompt ; aucune admission réelle déduite. |
| SC005 | Non-montage Claude traité avant offer, consignes explicites, absence de nouveau dispatch ; rejeu149/163/168 réutilisé. | Contrat/fixture/inspection ; limite FR010 conservée. |
| SC006 | OFF→ON→OFF, fichier/watch, resets après reprise à état inconnu en R2 ; Lineage149 et mcpSession149 conservés. | Intégration isolée et régression héritée ; reprise cible réservée àJ5. |
| SC007 | Fil T3 explicite, outils hors délégation, slash ; R2 vérifie slash ACP/Cursor sans cache avancé. | Contrat et fixtures ; aucun choix spontané de modèle prétendu. |
| SC008 | Manifeste35 fichiers et lecture owner/ROOT ; configurations/dépendances inchangées ; pas de nouveau service, stockage ou protocole public. | Inspection du périmètre. |
| SC009 | Handler HTTP corrigé et testé en isolation ; simulation du gros fil disponible. | **Ouvert, installation/activation/reprise réelle enJ5.** |

## Matrice des14 tâches acceptées par ROOT

Les lignes décrivent les preuves de l'acceptation d'implémentation. Elles ne cochent aucun jalon de livraison.

| Tâche | Preuve retenue | Réserve |
|---|---|---|
| T001 | Contrat, réglages et I/O :défaut, exclusion du patch, confiance, refus avant secrets, réparation. | Les premiers échecs de couche de test restent au registre. |
| T002 | Champ par tour, configureMcp=false, attachement réutilisé, cinq tours fichier/watch/builder, identité stable. | Fixture d'intégration, pas un fournisseur réel. |
| T003 | Compositeur off/prefer/blocked ; six contradictions corrigées ; constante OFF avec MCP conservée et exception d'enveloppe sans MCP en R2. | Le texte ne garantit pas l'obéissance. |
| T004 | Codex repris164 PASS ; matrice Claude162 cas dédupliqués ; snapshot:false et clé de query ; travail de fond préservé puis prochain tour à jour. | SDK/transport de fixtures ; coût cache/thinking non mesuré, pas de modèle réel. |
| T005 | Changements réels du fichier/watch entre tours, invalide après OFF, réparation, retour OFF. | Environnement isolé. |
| T006 | Textes et gardes de montages inchangés ; opt-out/révocation conservés. | Contrat et régression héritée ; pas de décision de flotte réelle169. |
| T007 | ACP, OpenCode1/2 et Cursor :réinjection, premier contexte et slash sans état avancé à tort. | Raccords simulés ; aucun montage Bridget inventé. |
| T008 | Extension et adaptateur Pi :mode fermé, nonce16hex, découverte avec provenance, reçu privé+ACK courants, tampon wake contourné pour ce reçu, nom utilisateur réservé, slash/compact intacts. | Contrat/fixtures seulement ; binaire Pi hors PATH. |
| T009 | Consignes de panne/refus/réponse perdue, références conservées, lecture du delta sans nouveau dispatch ; réemploi149/163/168. | Aucune admission native réelle produite par un mock T3. |
| T010 | Lineage149 :23 PASS ; mcpSession149 :14 PASS ; moteurs/permissions/Lineage inchangés. | Régression héritée, pas une nouvelle recette standalone. |
| T011 | Contrôles ciblés, comparaison complète base/current, lint/format, manifestes et self-review ; lecture ROOT des35 deltas. | Serveur typesRC1 historique75, delta0 ; aucune suite globale verte. |
| T012 | Banc HTTP réel intégré :5 cas au registre, comparaison du snapshot complet et bounded entre sujets. | Le reçu préparatoire du second worker disait non exécuté ; le reçu final et le registre du principal attestent ensuite le banc. Le fil de production attendJ5. |
| T013 | R2 :reset Codex par tour/cache de compaction et mode courant ACP/Cursor après reprise inconnue ; oracles des6 bancs affectés. | Frontière fournisseur simulée ; OpenCode1/2 inchangés. |
| T014 | R2 :vrai runner/prepareClaudeMcp, candidate finale seule destinataire du prompt, attente du Query.return physique et gardes de divergence/erreur. | SDK en fixture ; qualification native sans prompt limitée aux deux paires présentes, aucune reprise existante positive ni réponse de modèle. |

Le sous-reçu préparatoire T012 reste /Users/moi/.cache/bridget-delivery169/source-snapshot/t012-developpement.md ; source /home/moi/.local/state/bridget-delivery169/t012-developpement.md. Il n'est pas utilisé seul pour revendiquer un PASS.

## Self-review XIX — Minimalisme et réutilisation

Cette section retranscrit la self-review de l'owner, rapprochée du manifeste et de la lecture ROOT. Elle ne constitue pas un nouvel audit indépendant du code.

| Axe | Constat et raison de conservation |
|---|---|
| Réglage | Un booléen fichier, pas d'UI ni d'API de patch ; une cache réglages/confiance. |
| Portage | Input par tour existant pour tous les adaptateurs, sans map de politique, mutation MCP ou fausse identité. |
| Consignes | Composition partagée et seules injonctions contradictoires modifiées ; constante OFF avec MCP conservée byte-à-byte ; enveloppe sans MCP différente en R2. |
| Pi | Unique commande privée dans extension/RPC existants. Reçu et ACK nécessaires au comportement réel du SDK ; nom réservé, corrélation et masquage ferment ce canal. |
| HTTP | Branche du principal authentifié dans le handler et le banc existants ; aucun helper produit, hausse de plafond ou troncature de champ consommé. |
| Surface et dépendances |35 fichiers incluent produit, oracles et fixtures de service. Aucune dépendance, table, service, stockage durable ou protocole public ajouté. Le nombre de fichiers n'est pas présenté comme un gain de qualité. |

Potentiel minimalisme :l'owner ne chiffre pas les lignes supprimables dans son reçu. Après lecture indépendante des35 deltas, ROOT estime **environ0 ligne de produit supprimable à comportement constant nécessaire au verdict**. Le filtre sur texte constant reste à surveiller lors des évolutions ; aucun changement n'est requis sur ce point. Cette estimation vient de ROOT et n'est pas une mesure générée par le responsable documentaire.

## Self-review XX — Responsabilité future

- L'intention et les invariants sont localisables dans le contrat et l'ADR, sans second moteur ni règle cachée. Les preuves séparent réglage, identité, montage et admission.
- Les tests couvrent des comportements :fichier préservé, identité stable, mode courant, travail de fond gardé, canal Pi fermé et réponse HTTP fidèle. Les fixtures SDK ne sont pas présentées comme un fournisseur réel.
- Le registre conserve commandes, RC, durées, SHA, erreurs initiales et reprises. Les outils, dépendances et configurations comparées sont identifiés. Le delta0 ne transforme pas le serveur en PASS.
- Aucun contournement de droits, login, compte, modèle réel ou DB de production en écriture n'est ajouté. Les limites de consigne et le coût non mesuré de snapshot:false restent explicites.
- Le mainteneur peut modifier le texte partagé, suivre le champ par tour et refaire les seuls bancs affectés. Le contrat explique les deux pièges difficiles à déduire d'un simple booléen :identité par référence et ACK Pi insuffisant.

## Preuves qui restent trop faibles pour une conclusion réelle

1. **FR017/SC003/SC009 :ouverts.** Aucun paquet installé/activé169 ni sous-tâche réelle corrélée depuis le nouveau contexte n'est attesté ici. J2–J6 ne se ferment pas sur ces tests.
2. **FR002 :ON de la cible reste à constater.** Le défaut et les transitions de test ne prouvent pas le réglage effectif du processus installé.
3. **FR006 et la partie instructionnelle de FR009 :preuve de texte/transport.** Elle ne prouve pas qu'un modèle réel respecte tous les axes ou refuse toujours un repli. L'absence d'un nouveau dispatch d'hôte est, elle, constatée dans le delta.
4. **FR010 et le rejeu de SC005 :réemploi et inspection.** Les garanties149/163/168 restent natives et inchangées ; aucun mock T3 n'a exécuté une nouvelle admission Bridget réelle.
5. **FR013 :autonomie héritée.** Aucun parcours standalone réel169 n'a été refait. Les résultats149 ne sont pas renommés en nouvelle recette169.
6. **Pi réel :non attesté.** Son absence du PATH interdit de confondre les fixtures et une exécution du binaire. Aucune installation de fournisseur n'a été ajoutée.
7. **Warning de fixture :réserve de contre-revue.** `TS377033 multipleEffectProvide` ajoute un warning de types, sans ajouter d'erreur TS. Les12 warnings lint sont identiques à la base ; cette identité ne couvre pas les warnings du journal de types.

Ces limites ne changent aucun statut de livraison. Le pipeline reste In Progress. ROOT a clos le passage2 CONVERGED après lecture du nouveau gel ; la contre-revue R2 est reçue et arbitrée ; l'audit source et J1 sont clos. Aucune nouvelle campagne n'a été lancée pour ce rapport.

## Arbitrages de contre-revue R2 — Limites C1/C2

Revue /Users/moi/.cache/bridget-delivery169/review169-code-r2.md : APPROVE_WITH_CHANGES. ROOT ne confirme pas C1 dans le SDK installé0.3.276, SHA b1607967e0dfb39a0db45f143d3b57c7a85f2f6eaa77d6bee7d8184a9830a9f3 ; preuve /Users/moi/.cache/bridget-delivery169/review169-sdk-c1.md. Pas de sessionStore T3 donc pas de flush miroir actif. Le nettoyage libère le lecteur, attend une course de sortie de2 s et réutilise sa promesse ; close ne force pas un nouveau nettoyage. Pas de timeout+close ni faux test return infini. Une absence physique absolue de processus figé n'est pas prouvée.

C2 est conservé sans code nouveau : absence=OFF garde la politique habituelle mais l'enveloppe sans MCP change, même au premier tour inconnu. runOrdinal1 n'exclut pas import/reprise. Codex reçoit t3_code_orchestration et t3_code_runtime ; contexte identique dédupliqué, coût restant non mesuré. Seule la constante OFF avec MCP a6 986 octets byte-identiques base/R1/R2, SHA1a94b3934e669d2b0175756f63143eb3982582255f08a2429e39fc5529eafb89 : /Users/moi/.cache/bridget-delivery169/source-snapshot-r2/preuves/R2/historical-OFF-check.json. Les87 transcriptions adaptées ne sont pas87 bugs produit.

Query blocked jusqu'à réouverture/changement de clé ou processus, message de divergence UI non prouvé et reprise Claude existante positive non prouvée restent des limites. Les qualifications SDK0.3.276/CLI2.1.251 et2.1.300 portent sur des sorties normales, sans modèle ; aucun crash figé qualifié. Pi absent,75 erreurs TS non GREEN, TS377033 LOW et lintR2 6/9 distinct deR1 12/35 sont conservés. P2 reste clos ; l'en-tête de tâches est ensuite actualisé documentairement sur GO ROOT, sans toucher les14 cases/descriptions. Audit et J1 clos ; J2 partiel et FR017/SC003 réel/SC009/J3–J6 ouverts.

## Checkpoint documentaire après clôture J1

J1 est clos :P2 CONVERGED sans ajout, contre-revue R2 arbitrée et audit source pass. Les12 notes par axe,42 critères et réserves exactes sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/audit169.md ; le score de pénalités100 n'est pas une qualité100 ni un GREEN global. J2 est partiel :source T3 `58ddcf3a16205a24f940164cba500089f49ec6c0` intégrée/poussée fork/local/main-20261009,35 SHA R2 identiques après hook sans bypass ; documents Bridget à committer par ROOT. J3–J6 et FR017/SC003 réel/SC009 restent ouverts. Aucun paquet169 construit, installé ou activé n'est attesté à ce checkpoint. Suite canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/169-priorite-bridget/specs/169-priorite-bridget/validation/livraison169.md.
