# Tasks — SPEC168 Routage de la flotte

Date de génération : 2026-10-15.
Branche : session-168-routage-flotte.
Source de travail exclusive : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte
Les chemins /home/moi ci-dessous sont SUR CARTAE ; le miroir /Users/moi sert uniquement aux revues.

État après Analyze :49 tâches,1cochée (T001 documentaire),48noncochées. Aucun code ni test168 exécuté par cette préparation.
Réemploi PASS, gate5/5, SHA d36e094c11cdda16d1575c1450efe98e7450c4bcbeeec169da493b4e97b785cd.
Revue du plan : r1 REQUEST_CHANGES corrigé, r2 APPROVE communiqué par ROOT ; Analyze T001 documentaire validée après corrections A1–A4 et relecture2 ; GO code/permis Cargo restent du ressort ROOT.
Aucun développeur ne commence avant ce GO. Les références du plan restent les exigences complètes ;
le périmètre ne se réduit pas à une seule histoire utilisable.

## Règles d’exécution et ownership

A initial : registre/fixture et Lineage remis puis gelés. Les contributions A publiées restent la référence ; aucune édition active A supposée.
Transfert Lineage confirmé après arrêt C sur ces surfaces : A possède delegation_lineage.rs et ses tests, daemon/native_lineage.rs, cli_lineage.rs, lineage_client.rs et les bancs Lineage associés. A réalise T029 et les assertions/projections Lineage de T034.
B natif : publication initiale protocol.rs et lineage_protocol.rs ; édition terminée, STOP confirmé.
Transfert ROOT : credits_cartae_fix possède exclusivement host_resources.rs, remote_worker.rs et fixtureworker162, ainsi que le seul ajout PrepareV2 dans protocol.rs. Le reste des types B publiés est conservé. C garde le maître et les appels probe/Prepare.
C : daemon/native_delegation.rs, daemon.rs, cli.rs, delegation_mcp.rs, communication/client.rs,
provider_limits.rs, lifecycle.rs si nécessaire et bancs CLI/native d’admission associés.
C ne modifie plus les surfaces Lineage transférées à A ; C reste seul écrivain des oracles natifs de concurrence/reprise/rollback de T030 et des scénarios d’admission T034.
C possède aussi /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lib.rs pour les seuls enregistrements de modules nécessaires.
E initial relayé à ROOT/credits_delivery_prepare : delegation.rs uniquement ; schéma, snapshot, étatsCAS et testsSQL.
D : decision_service.rs, claude_stream_json.rs et managed_session.rs ; charge/transport/délais des décideurs.
D possède aussi /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lib.rs pour les seuls exports du mode isolé.
ROOT/credits_cartae_fix peut créer /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/host_resources.rs uniquement après gate anti-doublon R09 du reuse-audit ; C enregistre alors le module.
Conserver le collecteur pub(crate) dans son module existant si cela est plus simple ; le relais ROOT en est l’unique écrivain.
C natif a remis un résultat partiel ; sa continuabilité est fermée. Son périmètre restant C passe exclusivement à ROOT/exécutant /root/admission168_finish, mêmes fichiers admission/daemon/CLI/MCP/quotas et bancs. Les surfaces Lineage restent A.
D natif est capturé et fermé. Son périmètre D passe exclusivement à ROOT/exécutant /root/deciders168_finish, avec ajout limité de /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/Cargo.toml pour activer serde_json/raw_value déjà disponible. Aucun nouveau package ni parseur artisanal.
Les lettres C/D ci-dessous restent des noms de responsabilité ; leurs écrivains actuels sont ces relais ROOT. API publique D inchangée. B reste ROOT/credits_cartae_fix ; A natif et E natif inchangés.
Ownership E actuel : ROOT/exécutant credits_delivery_prepare, exclusivement delegation.rs et reçu propre ; E natif initial reste historique. A initial registre/Lineage est gelé.
ROOT : orchestration, permisCargo, validation finale, Git, configuration réelle, installation et nettoyage.

Les chemins exacts sont répétés dans chaque tâche. Un seul écrivain par fichier.
Le lotB publie ses types et E ses signatures avant les consommateurs, par message à ROOT
et chacun dans son reçu propre developpement168-b.md / developpement168-e.md.
L’index central /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/interfaces168.md
a pour seul écrivain le propriétaire documentaire ; ni B ni E ne l’écrivent.
ROOT lui transmet les publications à consolider ; aucun copropriétaire implicite de cet index.
C réemploie le collecteur B depuis le tick d’entretien local existant ; aucun nouveau thread.
D force inherit_stderr=false ; aucun buffer stderr ajouté.
Aucune écriture Git/service/prod par les développeurs délégués hors mandat ROOT.
Un seul compilateur Cargo sur Cartae : CARGO_BUILD_JOBS=2 et deux threads de tests.
Les marqueurs [P] signifient travail possible sur fichiers disjoints lorsque les dépendances sont satisfaites ;
ils ne donnent jamais un second permisCargo.

## Sémantique des dépendances et preuves différées

Une dépendance de construction est levée lorsque l’interface ou le code consommable est publié,
même si la tâche reste non cochée faute d’exécution. Une dépendance vers un oracle signifie que
son code/fixture est prêt ; elle ne signifie jamais que son succès est anticipé.
Les tâches de test gardent leur case vide jusqu’à exécution ROOT T035, sauf permis explicite plus tôt.
T035 peut exécuter les oracles publiés avant leur cochage : aucun cycle de validation.
Aucun propriétaire ne lance Cargo sans permis ROOT, même après Analyze.

## Phase1 — Préparation et gate final



- [x] T001 Owner ROOT. Après revue R2 APPROVE et réemploi PASS, effectuer Analyze, corriger les incohérences documentaires et publier son reçu pour décision ROOT avec ownership exclusif. Le cochage n’accorde automatiquement ni droit de coder ni permis Cargo. Résultat : mapping complet35FR/11SC/7US et aucun gate bloquant. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/tasks.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/analyze168.md. Dépendance : aucune réalisation avant Analyze.

## Phase2 — Fondations communes bloquantes

TypesB et schéma/E prêts avant leurs consommateurs ; T003 est indépendant de T002.

- [x] T002 Owner B. Publier les types fermés AutoDelegateV2, workload, expected_base_commit, catalogue routing_catalogue_v1, probev2 et Lineagev4 ; capacités dédiées négociées, IDs c001..c128, efforts compatibles et défaut nullable. Préserver v1/v3/c01..c08 ; fournir à ROOT par message et reçu propre les noms/champs et oracles de sérialisation avant raccord. B n’écrit jamais l’index central interfaces168.md. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lineage_protocol.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/developpement168-b.md. Dépendances : T001.

- [x] T003 [P] Owner ROOT / exécutant credits_delivery_prepare. Créer directement le DDL168 sur base neuve ; migrer transactionnellement une table163 connue avec metadata_version1/2, CHECK16/128Kio et copieBLOB exacte puis trois index recréés. Schéma inconnu/trigger/FK inattendu refuse, rollback intégral sur faute ; aucune table moteur supplémentaire. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs. Dépendances : T001.

- [x] T004 Owner ROOT / exécutant credits_delivery_prepare. Étendre snapshot/reçu/CAS existants : canon≤512Kio, matrice entière, workload/baseSHA, score/repli/niveau/connexion et source figés ; modes SQL explicit|automatic|fallback conservés. Publier par message à ROOT et reçu propre les signatures CAS niveau/époque et admission atomique au principal, sans écrire l’index central interfaces168.md, avec contrôle de compatibilité163 et tailles. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/developpement168-e.md. Dépendances : T002,T003.

## Phase3 — US1 / P1 — Catalogue réel consultable

Test indépendant : catalogue humain/JSON12+ horsT3, garde locale attestée, aucune Task ni appel modèle.

- [x] T005 [P] [US1] Owner A. Valider routing_policy v2 et connexions référençant les lanceurs existants ; séparations moteur/modèle/machine/connexion, axes par machine et plafond statique128 sur toutes connexions×modèles déclarées, désactivées comprises, avant filtres. Registre129 refuse ; jamais un profil par modèle ou option par effort. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs. Dépendances : T002.

- [x] T006 [US1] Owner A. Consommer les capacités déclarées et les observations model/list du préflight/runtime natif existant ou de l’opérateur T043 pour les vrais noms et efforts par connexion ; default_effort membre des efforts ou null/liste vide pour défaut natif sans flag. Aucun processus depuis le loader de registre ou la lecturecatalogue, aucune nouvelle découverte automatique. Conserver provenance et entitlement inconnu, aucun medium inventé, modèle masqué ou appel à chaque modèle. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs. Dépendances : T005.

- [x] T007 [US1] Owner ROOT / exécutant /root/admission168_finish. Raccorder routing_catalogue_v1 sur le clientcontract version existant et sa feature dédiée ; garde unix_peer_uid/peer_uids mêmeUID + Role::Client + négociation. Projection publique allowlistée, aucune session d’agent obligatoire ni human_authority_guard/T3, aucune mutation ni lancement fournisseur. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/communication/client.rs. Dépendances : T002,T006.

- [x] T008 [US1] Owner ROOT / exécutant /root/admission168_finish. Ajouter à delegate167 la lecture exacte --catalogue --json avec filtre --machine et rendu humain du même snapshot. Une lecture depuis terminal humain sans identité d’agent fonctionne ; un maître sans feature retourne capability_unavailable structuré, sans commande parallèle. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli.rs. Dépendances : T007.

- [x] T009 [US1] Owner ROOT / exécutant /root/admission168_finish. Étendre les bancs existants : catalogue12/128 complet, registre129 même désactivé refusé, modèles réels séparés des labels de profils, défaut natif null, filtre machine et lecture humaine sameUID sansT3. Prouver zéro Task et zéro appel modèle, pas seulement la forme JSON. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/cli_arguments_integration_test.rs. Dépendances : T008.

## Phase4 — US2 / P1 — Axes indépendants

Test indépendant : modèle seul ou machine seule laisse les autres axes libres ; contradiction refuse sans bascule.

- [x] T010 [US2] Owner ROOT / exécutant /root/admission168_finish. Raccorder AutoDelegateV2 sur CLI/MCP existants : filtres engine/model/machine/connection_id/effort, workload fermé general|compilation et expected_base_commit hors HandoffV1. Défaut general canonique, jamais un commutateur ; le nouveau client automatique CLI/MCP émet toujoursV2 sans flag supplémentaire ni choix selon ON/OFF. Négociation v2 requise ; sans politique168 active, refus structuré deV2, jamais rétrogradation. Le chemin explicite bridget_delegate/NativeDelegate et les anciennes enveloppes AutoDelegateV1 restent inchangés pourFR-034 ; version du canon stable lors d’un toggle du même request_id. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation_mcp.rs. Dépendances : T002,T008.

- [x] T011 [US2] Owner ROOT / exécutant /root/admission168_finish. Appliquer les cinq contraintes indépendantes avant score/décideur et résoudre les axes désactivés depuis les valeurs fixes. Modèle sans effort reste éligible avec défaut natif ; effort imposé incompatible ou filtres contradictoires refusent sans resélection ni gain de droits. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs. Dépendances : T004,T006,T010.

- [x] T012 [US2] Owner ROOT / exécutant /root/admission168_finish. Prouver modèle seul puis machine seule, connexion/effort imposés, cinq axes fixés, conflit et option désactivée : aucun écart de choix, aucun décideur inutile, aucun champ injecté dans le brief fermé153. Vérifier le refusV2 explicite sur ancien maître, le refus structuré de nouvelleV2 sans politique168 active, le parcours explicite bridget_delegate/NativeDelegate et AutoDelegateV1 inchangés, puis la stabilité du canon/version au toggle d’un même request_id ; workload=general ne provoque aucune bascule. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/cli_arguments_integration_test.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs. Dépendances : T011.

## Phase5 — US3 / P1 — Répartition selon faits frais

Test indépendant : mesures bornées et score stable ; saturation fraîche filtre avant choix, inconnus préservés.

- [x] T013 [P] [US3] Owner ROOT / exécutant credits_cartae_fix. Implémenter le petit collecteur CPU/RAM/charge/capacité compilation avec libc/primitives natives déjà présentes ; échantillon mémorisé rafraîchi au heartbeat/probe existants, premierCPU=null, une collecte par machine/lot. Publier une entrée réemployable au tick local du principal, sans nouveau daemon/thread ni benchmark. Après gate anti-doublon R09, utiliser si nécessaire /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/host_resources.rs sous ownership ROOT/credits_cartae_fix ; sinon garder le collecteur dans son module existant. C seul enregistre un module nouveau. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/remote_worker.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs. Dépendances : T002.

- [x] T014 [P] [US3] Owner ROOT / exécutant /root/admission168_finish. Associer les faits quota existants aux connexions via instance/définition/époque attestées ; invalider sur changement et garder unknown neutre. C enregistre si nécessaire le module dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lib.rs sans autre modification de cette surface. Réemployer le collecteur B au tick d’entretien local existant et contrôler âge source/réception/époque des faits ; jamais de pool de comptes ni transfert de crédit. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/provider_limits.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs. Dépendances : T004,T006,T011.

- [x] T015 [US3] Owner ROOT / exécutant /root/admission168_finish. Calculer une fois les termes machine/connexion puis le score stable O(C+P)+O(C log C), bonus continuité≤20 et quota/reset bornés, workload compilation explicite. Exclure la saturation fraîche avant décideurs lorsqu’une autre machine admissible est prouvée disponible ; machine imposée saturée refuse. Oracles : inconnus, fenêtre100% avec crédits utilisables, absence de somme entre comptes et ordre stable. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/provider_limits.rs. Dépendances : T011,T013,T014.

- [x] T016 [US3] Owner ROOT / exécutant credits_cartae_fix. Prouver probev2 groupé jusqu’à128 : première mesureCPU inconnue, TTL/horloge/époque périmés, un fait machine par lot et révocation fraîche. Aucun Prepare/worktree/slot/fournisseur pendant classement ; ancien worker refuse168 précisément et conserve163. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/remote_worker.rs. Dépendances : T013,T015.

## Phase6 — US4 / P1 — Chaîne de décideurs complète

Test indépendant : trois issues service/LLM/score, niveauxOFF sans appel, aucune cible exclue réintroduite.

- [x] T017 [P] [US4] Owner ROOT / exécutant /root/deciders168_finish. Construire la charge de décision168 entière, bornée et agrégée par machine/connexion : vrais modèles/efforts, workload, CPU/RAM/charge/âges, continuité et quotas attestés. IDs opaques comme seule autorité, contraintes figées, brief utilisateur inerte, aucun secret/config/auth automatiquement injecté. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs. Dépendances : T002,T004,T006.

- [x] T018 [US4] Owner ROOT / exécutant /root/deciders168_finish. Étendre le rôle interne Claude sans Task : env_clear et allowlist de connexion, cwd enfant neutre, safe-mode/restricted/outils vides/MCP strict vide/settings vides/slash désactivé/no-persistence/schema print-streamJSON. --bare interdit ; inherit_stderr=false forcé, aucun tampon stderr, stdout64Kio et réponse4Kio bornés avant parsing détaillé, via la seule feature serde_json/raw_value existante du Cargo.toml transport ; aucun nouveau package ni parseur artisanal. Aucun outil métier/système/réseau/MCP ; seul StructuredOutput imposé par json-schema est autorisé sans exécution. Rejeter toute autre capacité/outillage/tool_use et valider localement l’objet fermé ID/effort. Les politiques administrées et authOAuth restent applicables. Export conditionnel du seul mode isolé dans /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lib.rs. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/managed_session.rs. Dépendances : T002.

- [x] T019 [US4] Owner ROOT / exécutant /root/deciders168_finish. Adapter constructeur/parseur Laya aux IDs168 et défauts d’effort sans changer163 : modèle≤32octets, état compact agrégé sans perte de FR021. Liste entière qui ne tient pas → decision_capacity_unavailable ; ni top8, omission, tournoi ou promesse128. Valider localement la sortie fermée parmi les ID/efforts admissibles ; l’exception de formatage natif StructuredOutput ne change pas le contrat HTTP. Partager deadline et permis avec le secours natif ; timeout dès lancement et permis gardé jusqu’à fin réelle. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs. Dépendances : T017,T018.

- [x] T020 [US4] Owner ROOT / exécutant /root/admission168_finish. Raccorder une seule chaîne déterministe→service→petit décideur→score avec CAS publiés par E, snapshot/repli avant consultation et choix avant admission. Niveau désactivé/inaccessible=aucun essai ; candidat unique ou axes fixés évitent les appels. Rejeter tout autre outillage/capacité/tool_use que le seul formateur StructuredOutput sans droit d’exécution et valider localement ID/effort avant CAS. Aucune Task décideur, prompt pouvant créer un candidat ou second engine. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs. Dépendances : T004,T015,T019.

- [x] T021 [US4] Owner ROOT / exécutant /root/deciders168_finish. Étendre les bancs existants pour isolation enfant sans mutation d’environnement/cwd global, flags OAuth, absence d’outils métier/système/réseau/MCP, StructuredOutput seul formateur natif admis sans exécution, rejet des autres capacités/tool_use et concordance de l’objet fermé avec structured_output/resultJSON, inherit_stderr=false, limites stdout/JSON, réponse fermée, effort nul/incompatible, timeout dès spawn, retour tardif et arrêt du seul groupe détenu sans faux permis libéré. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/managed_session.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs. Dépendances : T018,T019.

- [x] T022 [US4] Owner ROOT / exécutant /root/admission168_finish. Prouver la chaîne réelle avec serveurs/transports contrôlés : service valide sur ensemble entier compatible, skipcapacité12+ puis secours sur liste entière, deux niveauxOFF→score, cible saturée retirée impossible à réintroduire. Vérifier modèle/machine/connexion/effort réellement choisis, SC005 au-delà d’un simple test de tri et zéro second appel sur rejeu. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/native_delegation_e2e.rs. Dépendances : T020,T021.

## Phase7 — US5 / P1 — Connexions isolées déjà authentifiées

Test indépendant : deux environnementsCodex restent distincts et Claude est représentable sans gestion de secrets ou facturation.

- [x] T023 [US5] Owner ROOT / exécutant /root/admission168_finish. Propager la connexion choisie au lanceur/définition épinglés et au puits local/distant existant ; réemployer build_environment/pin_delegation_model et le recontrôle quota enfant. Ne pas copier credential, faire login, modifier facturation ou confondre ClaudeCode avec fournisseur commercial. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lifecycle.rs. Dépendances : T006,T011,T020.

- [x] T024 [US5] Owner ROOT / exécutant credits_cartae_fix. Réemployer WorkerPrepare dans l’opération additive négociée WorkerRequestOperation::PrepareV2 { prepare: WorkerPrepare, workload: AutoDelegateWorkload, connection_id: String, engine: String, expected_definition_digest: String, expected_permission_digest: String }. Résoudre localement la connexion puis relire tuple machine/moteur/connexion/modèle/effort, droits et les deux digests avant tout effet ; le modèle/effort/base restent dans WorkerPrepare. workload est obligatoire dans PrepareV2 : seul Shared hors Git général peut garder baseNone ; Worktree ou compilation exigent Git/base vérifiée. Conserver protections de chemins et mapping existants, sans configuration de mapping nouvelle. Réemployer les gardes CandidateProbeV2 ; aucun envoi168 sans capacité négociée, Prepare v1 inchangé. Connexion inconnue/désactivée ou définition révoquée refuse sans lancer un autre compte ; ne jamais importer l’environnement privé d’une autre machine. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs (seul ajout PrepareV2) ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/remote_worker.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs. Dépendances : T002,T016,T023.

- [x] T025 [P] [US5] Owner A. Prouver deux environnements Codex distincts et représentation Claude générique avec fixtures sans secrets : un lanceur expose plusieurs modèles, labels non commerciaux, défaut natif préservé et aucune entitlement inventée. Vérifier qu’une seconde connexion Cartae absente reste déclarable mais non opérationnelle sans provisionnement autorisé. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/fixtures/registry/codex-claude.json. Dépendances : T005,T006.

## Phase8 — US6 / P1 — Idempotence, reprise et explication

Test indépendant : concurrence/crash/rejeu rendent une unique admission et aucune nouvelle consultation ; décisionv4 publique et v3compatible.

- [x] T026 [US6] Owner ROOT / exécutant credits_delivery_prepare. Compléter les oracles CAS du store : propriété de niveau/époque/révision, source détaillée en metadata et mode SQL historique, collision native/auto, Task+liaison atomiques, ancienne réponse incapable de remplacer choix/repli et conservation d’un état ambigu sous erreur technique. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs. Dépendances : T004,T020.

- [x] T027 [US6] Owner ROOT / exécutant /root/admission168_finish. Étendre l’entretien/reprise existants : pending/decision_started anciens utilisent seulement le repli figé, chosen reprend son canon ; aucun service/LLM supplémentaire au rejeu ou redémarrage. Recontrôler droits/connexion/base, pause/annulation/hold160 et préserver l’erreur technique sans refus terminal fabriqué. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs. Dépendances : T020,T023,T026.

- [x] T028 [US6] Owner ROOT / exécutant credits_delivery_prepare. Prouver sur SQLite persistante migration163→168 et création neuve directe, BLOBlegacy inchangés dans chaque état, CHECK16/128Kio, refus taille, rollback copie/index, réouverture/rejeu/CAS et rétentionUNION sans double compte. ÉchecSQL reste visible ; ancien binaire interdit avec lignev2 même168OFF. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs. Dépendances : T003,T004,T026.

- [x] T029 [US6] Owner A. Exposer Lineagev4 avec source/connexion et faits publics bornés ; mode SQL mappé explicitement. Tâche168 luev3 garde placement et decision:null, tâche163c01..c08 inchangée ; repli de version strict, journal/cancel inchangés, aucun updateT3 requis. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation_lineage.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_lineage.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli_lineage.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lineage_client.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/lineage_cli_test.rs. Dépendances : T002,T004,T020.

- [x] T030 [US6] Owner ROOT / exécutant /root/admission168_finish. Prouver deux demandes concurrentes mêmeclé, enveloppe différente, réponse tardive entre niveaux, crash aux frontières pending/decision_started/chosen/admitted, rollback admission et reprise après réouverture : une Task au plus et zéro nouvelle consultation après choix/reprise ; contrôle via états durables et compteurs réels des bancs existants. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/native_delegation_e2e.rs. Dépendances : T027,T028,T029.

## Phase9 — US7 / P1 — Base exacte et récupération du travail

Test indépendant : Shared/Worktree surSHAattendu, horsGit reconnu sans confondre erreurtechnique ; résultatGit contrôlable et aucun Git de livraison du daemon.

- [x] T031 [US7] Owner ROOT / exécutant /root/admission168_finish. Résoudre le SHA source côtéCLI avant enveloppe si nécessaire, exiger expected_base_commit pour chaque candidatGit168 Shared/Worktree et figer canon/snapshot/probe. Préflight fermé distingue horsGit d’une erreur technique : absentSHA exclut le candidatGit avec workspace_base_required ; horsGit Shared général autorisé sansSHA, SHA fourni horsGit refuse mismatch, Worktree exigeGit. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation_mcp.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs. Dépendances : T010,T011,T027.

- [x] T032 [US7] Owner ROOT / exécutant credits_cartae_fix. Propager expected_base_commit du probev2 au WorkerPrepare.base_commit existant et réemployer verify_base_commit ; Worktree.base_ref résolue égale le mêmeSHA, SharedGit compare la base attendue, SHA fourni horsGit refuse mismatch. Erreur technique de préflight n’est jamais interprétée outsideGit ; révocation fraîche interdit le lancement. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/remote_worker.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs. Dépendances : T002,T024,T031.

- [x] T033 [US7] Owner ROOT / exécutant /root/admission168_finish. Prouver local+distant SharedGit/Worktree : SHA manquant/inexact, base_ref divergente et erreurI/O/timeout refusent avant fournisseur ; Shared horsGit général sansSHA fonctionne, SHA fourni horsGit refuse. Aucun HEAD distant implicite ni fallback caché ; lecteur/rejeu163 inchangés. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/cli_arguments_integration_test.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/native_delegation_e2e.rs. Dépendances : T031,T032.

- [x] T034 [US7] Owner ROOT / exécutant /root/admission168_finish pour admission/CLI et scénarios natifs ; A pour la compatibilité/projection Lineage dans ses fichiers exclusifs. Vérifier que le résultat rapporte base/branche/tête/commits/dirty et que result_available ne vaut pas intégration. Deux sous-tâches indépendantes peuvent utiliser machines/worktrees distincts ; le daemon ne fetch/push/merge ni ne retire le travail. Conserver étatkept et preuve de récupération avant cleanup autorisé. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli.rs ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/native_delegation_e2e.rs. Dépendances : T030,T033.

## Phase10 — Vérification, livraison et nettoyage

Les tests contrôlés ne remplacent pas la recette réelle ; une installation ou un commit seul ne clôture pas la livraison.

- [x] T035 Owner ROOT. Exécuter une campagne ciblée groupée après raccords stables : bancs modifiés registry/protocol/worker/store/décideurs/CLI/MCP/Lineage/native, check et format appropriés. Un seul permisCargo surCartae, jobs2 et testthreads2 ; consigner commandes, nombres réels, durées et limites, pas de filtre0compté PASS. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md. Dépendances : T009,T012,T016,T022,T025,T028,T030,T033,T034.

- [x] T036 Owner ROOT. Lancer une seule campagne workspace finale sur source stabilisée, puis uniquement replays d’échecs affectés par correction. Conserver les échecs hérités167 et leur preuve, sans élargir le périmètre ni revendiquer GREENglobal depuis des succès ciblés. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md. Dépendances : T038.

- [x] T037 Owner ROOT et owners du diff. Faire la contre-revue de code et les passes Converge prévues. Pendant chaque passe Converge, seule une écriture append-only dans Tasks est permise, code inchangé. Ajouter les objections vérifiées puis corriger hors passe et rejouer les seuls contrôles affectés avant poursuite. Mettre les rapports en forme hors passe ; couvrir minimalisme, complexité, sûreté et compatibilité sans régénération massive. Surface d’append en passe : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/tasks.md. Rapports hors passe : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/adversarial-review-code168.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/converge168.md. Dépendances : T035.

- [x] T038 Owner ROOT. Exécuter l’audit du diff avec ses JSON/scoring/validateur et limites explicites ; corriger les blocages et conserver les findings non bloquants justifiés. Vérifier qu’aucun engine, stockageauth, wrapper décoratif, scheduler ou dépendanceT3 n’a été ajouté. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/audits ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/audit168.md. Dépendances : T037.

- [x] T039 Owner ROOT. Documenter les commandes réellement livrées, connexion distincte du modèle, catalogue nonentitlement, workload/baseSHA, niveaux désactivables, formats de résultat/reprise et procédure Git entre machines. Exemples sans secret et sans promesse de synchronisation du daemon ; relire la correspondance35FR/11SC. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/docs/delegation-native.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/skills/bridget/SKILL.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/quickstart.md. Dépendances : T036,T038.

- [x] T040 Owner ROOT. Au moment de livraison seulement, porter la nouvelle fonction compatible de0.3.0 à0.4.0, notes françaises et limites de validation honnêtes. Committer le lot validé avec identité utilisateur inchangée et message type(scope): Description, sans trailer/signature automatique ; fixer le commit source exact à publier. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/Cargo.toml ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/Cargo.lock ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/CHANGELOG.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/livraison168.md. Dépendances : T039.

- [x] T041 Owner ROOT. Réemployer les scripts de build/paquet existants depuis le commit exactT040 : Linux surCartae avecjobs2, binaireMac utile, version/BUILD_ID/hashes et signaturesMac vérifiés. Vérifier contenu/sommes des archives ; ne pas reconstruire silencieusement depuis un arbre sale ni changer les scripts sans nécessité prouvée. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/scripts/build.py ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/scripts/package-089-core.sh ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/paquets168.md. Dépendances : T040.

- [x] T042 Owner ROOT. Intégrer la branche validée sur la cible autorisée, pousser et vérifier le SHA distant. Réconcilier les checkouts durables Mac/Cartae par ff-only lorsqu’ils sont propres/nondivergents ; comparer chaqueSHA et préserver toute copie sale/divergente. Distinguer le commit binaire des documents ultérieurs. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/livraison168.md. Dépendances : T041.

- [x] T043 Owner ROOT. Inventorier connexions/modèles/efforts réels surMac/Cartae sans tour payant par entrée, vérifier flagsClaude des versions installées et politiques, config par machine/accèsLaya/adaptateurLLM. Raccorder les comptes déjà authentifiés ; aucun nouveau login/copie de secret non autorisé ni secondeconnexionCartae inventée. Sauvegarder binaires/config/unités/plists et bases cohérentes ; préflight de rollback compatible. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/preflight-activation168.md. Dépendances : T041,T042.

- [x] T044 Owner ROOT. Installer et activer les paquets utiles Mac/Cartae et la politique déclarée, avec quiescence et relances contrôlées nécessaires. Vérifier build chargé, daemon/pont/worker, registre/digests, quick_check/FK0 et configuration réellementON. Préserver les PID/naissancesT3 et tunnel ; aucun job temporaire de relance en boucle. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/activation168.md. Dépendances : T043.

- [ ] T045 Owner ROOT. Exécuter les recettes réelles bornées : namespace sansT3, catalogue humain12+, contraintes modèle/machine/connexion/effort, charge/saturation, comptes isolés, LayaON sur ensemble entier compatible≤8, LayaOFF+HaikuOAuth réel sur12+, deuxOFF+score sur12+, tâche distante hostcartae et SHAexact. Vérifier résultat corrélé, gardes soldées, rejeu sans redécision ; aucune centaine d’appels modèle, aucun claimentitlementuniversel. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/recette168.md. Dépendances : T044.

- [ ] T046 Owner ROOT. Publier depuisCartae le tagannoté et la releaseSemVer v0.4.0 sur le commit exact validé, avec paquets et sommesT041. Vérifier tag/cible/assetsdistants et intégrité ; notes sans secrets, identitéGit utilisateur et limites de tests conservées. Ne pas republier ni écraser une release existante. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/release168.md. Dépendances : T042,T045.

- [ ] T047 Owner ROOT. Rassembler preuves de tests/revue/audit/build/source/activation/recettes, critères et limitations, mise à jour des statuts fondée sur les faits. Committer/pousser les documents de livraison ; aucune case cochée depuis une intention, aucun succès global inventé. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/spec.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/plan.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/tasks.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/livraison168.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/AGENTS.md. Dépendances : T038,T045,T046.

- [ ] T048 Owner ROOT. Après résultat capturé et récupération/intégration vérifiée, nettoyer branches/worktrees/TMP/stages du lot surMac/Cartae seulement. Contrôler modifications/processus/handles et absence de travail étranger, conserver sauvegardes/preuves/bases/anciensbinaires utiles ; aucun force, kill9 ou suppression de Firefox. Conserver avec motif ce qu’une garde incertaine interdit de retirer. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/nettoyage168.md. Dépendances : T042,T045,T047.

- [ ] T049 Owner ROOT. Faire le contrôle final exigencepar exigence : sources intégrées/poussées, tag/assets, binaires réellement actifs, config et recettes complètes, machines synchronisées et nettoyage justifié. Finaliser reçus/statuts puis dernier commit/push documentaire et comparaison desSHA ; déclarer livré seulement quand toutes les tâches applicables, y compris les ajouts Converge, et35FR/11SC sont réellement prouvées. Fichiers/preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/livraison168.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/tasks.md. Dépendances : T047,T048.

## Matrice de couverture35FR /11SC /7US

Les critères restent ceux de la spec corrigée ; cette matrice n’est pas une preuve d’exécution.

| Exigence | Tâches porteuses |
|---|---|
| FR-001 | T007,T008,T029,T045 |
| FR-002 | T005,T006,T010,T023 |
| FR-003 | T006,T009,T043 |
| FR-004 | T005,T009 |
| FR-005 | T010,T011,T012 |
| FR-006 | T011,T012,T015,T022 |
| FR-007 | T005,T023,T025,T043 |
| FR-008 | T014,T023,T024,T025 |
| FR-009 | T014,T015,T023,T045 |
| FR-010 | T005,T024,T025,T043 |
| FR-011 | T013,T014,T016 |
| FR-012 | T010,T013,T015 |
| FR-013 | T015,T022,T045 |
| FR-014 | T016,T034,T045 |
| FR-015 | T027,T030 |
| FR-016 | T017,T019,T020,T022 |
| FR-017 | T011,T020,T022 |
| FR-018 | T005,T010,T020,T022 |
| FR-019 | T019,T020,T022 |
| FR-020 | T005,T019,T043,T045 |
| FR-021 | T017,T018,T019,T022,T045 |
| FR-022 | T002,T019,T021,T022 |
| FR-023 | T017,T018,T021 |
| FR-024 | T018,T019,T021 |
| FR-025 | T004,T020,T026 |
| FR-026 | T026,T027,T030 |
| FR-027 | T027,T030,T045 |
| FR-028 | T004,T023,T026,T027,T028 |
| FR-029 | T010,T031,T032,T033 |
| FR-030 | T029,T034,T042 |
| FR-031 | T031,T034,T039,T042 |
| FR-032 | T034,T048,T049 |
| FR-033 | T014,T017,T029,T039 |
| FR-034 | T002,T009,T012,T016,T028,T029 |
| FR-035 | T043,T044,T045,T049 |

| Critère de succès | Tâches de preuve |
|---|---|
| SC-001 | T007,T008,T034,T045 |
| SC-002 | T005,T006,T009,T022,T045 |
| SC-003 | T011,T012,T022,T045 |
| SC-004 | T014,T023,T024,T025,T045 |
| SC-005 | T015,T022,T045 |
| SC-006 | T019,T021,T022,T045 |
| SC-007 | T018,T019,T021,T045 |
| SC-008 | T026,T027,T028,T030,T045 |
| SC-009 | T017,T018,T021,T043 |
| SC-010 | T031,T032,T033,T034,T042,T048 |
| SC-011 | T006,T009,T043,T045 |

| Histoire | Tâches dédiées | Nombre |
|---|---|---|
| US1 / P1 | T005–T009 | 5 |
| US2 / P1 | T010–T012 | 3 |
| US3 / P1 | T013–T016 | 4 |
| US4 / P1 | T017–T022 | 6 |
| US5 / P1 | T023–T025 | 3 |
| US6 / P1 | T026–T030 | 5 |
| US7 / P1 | T031–T034 | 4 |

## Dépendances et possibilités parallèles

T001 documentaire est validée ; ROOT reste seul à donner le GO d’implémentation. Après ce GO, T002(B) et T003(E) sont indépendants ;
T004 attend les deux et publie le store compatible. A T005, B T013 et D T018 démarrent dès
publication de T002 sans attendre T004 ; les raccords C attendent bien E.
La numérotation regroupe les histoires ; les dépendances explicites déterminent le démarrage,
pas l’obligation d’attendre toutes les histoires précédentes pour un fichier disjoint.

- US1 : A prépare T005→T006 ; C raccorde T007→T009 après publication des types.
- US2 : C réalise T010→T012 ; aucun autre owner n’édite ses façades.
- US3 : ROOT/credits_cartae_fix T013 peut avancer avec A et D ; C T014 attend ses interfaces, puis T015→T016 qualifient le raccord.
- US4 : D T017→T019 peut avancer dès les fondations ; C T020 attend ce contrat, puis T021/T022 ferment les risques.
- US5 : A T025 peut avancer avec D ; ROOT/credits_cartae_fix T024 attend les champs C T023, sans partager de fichierC.
- US6 : E teste T026/T028 pendant les raccords de C ; C T027/T030 ferment reprise et concurrence ; A T029 réalise la projection Lineage.
- US7 : ROOT/credits_cartae_fix T032 réemploie ses types et attend T031 ; C T033/T034 consolide les scénarios d’admission ; A couvre la compatibilité Lineage de T034.
- Final : T035 ciblés → T037 contre-revue/Converge → T038 audit et corrections → T036 FULL unique
  sur source stabilisée → T039 docs → version, paquets, intégration, activation, recette et nettoyage.
  Tout changement après revue/audit entraîne seulement les contrôles affectés et sa preuve ; pas une nouvelle FULL par défaut.

L’ownership est une frontière réelle : aucune tâche ne délègue le même fichier à deux écrivains.
Les exemples parallèles portent sur éditions/lectures ; seul ROOT distribue le permis des exécutionsCargo.

## Chemin critique et estimation restante

Estimation après le GOAnalyze, distincte de l’ETA initialeROOT09:09 :
- fondations/types/migration et interfaces :45–75min ;
- code en parallèle A/B/D/E avec raccord C :180–270min, C est le principal chemin critique ;
- validations finales, revue/audit et corrections affectées :60–100min ;
- paquets, activation, recettes réelles bornées et nettoyage :75–95min.
Total restant estimé :360–540min (6–9h), hors fournisseur indisponible ou authentification nouvelle autorisée.
Si C doit aussi reprendre seul le storeE ou le transportD, ajouter60–120min plutôt que réduire le périmètre.
ROOT recalibre l’heure de fin à partir de l’avancement réel et conserve l’ETA initiale comme historique.

## Stratégie et frugalité / ArticleXX

Chaque histoire fournit un résultat indépendamment vérifiable, mais aucune histoire ne remplace
le périmètre complet168. Aucun périmètreMVP réduit n’est proposé.
Réemploi : T002/T004/T007/T013/T018/T019/T020/T027/T029/T031/T032 étendent les puits existants.
La complexité ajoutée est justifiée par schémas fermés/versionnés, sûreté sans outils,
connexions distinctes, mesures fraîches et migration durable ; aucun wrapper décoratif.
Les preuves observables permettent au mainteneur de reprendre chaque tâche sans contexte caché.
Les données et erreurstechniques ne sont jamais transformées en autorité, quota libre ou refus métier inventé.
La fin comprend obligatoirement livraisonGit, installation utile, recette et nettoyage vérifié.


## Preuves ciblées consolidées avant Converge

Reçu canonique : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md.
57 noms distincts ont leur dernière issue PASS, dont un oracle compatibilité163 ; doublons et échecs antérieurs restent distingués.
T002–T024 et T026–T032 cochées sur publications et preuves contrôlées de la matrice.
T025 reste ouverte pour la preuve de deux environnements Codex ; T033 pour I/O/timeout Git168 non identifié parmi les57.
T034 reste ouverte : rapport local/Lineage ne prouve pas deux worktrees sur deux machines.
T035 garde ces dépendances explicites ; T036–T049 opérationnelles restent ouvertes. Aucun livré, recette réelle168 ou GREEN global déduit.
GEL documentaire pour Converge après ce reçu : aucune écriture, sauf appendTasks explicitement ordonné par ROOT.


## Convergence

### Passage 1 — 2026-10-15 — NON CONVERGED

Lecture seule des documents et des chemins produit ; aucun test, build, formatage, service, modèle ou Git write exécuté.
Les 57 dernières issues PASS du reçu T035 sont des preuves contrôlées antérieures, pas une nouvelle exécution de cette passe.
Les opérations T036–T049 restent des étapes prévues, sans succès anticipé.
Les tâches ci-dessous reprennent uniquement des exigences déjà acceptées.

- [x] T050 [US3,US4] Owner ROOT / propriétaire admission et décideurs. Compléter la charge commune réellement envoyée aux deux décideurs avec les faits de continuité dépôt/cache et les termes déclarés effectivement utilisés par le classement, sans déduire ces faits du seul score agrégé. Conserver la capacité compilation déjà présente dans HostResourceSnapshotV1 ; rendre aussi inspectable la capacité déclarée utilisée quand la télémétrie est inconnue/périmée. Figer ces faits dans le snapshot existant, garder les inconnus et les bornes de charge ; si la charge entière ne tient pas, conserver le refus de capacité sans omission. Étendre les assertions des oracles de charge/POST existants, sans nouveau banc. Preuve du manque : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:243–334 (facts limités aux références machine/connexion), :799–824 (continuité20/10 et compilation déclarée dans le score) ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/contracts/168-routing.md:168–194. Exigences : FR-012,FR-013,FR-021,T017. Dépendances : T015,T017,T020.

- [x] T051 [US6] Owner ROOT / propriétaires admission et Lineage. Exposer depuis le snapshot figé une projection publique bornée des faits utiles de la décision et de leurs inconnues, avec âge et termes du score utiles, sans environnement, chemin privé ou credential. Réemployer les façades et projections existantes ; préserver strictement les lectures v1/v2/v3 et leurs bornes. Ne pas relire les ressources fraîches pour réécrire l'explication historique et ne pas compter une reason fournisseur facultative comme preuve des faits locaux. Étendre les oracles Lineage/lecture existants pour un choix déterministe sans reason et pour des mesures inconnues. Preuve du manque : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation_lineage.rs:219–243, :883–889, :993–1000 publient seulement metadata.decision ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lineage_protocol.rs:91–110 ne contient ni faits ni inconnus ; les autres données restent privées dans RoutingAdmissionSnapshot. Exigences : FR-033,US6.4,T029 et contrat168 section8. Dépendances : T029,T050.

- [x] T052 [US4] Owner ROOT / propriétaires admission et client décision. Réemployer pour le service168 la santé et le cache négatif/cooldown163 existants, avec l'identifiant de service issu de la politique figée et la même échéance bornée santé+décision. Une indisponibilité connue doit passer au secours sans POST réessayé pendant son cooldown ; les niveaux désactivés/inaccessibles et les ensembles hors capacité gardent zéro essai. Préserver le permis jusqu'à fin physique de l'appel. Étendre les compteurs des oracles HTTP contrôlés existants pour santé négative/cooldown puis secours, sans nouveau service ni nouvelle boucle. Preuve du manque : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:1555–1563 appelle directement decide_routing ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs:333–354 fait le POST sans health/cache, alors que les gardes réemployables existent :359–399. Exigences : FR-024,T019 et plan168 section3 (réemploi des gardes santé/URL/borne/permis). Dépendances : T019,T020,T021.

- [x] T053 [US3,US7] Owner ROOT / propriétaire des oracles natifs. Compléter le scénario déjà prévu quickstart cas5 par deux demandes168 indépendantes (clés distinctes), admises sur deux worktrees coexistants issus du même SHA exact. Vérifier deux Task et deux chemins/branches distincts, la base complète de chacun, les rapports base/branche/tête/commits/dirty et l'absence d'intégration implicite. Réemployer et étendre l'oracle Git168 existant ; aucun appel modèle réel supplémentaire ni obligation de deux machines physiques pour cette preuve contrôlée. Les recettes distantes et preuves de récupération/nettoyage restent T045/T048. Preuve du manque : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:10963–11079 n'admet qu'une Task/Worktree ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs:2507–2520 prépare un Worktree et en attend exactement un. Exigences : FR-014,US3 test indépendant,spec168:74,quickstart168:62,T034. Dépendances : T033,T034.

Limites de preuve déjà prévues, sans ajout de fonction :
- T025 : les deux observations opérateur model/list Codex Mac normal/pro dans /home/moi/.local/state/bridget-delivery168/config-draft attestent deux lanceurs consultés séparément, sans thread/turn et sans entitlement garantie. Elles complètent les fixtures de déclaration ; leur rattachement documentaire peut être fait hors passe, sans nouvel appel fournisseur.
- T033 : le Git168 local réemploie source_base_commit et preflight_classified ; les chemins distants réemploient verify_base_commit/preflight. Le banc commun workspace_git_tests.rs couvre déjà sortie tronquée et délai avec arrêt de l'enfant. La passe n'exige pas de nouveau banc de timeout ; les preuves exécutées de ces tests communs et leur raccord exact restent à référencer honnêtement hors passe.
- SC-001,SC-006 réel,SC-010 livraison et SC-011 activation restent liés aux recettes et opérations T042–T049 ; aucune absence de recette encore non exécutée n'est transformée ici en nouvelle fonction manquante.

- [x] T054 [US3,US4] Owner ROOT / propriétaire des oracles natifs. Compléter la preuve SC-005 déjà demandée par T022 dans les bancs de chaîne existants : saturation fraîche malgré continuité, alternative admissible attestée disponible, puis service ou secours contrôlé tentant de réintroduire une cible exclue ; vérifier le payload effectivement envoyé et l'admission finale. Conserver aussi le refus machine imposée sans bascule et le rejeu sans nouvelle consultation. Réemployer les fixtures HTTP/transport et le collecteur déjà présents ; aucun nouveau framework ou appel fournisseur réel. Preuve de la limite actuelle : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:10845–10925 teste directement score_routing_candidates puis validate_routing_choice, tandis que :11159–11248 exerce la chaîne avec des candidats locaux sans saturation injectée. Leur combinaison ne réalise pas le scénario de chaîne exigé T022 (« au-delà d'un simple test de tri »). Exigences : FR-013,SC-005,T022. Dépendances : T015,T020,T022,T050,T052.


### Rattachement documentaire hors passe P1 — T025

GO ROOT après clôture P1 : T025 cochée sur les deux observations opérateur model/list séparées Mac normal/pro.
Sources : /home/moi/.local/state/bridget-delivery168/config-draft/model-list-codex-mac-normal.json
et /home/moi/.local/state/bridget-delivery168/config-draft/model-list-codex-mac-pro.json.
Sept modèles exposés par lanceur, initialize/model/list uniquement, aucun thread/turn ni appel d’inférence ajouté.
Ces preuves complètent les fixtures ; elles n’attestent ni entitlement, ni installation/activation168.
T033 reste ouverte : le banc Git commun a passé en163 mais sa source produit a changé ; ROOT le prévoit dans la prochaine campagne affectée groupée.
T050–T054 restent ouvertes sans preuve d’exécution. État courant :32/54cochées,22ouvertes ; P1 NON CONVERGED.
Les rapports et compteurs31/49 antérieurs décrivent leur point de capture historique.


### Preuves hors passe après corrections P1 — gel avant P2

Campagne ROOT :33PASS/0FAIL (13natifs,7décision,1Lineage,10Git communs,1Lineage transport,1LineageCLI).
Reçus : /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p1-hashes.json et les six sélections associées.
55entrées avant/après identiques ; builds --no-run transport11,609s et daemon38,675s RC0, exclus du compte de tests.
70noms uniques cumulés à dernière issuePASS après déduplication ; aucun compteur de recette réelle ou FULL déduit.
T033 cochée sur le banc Git commun réellement exécuté à l’empreinte actuelle.
T034/T053 cochées sur deux demandes/Tasks et deux worktrees coexistants, SHA identique, rapports Git complets et aucune intégration implicite ; recette sur machines physiques et récupération restent T045/T048.
T050–T052/T054 cochées après lecture de leurs assertions et reçus exacts ; T035 campagne ciblée effectuée.
État courant :40/54cochées,14ouvertes (T036–T049). T037 non cochée : le verdict P2 n’est pas anticipé.
Nouveau GEL documentaire pour P2 : aucune écriture après cette publication hors appendTasks explicitement ordonné ROOT.


## Convergence — passage 2 : compléments résiduels bornés

Relecture de code seule sur le gel P2. Les corrections T050–T054 sont présentes et les 33 PASS affectés restent des preuves ROOT antérieures, pas des tests exécutés pendant cette passe. Les opérations T036–T049 restent leurs gates futures. Aucun contenu antérieur de ce fichier n’est modifié.

- [x] T055 [US2/US4/US6] Conserver les faits de crédits attestés utiles de chaque connexion dans la charge normalisée commune aux deux décideurs et dans la projection historique publique fermée. Réemployer les observations `ProviderLimits::routing_quota` déjà liées à connexion/instance/époque et leur fraîcheur ; distinguer quota inclus épuisé avec crédits utilisables, crédits inutilisables et inconnus, sans solde opaque ni estimation financière. Le compactage `daemon/native_delegation.rs:358–370,948–985` ne transmet actuellement que used_percent/resets_at/âge et termes du score ; les crédits présents dans `plan.connections` (`:597–606`) sont perdus. `LineageDecisionQuotaV4` (`crates/bridget-transport/src/lineage_protocol.rs:157–164`) ne les expose pas non plus ; le parseur commun (`decision_service.rs:181–239`) et la projection (`delegation_lineage.rs:219–247`) ne les reconstruisent pas. Cela laisse FR-021 et l’explication FR-033 incomplètes, malgré le filtre d’admissibilité qui sait déjà tenir compte des crédits (FR-009). Étendre les oracles existants du POST/snapshot et de Lineage avec quota100%+crédits utilisables, absence/inutilisabilité, et isolation des connexions ; conserver les inconnus et le rejeu byte-identique, sans appel réel de fournisseur supplémentaire.

- [x] T056 [US3/US6] Fermer les deux limites résiduelles des nouveaux faits figés T050/T051 : transmettre et projeter CPU/RAM/charge inconnus (`null`/`unknown`) lorsque la mesure est périmée/invalide, en conservant date/âge et fresh=false ainsi que les seuls faits déclarés encore valables ; valider aussi compilation_slots dans 0..=10 à l’écriture et à la lecture du DTO public. Actuellement `daemon/native_delegation.rs:830–850` calcule fresh=false mais conserve les nombres périmés, que `:335–355` envoie au décideur ; le contrat168 §4 impose valeur périmée=null et FR-011 impose inconnue. L’oracle stale (`:11100–11122`) prouve scoreCPU0 mais ne vérifie pas l’absence du nombre périmé. `LineageDecisionFactsV4::validate` (`crates/bridget-transport/src/lineage_protocol.rs:181–232`) borne compilation_capacity et pourcentages, mais omet resources.compilation_slots, pourtant borné à10 par le producteur. Étendre uniquement les oracles existants de fraîcheur/payload et de validation Lineage pour ces null et le rejet compilation_slots=11 ; préserver la capacité compilation déclarée et l’historique figé sans horloge fraîche.

Verdict P2 : NON CONVERGED pour T055/T056 seulement. Aucune source, configuration, recette, suite complète ou service modifié/exécuté pendant cette passe. Préfixe antérieur conservé byte-identique :57988octets, SHA-256 a6daed58160e4b3b1ccaa3bc1bc6726eecfc7b106f18c2323d2b6b6d959a8f03.


### Preuves hors passe après corrections P2 — gel avant P3

ROOT a exécuté six filtres exacts modifiés : score/payload/service/connections natifs et Lineage store/transport, chacun RC0/1PASS.
Reçu agrégé : /home/moi/.local/state/bridget-delivery168/tests/t035-converge-p2-hashes.json ;55entrées avant/après identiques, all_selections_ok=true.
Builds --no-run transport11,560s et daemon36,163s RC0 ; namespace /tmp/b168p2-h4hznyr5.
T055 cochée sur assertions crédits utilisables/inutilisables/inconnus, quota100%, fraîcheur/liaison des connexions, POST et secours, projection/rejeu figés.
T056 cochée sur mesures périmées→null/unknown avec dates/âges/fresh=false, capacité déclarée conservée, DTO compilation_slots11 refusé en lecture et écriture.
Les six PASS sont des replays de noms existants :70noms uniques à dernière issuePASS restent70 ;98PASS bruts et9FAIL historiques conservés.
État courant :42/56cochées,14ouvertes T036–T049. T037 non cochée ; revue indépendante, FULL et livraison non anticipées.
Format/diffcheck ciblés C/E rapportés RC0 sur leurs fichiers finaux ; aucun statique global ou parseJSON nouveau revendiqué.
GEL documentaire P3 après cette publication : aucune écriture hors appendTasks explicitement ordonné ROOT.

## Clôture T037 — contre-revue et contrôles affectés

ROOT autorise le cochage après revue Sonnet R1 APPROVE_WITH_CHANGES sans bloquant,
clarifications L3/R1 et corrections M1/L1/L2/L4. Quatre exacts ROOT : 4 PASS, 0 FAIL.
Build bibliothèque --no-run RC 0 ; manifeste 56 entrées avant/après identiques.
P3 CONVERGED conservé ; aucune passe P4 ou nouvelle revue intégrale revendiquée.
Preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md
et /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/adversarial-review-code168.md.
État : 43 tâches sur 56 cochées ; 13 ouvertes. T038 audit et T036 FULL restent ouvertes.
GEL final documentaire pour init V15 ; aucune autre écriture sans GO ROOT.


## Clôture T036 — campagne unique conservée

ROOT a relu les reçus terminaux et leurs limites le15octobre2026. FULLunique RC101 :2820PASS/32FAIL/71ignorés, sources stables ; reprises exactes :9PASS migrations,12PASS r1 et8PASS r2. Dernière issue de29échecs initiauxPASS ; trois annulations159 nonvalidables Linux (watch_marker kqueue Darwin/BSD), sans ignore ajouté. Aucun GREENglobal. Les neuf deltas postFULL concernent les tests uniquement et ont leur contre-lecture. Le diagnostic T3 nonatteint au PASS ne ferme pas sa cause initiale inconnue.
Preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md et /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/audit168.md.
T036 clôturée sur exécution et conservation des résultats exigées, sans attribuer un succès aux trois frontières nonportables. État45/56 ; première tâche restanteT039. Les étapes Git/build/activation/recettes/release/nettoyage restent à réaliser.


## Clôture T039 — commandes et configuration documentées

Les trois documents du manifeste T039 sont publiés après CAS avant/après. ROOT a relu le diff, les flags réels et la correspondance35FR/11SC. Le contrôle documentaire git diff --check estRC0 ; aucune nouvelle campagne de tests n’est nécessaire pour ces textes. Distinctions explicites : catalogue/entitlement, machine/connexion/modèle, niveaux OFF/inaccessibles, score/repli, SHA imposé et synchronisation Git manuelle. État46/56 ; première tâche restanteT040.


## Clôture T040 — source binaire fixée

Version workspace et trois packages Bridget0.4.0, dépendances inchangées. Commit source269de64e172f71f8856373fe8b09cb2d8642da52, parent1f69e5d8e651dd27907fd244aee6f90b6b734565 ;74fichiers autorisés, blobs relus, hooks conservés, arbre propre au commit/export. L’export exact3433fichiers et ses trois liens historiques omis sont qualifiés par le reçu privé. Aucun build terminé, intégration/push ou activation déduit de ce commit. État47/56 ; première tâche restanteT041.


## Clôture T041 — deux paquets vérifiés

ROOT a recoupé les deux reçus de build, leurs binaires et archives. LinuxRC0 en89,918s ; MacRC0 en126,022s, signature stricte valide. Même commit269de64e172f71f8856373fe8b09cb2d8642da52 ; sources inchangées et installations actives préservées. Preuve : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/paquets168.md. État48/56 ; première tâche restanteT042.


## Clôture T042 — source intégrée et synchronisée

Cartae main intégré ff-only puis poussé origin/main ; Mac main avancé ff-only depuis github/main. Les deux checkouts durables sont propres sur269de64e172f71f8856373fe8b09cb2d8642da52 et le SHA distant est vérifié. Reçus privés : /home/moi/.local/state/bridget-delivery168/git/integration-pushA-cartae.json SHA256 2f0f8750f5b7a975281d11fe55a0ae06cf2280813e8fdba8d03666cd2de34fd0 ; /Users/moi/.cache/bridget-delivery168/git/integration-pullA-mac.json SHA256 1bb0f3d9741ecdc0f87d860d6b36e449910dcc4df8a386c648e4e714add03446. L’intégration n’ajoute aucun delta de code aux binaires déjà construits ; les documents postérieurs restent dans le worktree de session. Aucun fetch/push n’a été fait par le daemon. État49/56 ; première tâche restanteT043.


## Clôture T043/T044 — qualification et activation utiles

Les reçus préflight/activation ont été relus par ROOT. Deux namespaces sans modèlePASS, catalogue27(17+10), DDL168 et DBvidesqualifiées ; sauvegardes froides cohérentes et contrôles de quiescence/handles exécutés. Daemon47823/pont47926 et worker3712026 sont réellement en0.4.0 sourceA ; configONet27options, quickOK/FK0, schémas7/12, identité native/MCP/pontRunning vérifiés. T3core65544/65608 et tunnel60206 sont conservés. Premierfreeze refusé avantmutation aprèsfermeture naturelleRenderer ; garde core corrigée et contre-relue sanssignaler T3. Aucun epochprodworkerdeviné, aucune recette réelle encore déduite. Preuves : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/preflight-activation168.md et /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/activation168.md, avec chemins absolus et hashes. État51/56 ; première tâche restanteT045.


## Point T045B — preuve de délai, sans clôture de recette

T045 reste non cochée ; état 51/56. Le correctif retire le plafond service fixe de 2 s : valeur configurée 1 à 30 s dans le budget global, défaut 2 s. Oracle exact existant daemon::native_delegation::tests::spec168_service_valide_choisit_la_combinaison_complete : RC 0, 1 PASS / 0 FAIL, trois sous-cas, source stable SHA-256 5668719253fb44f96ccf28a878c952da559c0a3349f73adf0ebf56462c90da7c. Reçu /home/moi/.local/state/bridget-delivery168/tests/t045-budget-service-config-r1.json ; SHA-256 738ac4d040cd13d59d5273821b229f323aa5da2cb406a0de991b0308f6f43149 ; contre-lecture statique APPROVE selon ROOT.

La recette distante A et son rejeu ont un résultat, mais source deterministic/caller_deadline : aucun succès Laya valide déduit. Reprise privée : même Task/canon/metadata/résultat, une Task/un reçu/une exécution ; payload binaire Task différent, projection métier stable ; compteur HTTP réel non observé. Nouveau commit B, paquets, activation et trois recettes restent requis avant release. Aucun autre Cargo, FULL, scanner V15 ou P4 déduit. Détails : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/tests168.md et /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/livraison168.md.
