# Audit de reutilisation de l'existant — 168-routage-flotte

## Decision

Statut: PASS
Date: 2026-10-15
Feature dir: /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte

Conclusion courte: aucune duplication évidente de service, base, ordonnanceur, moteur ou commande n'est proposée. Les extensions de wire, de snapshot et de sûreté sont nécessaires aux limites réelles du socle. Les précisions de catalogue hors T3, workload hors brief historique, SHA Shared/Worktreev2 et mode Claude compatible OAuth sont désormais arbitrées et reportées de façon cohérente dans les documents corrigés figés. Une relecture ciblée a fermé ces conditions ; aucun doublon bloquant ni arbitrage utilisateur ne reste ouvert.

Audit de réemploi unique, en lecture des sources sur Cartae. Seul ce fichier peut être écrit par l'auditeur. Aucun code, test, modèle, service, base de production, configuration ou Git n'a été modifié. Ce verdict porte sur la conception et le réemploi ; il ne qualifie pas les drapeaux Claude installés, l'exécution, l'activation ni la revue adversariale.

Périmètre lu intégralement : plan, spec, research, data-model, contrat et quickstart168. Source du tableau ci-dessous : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/plan.md, sections Architecture 1–5, migration, ownership, validation et livraison. Les numéros R01–R40 rendent chaque responsabilité extraite traçable. Les variantes de compatibilité sont auditées individuellement ; les fichiers de lot ne sont pas comptés comme une architecture indépendante.

Empreinte du plan initial audité : 75fb09bdbb3a6a431dd159a4cb4b52eddcb75bf257f3849a5f8a690dc705fc60.
Après corrections documentaires r1, relecture ciblée du plan figé : SHA-256 3078e58e0b135abe28208bc67e07085ea70acd7dcc961af63657454bb4bec401 ; contrat SHA-256 35f6b11fb992848f1a1ca538ee6251ee20eb31e5e0c8f46e6b94c20406e4a7b8. Réponses ROOT/docowner : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/plan-review168-r1.md. Les changements de conception r1 ont été relus uniquement sur ces points, sans second passage d'exploration large.
Le script /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/.specify/scripts/bash/check-prerequisites.sh est absent. Le chemin explicite fourni par ROOT et le fallback documentaire du plan ont été utilisés, sans recréer les scripts gérés.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 40 |
| Items audites | 40 |
| Reutilisations deja prevues | 33 |
| Existants potentiellement pertinents | 7 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 10 |
| Specs existantes applicables | 8 |
| Clarifications documentaires a fermer avant Tasks | 0 |

## Reutilisations correctement identifiees

Dans chaque ligne, « étendre » signifie modifier le point commun existant, pas ajouter un second parcours indépendant. « Nouveau justifié » désigne une donnée ou frontière absente ; cela n'autorise pas un nouveau framework.

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| R01 « section versionnée facultative » de registre168 | AgentRegistry, AdmissionPolicy et validation du registre | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs:109 | Étendre. routing_policy est une variante additive justifiée ; absence conserve163. Ne pas remplacer admission_policy163 ni dupliquer son chargement. |
| R02 « une connexion référence un lanceur » | AgentDefinition, pass_env, claude_config_dir et définition résolue | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs:71 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lifecycle.rs:435 | Réutiliser. Une référence opaque n'est ni un compte canonique ni un nouveau coffre d'authentification. Garder les environnements privés existants. |
| R03 catalogue Codex « model/list borné » | select_runtime et JSON-RPC request_with_timeout existants | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/codex_app_server.rs:1102 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/codex_app_server.rs:3157 | Extraire/réutiliser la lecture bornée, pas copier le client JSON-RPC. Rafraîchir dans la phase de découverte autorisée, jamais démarrer un fournisseur à chaque lecture humaine du catalogue. |
| R04 catalogue Claude déclaré et modèles/efforts | capacités du registre, validation et pin_delegation_model | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs:529 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs:585 | Réutiliser. La déclaration n'est pas une preuve d'entitlement. Pas d'alias commerciaux traduits arbitrairement ni de catalogue emprunté à une autre connexion. |
| R05 « delegate --catalogue » humain/JSON | façade delegate167, parseur et sortie existants | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli.rs:2983 | Étendre la même commande. Pas de sous-commande flotte concurrente. Rendu humain et JSON viennent du même snapshot public. Garde R06 séparée ci-dessous. |
| R07 « 128 options » et efforts compatibles | plan de candidats163 et validateurs fermés du protocole | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:33 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs:29 | Étendre par version. 128 n'est pas une hausse globale de c01–c08. L'option porte une liste d'efforts ; pas de duplication du lanceur par modèle ou de l'option par effort. |
| R08 « axes indépendants et réglages par machine » | filtres auto_admission_plan et registre de machines | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:91 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs:167 | Étendre aux cinq axes168. Une contrainte demandée reste un filtre, jamais une invitation au décideur à réparer la requête. |
| R10 « un probe groupé par worker » | CandidateProbe, candidate_probe et inspection commune à Prepare | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs:2568 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/remote_worker.rs:1816 | Étendre le même canal et les mêmes lectures. Aucun Prepare, worktree, token, slot ou fournisseur pour classer. Variante publique v2 traitée R32. |
| R11 « faits frais, époque et reçu maître » | RemoteWorkerConnection, corrélation pending_candidate_probes et contrôle des réponses | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs:1145 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs:18567 | Étendre avec received_at/TTL et snapshot hôte. Ne pas créer un registre parallèle d'état des workers. Epoch et identité gagnent sur un timestamp distant récent. |
| R14 « continuité dépôt/cache » | préflight Git et WorkspaceRecord | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/workspace_git.rs:436 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/workspace_git.rs:625 | Réutiliser les faits vérifiables et déclarations bornées. Ne pas appeler Git par candidat ni créer un watcher/fetch automatique. Aucun cache ne dispense du SHA exact ; Shared est traité R35. |
| R16 « quota par connexion rattachée à l'instance » | ProviderLimits::record_scoped/scoped_quota | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/provider_limits.rs:80 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/provider_limits.rs:115 | Étendre l'association attestée connexion→instance/définition, sans seconde mémoire de quota par nom de compte ni fusion entre machines. Une autre définition/époque invalide l'association. |
| R17 bonus bornés quota/reset | fenêtres attestées, credits.usable et règles launch_blocked | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/provider_limits.rs:186 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs:6384 | Étendre le calcul pur avec les mêmes faits. Aucun solde opaque interprété, aucun budget partagé ni somme entre comptes. Les bonus ne remplacent pas le refus bloquant. |
| R18 « chaîne de décision unique » | admit_auto_configured, finalise_auto_choice et reçu163 | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:926 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:794 | Étendre. Le score, service puis petit modèle partagent un plan figé et le même puits. Pas de deuxième saga ou de mission de routage récursive. |
| R19 service HTTP163 | DecisionServiceClient, health, decide, NetworkPermit | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs:142 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs:153 | Réutiliser santé, cache négatif, URL, contrôle de réponse et permis jusqu'au retour réel. Pas de nouveau client HTTP ou poller de santé. |
| R20 « liste entière ou niveau ignoré » Laya | constructeur et bornes attestées Laya | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs:16 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs:437 | Étendre l'adaptation sans changer implicitement JSON4096/state1536/descriptions32 ni c01–c08. Si la sémantique168 entière ne tient pas, motif de capacité ; pas top8, tournoi ou fausse compatibilité128. |
| R21 « petit décideur Claude » | ClaudeStreamJsonTransport et ManagedSession | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs:119 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/managed_session.rs:284 | Étendre avec un rôle interne borné. Pas de nouveau provider/framework, Task, AutoDelegate, gestionnaire de comptes ou pilote de protocole. Le transport seul ne prouve pas zéro outil : R23. |
| R22 « environnement/cwd neutres » | build_environment, launch_preflight et constructeur enfant existants | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lifecycle.rs:102 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/lifecycle.rs:435 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs:568 | Étendre le mode interne : env_clear et cwd du seul enfant, sans set_current_dir global. Le constructeur courant envs ne vide pas l'environnement hérité. Aucun profil de conversation réutilisé. |
| R24 « sortie fermée et flux bornés » | parseur stream-json et événements TurnFinished/Error | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs:960 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/managed_session.rs:150 | Étendre les bornes cumulées et le résultat structuré dans ce lecteur. Pas de second lecteur stdout ad hoc. Schéma JSON fermé plus validation d'appartenance au snapshot. |
| R25 « deadline et arrêt sans faux permis libéré » | ManagedSession cancel/stop, process_group, stop_owned_child et NetworkPermit163 | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/managed_session.rs:19 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/managed_session.rs:375 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/decision_service.rs:130 | Réutiliser le cycle de vie et étendre la deadline à démarrage+réponse+drain. Un timeout demandeur n'est pas une preuve d'arrêt. Pas de kill9, processus non détenu ou permis parallèle. |
| R26 snapshot versionné16/128Kio, canon512Kio | AutoAdmissionReceipt et AutoAdmissionSnapshot | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs:11 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs:232 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:43 | Étendre par version. Absence/v1 reste≤16384 et c01–c08 ; v2≤131072. Maintenir canon≤524288 et les autres budgets transport. La colonne SQL et le contenu doivent concorder. |
| R27 migration froide même table | ouverture transactionnelle du DelegationStore et schema auto_admission_receipts | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs:788 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs:852 | Étendre la migration existante. CHECK SQL16384 réel : hausse Rust seule impossible. Copie SQL froide, contrôle du schéma connu, remplacement atomique et recréation des trois index :872–877. Table de passage justifiée, aucune seconde table métier après commit. |
| R28 niveau courant, CAS et admission_pending | start_auto_decision/choose_auto_admission/admit_auto_task | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs:1428 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs:1490 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs:1596 | Étendre avec niveau/révision/époque attendus. Ne pas réutiliser Pending→DecisionStarted pour relancer un niveau déjà commencé. Un seul canon et une seule Task, réponses tardives sans autorité. |
| R29 reprise sans nouveau décideur | recover_auto_admissions, reprobe figé, batch anciennes époques et tick | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:6960 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs:1339 | Étendre l'entretien existant, sans scheduler ou file de reprise dédiée. Repli168 figé ; chosen conserve son canon. Erreur technique non terminalisée comme refus. |
| R30 puits d'admission et garde quota enfant | resolve_target, admit_delegate, delegate_remote et recheck_remote_dispatch | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:3270 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:2417 | Réutiliser. Le score/décideur n'accorde aucun droit, ne réserve pas de capacité et ne remplace pas la revalidation finale du même enfant. |
| R31 AutoDelegateV2/capacité auto_delegate_v2 | enum native et AutoDelegateV1 fermés, négociation MCP native existante | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs:2269 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation_mcp.rs:420 | Nouveau type versionné justifié, même façade/admission. Ne pas faire accepter les champs168 à v1, ni convertir silencieusement une requête168 vers163. |
| R32 CandidateProbeV2/ResultV2/capacité distincte | WorkerCapability, hello négocié et routage corrélé du probe163 | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs:2523 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs:18567 | Nouveaux DTO/variants justifiés par les schémas fermés et128IDs, même canal et inspections. Ne jamais envoyer v2 à un ancien worker ni dupliquer le handshake en service séparé. |
| R33 Lineagev4 et decision:null en v3 | LineageDecisionV3 et versionnement list/show | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lineage_protocol.rs:22 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/lineage_protocol.rs:68 | Étendre les lectures existantes. c001 ne doit jamais passer le validateurv3 c01–c08. Pas de profil163 fictif, perte de placementv2, mise à jour T3 requise ou variante de journal/cancel. |
| R34 façades CLI/MCP/projections | parseurs delegate/task167, delegation_mcp et cli_lineage | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli.rs:2983 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/cli_lineage.rs:105 | Étendre les mêmes entrées et rendus. Pas de commande parallèle ; négociation et refus de version structurés, jamais repli technique caché. |
| R36 base/branche/HEAD/résultat puis intégration autorisée | workspace_git::report et WorkspaceRecord public | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/workspace_git.rs:770 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:1504 | Réutiliser les rapports. Le coordinateur récupère/intègre le Git ; ne pas ajouter de fetch/pull/push/merge automatique du daemon ni assimiler résultat disponible à livraison. |
| R37 tests/fixtures et compatibilité | fixtures worker162, oracles163 et suites CLI167/store | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/fixtures/remote_worker_162.rs:1003 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/delegation.rs:3520 | Étendre les bancs existants avec12/128/129, migration et concurrence. Aucun nouveau runner, benchmark ou framework. Les tests prévus ne sont pas présentés comme passés par cet audit. |
| R38 paquets, activation, rollback et nettoyage | procédures et reçus de livraison167/163 | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/167-coordination-cli-autonome/plan.md:1 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/163-decision-admission/plan.md:251 | Réutiliser les gardes et binaires utiles. Migration168 interdit retour aveugle à un ancien lecteur ; sauver un ensemble cohérent. L'audit ne valide aucun runtime actuel. |
| R39 dépendances et stockage | workspace Rust, serde/serde_json, rusqlite, libc, sha2 existants | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/Cargo.toml:19 | Réutiliser. Aucun nouveau package, moteur, Prometheus obligatoire, base d'authentification ou comptabilité. |
| R40 ownership A–D et interfaces | modules registry/protocol/worker/admission/store/transport existants | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/plan.md:217 | Répartition des écritures, pas création de couches supplémentaires. Un seul écrivain par fichier ; les interfaces publiées avant consommateurs. Les anciens modules de transport ne sont pas régénérés. |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| R06 catalogue humain natif et « garde Lineage » | registre de peer UID/canal Client existe ; human_authority_guard impose un fil T3 | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs:7274 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs:7603 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs:17881 | ROOT a décidé routing_catalogue_v1 sur canal existant, avec Role::Client, version/capacité négociées et UID pair égal à geteuid. Réemployer les faits du canal et le pattern same-UID:1973 ; NE PAS appeler human_authority_guard. L'agent conserve son identité/racines ; catalogue humain public allowlisté. Décision désormais explicite dans le plan et le contrat corrigés ; lecture publique d'un snapshot existant, sans démarrage fournisseur. |
| R09 CPU/RAM/charge/occupation compilation | disk_hygiene mesure seulement espace disque ; getloadavg est présent dans un test, pas un collecteur de production | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/disk_hygiene.rs:85 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/tests/sc005_attach_budget.rs:901 | Nouveau snapshot/collecteur borné justifié. Réutiliser libc et cadence worker ; ne pas copier le banc de charge en daemon, faire passer disque pour RAM ou inventer une activité des compilations étrangères. |
| R12 exclusion saturation avant décideurs | filtre d'admissibilité163, aucun classement de ressources existant | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:580 | Étendre le filtre commun avec faits frais ; nouveau prédicat pur justifié. Une machine imposée saturée refuse. Un LLM ne peut réintroduire un candidat exclu. |
| R13 score pondéré et ordre stable | plan de candidats163 existe, pas de score CPU/RAM/continuité | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:91 | Nouveau calcul pur justifié dans le même chemin. Indexer connexions/modèles/faits, calculer une fois les termes machine ; O(C+P), tri unique O(C log C), pas de framework de scheduling. |
| R15 signal « compilation dans son brief structuré » | HandoffV1 ferme les champs et n'a aucun workload/compilation ; admission finale le valide | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/handoff.rs:64 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/handoff.rs:460 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:3545 | ROOT a décidé un champ fermé additif AutoDelegateV2 workload=general|compilation, HORS brief153. Nouveau champ justifié ; ne pas enrichir silencieusement HandoffV1 ni déduire compilation par mots-clés. Décision reportée dans plan/contrat/modèle/snapshot ; HandoffV1 reste inchangé. |
| R23 mode Claude sans outils métier/recursion | découverte actuelle = --restricted + plan ; le lecteur refuse des approbations mais ne retire pas tous les outils | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/registry.rs:1762 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/claude_stream_json.rs:1143 | Étendre le transport avec une frontière interne isolée, pas réemployer discovery comme si zéro outil était acquis. ROOT a écarté --bare car incompatible OAuth et retenu --safe-mode avec restricted/outils vides/MCP strict/settings/session/schema ; vérifier ces capacités sur2.1.251 et2.1.300 sans contourner les politiques administrées. Préqualification CLI ROOT sur Cartae2.1.251 : RC0, init.tools=[StructuredOutput] ; json-schema impose ce seul formateur sans droit d’exécution. Autoriser uniquement celui-ci, rejeter toute autre capacité/outillage/tool_use et valider localement ID/effort et sortie concordante. Aucun outil métier/système/réseau/MCP. Ce fait CLI ne prouve pas encore le transport168 codé. Nouvelle sûreté justifiée ; pas d'autre provider ni Task. |
| R35 SHA exact en Shared v2 | WorkspaceRequest::Shared{} n'a pas de SHA ; WorkerPrepare porte déjà base_commit et verify_base_commit compare HEAD quand renseigné | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs:2055 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon/native_delegation.rs:2208 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/remote_worker.rs:1800 | Arbitrage ROOT : expected_base_commit fermé v2 obligatoire pour tout workspace Git Shared/Worktree, figé/corrélé puis propagé au probe/Prepare et contrôle local. CLI lit le SHA source avant l'enveloppe ; MCP le fournit. Hors Git, Shared général peut l'omettre. Worktree réemploie base_ref résolue vers ce SHA. Ne pas modifier Shared v1, traiter HEAD exact comme preuve de dossier propre, ni garder base_commit=None pour une tâche de code censée être garantie. |

## Duplications evidentes

| Item propose | Doublon existant | Preuve | Action requise |
|---|---|---|---|
| Aucun | Aucun doublon 1:1 non justifié trouvé dans le plan | Tableau R01–R40 et journal ci-dessous | Conserver les extensions dans les puits communs. Les seules nouvelles surfaces sont des versions/données/frontières nécessaires ; aucune seconde base/service/commande n'est autorisée par cet audit. |

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| Instructions AGENTS utilisateur du fil et /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/AGENTS.md | français, chemins complets, travail serveur et ownership | Audit sur Cartae, rapports complets, aucune écriture de code concurrente. Les checkpoints historiques ne prouvent pas168. |
| /home/moi/.speckit/constitution.md, III et XVI | cycle SpecKit et worktree | Audit avant Tasks dans le worktree168 autorisé ; pas de code anticipé. |
| /home/moi/.speckit/constitution.md, XVIII | complexité explicite et pas de N+1 | Faits groupés par hôte/connexion, O(C+P), tri unique et bornes128/TTL/sorties ; aucune collecte par modèle. |
| /home/moi/.speckit/constitution.md, XIX | réutiliser avant créer, pas d'abstraction décorative | Pas de provider/runner/base parallèle. Nouveau rôle décideur justifié par isolement et compatibilité. |
| /home/moi/.speckit/constitution.md, XX | responsabilité et confidentialité | Réponse modèle sans autorité ; aucune donnée de credential, aucune preuve de disponibilité inventée. Écritures natives auth/state distinguées des écritures métier. |
| /home/moi/.speckit/constitution.md, XXI et instructions utilisateur | routines opt-in et anti-boucle | Pas de routine d'incident, trace observer ou fichier de passation bavard ajouté par cet audit. |
| /home/moi/.speckit/ref/code-quality-details.md, CXXIV/CXXV/CXXVII/CXXVIII/CXXIX | contrat, erreur, CAS, validation et dépendances | Schémas fermés, erreurs techniques visibles, migration transactionnelle, nouvelles versions explicites ; libc/rusqlite existants suffisent. |
| /home/moi/.speckit/ref/speckit-workflow.md | statuts et preuves distinctes | Spec168 reste Draft/Planned ; audit documentaire ne coche aucun test ni implémentation. |
| /home/moi/.speckit/ref/standards-tests.md et stratégie explicite de cette session | comportement et fixtures aux frontières | Tests Rust existants réemployés ; pas de framework Python/Gherkin importé dans un workspace Rust. Campagne ciblée puis FULL unique autorisée, preuves héritées séparées selon les instructions utilisateur prioritaires. |
| /home/moi/.speckit/ref/agent-orchestration.md et mandat ROOT | exploration déléguée, un seul écrivain | Un passage de recherche par cet auditeur ; seul reuse-audit.md est écrit. Pas de build/test concurrent. |

Sources projet absentes sur ce worktree : CLAUDE.md, .specify/memory/constitution.md, .specify/memory/standards.md et .specify/memory/user-layer.json. Le fallback est explicite dans la recherche168 et le plan. Aucune source gérée SpecKit n'a été recréée. Aucun accès aux secrets ni écriture DevKMS hors mandat n'a été effectué.

Minimalisme & Frugalité : nouveau service0, base métier0, ordonnanceur0, dépendance0, commande parallèle0. Les variantesv2/v4 ne sont pas des moteurs concurrents ; elles portent les contrats fermés et la compatibilité réelle. Potentiel minimalisme : ~0 ligne de code chiffrable à ce stade, aucun code168 n'existant ; éviter surtout de recopier JSON-RPC, stdout, santé/CAS et quotas. La liste R01–R40 est une couverture de responsabilités, pas une demande de40helpers.

Vertus LLM & Responsabilité Future : la charge cognitive augmente avec les versions nécessaires, mais reste contenue par une seule admission et les transports existants. Chaque nouvelle frontière a une raison explicite. Ni un prompt ni une probabilité du modèle ne vaut permission. Le volume réel et la simplicité de l'implémentation devront être relus sur le diff ; ils ne sont pas prouvés par le présent plan.

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/103-dossier-passation/plan.md (repère source effectif : handoff.rs:1) | brief de passation fermé et inerte | Ne pas transformer HandoffV1 en signal de workload implicite. Le dossier103 a été inventorié et son plan consulté ; le validateur de production lève l'ambiguïté. |
| /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/154-isolation-worktree/plan.md | worktree et base figée, rapport Git | Réemployer préflight/create/report ; pas de synchronisation Git du daemon. |
| /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/157-catalogue-quotas/plan.md:34 | catalogue et mémoire ProviderLimits | Réemployer observations et refus du lancement. |
| /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/160-reprise-quota/plan.md:41 | quota enfant, reprises durables et rondes | Pas de pool financier ou de faux zéro pour une absence de fait. |
| /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/162-execution-distante/plan.md:40 | worker/protocole/slot/époque/Prepare | Ajouter capacité/probev2 au même canal et mêmes gardes. |
| /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/163-decision-admission/plan.md:41 | admission automatique, service et reçus CAS | Étendre son puits, sa table et sa reprise ; conserver ses limites/v1. |
| /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/167-coordination-cli-autonome/plan.md:1 | façade delegate/task hors T3 | Étendre les commandes, sans alias de travail parallèle ou dépendance T3. |
| /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/149-sous-agents-lineage/plan.md (source effective : cli_lineage.rs:1 et lineage_protocol.rs:22) | lecture humaine et versions Lineage | Réemployer framing/projection/capacité ; ne pas emprunter l'autorité T3 pour le catalogue autonome. |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| Lecture de speckit-audit-existing et plan.md complet, puis spec/research/data-model/contrat/quickstart168 | /Users/moi/.agents/skills/speckit-audit-existing/SKILL.md ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte | Extraction R01–R40, aucune génération Tasks. |
| Préflight script et inventaire des sources de règles | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/.specify ; /home/moi/.speckit | Script/mémoires locales absents ; constitution globale et références pertinentes lues, fallback documenté. |
| Passage large unique : auto_admission_receipts/metadata/CAS, human guard/capabilities, CandidateProbe/epoch, plans154/157/160/162/163/167 et Cargo.toml | sources serveur du worktree168 | Table existante CHECK16384, trois index, gardes et puits communs, aucune dépendance nouvelle justifiée. |
| Lectures ciblées des occurrences et réemploi des recherches antérieures transport/registry/lifecycle | modules explicitement cités R01–R40 | Nouveaux DTO justifiés ; envs sans env_clear et discovery ne prouvent pas zéro outil. |
| Recherche CPU/RAM/getloadavg/compile_slots | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/scripts | Seulement getloadavg d'un banc de test ; pas de collecteur de production réutilisable complet. |
| Lecture handoff::validate_and_render et admission finale | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/handoff.rs:460 | Workload/compilation absent et clés inconnues refusées. |
| Lecture human_client_guard/human_authority_guard, unix_peer_uid et état de connexion | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/daemon.rs:7274 | Catalogue humain natif ne doit pas dépendre d'un fil T3. Le fait UID est déjà capturé sur le même canal. |
| Lecture WorkspaceRequest/verify_base_commit et création du WorkerPrepare | /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-transport/src/protocol.rs:2055 ; /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/crates/bridget-daemon/src/remote_worker.rs:1800 | Contrôle de SHA réutilisable ; Shared ancien ne transmet pas la base demandée. |
| Contrôle de SHA documentaire, sans mutation Git | six documents168 | Empreintes consignées ; corrections documentaires ROOT relues, SHA final plan/contrat vérifiés. |

## Arbitrage de raccord T024 — complément du 2026-10-15

Décision ROOT après lecture : WorkerPrepare v1 ne transporte pas connection_id.
Le contrôle final ne peut pas déduire une connexion depuis agent_type partagé.
R10 réemploie l’inspection Prepare et R35 la base ; ces lignes ne couvrent pas à elles seules
la preuve de connexion/digests entre probe et Prepare. L’extension additive fermée
WorkerRequestOperation::PrepareV2 { prepare: WorkerPrepare, workload: AutoDelegateWorkload, connection_id: String,
engine: String, expected_definition_digest: String, expected_permission_digest: String }
est donc justifiée. Réemployer le même canal négocié, WorkerPrepare et ses contrôles,
avec relecture locale tuple/droits/digests avant effets. Aucun nouveau runner/store ou workflow.
Modèle/effort/base restent dans WorkerPrepare ; le chemin v1 reste inchangé.
Ce complément est un arbitrage de réemploi, sans nouveau gate PASS ni preuve d’exécution.
Ownership après STOP B : ROOT/credits_cartae_fix possède worker/collecteur/fixture et
le seul ajout PrepareV2 de protocol.rs ; C garde maître/admission/façades.

## Complément de raccord — workload et borne JSON avant parsing

PrepareV2 porte workload: AutoDelegateWorkload obligatoire avant connection_id.
Le worker réemploie l’inspection/protection des chemins et le mapping existants :
Shared hors Git général peut omettre la base, Worktree ou compilation exigent Git/base vérifiée.
Aucune nouvelle configuration de mapping n’est justifiée.
Pour la limite4Kio avant parsing détaillé, activer la seule feature serde_json/raw_value
existante dans le Cargo.toml transport. R24/R39 imposent le réemploi du parseur :
aucune dépendance nouvelle ni analyse JSON artisanale. API publique D inchangée.
Relais d’écriture : /root/admission168_finish remplace C natif fermé ;
/root/deciders168_finish remplace D natif fermé et possède ce changement Cargo limité.
ROOT/credits_cartae_fix conserve worker/collecteur/PrepareV2 ; A et E natifs inchangés.
Ce complément ne crée aucune tâche, aucun gate PASS ni preuve d’exécution.

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| R06 catalogue hors T3 | réutiliser les faits Client/peer UID et ajouter routing_catalogue_v1 ; décision ROOT actée et report documentaire vérifié | Catalogue163 exige un parent. Autorité Lineage complète exige T3 ; seule l'identité du canal local est réutilisable pour ce catalogue public. | 2026-10-15 |
| R15 workload | nouveau champ fermé additif AutoDelegateV2 general/compilation, hors HandoffV1 ; décision ROOT actée et report documentaire vérifié | Aucun signal compilation existant ; modifier brief153 casserait son validateur commun. | 2026-10-15 |
| R35 SHA Shared v2 | champ expected_base_commit v2 obligatoire pour Shared/Worktree Git, absence autorisée seulement pour Shared général hors Git ; décision ROOT actée et report documentaire vérifié | Le worker peut déjà vérifier base_commit mais le maître transmet None. Une preuve de HEAD exact ne prouve pas un dossier sans modifications. | 2026-10-15 |
| Snapshot et migration | étendre même table, col version et remplacement transactionnel précis déjà prévu | CHECK SQL réel empêche une hausse Rust seule ; aucune seconde table métier permanente nécessaire. | 2026-10-15 |
| Mode décideur Claude | étendre le transport existant avec frontière de sûreté et preuve des flags | Aucune primitive actuelle n'offre zéro outil. Ce besoin réel justifie l'extension, pas un nouveau provider. | 2026-10-15 |
| Versions de wire et Lineage | nouveaux DTOv2/v4 sur le même canal, v1/v3 inchangés | Les schémas actuels sont fermés ; 128IDs incompatibles avec c01–c08 ne doivent pas passer en cachette. | 2026-10-15 |
| Capture de faits et score | nouveau calcul/collecte bornés dans worker/admission existants | Aucun collecteur CPU/RAM/charge de production trouvé ; pas de service système ou framework supplémentaire. | 2026-10-15 |

| OAuth et mode Claude | remplacer --bare par --safe-mode et les restrictions explicites ; ne pas conserver stderr | ROOT a vérifié les aides locales : --bare exige une clé API, incompatible avec la connexion OAuth visée. Les bornes stdout et l'arrêt détenu restent obligatoires ; aucune transcription stderr privée à stocker. | 2026-10-15 |
| SQL neuf et legacy | DDL168 pour DB neuve, migration163 connue pour DB existante, mode SQL garde explicit/automatic/fallback | La source détaillée168 appartient aux métadonnées versionnées, pas à une nouvelle valeur rejetée par le CHECK SQL historique. | 2026-10-15 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

Conditions fermées avant Tasks : R06 catalogue natif same-UID/Client sans T3 ; R15 workload fermé v2 hors brief153 ; R35 expected_base_commit obligatoire pour Shared/Worktree Git, corrélé au snapshot/probe/Prepare et recontrôlé localement ; mode Claude safe-mode/OAuth et stdout borné sans conservation stderr ; DDL168 neuf et trois modes SQL historiques conservés. Aucun autre doublon à arbitrer, aucune nouvelle question humaine exigée.

PASS signifie passage possible à la génération des tâches après la revue de plan gérée par ROOT. Il ne transforme pas les capacités Claude projetées, les mesures CPU/RAM, la migration SQL ou les recettes de décision168 en preuves d'exécution.


## Complément revue code R1 — coefficients et canon

Arbitrage ROOT, 2026-10-15 ; aucune nouvelle tâche ni nouvelle exécution.
RoutingScoreWeights dans le registre représente seulement les sept coefficients privés fermés
déjà demandés par R13 et le contrat : cpu/ram/load/continuity/compilation/quota/reset.
Aucun type équivalent de coefficients n’a été trouvé dans le registre ; les seuils de saturation
préexistants sont des conditions d’admissibilité, pas des poids. Réemployer la politique/snapshot
et le calcul pur commun. Aucun service, couche, dépendance ou stockage supplémentaire.
Les poids de la coordinatrice s’appliquent à toutes les cibles ; seuils et fraîcheur restent par cible.
Bornes/défauts20/20/20/20/10/5/3, arrondi CPU/RAM/charge au plus proche ; pénalité100 fixe.
E finalise skip_serializing_if pour weights==Default : omission du JSON canonique afin de
préserver le digest d’un ancien snapshot168 ; non défauts sérialisés et figés.
C a publié calcul/horloge/note/aide selon ROOT. La garde source≤réception des mesures reste
requise avant validation ; aucune lecture théorique ou publication de code ne vaut résultat de test.
Ce complément précise R13 sans nouveau grade, gate PASS ou audit global.


## Complément audit T038 — correction des raccords, validation future

Le probe groupé ne suffisait pas : la boucle worker répétait origin/HEAD/préflight Git par modèle.
Le RED instrumenté ROOT a compté 521 appels Git réels pour 128 options et 2 tuples.
Réemploi R14 : cache éphémère des seuls faits Git par tuple complet dans le lot, succès et erreurs
compris. Le tuple distingue worker/époque/provision, projet/révision mapping, racines/cwd/identité Git,
workload, workspace/base_ref/branche et SHA. Aucun cache de droits par simple chemin.
La connexion, la définition, le modèle, l’effort et les droits restent individuels ; les gardes
fraîches Prepare/Start déjà présentes restent l’autorité. Pas de watcher, service ou cache global.

Le premier contrôle distant du puits Worktree ne doit pas changer sa stratégie en Shared :
PrepareV2 ajoute inspect_only, false par défaut et absent de V1. En true, vérifier le Worktree
futur/base_ref/SHA et renvoyer WorkerPrepared avec token réservé inspect-only-v1 sans état Prepared,
worktree ni slot. Start refuse ce token ; second Prepare normal après Task durable.
Ce champ fermé étend le canal et les gardes existants ; aucun runner, store ou saga supplémentaire.

Réemploi R25 : passer le même Instant absolu admission→préparation→scheduling→I/O, sans renouveler
un budget relatif. Réemploi R11/R24 : si l’effort exige une seconde sonde, comparer définition
brute, époque et mapping avec la première observation fraîche validée ; droits peuvent différer
avec l’effort, jamais la session ou la définition par dérive silencieuse.

Ces arbitrages répondent aux défauts d’audit. Les corrections et leur exécution finale ne sont
pas encore attestées ici : aucun finding fixed, nouveau gate PASS ou audit global revendiqué.
T038 demeure ouverte ; ses recettes/livraison ultérieures ne sont pas déduites.
