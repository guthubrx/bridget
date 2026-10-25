# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/tasks.md](../../../../../../specs/147-panneau-bridget-vivant/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-08 ; extensions US6–US9 approuvées le 2026-10-09. Développement : Implemented, Validé,54/54 tâches après deux passes Converge et GO final,31FR/20SC/9US. Sources committées/fusionnées/poussées ; Bridget installé/actif, T3 package prêt et activation encore pending. Les54 tâches de développement restent closes ; reçus de livraison et cleanup distincts.

<a id="extrait-2"></a>

## Extrait 2

- [x] T055 [US10] Ajouter les tests de parsing V2, états et corrélation dans t3code_contract_v2.rs ; observer RED sans toucher aux fixtures V1.
- [x] T056 [US10] Adapter détection, lectures et dispatch RPC borné dans t3code_contract.rs/t3code_contract_v2.rs ; tester auth, rejeu et panne après envoi sans repli.
- [x] T057 [US10] Adapter la lecture seule des identités natives V2 dans t3code_identity.rs, choisie par protocole actif ; tester base V1 résiduelle, session étrangère et refus d'ambiguïté.
- [x] T058 [US10] Exercer corrélation et observations existantes avec projections V2 ; tester queue, steer, erreurs et sous-agents sans faux succès.
- [x] T059 [US10] Faire une recette API sur vrai T3 isolé sans modèle ; lancer les régressions ciblées, format/clippy et construire le binaire.
- [x] T060 [US10] Relire le diff, consigner les preuves et limites puis préparer les deux paquets sans installer ni redémarrer. Aucun push/PR.

<a id="extrait-3"></a>

## Extrait 3

Ces tâches vivent dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/session-147-t3-v2.
Ordre : T055 → T056 → T057 → T058 → T059 → T060. Les54 tâches antérieures restent closes.
US10 validée :124 tests ciblés +2 tests natifs ; build release réussi. Total60/60.
Preuves et limites : validation/t3-v2.json. Activation des deux applications non effectuée.

<a id="extrait-4"></a>

## Extrait 4

- [x] T003 [P] Écrire et observer les tests RED de capacité, requête Client humaine, signal fermé version/generation/seq/status et refus ancien daemon dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-transport/src/protocol.rs ; vérifier rejet de body/title/members/UUID de fil, champs inconnus et seq hors entier sûr.
- [x] T004 [P] Écrire et observer les tests RED des schémas RPC bridget.watch et du scope de lecture dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/bridget.test.ts et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/auth/RpcAuthorization.test.ts ; aucune racine, identité agent ou propriété d'autorité ne vient du navigateur.
- [x] T005 Étendre la capacité et les enums de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-transport/src/protocol.rs, puis extraire la garde humaine commune dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs ; rendre T003 GREEN sans élargir ThreadRequest ni déclencher maintenance/identité agent.
- [x] T006 Étendre les schémas fermés et le tagstream dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/bridget.ts et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/rpc.ts ; ajouter bridget.watch au scope existant dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/auth/RpcAuthorization.ts ; rendre T004 GREEN sans nouveau transport.

<a id="extrait-5"></a>

## Extrait 5

Test indépendant : vrai watch de fixture, puis nouvelles publications/créations/clôtures et vue T3 isolée ; observer état exact et signal silencieux hors changements.

<a id="extrait-6"></a>

## Extrait 6

- [x] T007 [US1] Écrire et observer les tests RED de mutations Done versus Replayed, Create/Post/supersedes/Close, nom d'annuaire pertinent et fil étranger dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs ; inclure inscription avant ready, ready seq0 toujours premier après rafale>16 avant écriture, saturation→resync, seq overflow→nouvelle génération et aucune émission sur ACK/read/refus.
- [x] T008 [US1] Étendre le module spec145_human_view_tests de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs, fixture11032 et vrai handle_connection/CLI11308 ; observer un RED de flux147 réel daemon/socket/processus puis vérifier handshake borné, génération/ordre JSONL, absence Register/autostart, mutation/replay, interruption du pipe et ancien daemon non capable. Réutiliser les tests CLI146 de capacité/replay si nécessaire ; aucun nouveau harness et aucun kill de groupe du harness098.
- [x] T009 [P] [US1] Écrire et observer les tests RED de résolution de contexte serveur et JSONL fractionné/invalides/limites dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.test.ts ; compléter les raccords RPC réels dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/server.test.ts (filtre Bridget) et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.test.ts. Vérifier ressource enfant scoped, signal sans corps et absence de timeout unary après handshake ; les tests purs de rétention utilisent seulement le fichier state/runtime.test.ts existant si leur objet y appartient.
- [x] T010 [P] [US1] Écrire et observer les tests RED d'actualisation/coalescence dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.test.tsx ;60 secondes sans relecture périodique,100 changements avec un in-flight et un dirty au plus, état final exact et signaux obsolètes ignorés.
- [x] T011 [US1] Implémenter le registre borné et sa publication après commit réel dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs ; étendre client/CLI dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/communication/client.rs. Résultat : T007/T008 GREEN, writes hors verrou, aucun body/UUID de fil dans les signaux, aucune file non bornée.
- [x] T012 [US1] Étendre BridgetReader.watch, handler RPC et subscription native dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts, /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/ws.ts et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/runtime.ts ; réutiliser ChildProcessSpawner, supervisor et Schema, idleTtlMs0 explicite ; T009 GREEN.
- [x] T013 [US1] Consommer ready/changed/resync seulement dans le panneau visible, relire liste+détail en groupe et coalescer les versions dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx ; aucun intervalle de relecture, aucune réponse dépassée publiée comme courante ; T010 GREEN.
- [x] T014 [US1] Consigner trois preuves distinctes dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json : vrai daemon→CLI147 ; vrai binaire CLI147 exécuté par le service/RPC typé ; runtime→DOM visuel synthétique. Mesurer validation→affichage sous deux secondes au raccord effectivement testé. Vérifier mutation étrangère silencieuse et schémas exacts ; ne jamais qualifier un DOM mock de parcours réel bout en bout.

<a id="extrait-7"></a>

## Extrait 7

- [x] T015 [US2] Écrire et observer les tests RED de persistance UUID seule et clé environment/project/conversation dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/rightPanelStore.test.ts ; compléter /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.test.tsx pour A→B→A, choix hors page1, réponses tardives, ancien snapshot, panne≠refus du fil et aucun remplacement par le premier fil.
- [x] T016 [US2] Ajouter bridgetSelectionByContextKey distinct de ThreadRightPanelState dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/rightPanelStore.ts ; restaurer show/history_recent par référence dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx ; rendre T015 GREEN sans corps/titres/membres persistés ni inventaire complet.
- [x] T017 [US2] Vérifier A→B→A→B et les séparations environnement/projet avec contenu visible sans clic supplémentaire, puis contrôler la représentation persistée dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json ;0 UUID cross-context,0 corps persistés et choix correct hors première page.

<a id="extrait-8"></a>

## Extrait 8

- [x] T018 [US3] Étendre les tests RED du module spec145_human_view_tests existant de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs : binding/projet/identité/connexion révoqués, client lent, fermeture réelle et absence de mutation métier. Comparer données/cursors/verdicts et compter0 message/ACK/réveil/appel modèle produits par le watch ; réutiliser le harness de connexion/CLI local, jamais créer un second harness ou tuer un groupe de processus.
- [x] T019 [P] [US3] Écrire et observer les tests RED de refus typé terminal sans autoretry, transport transitoire avec reprise, idleTtl0 et cancel enfant réel dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.test.ts, /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/server.test.ts (filtre Bridget) et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.test.ts ; les tests purs de rétention utilisent state/runtime.test.ts existant si approprié. Dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.test.tsx, couvrir page masquée, A→B→A et seuls thread_unavailable confirmé par read choisi effaceUUID, panne/binding gardeUUID avec contenu masqué.
- [x] T020 [US3] Fermer et annuler les ressources aux refus/fin de portée, brancher reprise sur transport seulement et purge par code dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs et /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts, /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/runtime.ts, /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx ; chaque propriétaire modifie ses seuls fichiers ; T018/T019 GREEN, aucun repli agent/polling.
- [x] T021 [US3] Observer dix cycles réels ouverture/fermeture et une reprise avec publications manquées dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json ; zéro enfant/IPC/subscription restant après clôture, nouvelle génération ready à reprise, aucun doublon, contenus refusés absents et aucun retry infini sur erreur métier.

<a id="extrait-9"></a>

## Extrait 9

- [x] T022 [US4] Écrire et observer le test RED du remplacement en page2 pendant chargement page3 dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.test.tsx ; exiger acquisition S, toutes pages consultées sous to_seq=S, staging atomique et réponse ancienne ignorée. Couvrir dépliages/détails/focus/texte sélectionné et copie originale des lignes inchangées.
- [x] T023 [US4] Réconcilier uniquement le segment déjà chargé sous nouveau snapshot S dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx ; staging puis publication atomique, index séquence/UUID et clés/DOM stables, génération de pagination invalidée ; rendre T022 GREEN sans nouvelle base locale, reset visuel ou I/O par message.
- [x] T024 [US4] Faire la recette isolée autorisée pour cette session du clavier, focus, sélection de texte, dépliage, copie complète, supersession page2/page3 et pagination dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json ; une simple assertion de mocks ne prouve pas la sélection DOM. Aucun T3 ou agent actif modifié.

<a id="extrait-10"></a>

## Extrait 10

- [x] T027 [US5] Écrire et observer les tests RED none/targets/all/nom absent/multidestinations dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.test.tsx. Exiger Auteur → Noms ou Auteur · Sans sollicitation, targets effectifs plutôt que tous les membres actuels, fallback honnête avec UUID court/détail exact et zéro appel supplémentaire ; copie égale au corps original. Reçu Web RED2026-10-09 00:03:42 :5failed39skipped, filtre requested recipients.
- [x] T028 [US5] Ajouter la présentation à la seule ligne auteur/date de /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx, en utilisant notify/targets et detail.members déjà autorisés ; rendre T027 GREEN. Aucun nouveau contrat/API/annuaire, mutation, réveil, ligne dédiée ou statut de livraison inventé. Reçu Web GREEN2026-10-09 00:06:02 :116/116,72store44panel ; typecheck/lint/format exit0 et relecture locale sans nouveau finding. Auteur exclu selon GO du principal.
- [x] T029 [US5] Vérifier sur l'aperçu isolé autorisé les cinq variantes, date conservée, lisibilité multidestinations, détail exact et copie du corps, puis consigner les observations dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json ; contrôle synthétique identifié comme tel et aucune conversation active modifiée.
- [x] T030 [US5] Consigner les preuves RED/GREEN/QA et converger spec/plan/modèle/quickstart/analysis dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/implementation.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json. Cocher uniquement les reçus vérifiés, sans completion de production ni installation/commit/restart.

<a id="extrait-11"></a>

## Extrait 11

Trois lanes avec propriété stricte : Rust (crates Bridget), RPC/runtime (contrats T3, server, client-runtime), Web (store et panel T3). Les tâches [P] écrivent des fichiers disjoints entre lanes ; deux tâches de la même lane ne sont pas lancées ensemble si elles touchent le même fichier. T020 est coordonnée par propriétaire, jamais une écriture concurrente du même module.

<a id="extrait-12"></a>

## Extrait 12

Les 30 tâches sont cochées uniquement après son GO final. Preuves : Rust16PASS, transport298PASS, threads16PASS, RPC136PASS, Web116PASS, WS1PASS, qualité/builds ciblés et recettes natives attribuées. Les ignorés, avertissements préexistants et limites synthétiques restent dans results.json. Validation des audits Rust/T3 RC0, zéro erreur/alerte. Aucun finding actif. La validation isolée ne constitue ni livraison ni activation en production.

<a id="extrait-13"></a>

## Extrait 13

- [x] T035 [US7] Ajouter et observer les six groupes RED dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeMcp.test.ts et /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.test.ts : new avant sessions.set/ready/prompt, resume sans restart actif, homonymes tous scopes/statuts, opt-out/config/I/O fermé, identité héritée non vide/espaces, rejet/pending/deadline/promise tardive. Exiger t3-code conservé, query candidate fermée et consumer annulé, diagnostics codés sans secrets et zéro modèle.
- [x] T036 [P] [US7] Ajouter les RED de chemins/configuration dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeSettingsPaths.test.ts et /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetExecutable.test.ts. Vérifier scope utilisateur/projet/repositoryroot, NotFound normal versus I/O refus/métadonnée invalide, MCP fail-closed versus Skills fail-open ; priorité executable inchangée et rejet cmd/bat. Aucun profil ou variable d'identité de production modifié.
- [x] T037 [US7] Extraire les calculs existants dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeSettingsPaths.ts et /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetExecutable.ts ; réutiliser depuis /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeSkills.ts et /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts. T036 GREEN ; comportement Skills/Reader identique, trois usages read/watch/MCP, exception architecture<=3 consumers paths validée, pas nouveau service ou dépendance.
- [x] T038 [US7] Implémenter la fonction Effect de préparation native dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeMcp.ts puis son raccord ClaudeQueryRuntime dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.ts. mcpServerStatus→setMcpServers avant premier prompt/new/resume ; préserver t3-code, homonymes même failed/disabled, opt-out, skip codé identité héritée, normalpermissions/canUseTool. T035 GREEN, timeout ferme candidate/consumer et empêche montage tardif ; aucun alwaysLoad, MCPenv d'identité ou fausse autorité t3-code.
- [x] T039 [US7] Exécuter les tests ciblés et régressions adapter/Skills/Reader/runtime concernés puis typecheck/lint/build ciblés sur /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.test.ts, /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeMcp.test.ts et /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.test.ts. Consigner les commandes, compteurs et avertissements réels dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json ; aucun catalogue d'un modèle actif ou E2E modèle déduit des fixtures SDK.
- [x] T040 [US7] Exercer la façade MCP OS privée vers handle_connection/credentials/SQLite/socket dans le module spec145_human_view_tests existant de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs : identité native/no-forgery/preuve birth, catalogue, snapshots stables et cleanup. Reçus3PASS existants conservés ; plus proche ancêtre VALIDE est le contrat, aucun faux RED/fix d'identité. Compléter dans les tests spec101_identity existants de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/t3code_identity.rs : IdentityBindings.refresh réel, DBT3 provider_session_runtime privée, enfant neutre arg0claude/--session-id et PID/birth/lineage natifs. Tuple live T3/HTTP synthétique explicitement limité, pas chaîne complète T3+SDK+modèle. Ne cocher qu'après reçu de ce complément. Aucun code fonctionnel/helper/module neuf, mcp.rs, modèle, identité MCPenv, agent actif ou configuration production modifiés.
- [x] T041 [US7] Consolider les deux preuves distinctes SDK/configuration et MCP OS/autorité dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/implementation.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json, avec contre-revue sécurité/qualité et mapping FR21/22/23↔SC13/14. Conserver non-installé/non-activé ; aucun vrai catalogue runtime d'agent actif ou E2E modèle revendiqué sans preuve.

<a id="extrait-14"></a>

## Extrait 14

Dépendances US7 : gate plan/reuse principal → T035/T036 disjoints → T037 → T038 → T039 ; T040 preuve OS reçue puis complément refresh isolé t3code_identity.rs sous ownership Rust, pas du fauxSDK pour son autorité. T041 attend T039 et T040 complet. Daemon AUTH est gelé/libéré au propriétaire membership ; le complément refresh n'écrit pas daemon et ne bloque pas ce writer.

<a id="extrait-15"></a>

## Extrait 15

- [x] T042 [US8] Écrire et observer les RED du contrat AddMembers/ThreadMembersV1 (wire thread_members_v1, sérialisation snake_case) dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-transport/src/protocol.rs et des façades dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/cli.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/mcp.rs : action/champs fermés, UUID casse canonique, membres/noms, id opération, refus invalides et ancien peer. Vérifier vrai hello auxiliary combiné projects+capacité, allowlist compatible et garde daemon attestée ; aucune inscription d'identité inventée.
- [x] T043 [US8] Écrire et observer les RED métier/migration dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/store/threads.rs : créateur/ouvert/max16/doublons/invalides/projetunion/concurrence, NoChange sans reçu/clé engagée, replay unique, cursor0 et migrationsv1/v2→v3 via fixtures136 existantes, seconde ouverture no-op et rollback atomique conservant opérations/index/reçus exacts. Recheck transactionnel refuse union divergente. Comparer données exactes avant/après refus.
- [x] T044 [US8] Implémenter la transaction d'ajout et la migration ciblée CHECK v2→v3 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/store/threads.rs et le raccord handle_with_change/digest/replay/require_uuid dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs. T043 GREEN ; limite entries+16, NoChange sans consommation clé, aucun entry/body/activité réécrit, pas table ni dependency nouvelle.
- [x] T045 [US8] Implémenter capacité/hello/allowlist et contrôle d'union complète dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-transport/src/protocol.rs, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/communication/client.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs. Exclure AddMembers du chemin mutates/dispatch_thread_wakes, même ancien wakepending ; publier watch humain uniquement après commit réel, rien sur replay/refus/NoChange. T042 GREEN côté transport ; aucun wake, ACK, modèle ou activité nouvelle.
- [x] T046 [US8] Ajouter commande CLI add-members et action MCP add_members dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/cli.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/mcp.rs en réutilisant parsers/schema/résolveur. T042 GREEN côté façades ; ancien catalogue/kinds et noms/préfixes inchangés, aucun composeur T3 ou API indépendante.
- [x] T047 [US8] Exercer les fixtures privées existantes dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs : vrai CLI et façade MCP/négociation attestée, historique multipage antérieur/postérieur lu sans doublon via read/history, corps/supersessions exacts, cursor0, futur all incluant nouveau membre et targets anciens inchangés. Préparer wakepending ancien puis ajouter : aucune dispatch/Deliver ni réveil, données pending intactes ; watch autorisé seul postcommit. Exécuter non-régressions102/127/138/145/146/147, migrationsv1/v2→v3, seconde ouverture/rollback/reçus exacts et qualité ciblées. Aucun modèle ni processus actif ; preuves OS distinctes des stubs.
- [x] T048 [US8] Actualiser les docs locales /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/docs/reference-communication.md, /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/skills/bridget/SKILL.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/skills/bridget/references/commandes.md par le propriétaire membership. Consolider les reçus RED/GREEN/interop/régressions et mapping FR24–28/SC15–18 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json et implementation.md par l'owner documentaire ; contre-revue et convergence globale des extensions avant cochage final. Aucun publisher global, commit/install/restart autorisé.

<a id="extrait-16"></a>

## Extrait 16

Dépendances US8 : gate principal5/5 et Analyze2passes acceptés → T042/T043 RED puis T044/T046. Daemon AUTH gelé/libéré au propriétaire membership ; complément T040 refresh écrit seulement t3code_identity.rs. Aucun writer daemon concurrent. T048 attend les reçus T047/T049/T050 pour convergence cumulée ; aucun faux PASS global basé sur audits du premier lot.

<a id="extrait-17"></a>

## Extrait 17

- [x] T049 [US7] Observer RED dedicated disabled explicite+SDKstatus rejeté dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeMcp.test.ts puis corriger par le seul propriétaire RPC la priorité metadata avant SDKstatus dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeMcp.ts. Exiger skip fermé pour refus explicite, flags/identity sans I/O, Skills/t3-code/permissions inchangés ; GREEN ciblé+régressions/tsc/lint/build et contre-revue renouvelée. Consigner reçus dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json. Préserver T035–T041/preuves292PASS préamendement, aucun nouveau scope, modèle, configuration production ou restart. Gate principal préalable reçu ; T049 et T048 attendent Converge final.

<a id="extrait-18"></a>

## Extrait 18

- [x] T050 [US7] Observer RED prepare MCP réussi puis close ancienne query en échec dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.test.ts. Le seul propriétaire RPC corrige /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.ts : un propriétaire/finalizer local unique garde candidate/consumer jusqu'à sessions.set ; tout échec prépublication ferme candidate/queue, aucun ready/prompt ni orphelin. Pas nouveau framework/helper/module. Vérifier T049 flags avec automaticMcp:true et environnement privé sans identité héritée, sans masquer la garde par le legacy skip. GREEN ciblé+régressions/tsc/lint/build/revue puis reçus dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json. Gate principal reçu ; mêmes FR21/22/lifetime, aucun scope nouveau/modèle/profil réel. Conserver293PASS pré-lifetime, attendre Converge global avantcochage.

<a id="extrait-19"></a>

## Extrait 19

- [x] T051 [US9] Écrire et observer cinq groupes RED dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.test.tsx : row-active/foreground/aria-current-pressed et focus-ring natifs ; détails frère accessible et header h2/membres inférieur absent ; choix horspage1/filtré unique puis A→B→A ; refus/masque sans ligne épinglée ni contenu divulgué ; refresh avec mêmes nœuds seq/ScrollArea/copie et détails conservés. Tester uniquement comportements observables, pas snapshots décoratifs seuls.
- [x] T052 [US9] Modifier seulement /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx : bg-sidebar sur surface liste, row-active/foreground pour choix, hover sur autres lignes, ring focus-visible inset et état accessible. Déplacer details sous ligne choisie comme frère du bouton ; épingler une ligne unique mêmeparent/key si détail autorisé horspage/filtre, jamais sur masque/refus. Retirer header h2/membres en bas sans déplacer les nœuds messages seq/ScrollArea/copie. T051 GREEN, aucune API/store/helper/dépendance ou changement communications.
- [x] T053 [US9] Faire la recette native dans l'aperçu isolé déjà autorisé : thèmes clair/sombre, largeur360px, longtitre, choix courant/horspage/recherche/A→B→A, details/clavier/focus, masque/refus, absence header inférieur et exactcopy. Réutiliser les preuves DOM existantes de stabilité et distinguer données synthétiques de transport réel. Exécuter suites Web store/panel, tsc/lint/format/build ciblés après synchronisation avec autres lanes ; consigner reçus dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/validation/results.json, jamais de conversation/agent actif modifié.
- [x] T054 [US9] Consigner RED/GREEN/recette et mapping FR29–31/SC19–20 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/implementation.md et analysis-report.md ; revue qualité/audit diff Web puis Converge global31FR/20SC sans orphelin. Attendre aussi US8 T047/T048 et reçus finaux US7 ; cocher seulement après GO principal. Aucun nouveau publisher, installation/activation/commit/restart autorisé.
