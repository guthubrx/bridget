# Contre-revue du plan158B R1 — décisions ROOT

Date : 2026-10-12. Fournisseur et verdict reçus : claudeAgent / claude-haiku-5-5 / APPROVE_WITH_CHANGES. Revue en lecture seule, aucune preuve de test. Les hypothèses sont vérifiées contre les sources existantes et les contrats ; les changements de périmètre ne sont pas appliqués.

| Objection | Vérification | Retenue | Décision |
|---|---|---|---|
| 1 | for_child retourne aussi Task terminale ; CallerScope=false ne prouve pas mission active. Preuve native ciblée. | Partiel | FR104 self/WritesOwnState conservé ; origine native et instance gelées puis lifecycle revérifié au gel/départ, fin de vie annule futures occurrences. Aucun autostart. |
| 2 | BridgetMessage originSystem existe et US5 le demande ; label2048 + préfixe < BODY_MAX_BYTES16384. | Partiel | Conserver notificationSystem daemon ; corps figé attribue explicitement propriétaire attesté, label n’accorde aucun droit. |
| 3 | Provenance once/worker et ACK099 réels relus ; owner metadata séparée requise. | Oui | Summary unknownterminal, nouvelle génération nouveau retour seulement ; protéger pendingnonterminal, saturation8192→gapglobal ; no callbackrecreate ; cutoff après pendingwrite soldés. |
| 4 | Freeze garde une occurrence unique ; aucune terminalité inconnue écrite avant R1. | Oui | Unknown terminal sans renvoi, génération suivante future. |
| 5 | begin_send_delivery TX commit099 avant socket ; thread_post commit102 distinct. | Oui | Table crash précise absence seulement lecture réussie ; stockageilllisible aucune conclusion. |
| 6 | daemon.rs1804–1811 deadline5s ;2064 budget1s. | Oui | 5s durable initial,1s I/O existant ; résultatqueueplein→unknown à échéance. |
| 7 | Pause garde unique admit_autonomous_effect ; cadence et grâce initiale pouvaient doubler rattrapage. | Oui | Ne pasfreeze pendantpause ; intervallelastdue/futureNext ; préparée600snonrenouvelée, terminalavanceancrefuture. |
| 8 | transition962 Prepared→Dispatching ; finalize1004 requiertDispatching ; post102uuidreplay. | Oui | UUID102 avantcanon099 ; transition avantpost, finalize aprèscommit, pas senddelivery/ACKfaux. |
| 9 | reserve issued_at+horizon ; lookup expired. | Oui | CLI7jours initial sansrenouvellement. |
| 10 | Factfile4096 ; host:String nonborné ; snapshot262K nebornepasfait. | Oui | JSONcomplet16384B CHECKBLOB ; dépassementgapdaté sanscorps ; summaryagrégé16KiB. |
| 11 | CircuitBreaker paire symétrique mémoire 8/180s ; prepare_dispatch check puisrecordaprès099. | Partiel | Réemploi mêmecheckrecord ownerlogique ; pasnouvellequotabusiness, quotaCLI60persistantdistinct. |
| 12 | Politiqueheureinexistante explicitementsaute dansspec. | Partiel | ListannonceTZ,next_at,règletrou ; pasnouvellealerte horsscope. |
| 13 | Planoracle label16 ambigu,64rôlesdistincts. | Oui | Libellé2048 séparé de16/128lignes ; 64faits/sub,64jobs,64résultats. |
| 14 | Halt158A admissionDelegate/Continue uniquement. | Oui | Documentation explicite cancel_active délégationsseules, programmationsinchangées. |
| 15 | FR101 limitephysiqueinclannulés ; purgeretiretoutUUIDvisible. | Oui | target_unknown aprèspurge ne prouve paséchecannulation ; list, sansregistreinfini. |

Les15objections sont tranchées. Les corrections contractuelles sont intégrées. Architecture et40items de réemploi inchangés. Les exigences feront l’objet des10groupes d’oracles déjà prévus, sans ajout de scénario de confort. Aucun test B ni résultatAnalyze déduit de cette décision. Le reçu complet de revue est conservé dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/plan-review158B-r1.json. Les preuves ciblées natives et missed sont dans le même dossier validation après leur copie vérifiée.
