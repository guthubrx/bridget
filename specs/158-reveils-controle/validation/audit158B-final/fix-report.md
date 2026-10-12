# Corrections158B

CinqMrésolus,MIN001LOWconservé ;ROOTseulWT/Git,AUTO_COMMITfalse,commit_sha:null honnête. MAX_CYCLES1/MAX_FIXES10 :C1puisscoringreadonly,pasCargo neuf.

[
  {
    "original_id": "PERF-001",
    "original_fingerprint": "a6dae79804fa",
    "original": {
      "id": "PERF-001",
      "category": "performance",
      "subcategory": "algorithmic_complexity",
      "severity": "MEDIUM",
      "file": "crates/bridget-daemon/src/communication/client.rs",
      "line": 924,
      "title": "La nouvelle façade wake_request ne documente pas sa complexité",
      "description": "Cette fonction pub(crate) constitue la frontière CLI/MCP des réveils. Elle choisit une voie d'identité et effectue plusieurs échanges, mais ses commentaires ne précisent pas la complexité exigée par le module04. Le constat est documentaire ; aucune lenteur n'est démontrée.",
      "evidence": "/// Frontière158B : identité auxiliaire pour MCP ; rôle client pour CLI local.\n/// Les actions administratives gardent la négociation de contrôle existante.\npub(crate) fn wake_request(\n    identity: Option<(&str, &str)>,\n    socket: &Path,\n    request: bridget_transport::protocol::WakeRequest,\n) -> Result<bridget_transport::protocol::WakeResult, ClientError> {",
      "exploitability": "N/A",
      "impact": "Coût de sérialisation et distinction entre calcul et attente socket non explicités pour cette nouvelle API.",
      "path_type": "hot",
      "remediation": "Ajouter O(B), B total des octets de requêtes/réponses, avec nombre constant d'échanges ; distinguer l'attente I/O.",
      "remediation_code": "/// Complexité O(B), B octets échangés ; nombre constant d’échanges, attente I/O bornée par le client.",
      "verification": "Relire wake_request et les échanges existants pour confirmer B. Commentaire seul : aucun nouveau test ou benchmark.",
      "effort": "XS",
      "regression_risk": "LOW",
      "status": "open",
      "references": [
        "Constitution Article XVIII",
        "audit-v14 module04"
      ],
      "fingerprint": "a6dae79804fa"
    },
    "status": "resolved",
    "proof": {
      "file": "crates/bridget-daemon/src/communication/client.rs",
      "line": 922,
      "evidence": "/// Frontière158B : identité auxiliaire pour MCP ; rôle client pour CLI local.\n/// Les actions administratives gardent la négociation de contrôle existante.\n/// Coût O(B), B octets sérialisés et lus ; nombre constant d'échanges.\n/// Cette borne ne décrit ni la latence réseau ni le temps de réponse du daemon.\npub(crate) fn wake_request(\n    identity: Option<(&str, &str)>,\n    socket: &Path,\n    request: bridget_transport::protocol::WakeRequest,\n) -> Result<bridget_transport::protocol::WakeResult, ClientError> {",
      "reason": "Annotation O(B), octets sérialisés/lus et nombre constant d’échanges ; latence explicitement exclue.",
      "sha256": "7bf47bcacaba52300ecc5689fee9cc311b9822b7358b2be1c20d674c5585aee9",
      "read_mode": "fresh_cycle_scoring",
      "verified_at_utc": "2026-10-12T12:50:20.505653+00:00"
    },
    "resolution_report": "/Users/moi/.cache/bridget-audit158B-20261012/session-2026-10-12-spec-158B-01/cycle-scoring/findings_complexite.json",
    "report_sha256": "28227f41232db3a18bd7817509bcd74636f103d861881b0e9b67f2ca6d35ecb1",
    "commit_sha": null
  },
  {
    "original_id": "PERF-003",
    "original_fingerprint": "80645aaff568",
    "original": {
      "id": "PERF-003",
      "category": "performance",
      "subcategory": "algorithmic_complexity",
      "severity": "MEDIUM",
      "file": "crates/bridget-daemon/src/store/observation_missed.rs",
      "line": 168,
      "title": "Le coût des propriétaires est absent de l’annotation du lot",
      "description": "L'annotation publique annonce O(F+B log F). Pour chaque nouveau propriétaire, ensure_owner compte les lignes owner et, à la borne8192, cherche une ligne retirable par date. Ce travail sur O propriétaires, distinct des F faits, est absent du coût annoncé. Les deux collections sont bornées : aucune saturation ou latence excessive n'est démontrée.",
      "evidence": "    /// O(F+B log F), F<=8192 ; snapshotonce et provenance avantenqueue.\n    pub(crate) fn save_observation_batch(\n        &mut self,\n        snapshot: Option<&[u8]>,\n        notices: &[Notification],\n        now: i64,\n    ) -> Result<Vec<String>, StoreError> {\n        if snapshot.is_some_and(|b| b.len() > 262144) {\n            return Err(StoreError::Invariant(\"snapshot observation trop grand\"));",
      "exploitability": "N/A",
      "impact": "Documentation incomplète du parcours des8192 propriétaires possibles dans les nouvelles attributions.",
      "path_type": "hot",
      "remediation": "Inclure explicitement O, propriétaires≤8192, dans la borne prudente du commentaire ; conserver les contrôles et l'éviction.",
      "remediation_code": "/// Coût borné par F faits et B notifications, plus les parcours de O propriétaires par nouvel owner ; F/O<=8192.",
      "verification": "Relire ensure_owner111–148 et la boucle197 ; confirmer8192. Documentation seulement, aucun nouveau test/index.",
      "effort": "XS",
      "regression_risk": "LOW",
      "status": "open",
      "references": [
        "Constitution Article XVIII"
      ],
      "fingerprint": "80645aaff568",
      "supporting_evidence": {
        "file": "crates/bridget-daemon/src/store/observation_missed.rs",
        "line": 122,
        "evidence": "    let count: i64 = tx.query_row(\n        \"SELECT COUNT(*) FROM observation_missed_owners WHERE kind='owner'\",\n        [],\n        |r| r.get(0),\n    )?;\n    if count >= MAX_OWNERS {\n        let old: Option<(String, i64)> = tx\n            .query_row(\n                \"SELECT owner,last_fact_or_gap_at FROM observation_missed_owners o"
      }
    },
    "status": "resolved",
    "proof": {
      "file": "crates/bridget-daemon/src/store/observation_missed.rs",
      "line": 169,
      "evidence": "    /// O(F+B*(log F+Cowner)), Cowner=O(O*(log F+log O)), F/O<=8192.\n    /// Snapshotonce et provenance committés avant enqueue, même sans avis.\n    pub(crate) fn save_observation_batch(\n        &mut self,\n        snapshot: Option<&[u8]>,\n        notices: &[Notification],\n        now: i64,\n    ) -> Result<Vec<String>, StoreError> {",
      "reason": "Cowner est explicite, dépend des owners et probes de faits ; F/O<=8192 documenté.",
      "sha256": "17004b1c61a01e14d72e9a52cb76ac182ff127e62cd391bccf707f43714f26a9",
      "read_mode": "fresh_cycle_scoring",
      "verified_at_utc": "2026-10-12T12:50:20.505653+00:00"
    },
    "resolution_report": "/Users/moi/.cache/bridget-audit158B-20261012/session-2026-10-12-spec-158B-01/cycle-scoring/findings_complexite.json",
    "report_sha256": "28227f41232db3a18bd7817509bcd74636f103d861881b0e9b67f2ca6d35ecb1",
    "commit_sha": null
  },
  {
    "original_id": "MIN-001",
    "original_fingerprint": "096f382ea2c7",
    "original": {
      "id": "MIN-001",
      "category": "minimalism",
      "subcategory": "unused_parameter",
      "severity": "LOW",
      "file": "crates/bridget-daemon/src/store/observation_missed.rs",
      "line": 285,
      "title": "Deux paramètres d’horloge ignorés dans les mutations missed",
      "description": "finish_observation_write et observation_owner_presence reçoivent un paramètre _now jamais lu, aux lignes285 et423. Les corps complets ne prennent aucune décision à partir de cette valeur. Les9 appels de la première fonction et11 de la seconde alimentent pourtant cet argument ; les délais sont traités par maintain_observation_missed ou freeze_observation_summary. Ces APIs pub(crate) sont nouvelles, sans compatibilité publique à préserver ni justification écrite trouvée pour ces arguments.",
      "evidence": "    pub(crate) fn finish_observation_write(\n        &mut self,\n        owner: &str,\n        id: &str,\n        outcome: ObservationWriteOutcome,\n        _now: i64,\n    ) -> Result<bool, StoreError> {",
      "exploitability": "N/A",
      "impact": "Deux arguments inutiles font croire que ces mutations dépendent de l’horloge ; aucun coût runtime ou défaut de délai n’est revendiqué par ce finding.",
      "path_type": "hot",
      "removable_lines_estimate": 2,
      "remediation": "Retirer uniquement _now de ces deux signatures internes et le dernier argument de leurs9/11 appels réels, sans modifier les corps, les transactions, les deadlines ou les oracles existants.",
      "verification": "ROOT peut rejouer les groupes existants observation158b_g8_provenance_once_writer_et_transaction, observation158b_g9_bornes_bytes_ttl_proprietaires_et_lacunes et observation158b_g10_retour_cutoff_resume_uuid_et_ack_cas ; mêmes phases, cutoffs et résultats. Aucun test exécuté par cet auditeur.",
      "effort": "XS",
      "regression_risk": "LOW",
      "references": [
        "Constitution ArticleXIX.1",
        "Auditv14 module12 checklist1"
      ],
      "status": "open",
      "fingerprint": "096f382ea2c7"
    },
    "status": "persisting",
    "proof": [
      {
        "file": "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/158B-reveils-controle/crates/bridget-daemon/src/store/observation_missed.rs",
        "line": 290,
        "sha256": "17004b1c61a01e14d72e9a52cb76ac182ff127e62cd391bccf707f43714f26a9",
        "evidence": "    pub(crate) fn finish_observation_write(\n        &mut self,\n        owner: &str,\n        id: &str,\n        outcome: ObservationWriteOutcome,\n        _now: i64,\n    ) -> Result<bool, StoreError> {"
      }
    ],
    "resolution_report": "/Users/moi/.cache/bridget-audit158B-20261012/session-2026-10-12-spec-158B-01/cycle-scoring/findings_minimalisme.json",
    "report_sha256": "af9037a17bd8521687f4ed609f87d611fd1b8b8bf58452dc26b356e116283042",
    "commit_sha": null
  },
  {
    "original_id": "PERF-002",
    "original_fingerprint": "a9af7e4637a8",
    "original": {
      "id": "PERF-002",
      "category": "performance",
      "subcategory": "redundant_sql_empty_batch",
      "severity": "MEDIUM",
      "file": "crates/bridget-daemon/src/store/observation_missed.rs",
      "line": 178,
      "title": "Un lot vide sans snapshot ouvre une transaction et agrège les faits",
      "description": "observe_fact_at appelle le dépôt même quand les notifications sont vides et la révision inchangée, donc snapshot=None. save_observation_batch lit/règle le timeout, ouvre une TXImmediate, puis exécute COUNT et GROUP BY sur les faits conservés. La boucle vide ne produit aucune mutation utile. Le travail est borné par8192 faits, mais se répète sous le verrou commun à chaque fait concerné.",
      "evidence": "            let mut total: i64 =\n                tx.query_row(\"SELECT COUNT(*) FROM observation_missed\", [], |r| r.get(0))?;\n            let mut counts: HashMap<String, i64> = {\n                let mut q=tx.prepare(\"SELECT subscription_id,COUNT(*) FROM observation_missed GROUP BY subscription_id\")?;\n                q.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?\n                    .collect::<Result<_, _>>()?\n            };\n            let mut accepted = Vec::new();\n            for notice in notices {",
      "exploitability": "CONFIRMED",
      "exploitation_scenario": "Conserver des faits missed puis traiter un fait par observe_fact_at sans abonnement correspondant, ou un turn_ended volontairement journal_only. Observations::observe rend une liste vide sans modifier la révision ; snapshot=None et queue_observation_notifications_at appelle save_observation_batch(None,&[],now). Constat de branches lues, aucune exécution runtime.",
      "impact": "Transaction et deux agrégations SQL inutiles sur le chemin de réception ; durée CPU/latence non mesurée.",
      "path_type": "hot",
      "remediation": "Retourner Ok(Vec::new()) au début du dépôt uniquement si notices.is_empty() && snapshot.is_none(). SnapshotSome doit toujours être committé, notamment après once. L'entretien reste dans observer_return_tick.",
      "remediation_code": "if notices.is_empty() && snapshot.is_none() {\n    return Ok(Vec::new());\n}",
      "verification": "ROOT vérifie les oracles T116 prévus et la persistance once avec snapshotSome. Lecture attendue : aucune TX/COUNT/GROUP BY pour lot vide sans snapshot ; commit conservé pour Some. Aucun banc/test de confort demandé.",
      "effort": "XS",
      "regression_risk": "LOW",
      "status": "open",
      "metric_impacted": "Nombre de transactions/agrégations SQL et temps CPU ; latence mesurée inconnue.",
      "qualitative_gain": "Lot vide sans snapshot : deux agrégations et une TX évitées ; coût O(F) supprimé sur cette branche.",
      "measured_latency_ms": null,
      "rollback": "Retirer seulement cette garde si les oracles existants révèlent un effet utile perdu ; conserver droits/provenance.",
      "references": [
        "daemon.rs:2833–2885",
        "observation.rs:456–537",
        "daemon.rs:2944 maintenance indépendante"
      ],
      "fingerprint": "a9af7e4637a8"
    },
    "status": "resolved",
    "proof": {
      "file": "crates/bridget-daemon/src/store/observation_missed.rs",
      "line": 177,
      "evidence": "        if notices.is_empty() && snapshot.is_none() {\n            return Ok(Vec::new());\n        }\n        if snapshot.is_some_and(|b| b.len() > 262144) {\n            return Err(StoreError::Invariant(\"snapshot observation trop grand\"));\n        }\n        let timeout: u64 = self\n            .conn\n            .query_row(\"PRAGMA busy_timeout\", [], |r| r.get(0))?;",
      "reason": "Retour None+empty avant PRAGMA/TX ; snapshotSome rejoint INSERT192 et commit276, entretien distinct daemon3008.",
      "sha256": "17004b1c61a01e14d72e9a52cb76ac182ff127e62cd391bccf707f43714f26a9",
      "read_mode": "fresh_cycle_scoring",
      "verified_at_utc": "2026-10-12T12:50:20.505653+00:00"
    },
    "resolution_report": "/Users/moi/.cache/bridget-audit158B-20261012/session-2026-10-12-spec-158B-01/cycle-scoring/findings_performance.json",
    "report_sha256": "059e263810c3f1df98611f21bc8ae2f9c0aa14e2dfcf94eadd4158c7e06542cc",
    "commit_sha": null
  },
  {
    "original_id": "TEST-001",
    "original_fingerprint": "b36720886694",
    "original": {
      "id": "TEST-001",
      "fingerprint": "b36720886694",
      "category": "tests",
      "subcategory": "planned_attested_cli_frontier_missing",
      "severity": "MEDIUM",
      "file": "crates/bridget-daemon/src/cli.rs",
      "line": 10337,
      "title": "Le groupe7 ne prouve pas le réveil de fil headless attesté",
      "description": "T111 et T113 exigent le parcours CLI-fil avec identité auxiliaire et credential réels. Le test G7 actuel exerce seulement une commande bare vers agent-connu et un faux serveur RoleClient. G6 traite Schedule Thread depuis une connexion primaire ; les autres tests de credentials ne remplacent pas ce parcours CliOnce auxiliaire.",
      "evidence": "            let (output, wire) = guidance156_cli_run(\n                \"wake158b_cli_g7_formes_fermees_et_identifiant_retry\",\n                &[\"agent-connu\", \"--label\", \"libellé\", \"--id\", id],\n            );\n            assert_eq!(output.status.code(), Some(exit_code), \"{id}: {output:?}\");\n            let wire: serde_json::Value =\n                serde_json::from_str(&wire.expect(\"WakeRequest réellement reçu\")).unwrap();",
      "exploitability": "N/A",
      "impact": "La validation de T111/T113 reste partielle : le raccord CLI identité auxiliaire vers fil, et son refus avant réservation sans contexte, ne sont pas prouvés par ce groupe. Aucun défaut de production ni échec de credential n’est déduit.",
      "path_type": "cold",
      "remediation": "Compléter le même test G7 avec la fixture existante de credential et une vraie CLI headless vers fil ; conserver les oracles actuels, sans nouvelle API ni nom de test.",
      "verification": "ROOT exécute le même filtre G7 : avec contexte attesté membre/projet, observer RegisterAuxiliary puis WakeRequest CliOnce et commit102 ; sans contexte, aucune réservation099 ni post102.",
      "effort": "S",
      "regression_risk": "LOW",
      "references": [
        "SPEC158B T111",
        "SPEC158B T113",
        "wake-158B.md:34"
      ],
      "status": "open"
    },
    "status": "resolved",
    "proof": {
      "file": "crates/bridget-daemon/src/cli.rs",
      "line": 10316,
      "sha256": "a72a404dafb2de209e24c5db0128ac16bfcb0c72848648718e0a0d8b70f406a2",
      "evidence": "                let id = args.windows(2).find(|w| w[0] == \"--id\").unwrap()[1].clone();\n                args[0] = thread_id.to_owned();\n                unsafe {\n                    std::env::set_var(\"BRIDGET_HOME\", daemon_root);\n                    std::env::set_var(\"BRIDGET_SOCKET\", &socket);\n                    if id != \"missing-context158b\" {\n                        std::env::set_var(\"BRIDGET_AGENT_ID_FILE\", &name_file);\n                        std::env::set_var(\n                            \"BRIDGET_AGENT_INSTANCE_ID\",\n                            if id == \"invalid-context158b\" {\n                                \"autre-instance158b\"\n                            } else {\n                                &identity.instance_id\n                            },\n                        );\n                    }",
      "verified_at": "2026-10-12T13:12:40.342548+00:00"
    },
    "resolution_report": "/Users/moi/.cache/bridget-audit158B-20261012/session-2026-10-12-spec-158B-01/cycle-scoring/findings_tests.json",
    "report_sha256": "093376662ad37abb280a82974f99782796b82e9a76af11b1da4bcdb997dab343",
    "commit_sha": null,
    "owner_proof_text": "CLI G7:10267–10346 démarre le daemonprivé existant dans le sous-processus, owner_connection:9132 reçoit/sauve credential primaire149 réel ; BRIDGET_AGENT_ID_FILE/INSTANCE_ID existants alimentent cmd_wake/stdin null. G7:10430–10454 vérifie réservations0991/0 et owner/ciblecanonique ; helper10665 relitSQLreadonly. G6:24680–24850 fait RoleClient/RegisterAuxiliary avec credential primaire149 puis CliOnceThread ; succès commit102+099Terminal, retraits membre/projet et absent/invalide zéroreserva. Final4P0F28,82s reprend les deux mêmes oracles."
  },
  {
    "original_id": "REL-001",
    "original_fingerprint": "235f9a62cf67",
    "original": {
      "id": "REL-001",
      "fingerprint": "235f9a62cf67",
      "category": "reliability",
      "subcategory": "contradictory_eviction_documentation",
      "severity": "MEDIUM",
      "file": "specs/158-reveils-controle/spec.md",
      "line": 230,
      "title": "FR106 initiale contredit la protection finale des propriétaires",
      "description": "La phrase initiale FR106 prévoit une cascade des faits quand aucun owner sans fait ne peut être retiré. Le contrat final R1 et T115 protègent au contraire les owners avec faits ou résumé non terminal et signalent une lacune globale lorsque les8192 lignes sont protégées. Le code et G9 suivent ce dernier choix ; il s’agit d’un écart documentaire, pas d’un bug de purge en production.",
      "evidence": "- **FR-105** : `bridget wake` (CLI) réveille seulement un agent ou un fil connu du registre.\n- **FR-106** : observation_missed≤8192 faits globaux,64 par abonnement,TTL 7 jours. Métadonnéesowner séparées≤8192 owners,7jours après le dernier fait/gap, retenues tantque faits restent, plus une ligne globale datée de taille fixe sansUUID/corpsalien. Évictionownerancien sansfaits d'abord, sinon cascadefacts+gapglobal atomique. Un résumé des nouveaux faits par générationRegister/DND de l'épisode, cutoff avancé seulementACK099, canonpending immuable. Pas suppression àlecture. Provenance structurée jusqu'au résultat du worker existant, sans parsebody.\n- **FR-107** : toute lacune connue est annoncée avec ses dates, jamais reconstruite. AprèsTTL/éviction de métadonnées, la preuve reste agrégée au global ; pas histoireattribuable illimitée. journal_only/throttlevolontaire ne sont pas des pertes.",
      "exploitability": "N/A",
      "impact": "Les artefacts donnent deux règles opposées à un prochain mainteneur. Une implémentation suivant le texte initial supprimerait les faits que la règle finale protège. Research43 et data-model59 conservent aussi la phrase initiale.",
      "path_type": "cold",
      "remediation": "Aligner FR106, research43 et data-model59 sur la protection finale de T115 et du contrat R1 ; conserver le code et les oracles G9.",
      "remediation_code": "Les owners avec faits vivants ou résumé non terminal sont protégés. L'entretien retire le plus ancien owner sans faits ni résumé non terminal. Si les8192 owners sont protégés, la nouvelle attribution est refusée avec lacune globale datée dans la même transaction, sans8193e ligne.",
      "verification": "Comparer les trois phrases avec contracts/wake-158B.md:110 et tasks.md:132 ; aucune cascade de faits vivants ni8193e ligne promise, aucun changement source métier.",
      "effort": "XS",
      "regression_risk": "LOW",
      "references": [
        "SPEC158B FR106",
        "SPEC158B T115",
        "wake-158B.md:110"
      ],
      "status": "open"
    },
    "status": "resolved",
    "proof": {
      "file": "specs/158-reveils-controle/spec.md",
      "line": 230,
      "sha256": "9b9d3c8c76fd0b9e0f4811f8ea0cf5634a6bff0c2f7b2c184967ae8b0cff03b4",
      "evidence": "- **FR-106** : observation_missed≤8192 faits globaux,64 par abonnement,TTL 7 jours. Métadonnéesowner séparées≤8192 owners,7jours après le dernier fait/gap, protégées tant que des faits ou un résumé non terminal restent, plus une ligne globale datée de taille fixe sansUUID/corpsalien. En saturation, retirer seulement un ancien propriétaire sans faits et sans résumé non terminal. Si les 8192 lignes sont protégées, enregistrer une lacune globale datée dans la même transaction, sans supprimer leurs faits ni créer une 8193e ligne. Un résumé des nouveaux faits par générationRegister/DND de l'épisode, cutoff avancé seulementACK099, canonpending immuable. Pas suppression àlecture. Provenance structurée jusqu'au résultat du worker existant, sans parsebody.",
      "verified_at": "2026-10-12T13:12:40.342548+00:00"
    },
    "resolution_report": "/Users/moi/.cache/bridget-audit158B-20261012/session-2026-10-12-spec-158B-01/cycle-scoring/findings_tests.json",
    "report_sha256": "093376662ad37abb280a82974f99782796b82e9a76af11b1da4bcdb997dab343",
    "commit_sha": null,
    "owner_proof_text": "FR106spec230/research43/data-model55/59 alignés sur contratfinal110/T115132 : protéger propriétaires avecfaits/résumé nonterminal ; saturation8192→lacuneglobaledatée, zéro8193eligne, sanscascadefacts vivants."
  }
]
