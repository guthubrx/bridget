# Réponses ROOT à la revue du plan159A, ronde1

Le GO implémentation reste fermé avant Analyze et la référence T001. A13 seul ; B14 reste autorisé pour la suite mais hors implémentation A.

| Objection R1 | Décision ROOT et preuve |
|---|---|
| 1 Atomicité | Le mutex State reste acquis du contrôle du repos au commit Immediate. Les magasins ouvrent le même config.db_path (daemon4806–4818). La transaction de saga lit control_state et control_pause_interruptions dans sa propre connexion ; aucun appel writer sur une seconde connexion. Le commit est le point de validation ; push différé hors verrou. La pause validée après commit ne retire pas une remise déjà libérée. Preuve : validation/atomicity-readonly.md. Aucune colonne miroir. Courses T007 déjà prévues. |
| 2 Réattestation | Retenue : échéance Instant du heartbeat vérifiée à chaque boucle, même sous trafic continu. Pas seulement WouldBlock. T006 déjà prévu. |
| 3 Octets fermés | SC001 porte sur octets du message canonique, phase et remise, pas tout le flux auxiliaire TurnState prévu FR023. Le faux repos est corrigé pour les transports attestés même held_queue fermé. Aucun faux repos conservé volontairement. Suites existantes et preuve d'équivalence T010 maintenues. |
| 4 Horloges | busy_period_start pour toutes instances turn_attest ; busy_since legacy pour non attestées. Début conservé sur true répété. Limite configurable notify_timeout_secs + 30s, 2730s uniquement par défaut. |
| 5 Coût du rang | MAX+1 supprimé. queue_rank INTEGER PRIMARY KEY AUTOINCREMENT ; INSERT omet rang. Index existant étendu, pas doublon. R13 coûts bornés et parcours global explicités. |
| 6 Purge | FR008 strict conservé grâce à sqlite_sequence interne. delivery_id UNIQUE NOT NULL garde cible FK. Copie anciens rowid ; séquence survit DELETE. B permutera valeurs existantes via négatifs transitoires dans TX sans changer watermark. |
| 7 FAIL historiques | Liste nominative fermée du FULL158B et rejoués copiée dans baseline avant T013. Aucun FULL préparatoire supplémentaire. Neuf filtres OR + transport et mesure T001 exécutés avant code ; une FULL finale T013. Référence historique ne vaut pas exécution159. |
| 8 Repères/ordre | Sources intégrées158B mêmes empreintes dans WT159A ; T002-source-correspondence159A.json. Tri list et recovery passent queue_rank. Correctif refus au wrapper6666. Anciennes plages citées dans rapports restent historiques, pas nouvelle lecture. |
| 9 Index | Étendre le même idx_send_deliveries_recipient_phase, non unique, à recipient_instance_id,phase,queue_rank. |
| 10 Retour arrière | Vrai binaire158B préservé /Users/moi/.cache/bridget-restoration159A-reference-20261012/bridget158B-reference. T011 obligatoire : backup SQLite cohérente en privé, ouverture v7 par neuf, refus par ancien, restauration backup privée et contrôle FK/comptages/liens. Aucun copie WAL/SHM séparée. |
| C01 Analyze ADR | ADR initiale adoptée avant code dans docs/decisions/159-file-retenue-et-edition.md ; T012 complète les preuves, T025 étend pour B. |
| C02 Analyze langue | Checklist traduite, 16 cases inchangées ; 27 IDs et périmètres A13/B14 inchangés. |

Réemploi frais PASS5/5 publié. Aucun code159 implémenté, aucune mesure ou réussite inventée. Le test seulement T001 reste à exécuter. La preuve T002 a été lue sur arbre158B avant livraison ; correspondance vérifiée sur les sources identiques de159A. HEAD b2359986 contient e52a9624, pas l'inverse.
